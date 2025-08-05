// use crate::state::AppState;
// use file_vec::{create_default_config, VectorDBConfig, VectorDb};
// use log::{debug, error, info, warn};
// use serde::{Deserialize, Serialize};
// use serde_json::Value;
// use std::collections::HashMap;
// use tauri::State;

// #[derive(Debug, Serialize, Deserialize)]
// pub struct VectorSearchResult {
//     pub id: String,
//     pub score: f32,
//     pub file_path: Option<String>,
//     pub file_name: Option<String>,
//     pub content_preview: Option<String>,
//     pub metadata: HashMap<String, Value>,
// }

// #[derive(Debug, Serialize, Deserialize)]
// pub struct VectorDbStats {
//     pub point_count: u64,
//     pub mapped_files: usize,
//     pub embedding_model: String,
//     pub collection_info: Value,
// }

// #[derive(Debug, Serialize, Deserialize)]
// pub struct SearchOptions {
//     pub query: String,
//     pub top_k: Option<usize>,
//     pub score_threshold: Option<f32>,
//     pub file_types: Option<Vec<String>>,
//     pub include_metadata: Option<bool>,
// }

// impl Default for SearchOptions {
//     fn default() -> Self {
//         Self {
//             query: String::new(),
//             top_k: Some(10),
//             score_threshold: Some(0.5),
//             file_types: None,
//             include_metadata: Some(true),
//         }
//     }
// }

// /// Initialize vector database with default configuration
// #[tauri::command]
// pub async fn init_vector_db(app_state: State<'_, AppState>) -> Result<String, String> {
//     info!("Initializing vector database with default configuration");

//     match create_default_config() {
//         Ok(config) => {
//             match VectorDb::new(config.embed_config, config.qdrant_config).await {
//                 Ok(vector_db) => {
//                     // Store the vector_db in app_state if needed
//                     // For now, we'll create it on-demand in each command
//                     info!("Vector database initialized successfully");
//                     Ok("Vector database initialized successfully".to_string())
//                 }
//                 Err(e) => {
//                     error!("Failed to initialize vector database: {}", e);
//                     Err(format!("Failed to initialize vector database: {}", e))
//                 }
//             }
//         }
//         Err(e) => {
//             error!("Failed to create vector database configuration: {}", e);
//             Err(format!("Failed to create configuration: {}", e))
//         }
//     }
// }

// /// Add a file to the vector database
// #[tauri::command]
// pub async fn add_file_to_vector_db(
//     file_path: String,
//     app_state: State<'_, AppState>,
// ) -> Result<String, String> {
//     info!("Adding file to vector database: {}", file_path);

//     let config = create_default_config().map_err(|e| e.to_string())?;
//     let vector_db = VectorDb::new(config.embed_config, config.qdrant_config)
//         .await
//         .map_err(|e| e.to_string())?;

//     match vector_db.add_file(&file_path).await {
//         Ok(point_id) => {
//             info!(
//                 "Successfully added file '{}' with ID '{}'",
//                 file_path, point_id
//             );
//             Ok(point_id)
//         }
//         Err(e) => {
//             error!("Failed to add file '{}': {}", file_path, e);
//             Err(format!("Failed to add file: {}", e))
//         }
//     }
// }

// /// Search for similar files in the vector database
// #[tauri::command]
// pub async fn search_vector_db(
//     options: SearchOptions,
//     app_state: State<'_, AppState>,
// ) -> Result<Vec<VectorSearchResult>, String> {
//     debug!("Searching vector database with query: '{}'", options.query);

//     let config = create_default_config().map_err(|e| e.to_string())?;
//     let vector_db = VectorDb::new(config.embed_config, config.qdrant_config)
//         .await
//         .map_err(|e| e.to_string())?;

//     let top_k = options.top_k.unwrap_or(10);

//     let search_results = if let Some(file_types) = options.file_types {
//         // Create filter for file types
//         let mut filter = HashMap::new();
//         let extensions_json = Value::Array(file_types.into_iter().map(Value::String).collect());
//         filter.insert("extension".to_string(), extensions_json);

//         vector_db
//             .search_with_filter(&options.query, top_k, filter)
//             .await
//             .map_err(|e| e.to_string())?
//     } else {
//         vector_db
//             .search(&options.query, top_k)
//             .await
//             .map_err(|e| e.to_string())?
//     };

//     let results: Vec<VectorSearchResult> = search_results
//         .into_iter()
//         .filter(|result| {
//             if let Some(threshold) = options.score_threshold {
//                 result.score >= threshold
//             } else {
//                 true
//             }
//         })
//         .map(|result| {
//             let file_path = result
//                 .payload
//                 .get("file_path")
//                 .and_then(|v| v.as_str())
//                 .map(|s| s.to_string());

