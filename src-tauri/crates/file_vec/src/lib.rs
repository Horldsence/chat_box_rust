pub mod embed;
pub mod vec_db;

use cb_config::{EmbedConfig, QdrantConfig};
use embed::FastEmbedWrapper;
use file_finder::FileFinder;
use log::{debug, info, warn};
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;
use vec_db::{QdrantVectorDb, SearchQuery, SearchResult, VectorPoint};

pub struct VectorDBConfig {
    pub qdrant_config: QdrantConfig,
    pub embed_config: EmbedConfig,
}

pub struct VectorDb {
    database: QdrantVectorDb,
    file_embed: FastEmbedWrapper,
    file_finder: FileFinder,
    file_id_map: std::sync::Arc<std::sync::Mutex<HashMap<String, Uuid>>>, // file_path -> point_id
}

impl VectorDb {
    pub async fn new(
        embed_config: EmbedConfig,
        qdrant_config: QdrantConfig,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let database = QdrantVectorDb::new(qdrant_config).await?;
        let file_embed = FastEmbedWrapper::new(embed_config);

        // Initialize the embedding model
        file_embed.initialize().await?;

        let file_finder = FileFinder::new()?;

        Ok(Self {
            database,
            file_embed,
            file_finder,
            file_id_map: std::sync::Arc::new(std::sync::Mutex::new(HashMap::new())),
        })
    }

    /// Add a file to the vector database
    pub async fn add_file(&self, file_path: &str) -> Result<String, Box<dyn std::error::Error>> {
        let file_content = std::fs::read_to_string(file_path)?;

        // Generate embedding for the file content
        let embed_result = self.file_embed.embed_passage(&file_content).await?;

        // Generate unique ID for this file
        let point_id = Uuid::new_v4();

        // Create payload with file metadata
        let mut payload = HashMap::new();
        payload.insert(
            "file_path".to_string(),
            Value::String(file_path.to_string()),
        );
        payload.insert(
            "content_preview".to_string(),
            Value::String(file_content.chars().take(200).collect()),
        );

        // Get file info if possible
        if let Ok(Some(file_info)) = self
            .file_finder
            .get_file_info(std::path::Path::new(file_path))
            .await
        {
            payload.insert("file_name".to_string(), Value::String(file_info.name));
            payload.insert(
                "file_size".to_string(),
                Value::Number(serde_json::Number::from(file_info.size)),
            );
            payload.insert(
                "modified".to_string(),
                Value::Number(serde_json::Number::from(file_info.modified)),
            );
            if let Some(ext) = file_info.extension {
                payload.insert("extension".to_string(), Value::String(ext));
            }
        }

        let vector_point = VectorPoint {
            id: point_id,
            vector: embed_result.vector,
            payload,
            timestamp: chrono::Utc::now().timestamp_millis() as i64,
        };

        // Store in database
        self.database.upsert_point(vector_point).await?;

        // Update file ID mapping
        let mut file_map = self.file_id_map.lock().unwrap();
        file_map.insert(file_path.to_string(), point_id);

        info!(
            "Added file '{}' with ID '{}'",
            file_path,
            point_id.to_string()
        );
        Ok(point_id.to_string())
    }

    /// Search for similar files based on query text
    pub async fn search(
        &self,
        query: &str,
        top_k: usize,
    ) -> Result<Vec<SearchResult>, Box<dyn std::error::Error>> {
        // Generate embedding for the query
        let embed_result = self.file_embed.embed_query(query).await?;

        let search_query = SearchQuery {
            vector: embed_result.vector,
            filter: None,
            limit: top_k,
            with_payload: true,
            with_vector: false,
            score_threshold: Some(0.5), // Minimum similarity threshold
        };

        let results = self.database.search(search_query).await?;

        debug!(
            "Search query '{}' returned {} results",
            query,
            results.len()
        );
        Ok(results)
    }

    /// Search with additional filters
    pub async fn search_with_filter(
        &self,
        query: &str,
        top_k: usize,
        filter: HashMap<String, Value>,
    ) -> Result<Vec<SearchResult>, Box<dyn std::error::Error>> {
        let embed_result = self.file_embed.embed_query(query).await?;

        let search_query = SearchQuery {
            vector: embed_result.vector,
            filter: Some(filter),
            limit: top_k,
            with_payload: true,
            with_vector: false,
            score_threshold: Some(0.5),
        };

        let results = self.database.search(search_query).await?;
        Ok(results)
    }

    /// Delete a file from the vector database
    pub async fn delete_file(&self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut file_map = self.file_id_map.lock().unwrap();

        if let Some(point_id) = file_map.remove(file_path) {
            drop(file_map); // Release the lock before async call
            self.database.delete_points(vec![point_id.clone()]).await?;
            info!("Deleted file '{}' with ID '{}'", file_path, point_id);
        } else {
            warn!("File '{}' not found in vector database", file_path);
        }

        Ok(())
    }

