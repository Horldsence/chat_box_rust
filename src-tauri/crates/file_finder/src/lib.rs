use anyhow::{Context, Result};
use ignore::WalkBuilder;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::fs;

/// File information structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub path: PathBuf,
    pub name: String,
    pub size: u64,
    pub modified: u64,
    pub is_dir: bool,
    pub extension: Option<String>,
}

/// Search options for file queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchOptions {
    pub pattern: String,
    pub use_regex: bool,
    pub include_hidden: bool,
    pub max_depth: Option<usize>,
    pub file_types: Option<Vec<String>>,
    pub max_results: Option<usize>,
    pub search_content: bool,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            pattern: String::new(),
            use_regex: false,
            include_hidden: false,
            max_depth: None,
            file_types: None,
            max_results: Some(1000),
            search_content: false,
        }
    }
}

/// File index for fast searching
#[derive(Debug, Clone)]
pub struct FileIndex {
    files: Arc<Mutex<HashMap<PathBuf, FileInfo>>>,
    last_updated: Arc<Mutex<SystemTime>>,
}

impl FileIndex {
    pub fn new() -> Self {
        Self {
            files: Arc::new(Mutex::new(HashMap::new())),
            last_updated: Arc::new(Mutex::new(UNIX_EPOCH)),
        }
    }

    /// Build index for the given directory
    pub async fn build_index(&self, root_path: &Path) -> Result<()> {
        let mut files = HashMap::new();

        let walker = WalkBuilder::new(root_path)
            .hidden(false)
            .git_ignore(true)
            .build();

        for entry in walker {
            match entry {
                Ok(entry) => {
                    let path = entry.path();
                    if let Ok(metadata) = entry.metadata() {
                        let file_info = FileInfo {
                            path: path.to_path_buf(),
                            name: path
                                .file_name()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .to_string(),
                            size: metadata.len(),
                            modified: metadata
                                .modified()
                                .unwrap_or(UNIX_EPOCH)
                                .duration_since(UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs(),
                            is_dir: metadata.is_dir(),
                            extension: path
                                .extension()
                                .map(|ext| ext.to_string_lossy().to_string()),
                        };
                        files.insert(path.to_path_buf(), file_info);
                    }
                }
                Err(e) => {
                    log::warn!("Failed to read entry: {}", e);
                }
            }
        }

        *self.files.lock().unwrap() = files;
        *self.last_updated.lock().unwrap() = SystemTime::now();

        Ok(())
    }

    /// Search files in the index
    pub fn search(&self, options: &SearchOptions) -> Result<Vec<FileInfo>> {
        let files = self.files.lock().unwrap();
        let mut results = Vec::new();

        let regex = if options.use_regex {
            Some(Regex::new(&options.pattern).context("Invalid regex pattern")?)
        } else {
            None
        };

        for file_info in files.values() {
            // Skip hidden files if not included
            if !options.include_hidden && file_info.name.starts_with('.') {
                continue;
            }

            // Filter by file types
            if let Some(ref file_types) = options.file_types {
                if let Some(ref ext) = file_info.extension {
                    if !file_types.contains(ext) {
                        continue;
                    }
                } else if !file_types.is_empty() {
                    continue;
                }
            }

            // Match pattern
            let matches = if let Some(ref regex) = regex {
                regex.is_match(&file_info.name)
            } else {
                file_info
                    .name
                    .to_lowercase()
                    .contains(&options.pattern.to_lowercase())
            };

            if matches {
                results.push(file_info.clone());

                // Check max results limit
                if let Some(max) = options.max_results {
                    if results.len() >= max {
                        break;
                    }
                }
            }
        }

        // Sort by name
        results.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(results)
    }

    /// Get file count in index
    pub fn file_count(&self) -> usize {
        self.files.lock().unwrap().len()
    }

    /// Check if index needs update
    pub fn needs_update(&self, max_age: Duration) -> bool {
        let last_updated = *self.last_updated.lock().unwrap();
        SystemTime::now()
            .duration_since(last_updated)
            .unwrap_or_default()
            > max_age
    }
}

/// File finder service
#[derive(Debug)]
pub struct FileFinder {
    pub index: FileIndex,
    home_dir: PathBuf,
}

impl FileFinder {
    pub fn new() -> Result<Self> {
        let home_dir = dirs::home_dir().context("Failed to get home directory")?;

        Ok(Self {
            index: FileIndex::new(),
            home_dir,
        })
    }

    /// Initialize the file finder with home directory indexing
    pub async fn initialize(&self) -> Result<()> {
        log::info!(
            "Initializing file finder for home directory: {:?}",
            self.home_dir
        );
        self.index
            .build_index(&self.home_dir)
            .await
            .context("Failed to build file index")?;
        log::info!(
            "File index built successfully with {} files",
            self.index.file_count()
        );
        Ok(())
    }

    /// Refresh the file index
    pub async fn refresh_index(&self) -> Result<()> {
        self.index.build_index(&self.home_dir).await
    }

    /// Search files using the index
    pub fn search_files(&self, options: SearchOptions) -> Result<Vec<FileInfo>> {
        self.index.search(&options)
    }

    /// Get the home directory path
    pub fn home_dir(&self) -> &Path {
        &self.home_dir
    }