//             let file_name = result
//                 .payload
//                 .get("file_name")
//                 .and_then(|v| v.as_str())
//                 .map(|s| s.to_string());

//             let content_preview = result
//                 .payload
//                 .get("content_preview")
//                 .and_then(|v| v.as_str())
//                 .map(|s| s.to_string());

//             VectorSearchResult {
//                 id: result.id,
//                 score: result.score,
//                 file_path,
//                 file_name,
//                 content_preview,
//                 metadata: if options.include_metadata.unwrap_or(true) {
//                     result.payload
//                 } else {
//                     HashMap::new()
//                 },
//             }
//         })
//         .collect();

//     info!("Vector search returned {} results", results.len());
//     Ok(results)
// }

// /// Delete a file from the vector database
// #[tauri::command]
// pub async fn delete_file_from_vector_db(
//     file_path: String,
//     app_state: State<'_, AppState>,
// ) -> Result<String, String> {
//     info!("Deleting file from vector database: {}", file_path);

//     let config = create_default_config().map_err(|e| e.to_string())?;
//     let vector_db = VectorDb::new(config.embed_config, config.qdrant_config)
//         .await
//         .map_err(|e| e.to_string())?;

//     match vector_db.delete_file(&file_path).await {
//         Ok(_) => {
//             info!("Successfully deleted file '{}'", file_path);
//             Ok("File deleted successfully".to_string())
//         }
//         Err(e) => {
//             error!("Failed to delete file '{}': {}", file_path, e);
//             Err(format!("Failed to delete file: {}", e))
//         }
//     }
// }

// /// Sync files from a directory to the vector database
// #[tauri::command]
// pub async fn sync_directory_to_vector_db(
//     directory_path: String,
//     app_state: State<'_, AppState>,
// ) -> Result<Vec<String>, String> {
//     info!("Syncing directory to vector database: {}", directory_path);

//     let config = create_default_config().map_err(|e| e.to_string())?;
//     let vector_db = VectorDb::new(config.embed_config, config.qdrant_config)
//         .await
//         .map_err(|e| e.to_string())?;

//     match vector_db.sync_files(&directory_path).await {
//         Ok(added_files) => {
//             info!(
//                 "Successfully synced {} files from directory '{}'",
//                 added_files.len(),
//                 directory_path
//             );
//             Ok(added_files)
//         }
//         Err(e) => {
//             error!("Failed to sync directory '{}': {}", directory_path, e);
//             Err(format!("Failed to sync directory: {}", e))
//         }
//     }
// }

// /// Get vector database statistics
// #[tauri::command]
// pub async fn get_vector_db_stats(app_state: State<'_, AppState>) -> Result<VectorDbStats, String> {
//     debug!("Getting vector database statistics");

//     let config = create_default_config().map_err(|e| e.to_string())?;
//     let vector_db = VectorDb::new(config.embed_config, config.qdrant_config)
//         .await
//         .map_err(|e| e.to_string())?;

//     match vector_db.get_stats().await {
//         Ok(stats_json) => {
//             let stats = VectorDbStats {
//                 point_count: stats_json
//                     .get("point_count")
//                     .and_then(|v| v.as_u64())
//                     .unwrap_or(0),
//                 mapped_files: stats_json
//                     .get("mapped_files")
//                     .and_then(|v| v.as_u64())
//                     .unwrap_or(0) as usize,
//                 embedding_model: stats_json
//                     .get("embedding_model")
//                     .and_then(|v| v.as_str())
//                     .unwrap_or("unknown")
//                     .to_string(),
//                 collection_info: stats_json
//                     .get("collection_info")
//                     .cloned()
//                     .unwrap_or(Value::Null),
//             };

//             debug!(
//                 "Vector database stats: {} points, {} mapped files",
//                 stats.point_count, stats.mapped_files
//             );
//             Ok(stats)
//         }
//         Err(e) => {
//             error!("Failed to get vector database statistics: {}", e);
//             Err(format!("Failed to get statistics: {}", e))
//         }
//     }
// }

// /// Get list of indexed files
// #[tauri::command]
// pub async fn get_indexed_files(app_state: State<'_, AppState>) -> Result<Vec<String>, String> {
//     debug!("Getting list of indexed files");

//     let config = create_default_config().map_err(|e| e.to_string())?;
//     let vector_db = VectorDb::new(config.embed_config, config.qdrant_config)
//         .await
//         .map_err(|e| e.to_string())?;