    /// Delete a vector point by its ID
    pub async fn delete_point(&self, point_id: &Uuid) -> Result<(), Box<dyn std::error::Error>> {
        self.database.delete_points(vec![point_id.clone()]).await?;

        // Remove from file mapping if exists
        let mut file_map = self.file_id_map.lock().unwrap();
        file_map.retain(|_, id| id != point_id);

        Ok(())
    }

    /// Delete all embeddings from the database
    pub async fn delete_all_embeddings(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Clear all points using filter (match all)
        let filter = HashMap::new(); // Empty filter matches all
        self.database.delete_points_by_filter(filter).await?;

        // Clear file mapping
        let mut file_map = self.file_id_map.lock().unwrap();
        file_map.clear();

        info!("Deleted all embeddings from vector database");
        Ok(())
    }

    /// Get collection information and stats
    pub async fn get_stats(&self) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let collection_info = self.database.get_collection_info().await?;
        let point_count = self.database.count_points().await?;

        let file_map = self.file_id_map.lock().unwrap();
        let mapped_files = file_map.len();
        drop(file_map);

        let stats = serde_json::json!({
            "collection_info": collection_info,
            "point_count": point_count,
            "mapped_files": mapped_files,
            "embedding_model": self.file_embed.get_config().model_name
        });

        Ok(stats)
    }

    /// Check for new files and add them to the database
    pub async fn sync_files(
        &self,
        directory_path: &str,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        // Build/refresh file index for the directory
        self.file_finder
            .index
            .build_index(std::path::Path::new(directory_path))
            .await?;

        // Search for all files in the directory
        let search_options = file_finder::SearchOptions {
            pattern: "*".to_string(),
            use_regex: false,
            include_hidden: false,
            max_depth: None,
            file_types: Some(vec![
                "txt".to_string(),
                "md".to_string(),
                "rs".to_string(),
                "py".to_string(),
            ]),
            max_results: None,
            search_content: false,
        };

        let files = self.file_finder.search_files(search_options)?;
        let mut added_files = Vec::new();

        for file_info in files {
            if !file_info.is_dir {
                let file_path = file_info.path.to_string_lossy();

                // Check if file is already in database
                let file_map = self.file_id_map.lock().unwrap();
                let already_exists = file_map.contains_key(file_path.as_ref());
                drop(file_map);

                if !already_exists {
                    match self.add_file(&file_path).await {
                        Ok(point_id) => {
                            added_files.push(file_path.to_string());
                            debug!("Added new file: {} (ID: {})", file_path, point_id);
                        }
                        Err(e) => {
                            warn!("Failed to add file {}: {}", file_path, e);
                        }
                    }
                }
            }
        }

        info!("Synced {} new files to vector database", added_files.len());
        Ok(added_files)
    }

    /// Update an existing file in the database
    pub async fn update_file(&self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        // First delete the existing entry
        self.delete_file(file_path).await?;

        // Then add it again with updated content
        self.add_file(file_path).await?;

        info!("Updated file '{}' in vector database", file_path);
        Ok(())
    }

    /// Get file paths that are currently indexed
    pub fn get_indexed_files(&self) -> Vec<String> {
        let file_map = self.file_id_map.lock().unwrap();
        file_map.keys().cloned().collect()
    }

    /// Test database connection
    pub async fn test_connection(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.database.test_connection().await?;
        Ok(())
    }

    /// Get embedding dimension
    pub async fn get_embedding_dimension(&self) -> Result<usize, Box<dyn std::error::Error>> {
        let dimension = self.file_embed.get_vector_dimension().await?;
        Ok(dimension)
    }
}

// Helper functions for creating configurations
pub fn create_default_config() -> Result<VectorDBConfig, Box<dyn std::error::Error>> {
    let embed_config = EmbedConfig {
        model_name: "BAAI/bge-small-en-v1.5".to_string(),
        max_length: 512,
        batch_size: 32,
        show_download_progress: true,
        cache_dir: None,
    };

    let qdrant_config = QdrantConfig {
        enabled: true,
        server_url: "http://localhost".to_string(),
        server_port: 6333,
        collection_name: "file_vectors".to_string(),
        vector_size: 384, // BGE-small dimension
        distance_metric: "cosine".to_string(),
        timeout_seconds: 30,
        use_grpc: false,
    };

    Ok(VectorDBConfig {
        qdrant_config,
        embed_config,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_vector_db_creation() {
        let config = create_default_config().unwrap();

        // This test requires a running Qdrant instance
        // Skip if not available
        if let Ok(_db) = VectorDb::new(config.embed_config, config.qdrant_config).await {
            // Test passed - database created successfully
        } else {
            // Skip test if Qdrant is not available
            println!("Skipping test - Qdrant not available");
        }
    }

    #[test]
    fn test_config_creation() {
        let config = create_default_config().unwrap();
        assert_eq!(config.embed_config.model_name, "BAAI/bge-small-en-v1.5");
        assert_eq!(config.qdrant_config.collection_name, "file_vectors");
    }
}
