pub mod error_handler;
pub mod initializer;
pub mod model_manager;

pub use cb_config::*;
pub use error_handler::*;
pub use initializer::*;
pub use model_manager::*;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InitError {
    ModelUnavailable(String),
    ConfigLoadFailed(String),
    DatabaseInitFailed(String),
    VoiceInitFailed(String),
    NetworkError(String),
    FileSystemError(String),
}

impl std::fmt::Display for InitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InitError::ModelUnavailable(msg) => write!(f, "模型不可用: {}", msg),
            InitError::ConfigLoadFailed(msg) => write!(f, "配置加载失败: {}", msg),
            InitError::DatabaseInitFailed(msg) => write!(f, "数据库初始化失败: {}", msg),
            InitError::VoiceInitFailed(msg) => write!(f, "语音初始化失败: {}", msg),
            InitError::NetworkError(msg) => write!(f, "网络错误: {}", msg),
            InitError::FileSystemError(msg) => write!(f, "文件系统错误: {}", msg),
        }
    }
}

impl std::error::Error for InitError {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserAction {
    Ignore,
    Retry,
    Exit,
}

#[derive(Debug, Clone)]
pub struct InitializationResult {
    pub success: bool,
    pub failed_components: Vec<String>,
    pub ignored_components: Vec<String>,
}

// 重新导出主要初始化函数
pub use initializer::initialize_app;

#[cfg(test)]
mod tests;