//     let indexed_files = vector_db.get_indexed_files();
//     debug!("Found {} indexed files", indexed_files.len());
//     Ok(indexed_files)
// }

// /// Update an existing file in the vector database
// #[tauri::command]
// pub async fn update_file_in_vector_db(
//     file_path: String,
//     app_state: State<'_, AppState>,
// ) -> Result<String, String> {
//     info!("Updating file in vector database: {}", file_path);

//     let config = create_default_config().map_err(|e| e.to_string())?;
//     let vector_db = VectorDb::new(config.embed_config, config.qdrant_config)
//         .await
//         .map_err(|e| e.to_string())?;

//     match vector_db.update_file(&file_path).await {
//         Ok(_) => {
//             info!("Successfully updated file '{}'", file_path);
//             Ok("File updated successfully".to_string())
//         }
//         Err(e) => {
//             error!("Failed to update file '{}': {}", file_path, e);
//             Err(format!("Failed to update file: {}", e))
//         }
//     }
// }

// /// Clear all embeddings from the vector database
// #[tauri::command]
// pub async fn clear_vector_db(app_state: State<'_, AppState>) -> Result<String, String> {
//     warn!("Clearing all embeddings from vector database");

//     let config = create_default_config().map_err(|e| e.to_string())?;
//     let vector_db = VectorDb::new(config.embed_config, config.qdrant_config)
//         .await
//         .map_err(|e| e.to_string())?;

//     match vector_db.delete_all_embeddings().await {
//         Ok(_) => {
//             info!("Successfully cleared all embeddings from vector database");
//             Ok("Vector database cleared successfully".to_string())
//         }
//         Err(e) => {
//             error!("Failed to clear vector database: {}", e);
//             Err(format!("Failed to clear database: {}", e))
//         }
//     }
// }

// /// Test vector database connection
// #[tauri::command]
// pub async fn test_vector_db_connection(app_state: State<'_, AppState>) -> Result<String, String> {
//     debug!("Testing vector database connection");

//     let config = create_default_config().map_err(|e| e.to_string())?;
//     let vector_db = VectorDb::new(config.embed_config, config.qdrant_config)
//         .await
//         .map_err(|e| e.to_string())?;

//     match vector_db.test_connection().await {
//         Ok(_) => {
//             info!("Vector database connection test successful");
//             Ok("Connection successful".to_string())
//         }
//         Err(e) => {
//             error!("Vector database connection test failed: {}", e);
//             Err(format!("Connection failed: {}", e))
//         }
//     }
// }

// /// Get embedding vector dimension
// #[tauri::command]
// pub async fn get_vector_dimension(app_state: State<'_, AppState>) -> Result<usize, String> {
//     debug!("Getting vector embedding dimension");

//     let config = create_default_config().map_err(|e| e.to_string())?;
//     let vector_db = VectorDb::new(config.embed_config, config.qdrant_config)
//         .await
//         .map_err(|e| e.to_string())?;

//     match vector_db.get_embedding_dimension().await {
//         Ok(dimension) => {
//             debug!("Vector embedding dimension: {}", dimension);
//             Ok(dimension)
//         }
//         Err(e) => {
//             error!("Failed to get vector dimension: {}", e);
//             Err(format!("Failed to get dimension: {}", e))
//         }
//     }
// }

// /// Batch add multiple files to vector database
// #[tauri::command]
// pub async fn batch_add_files_to_vector_db(
//     file_paths: Vec<String>,
//     app_state: State<'_, AppState>,
// ) -> Result<Vec<String>, String> {
//     info!("Batch adding {} files to vector database", file_paths.len());

//     let config = create_default_config().map_err(|e| e.to_string())?;
//     let vector_db = VectorDb::new(config.embed_config, config.qdrant_config)
//         .await
//         .map_err(|e| e.to_string())?;

//     let mut added_files = Vec::new();
//     let mut errors = Vec::new();

//     for file_path in file_paths {
//         match vector_db.add_file(&file_path).await {
//             Ok(point_id) => {
//                 added_files.push(format!("{}:{}", file_path, point_id));
//                 debug!("Added file: {} (ID: {})", file_path, point_id);
//             }
//             Err(e) => {
//                 errors.push(format!("{}:{}", file_path, e));
//                 warn!("Failed to add file {}: {}", file_path, e);
//             }
//         }
//     }

//     if !errors.is_empty() {
//         warn!("Batch add completed with {} errors", errors.len());
//         // You might want to return errors as well, but for now we'll just log them
//     }

//     info!(
//         "Successfully added {} files to vector database",
//         added_files.len()
//     );
//     Ok(added_files)
// }
