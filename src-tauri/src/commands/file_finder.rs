use file_finder::{FileFinder, FileInfo, SearchOptions};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::State;
use tokio::time::Duration;

#[tauri::command]
pub async fn search_files(
    finder: State<'_, Arc<FileFinder>>,
    options: SearchOptions,
) -> Result<Vec<FileInfo>, String> {
    finder.search_files(options).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn search_files_with_content(
    finder: State<'_, Arc<FileFinder>>,
    options: SearchOptions,
) -> Result<Vec<FileInfo>, String> {
    finder
        .search_with_content(options)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn refresh_file_index(finder: State<'_, Arc<FileFinder>>) -> Result<(), String> {
    finder.refresh_index().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_file_info(
    finder: State<'_, Arc<FileFinder>>,
    path: String,
) -> Result<Option<FileInfo>, String> {
    let path = PathBuf::from(path);
    finder.get_file_info(&path).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_index_stats(
    finder: State<'_, Arc<FileFinder>>,
) -> Result<HashMap<String, serde_json::Value>, String> {
    let mut stats = HashMap::new();
    stats.insert("file_count".to_string(), finder.index.file_count().into());
    stats.insert(
        "needs_update".to_string(),
        finder.index.needs_update(Duration::from_secs(3600)).into(),
    );
    Ok(stats)
}
