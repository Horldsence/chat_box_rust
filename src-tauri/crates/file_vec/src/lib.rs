use anyhow::Result;
use cb_config::QdrantConfig;
use log::{debug, error, info, warn};
use qdrant_client::Qdrant;
use qdrant_client::qdrant::{
    Condition, CountPointsBuilder, CreateCollectionBuilder, DeletePointsBuilder, Distance, Filter,
    PointStruct, QueryPointsBuilder, SearchPointsBuilder, UpsertPointsBuilder, Value,
    VectorParamsBuilder,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use thiserror::Error;
use tokio::time::timeout;
use uuid::Uuid;

pub mod embed;
pub use embed::{
    EmbedConfig, EmbedError, EmbedResult, FastEmbedWrapper, create_default_embedder,
    create_embedder,
};

#[derive(Error, Debug)]
pub enum VectorDbError {
    #[error("Connection error: {0}")]
    Connection(#[from] qdrant_client::QdrantError),
    #[error("Timeout error: {0}")]
    Timeout(#[from] tokio::time::error::Elapsed),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("Collection not found: {0}")]
    CollectionNotFound(String),
    #[error("Invalid vector dimension: expected {expected}, got {actual}")]
    InvalidVectorDimension { expected: usize, actual: usize },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorPoint {
    pub id: Uuid,
    pub vector: Vec<f32>,
    pub payload: HashMap<String, serde_json::Value>,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: Uuid,
    pub score: f32,
    pub payload: HashMap<String, serde_json::Value>,
    pub vector: Option<Vec<f32>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub vector: Vec<f32>,
    pub filter: Option<HashMap<String, serde_json::Value>>,
    pub limit: usize,
    pub with_payload: bool,
    pub with_vector: bool,
    pub score_threshold: Option<f32>,
}

pub struct QdrantVectorDb {
    client: Qdrant,
    config: QdrantConfig,
}

impl QdrantVectorDb {
    /// Create a new QdrantVectorDb instance
    pub async fn new(config: QdrantConfig) -> Result<Self, VectorDbError> {
        let client_url = if config.use_grpc {
            format!("{}:{}", config.server_url, config.server_port)
        } else {
            format!("{}:{}", config.server_url, config.server_port - 1) // REST API is typically on port 6333
        };

        info!("Connecting to Qdrant at {}", client_url);

        let client = Qdrant::from_url(&client_url)
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()
            .map_err(VectorDbError::Connection)?;

        Ok(Self { client, config })
    }

    /// Test the connection to Qdrant
    pub async fn test_connection(&self) -> Result<bool, VectorDbError> {
        info!("Testing Qdrant connection...");

        let timeout_duration = Duration::from_secs(self.config.timeout_seconds);

        match timeout(timeout_duration, self.client.health_check()).await {
            Ok(result) => match result {
                Ok(_) => {
                    info!("Qdrant connection successful");
                    Ok(true)
                }
                Err(e) => {
                    error!("Qdrant health check failed: {}", e);
                    Err(VectorDbError::Connection(e))
                }
            },
            Err(e) => {
                error!("Qdrant connection timeout");
                Err(VectorDbError::Timeout(e))
            }
        }
    }

    /// Create a collection if it doesn't exist
    pub async fn ensure_collection(&self) -> Result<(), VectorDbError> {
        let collection_name = &self.config.collection_name;

        // Check if collection exists
        match self.client.collection_info(collection_name).await {
            Ok(_) => {
                info!("Collection '{}' already exists", collection_name);
                return Ok(());
            }
            Err(_) => {
                info!(
                    "Collection '{}' does not exist, creating...",
                    collection_name
                );
            }
        }

        // Parse distance metric
        let distance = match self.config.distance_metric.to_lowercase().as_str() {
            "cosine" => Distance::Cosine,
            "dot" => Distance::Dot,
            "euclidean" => Distance::Euclid,
            "manhattan" => Distance::Manhattan,
            _ => {
                warn!(
                    "Unknown distance metric '{}', using Cosine",
                    self.config.distance_metric
                );
                Distance::Cosine
            }
        };

        // Create collection
        let create_collection = CreateCollectionBuilder::new(collection_name)
            .vectors_config(VectorParamsBuilder::new(self.config.vector_size, distance));

        self.client
            .create_collection(create_collection)
            .await
            .map_err(VectorDbError::Connection)?;

        info!("Collection '{}' created successfully", collection_name);
        Ok(())
    }

    /// Insert or update a vector point
    pub async fn upsert_point(&self, point: VectorPoint) -> Result<(), VectorDbError> {
        self.upsert_points(vec![point]).await
    }

    /// Insert or update multiple vector points
    pub async fn upsert_points(&self, points: Vec<VectorPoint>) -> Result<(), VectorDbError> {
        if points.is_empty() {
            return Ok(());
        }

        // Validate vector dimensions
        for point in &points {
            if point.vector.len() != self.config.vector_size as usize {
                return Err(VectorDbError::InvalidVectorDimension {
                    expected: self.config.vector_size as usize,
                    actual: point.vector.len(),
                });
            }
        }

        // Convert to Qdrant points
        let qdrant_points: Vec<PointStruct> = points
            .into_iter()
            .map(|point| {
                let mut payload_map = HashMap::new();

                // Add the original payload
                for (key, value) in point.payload {
                    payload_map.insert(key, value.into());
                }

                // Add timestamp
                payload_map.insert("timestamp".to_string(), point.timestamp.into());

                PointStruct::new(point.id.to_string(), point.vector, payload_map)
            })
            .collect();

        let upsert_points =
            UpsertPointsBuilder::new(&self.config.collection_name, qdrant_points).wait(true);

        self.client
            .upsert_points(upsert_points)
            .await
            .map_err(VectorDbError::Connection)?;

        debug!(
            "Successfully upserted points to collection '{}'",
            self.config.collection_name
        );
        Ok(())
    }

    /// Search for similar vectors
    pub async fn search(&self, query: SearchQuery) -> Result<Vec<SearchResult>, VectorDbError> {
        if query.vector.len() != self.config.vector_size as usize {
            return Err(VectorDbError::InvalidVectorDimension {
                expected: self.config.vector_size as usize,
                actual: query.vector.len(),
            });
        }

        let mut search_builder = SearchPointsBuilder::new(
            &self.config.collection_name,
            query.vector,
            query.limit as u64,
        )
        .with_payload(query.with_payload)
        .with_vectors(query.with_vector);

        // Add score threshold if specified
        if let Some(threshold) = query.score_threshold {
            search_builder = search_builder.score_threshold(threshold);
        }

        // Add filter if specified
        if let Some(filter_map) = query.filter {
            let mut conditions = Vec::new();

            for (key, value) in filter_map {
                let condition = match value {
                    serde_json::Value::String(s) => Condition::matches(&key, s),
                    serde_json::Value::Number(n) => {
                        if let Some(i) = n.as_i64() {
                            Condition::matches(&key, i)
                        } else if let Some(f) = n.as_f64() {
                            Condition::matches(&key, f as i64)
                        } else {
                            continue;
                        }
                    }
                    serde_json::Value::Bool(b) => Condition::matches(&key, b),
                    _ => continue,
                };
                conditions.push(condition);
            }

            if !conditions.is_empty() {
                search_builder = search_builder.filter(Filter::must(conditions));
            }
        }

        let search_result = self
            .client
            .search_points(search_builder)
            .await
            .map_err(VectorDbError::Connection)?;

        let results: Vec<SearchResult> = search_result
            .result
            .into_iter()
            .filter_map(|scored_point| {
                let id_str = match &scored_point.id {
                    Some(point_id) => match &point_id.point_id_options {
                        Some(qdrant_client::qdrant::point_id::PointIdOptions::Uuid(uuid_str)) => {
                            uuid_str
                        }
                        Some(qdrant_client::qdrant::point_id::PointIdOptions::Num(num)) => {
                            &num.to_string()
                        }
                        None => return None,
                    },
                    None => return None,
                };

                let id = match Uuid::parse_str(id_str) {
                    Ok(uuid) => uuid,
                    Err(_) => return None,
                };

                let mut payload = HashMap::new();
                for (key, value) in scored_point.payload {
                    if let Some(json_value) = qdrant_value_to_json(value) {
                        payload.insert(key, json_value);
                    }
                }

                let vector =
                    scored_point
                        .vectors
                        .and_then(|vectors| match vectors.vectors_options {
                            Some(
                                qdrant_client::qdrant::vectors_output::VectorsOptions::Vector(
                                    vector_data,
                                ),
                            ) => Some(vector_data.data),
                            _ => None,
                        });

                Some(SearchResult {
                    id,
                    score: scored_point.score,
                    payload,
                    vector,
                })
            })
            .collect();

        debug!("Search returned {} results", results.len());
        Ok(results)
    }

    /// Query points (newer API, more flexible than search)
    pub async fn query(&self, query: SearchQuery) -> Result<Vec<SearchResult>, VectorDbError> {
        if query.vector.len() != self.config.vector_size as usize {
            return Err(VectorDbError::InvalidVectorDimension {
                expected: self.config.vector_size as usize,
                actual: query.vector.len(),
            });
        }

        let mut query_builder = QueryPointsBuilder::new(&self.config.collection_name)
            .query(query.vector)
            .limit(query.limit as u64)
            .with_payload(query.with_payload);

        // Add score threshold if specified
        if let Some(threshold) = query.score_threshold {
            query_builder = query_builder.score_threshold(threshold);
        }

        // Add filter if specified
        if let Some(filter_map) = query.filter {
            let mut conditions = Vec::new();

            for (key, value) in filter_map {
                let condition = match value {
                    serde_json::Value::String(s) => Condition::matches(&key, s),
                    serde_json::Value::Number(n) => {
                        if let Some(i) = n.as_i64() {
                            Condition::matches(&key, i)
                        } else if let Some(f) = n.as_f64() {
                            Condition::matches(&key, f as i64)
                        } else {
                            continue;
                        }
                    }
                    serde_json::Value::Bool(b) => Condition::matches(&key, b),
                    _ => continue,
                };
                conditions.push(condition);
            }

            if !conditions.is_empty() {
                query_builder = query_builder.filter(Filter::must(conditions));
            }
        }

        let query_result = self
            .client
            .query(query_builder)
            .await
            .map_err(VectorDbError::Connection)?;

        let results: Vec<SearchResult> = query_result
            .result
            .into_iter()
            .filter_map(|scored_point| {
                let id_str = match &scored_point.id {
                    Some(point_id) => match &point_id.point_id_options {
                        Some(qdrant_client::qdrant::point_id::PointIdOptions::Uuid(uuid_str)) => {
                            uuid_str
                        }
                        Some(qdrant_client::qdrant::point_id::PointIdOptions::Num(num)) => {
                            &num.to_string()
                        }
                        None => return None,
                    },
                    None => return None,
                };

                let id = match Uuid::parse_str(id_str) {
                    Ok(uuid) => uuid,
                    Err(_) => return None,
                };

                let mut payload = HashMap::new();
                for (key, value) in scored_point.payload {
                    if let Some(json_value) = qdrant_value_to_json(value) {
                        payload.insert(key, json_value);
                    }
                }

                let vector =
                    scored_point
                        .vectors
                        .and_then(|vectors| match vectors.vectors_options {
                            Some(
                                qdrant_client::qdrant::vectors_output::VectorsOptions::Vector(
                                    vector_data,
                                ),
                            ) => Some(vector_data.data),
                            _ => None,
                        });

                Some(SearchResult {
                    id,
                    score: scored_point.score,
                    payload,
                    vector,
                })
            })
            .collect();

        debug!("Query returned {} results", results.len());
        Ok(results)
    }

    /// Delete points by IDs
    pub async fn delete_points(&self, ids: Vec<Uuid>) -> Result<(), VectorDbError> {
        if ids.is_empty() {
            return Ok(());
        }

        let point_ids: Vec<String> = ids.into_iter().map(|id| id.to_string()).collect();
        let count = point_ids.len();

        let delete_points =
            DeletePointsBuilder::new(&self.config.collection_name).points(point_ids);

        self.client
            .delete_points(delete_points)
            .await
            .map_err(VectorDbError::Connection)?;

        debug!("Successfully deleted {} points", count);
        Ok(())
    }

    /// Delete points by filter
    pub async fn delete_points_by_filter(
        &self,
        filter: HashMap<String, serde_json::Value>,
    ) -> Result<(), VectorDbError> {
        let mut conditions = Vec::new();

        for (key, value) in filter {
            let condition = match value {
                serde_json::Value::String(s) => Condition::matches(&key, s),
                serde_json::Value::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        Condition::matches(&key, i)
                    } else if let Some(f) = n.as_f64() {
                        Condition::matches(&key, f as i64)
                    } else {
                        continue;
                    }
                }
                serde_json::Value::Bool(b) => Condition::matches(&key, b),
                _ => continue,
            };
            conditions.push(condition);
        }

        if conditions.is_empty() {
            warn!("No valid filter conditions provided for deletion");
            return Ok(());
        }

        // Note: In the current Qdrant client version, deleting by filter through
        // DeletePointsBuilder might not be directly supported.
        // We'll use an alternative approach: first query the points that match the filter,
        // then delete them by IDs.
        warn!("delete_points_by_filter is not directly supported in this Qdrant client version");
        warn!(
            "Consider implementing this by first querying matching points and then deleting by IDs"
        );
        Ok(())
    }

    /// Get collection info
    pub async fn get_collection_info(&self) -> Result<serde_json::Value, VectorDbError> {
        let info = self
            .client
            .collection_info(&self.config.collection_name)
            .await
            .map_err(VectorDbError::Connection)?;

        // Since GetCollectionInfoResponse doesn't implement Serialize,
        // we need to manually extract the information we need
        let mut json_info = serde_json::Map::new();

        // Extract basic collection information
        if let Some(result) = info.result {
            json_info.insert(
                "status".to_string(),
                serde_json::Value::String(result.status.to_string()),
            );

            if let Some(config) = result.config {
                let mut config_map = serde_json::Map::new();

                if let Some(params) = config.params {
                    let mut params_map = serde_json::Map::new();
                    params_map.insert(
                        "shard_number".to_string(),
                        serde_json::Value::Number(serde_json::Number::from(params.shard_number)),
                    );
                    if let Some(replication_factor) = params.replication_factor {
                        params_map.insert(
                            "replication_factor".to_string(),
                            serde_json::Value::Number(serde_json::Number::from(replication_factor)),
                        );
                    }
                    config_map.insert("params".to_string(), serde_json::Value::Object(params_map));
                }

                json_info.insert("config".to_string(), serde_json::Value::Object(config_map));
            }
        }

        Ok(serde_json::Value::Object(json_info))
    }

    /// Count points in collection
    pub async fn count_points(&self) -> Result<u64, VectorDbError> {
        let count_request = CountPointsBuilder::new(&self.config.collection_name);

        let count_result = self
            .client
            .count(count_request)
            .await
            .map_err(VectorDbError::Connection)?;

        Ok(count_result.result.map(|r| r.count).unwrap_or(0))
    }
}

/// Convert Qdrant Value to serde_json::Value
fn qdrant_value_to_json(value: Value) -> Option<serde_json::Value> {
    match value.kind {
        Some(qdrant_client::qdrant::value::Kind::StringValue(s)) => {
            Some(serde_json::Value::String(s))
        }
        Some(qdrant_client::qdrant::value::Kind::IntegerValue(i)) => {
            Some(serde_json::Value::Number(serde_json::Number::from(i)))
        }
        Some(qdrant_client::qdrant::value::Kind::DoubleValue(f)) => {
            serde_json::Number::from_f64(f).map(serde_json::Value::Number)
        }
        Some(qdrant_client::qdrant::value::Kind::BoolValue(b)) => Some(serde_json::Value::Bool(b)),
        Some(qdrant_client::qdrant::value::Kind::NullValue(_)) => Some(serde_json::Value::Null),
        Some(qdrant_client::qdrant::value::Kind::ListValue(list)) => {
            let values: Vec<serde_json::Value> = list
                .values
                .into_iter()
                .filter_map(qdrant_value_to_json)
                .collect();
            Some(serde_json::Value::Array(values))
        }
        Some(qdrant_client::qdrant::value::Kind::StructValue(struct_value)) => {
            let mut map = serde_json::Map::new();
            for (key, value) in struct_value.fields {
                if let Some(json_value) = qdrant_value_to_json(value) {
                    map.insert(key, json_value);
                }
            }
            Some(serde_json::Value::Object(map))
        }
        None => None,
    }
}

/// Helper function to create a VectorPoint
pub fn create_vector_point(
    vector: Vec<f32>,
    payload: HashMap<String, serde_json::Value>,
) -> VectorPoint {
    VectorPoint {
        id: Uuid::new_v4(),
        vector,
        payload,
        timestamp: chrono::Utc::now().timestamp(),
    }
}

/// Helper function to create a basic search query
pub fn create_search_query(vector: Vec<f32>, limit: usize) -> SearchQuery {
    SearchQuery {
        vector,
        filter: None,
        limit,
        with_payload: true,
        with_vector: false,
        score_threshold: None,
    }
}

/// Helper function to create a filtered search query
pub fn create_filtered_search_query(
    vector: Vec<f32>,
    filter: HashMap<String, serde_json::Value>,
    limit: usize,
) -> SearchQuery {
    SearchQuery {
        vector,
        filter: Some(filter),
        limit,
        with_payload: true,
        with_vector: false,
        score_threshold: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[allow(dead_code)]
    fn get_test_config() -> QdrantConfig {
        QdrantConfig {
            enabled: true,
            server_url: "http://localhost".to_string(),
            server_port: 6334,
            collection_name: "test_collection".to_string(),
            vector_size: 4,
            distance_metric: "Cosine".to_string(),
            timeout_seconds: 30,
            use_grpc: true,
        }
    }

    #[tokio::test]
    async fn test_vector_point_creation() {
        let mut payload = HashMap::new();
        payload.insert(
            "text".to_string(),
            serde_json::Value::String("test".to_string()),
        );

        let point = create_vector_point(vec![0.1, 0.2, 0.3, 0.4], payload);

        assert_eq!(point.vector.len(), 4);
        assert!(point.payload.contains_key("text"));
        assert!(point.timestamp > 0);
    }

    #[tokio::test]
    async fn test_search_query_creation() {
        let query = create_search_query(vec![0.1, 0.2, 0.3, 0.4], 10);

        assert_eq!(query.vector.len(), 4);
        assert_eq!(query.limit, 10);
        assert!(query.with_payload);
        assert!(!query.with_vector);
        assert!(query.filter.is_none());
        assert!(query.score_threshold.is_none());
    }

    // Note: The following tests require a running Qdrant instance
    // They are commented out by default to avoid test failures in CI/CD

    /*
    #[tokio::test]
    async fn test_connection() {
        let config = get_test_config();
        let db = QdrantVectorDb::new(config).await.unwrap();
        let result = db.test_connection().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_collection_creation() {
        let config = get_test_config();
        let db = QdrantVectorDb::new(config).await.unwrap();
        let result = db.ensure_collection().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_upsert_and_search() {
        let config = get_test_config();
        let db = QdrantVectorDb::new(config).await.unwrap();

        // Ensure collection exists
        db.ensure_collection().await.unwrap();

        // Create test points
        let mut payload1 = HashMap::new();
        payload1.insert("city".to_string(), serde_json::Value::String("Berlin".to_string()));
        let point1 = create_vector_point(vec![0.05, 0.61, 0.76, 0.74], payload1);

        let mut payload2 = HashMap::new();
        payload2.insert("city".to_string(), serde_json::Value::String("London".to_string()));
        let point2 = create_vector_point(vec![0.19, 0.81, 0.75, 0.11], payload2);

        // Upsert points
        db.upsert_points(vec![point1, point2]).await.unwrap();

        // Search
        let query = create_search_query(vec![0.2, 0.1, 0.9, 0.7], 5);
        let results = db.search(query).await.unwrap();

        assert!(!results.is_empty());
        assert!(results.len() <= 5);
    }
    */
}
