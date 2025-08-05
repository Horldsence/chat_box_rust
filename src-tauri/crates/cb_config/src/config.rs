use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub enabled: bool,
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AiModelConfig {
    pub model_type: String, // "ollama" or "candle"
    pub model_name: String,
    pub server_url: String,
    pub server_port: u16,
    pub system_prompt: String,
    // Candle specific settings
    pub candle_model_id: Option<String>,
    pub candle_revision: Option<String>,
    pub candle_use_flash_attn: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VoiceConfig {
    pub enabled: bool,
    pub model_path: String,
    pub timeout_seconds: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TtsConfig {
    pub enabled: bool,
    pub model_path: PathBuf,
    pub voice_path: PathBuf,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UiConfig {
    pub theme: String,
    pub language: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppBehaviorConfig {
    pub log_level: String,
    pub default_conversation_title: String,
    pub welcome_message: String,
    pub message_chunk_buffer_size: usize,
    pub message_chunk_send_interval_ms: u64,
    pub show_error_dialogs: bool,
    pub auto_retry_failed_init: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InitConfig {
    pub config_path: PathBuf,
    pub ai_model: AiModelConfig,
    pub voice: VoiceConfig,
    pub tts_engine: TtsConfig,
    pub ui: UiConfig,
    pub database: DatabaseConfig,
    pub app_behavior: AppBehaviorConfig,
    pub live2d: Live2DConfig,
    pub qdrant: QdrantConfig,
    pub embed: EmbedConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Live2DConfig {
    pub enabled: bool,
    pub model_path: String,
    pub model_name: String,
    pub scale: f32,
    pub position_x: f32,
    pub position_y: f32,
    pub auto_blink: bool,
    pub auto_breath: bool,
    pub check_model_on_startup: bool,
    pub fallback_to_simple_character: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct QdrantConfig {
    pub enabled: bool,
    pub server_url: String,
    pub server_port: u16,
    pub collection_name: String,
    pub vector_size: u64,
    pub distance_metric: String,
    pub timeout_seconds: u64,
    pub use_grpc: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbedConfig {
    /// 模型名称，默认使用 BAAI/bge-small-en-v1.5
    pub model_name: String,
    /// 最大序列长度，默认 512
    pub max_length: usize,
    /// 批处理大小，默认 32
    pub batch_size: usize,
    /// 是否显示下载进度，默认 true
    pub show_download_progress: bool,
    /// 缓存目录，默认使用系统缓存目录
    pub cache_dir: Option<String>,
}

impl Default for InitConfig {
    fn default() -> Self {
        Self {
            config_path: "config.yaml".into(),
            ai_model: AiModelConfig {
                model_type: "candle".to_string(),
                model_name: "qwen2.5:0.5b".to_string(),
                server_url: "http://localhost".to_string(),
                server_port: 11434,
                system_prompt: "你是一个友好、乐于助人的AI助手，使用中文回答问题。".to_string(),
                candle_model_id: Some("Qwen/Qwen2.5-0.5B".to_string()),
                candle_revision: Some("main".to_string()),
                candle_use_flash_attn: false,
            },
            voice: VoiceConfig {
                enabled: false,
                model_path: "model/vosk-model-small-cn-0.22".to_string(),
                timeout_seconds: 15,
            },
            tts_engine: TtsConfig {
                enabled: true,
                model_path: "model/kokoro/kokoro-v1.1-zh.onnx".into(),
                voice_path: "model/kokoro/voices-v1.1-zh.bin".into(),
            },
            ui: UiConfig {
                theme: "light".to_string(),
                language: "zh-CN".to_string(),
            },
            app_behavior: AppBehaviorConfig {
                log_level: "info".to_string(),
                default_conversation_title: "新对话".to_string(),
                welcome_message: "欢迎使用聊天应用!".to_string(),
                message_chunk_buffer_size: 2,
                message_chunk_send_interval_ms: 3,
                show_error_dialogs: true,
                auto_retry_failed_init: false,
            },
            database: DatabaseConfig {
                enabled: true,
                path: "database/chat_database.db".to_string(),
            },
            live2d: Live2DConfig {
                enabled: true,
                model_path: "UG/ugofficial.model3.json".to_string(),
                model_name: "Hiyori".to_string(),
                scale: 1.0,
                position_x: 0.0,
                position_y: 0.0,
                auto_blink: true,
                auto_breath: true,
                check_model_on_startup: true,
                fallback_to_simple_character: true,
            },
            qdrant: QdrantConfig {
                enabled: false,
                server_url: "http://localhost".to_string(),
                server_port: 6334,
                collection_name: "chat_vectors".to_string(),
                vector_size: 768,
                distance_metric: "Cosine".to_string(),
                timeout_seconds: 30,
                use_grpc: true,
            },
            embed: EmbedConfig {
                model_name: "BAAI/bge-small-en-v1.5".to_string(),
                max_length: 512,
                batch_size: 32,
                show_download_progress: true,
                cache_dir: None,
            },
        }
    }
}

impl InitConfig {
    pub fn new(config_path: PathBuf) -> Self {
        Self {
            config_path,
            ..Self::default()
        }
    }

    pub fn load_config(self) -> InitConfig {
        // 尝试从配置文件加载配置
        match self.clone().get_config_file_path() {
            Some(config_path) => {
                if config_path.exists() {
                    match std::fs::read_to_string(&config_path) {
                        Ok(yaml_str) => match serde_yaml::from_str(&yaml_str) {
                            Ok(config) => {
                                log::info!("配置已从 {:?} 加载", config_path);
                                return config;
                            }
                            Err(e) => {
                                log::error!("解析配置文件失败: {}", e);
                            }
                        },
                        Err(e) => {
                            log::error!("读取配置文件失败: {}", e);
                        }
                    }
                }

                // 文件不存在，创建默认配置文件
                let default_config = InitConfig::default();
                self.save_config(&default_config, &config_path);
                default_config
            }
            None => {
                log::error!("无法确定配置文件路径");
                InitConfig::default()
            }
        }
    }

    pub fn save_config(&self, config: &InitConfig, path: &PathBuf) {
        // 确保目录存在
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                if let Err(e) = std::fs::create_dir_all(parent) {
                    log::error!("创建配置目录失败: {}", e);
                    return;
                }
            }
        }

        // 写入配置文件
        match serde_yaml::to_string(config) {
            Ok(yaml_str) => match std::fs::write(path, yaml_str) {
                Ok(_) => {
                    log::info!("配置已保存到 {:?}", path);
                }
                Err(e) => {
                    log::error!("写入配置文件失败: {}", e);
                }
            },
            Err(e) => {
                log::error!("序列化配置失败: {}", e);
            }
        }
    }

    pub fn get_config_file_path(self) -> Option<PathBuf> {
        let config_path = self.config_path.clone();
        Some(config_path)
    }
}