    /// Get file count from index
    pub fn file_count(&self) -> usize {
        self.index.file_count()
    }

    /// Check if index needs update
    pub fn needs_update(&self, max_age: Duration) -> bool {
        self.index.needs_update(max_age)
    }

    /// Search files with content matching (slower but more thorough)
    pub async fn search_with_content(&self, options: SearchOptions) -> Result<Vec<FileInfo>> {
        if !options.search_content {
            return self.search_files(options);
        }

        let regex = if options.use_regex {
            Some(Regex::new(&options.pattern).context("Invalid regex pattern")?)
        } else {
            None
        };

        let mut results = Vec::new();
        let home_dir = self.home_dir.clone();

        let walker = WalkBuilder::new(&home_dir)
            .hidden(options.include_hidden)
            .git_ignore(true)
            .build();

        for entry in walker {
            match entry {
                Ok(entry) => {
                    let path = entry.path();

                    if let Ok(metadata) = entry.metadata() {
                        if metadata.is_file() {
                            // Check file name first
                            let file_name = path.file_name().unwrap_or_default().to_string_lossy();

                            let name_matches = if let Some(ref regex) = regex {
                                regex.is_match(&file_name)
                            } else {
                                file_name
                                    .to_lowercase()
                                    .contains(&options.pattern.to_lowercase())
                            };

                            let mut content_matches = false;

                            // Search content if requested and name doesn't match
                            if !name_matches && options.search_content {
                                if let Ok(content) = fs::read_to_string(path).await {
                                    content_matches = if let Some(ref regex) = regex {
                                        regex.is_match(&content)
                                    } else {
                                        content
                                            .to_lowercase()
                                            .contains(&options.pattern.to_lowercase())
                                    };
                                }
                            }

                            if name_matches || content_matches {
                                let file_info = FileInfo {
                                    path: path.to_path_buf(),
                                    name: file_name.to_string(),
                                    size: metadata.len(),
                                    modified: metadata
                                        .modified()
                                        .unwrap_or(UNIX_EPOCH)
                                        .duration_since(UNIX_EPOCH)
                                        .unwrap_or_default()
                                        .as_secs(),
                                    is_dir: false,
                                    extension: path
                                        .extension()
                                        .map(|ext| ext.to_string_lossy().to_string()),
                                };

                                results.push(file_info);

                                if let Some(max) = options.max_results {
                                    if results.len() >= max {
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    log::warn!("Failed to read entry: {}", e);
                }
            }
        }

        results.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(results)
    }

    /// Get file info for a specific path
    pub async fn get_file_info(&self, path: &Path) -> Result<Option<FileInfo>> {
        if !path.exists() {
            return Ok(None);
        }

        let metadata = fs::metadata(path)
            .await
            .context("Failed to read file metadata")?;

        let file_info = FileInfo {
            path: path.to_path_buf(),
            name: path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            size: metadata.len(),
            modified: metadata
                .modified()
                .unwrap_or(UNIX_EPOCH)
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            is_dir: metadata.is_dir(),
            extension: path
                .extension()
                .map(|ext| ext.to_string_lossy().to_string()),
        };

        Ok(Some(file_info))
    }
}

// /// Tauri commands for the file finder

// /// Initialize the file finder plugin
// pub fn init<R: Runtime>() -> tauri::plugin::TauriPlugin<R> {
//     tauri::plugin::Builder::new("file_finder")
//         .invoke_handler(tauri::generate_handler![
//             search_files,
//             search_files_with_content,
//             refresh_file_index,
//             get_file_info,
//             get_index_stats
//         ])
//         .setup(|app, _api| {
//             let finder = Arc::new(
//                 FileFinder::new().map_err(|e| format!("Failed to create file finder: {}", e))?,
//             );

//             // Initialize the file finder asynchronously
//             let finder_clone = finder.clone();
//             tauri::async_runtime::spawn(async move {
//                 if let Err(e) = finder_clone.initialize().await {
//                     log::error!("Failed to initialize file finder: {}", e);
//                 }
//             });

//             app.manage(finder);
//             Ok(())
//         })
//         .build()
// }

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_file_index() {
        let temp_dir = TempDir::new().unwrap();
        let index = FileIndex::new();

        // Create some test files
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "test content").await.unwrap();

        // Build index
        index.build_index(temp_dir.path()).await.unwrap();

        // Search
        let options = SearchOptions {
            pattern: "test".to_string(),
            ..Default::default()
        };

        let results = index.search(&options).unwrap();
        assert!(!results.is_empty());
    }

    #[tokio::test]
    async fn test_regex_search() {
        let temp_dir = TempDir::new().unwrap();
        let index = FileIndex::new();

        // Create test files
        fs::write(temp_dir.path().join("file1.txt"), "content")
            .await
            .unwrap();
        fs::write(temp_dir.path().join("image.png"), "image")
            .await
            .unwrap();

        // Build index
        index.build_index(temp_dir.path()).await.unwrap();

        // Regex search for files ending with .txt
        let options = SearchOptions {
            pattern: r"\.txt$".to_string(),
            use_regex: true,
            ..Default::default()
        };

        let results = index.search(&options).unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].name.ends_with(".txt"));
    }
}
