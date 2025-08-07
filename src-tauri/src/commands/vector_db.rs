use crate::state::AppState;
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use tauri::{State, Window};

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResultResponse {
    pub id: String,
    pub score: f32,
    pub payload: HashMap<String, Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VectorDbStats {
    pub collection_info: Value,
    pub point_count: u64,
    pub mapped_files: usize,
    pub embedding_model: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddFileResponse {
    pub point_id: String,
    pub file_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncFilesResponse {
    pub added_files: Vec<String>,
    pub count: usize,
}

#[tauri::command]
pub async fn create_vector_db(state: State<'_, AppState>, name: String) -> Result<(), String> {
    #[cfg(feature = "vector_db")]
    {
        let mut db_guard = state.vector_db.lock().await;
        if db_guard.is_some() {
            return Err("Vector database already exists".to_string());
        }

        // Create new vector database with the given name as collection
        let embed_config = file_vec::create_default_config()
            .map_err(|e| format!("Failed to create embed config: {}", e))?;

        let mut qdrant_config = embed_config.qdrant_config;
        qdrant_config.collection_name = name.clone();

        match file_vec::VectorDb::new(embed_config.embed_config, qdrant_config).await {
            Ok(db) => {
                *db_guard = Some(db);
                info!("Created vector database with collection: {}", name.clone());
                Ok(())
            }
            Err(e) => Err(format!("Failed to create vector database: {}", e)),
        }
    }
    #[cfg(not(feature = "vector_db"))]
    {
        Err("Vector database feature not enabled".to_string())
    }
}

#[tauri::command]
pub async fn delete_vector_db(state: State<'_, AppState>) -> Result<(), String> {
    #[cfg(feature = "vector_db")]
    {
        let mut db_guard = state.vector_db.lock().await;
        if let Some(db) = db_guard.as_ref() {
            db.delete_all_embeddings()
                .await
                .map_err(|e| format!("Failed to clear vector database: {}", e))?;
        }
        *db_guard = None;
        info!("Deleted vector database");
        Ok(())
    }
    #[cfg(not(feature = "vector_db"))]
    {
        Err("Vector database feature not enabled".to_string())
    }
}

#[tauri::command]
pub async fn add_file_to_vector_db(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<AddFileResponse, String> {
    #[cfg(feature = "vector_db")]
    {
        let db_guard = state.vector_db.lock().await;
        if let Some(ref db) = *db_guard {
            match db.add_file(&file_path).await {
                Ok(point_id) => {
                    info!("Added file '{}' to vector database", file_path);
                    Ok(AddFileResponse {
                        point_id,
                        file_path,
                    })
                }
                Err(e) => Err(format!("Failed to add file to vector database: {}", e)),
            }
        } else {
            Err("Vector database not initialized".to_string())
        }
    }
    #[cfg(not(feature = "vector_db"))]
    {
        Err("Vector database feature not enabled".to_string())
    }
}

#[tauri::command]
pub async fn search_vector_db(
    state: State<'_, AppState>,
    query: String,
    top_k: Option<usize>,
) -> Result<Vec<SearchResultResponse>, String> {
    #[cfg(feature = "vector_db")]
    {
        let db_guard = state.vector_db.lock().await;
        if let Some(ref db) = *db_guard {
            let limit = top_k.unwrap_or(10);
            match db.search(&query, limit).await {
                Ok(results) => {
                    let response: Vec<SearchResultResponse> = results
                        .into_iter()
                        .map(|r| SearchResultResponse {
                            id: r.id.to_string(),
                            score: r.score,
                            payload: r.payload,
                        })
                        .collect();

                    debug!(
                        "Search query '{}' returned {} results",
                        query,
                        response.len()
                    );
                    Ok(response)
                }
                Err(e) => Err(format!("Failed to search vector database: {}", e)),
            }
        } else {
            Err("Vector database not initialized".to_string())
        }
    }
    #[cfg(not(feature = "vector_db"))]
    {
        Err("Vector database feature not enabled".to_string())
    }
}

#[tauri::command]
pub async fn search_vector_db_with_filter(
    state: State<'_, AppState>,
    query: String,
    top_k: Option<usize>,
    filter: HashMap<String, Value>,
) -> Result<Vec<SearchResultResponse>, String> {
    #[cfg(feature = "vector_db")]
    {
        let db_guard = state.vector_db.lock().await;
        if let Some(ref db) = *db_guard {
            let limit = top_k.unwrap_or(10);
            match db.search_with_filter(&query, limit, filter).await {
                Ok(results) => {
                    let response: Vec<SearchResultResponse> = results
                        .into_iter()
                        .map(|r| SearchResultResponse {
                            id: r.id.to_string(),
                            score: r.score,
                            payload: r.payload,
                        })
                        .collect();

                    debug!(
                        "Filtered search query '{}' returned {} results",
                        query,
                        response.len()
                    );
                    Ok(response)
                }
                Err(e) => Err(format!(
                    "Failed to search vector database with filter: {}",
                    e
                )),
            }
        } else {
            Err("Vector database not initialized".to_string())
        }
    }
    #[cfg(not(feature = "vector_db"))]
    {
        Err("Vector database feature not enabled".to_string())
    }
}

#[tauri::command]
pub async fn delete_file_from_vector_db(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<(), String> {
    #[cfg(feature = "vector_db")]
    {
        let db_guard = state.vector_db.lock().await;
        if let Some(ref db) = *db_guard {
            match db.delete_file(&file_path).await {
                Ok(_) => {
                    info!("Deleted file '{}' from vector database", file_path);
                    Ok(())
                }
                Err(e) => Err(format!("Failed to delete file from vector database: {}", e)),
            }
        } else {
            Err("Vector database not initialized".to_string())
        }
    }
    #[cfg(not(feature = "vector_db"))]
    {
        Err("Vector database feature not enabled".to_string())
    }
}

#[tauri::command]
pub async fn delete_point_from_vector_db(
    state: State<'_, AppState>,
    point_id: String,
) -> Result<(), String> {
    #[cfg(feature = "vector_db")]
    {
        let db_guard = state.vector_db.lock().await;
        if let Some(ref db) = *db_guard {
            let uuid = uuid::Uuid::parse_str(&point_id)
                .map_err(|e| format!("Invalid UUID format: {}", e))?;

            match db.delete_point(&uuid).await {
                Ok(_) => {
                    info!("Deleted point '{}' from vector database", point_id);
                    Ok(())
                }
                Err(e) => Err(format!(
                    "Failed to delete point from vector database: {}",
                    e
                )),
            }
        } else {
            Err("Vector database not initialized".to_string())
        }
    }
    #[cfg(not(feature = "vector_db"))]
    {
        Err("Vector database feature not enabled".to_string())
    }
}

#[tauri::command]
pub async fn delete_all_embeddings(state: State<'_, AppState>) -> Result<(), String> {
    #[cfg(feature = "vector_db")]
    {
        let db_guard = state.vector_db.lock().await;
        if let Some(ref db) = *db_guard {
            match db.delete_all_embeddings().await {
                Ok(_) => {
                    info!("Deleted all embeddings from vector database");
                    Ok(())
                }
                Err(e) => Err(format!("Failed to delete all embeddings: {}", e)),
            }
        } else {
            Err("Vector database not initialized".to_string())
        }
    }
    #[cfg(not(feature = "vector_db"))]
    {
        Err("Vector database feature not enabled".to_string())
    }
}

#[tauri::command]
pub async fn get_vector_db_stats(state: State<'_, AppState>) -> Result<VectorDbStats, String> {
    #[cfg(feature = "vector_db")]
    {
        let db_guard = state.vector_db.lock().await;
        if let Some(ref db) = *db_guard {
            match db.get_stats().await {
                Ok(stats_value) => {
                    let stats = VectorDbStats {
                        collection_info: stats_value["collection_info"].clone(),
                        point_count: stats_value["point_count"].as_u64().unwrap_or(0),
                        mapped_files: stats_value["mapped_files"].as_u64().unwrap_or(0) as usize,
                        embedding_model: stats_value["embedding_model"]
                            .as_str()
                            .unwrap_or("unknown")
                            .to_string(),
                    };
                    Ok(stats)
                }
                Err(e) => Err(format!("Failed to get vector database stats: {}", e)),
            }
        } else {
            Err("Vector database not initialized".to_string())
        }
    }
    #[cfg(not(feature = "vector_db"))]
    {
        Err("Vector database feature not enabled".to_string())
    }
}

#[tauri::command]
pub async fn sync_files_to_vector_db(
    state: State<'_, AppState>,
    directory_path: String,
) -> Result<SyncFilesResponse, String> {
    #[cfg(feature = "vector_db")]
    {
        let db_guard = state.vector_db.lock().await;
        if let Some(ref db) = *db_guard {
            match db.sync_files(&directory_path).await {
                Ok(added_files) => {
                    let count = added_files.len();
                    info!(
                        "Synced {} files to vector database from '{}'",
                        count, directory_path
                    );
                    Ok(SyncFilesResponse { added_files, count })
                }
                Err(e) => Err(format!("Failed to sync files to vector database: {}", e)),
            }
        } else {
            Err("Vector database not initialized".to_string())
        }
    }
    #[cfg(not(feature = "vector_db"))]
    {
        Err("Vector database feature not enabled".to_string())
    }
}

#[tauri::command]
pub async fn update_file_in_vector_db(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<(), String> {
    #[cfg(feature = "vector_db")]
    {
        let db_guard = state.vector_db.lock().await;
        if let Some(ref db) = *db_guard {
            match db.update_file(&file_path).await {
                Ok(_) => {
                    info!("Updated file '{}' in vector database", file_path);
                    Ok(())
                }
                Err(e) => Err(format!("Failed to update file in vector database: {}", e)),
            }
        } else {
            Err("Vector database not initialized".to_string())
        }
    }
    #[cfg(not(feature = "vector_db"))]
    {
        Err("Vector database feature not enabled".to_string())
    }
}

#[tauri::command]
pub async fn get_indexed_files(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    #[cfg(feature = "vector_db")]
    {
        let db_guard = state.vector_db.lock().await;
        if let Some(ref db) = *db_guard {
            let files = db.get_indexed_files().await;
            Ok(files)
        } else {
            Err("Vector database not initialized".to_string())
        }
    }
    #[cfg(not(feature = "vector_db"))]
    {
        Err("Vector database feature not enabled".to_string())
    }
}

#[tauri::command]
pub async fn test_vector_db_connection(state: State<'_, AppState>) -> Result<(), String> {
    #[cfg(feature = "vector_db")]
    {
        let db_guard = state.vector_db.lock().await;
        if let Some(ref db) = *db_guard {
            match db.test_connection().await {
                Ok(_) => {
                    info!("Vector database connection test successful");
                    Ok(())
                }
                Err(e) => Err(format!("Vector database connection test failed: {}", e)),
            }
        } else {
            Err("Vector database not initialized".to_string())
        }
    }
    #[cfg(not(feature = "vector_db"))]
    {
        Err("Vector database feature not enabled".to_string())
    }
}

#[tauri::command]
pub async fn get_embedding_dimension(state: State<'_, AppState>) -> Result<usize, String> {
    #[cfg(feature = "vector_db")]
    {
        let db_guard = state.vector_db.lock().await;
        if let Some(ref db) = *db_guard {
            match db.get_embedding_dimension().await {
                Ok(dimension) => Ok(dimension),
                Err(e) => Err(format!("Failed to get embedding dimension: {}", e)),
            }
        } else {
            Err("Vector database not initialized".to_string())
        }
    }
    #[cfg(not(feature = "vector_db"))]
    {
        Err("Vector database feature not enabled".to_string())
    }
}
