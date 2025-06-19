use log::{error, info};
use serde::{Deserialize, Serialize};
use std::fs;
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
pub struct AppConfig {
    pub config_path: PathBuf,
    pub ai_model: AiModelConfig,
    pub voice: VoiceConfig,
    pub ui: UiConfig,
    pub database: DatabaseConfig,
    pub app_behavior: AppBehaviorConfig,
    pub live2d: Live2DConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            config_path: "config.yaml".into(),
            ai_model: AiModelConfig {
                model_type: "candle".to_string(),
                model_name: "microsoft/DialoGPT-medium".to_string(),
                server_url: "http://localhost".to_string(),
                server_port: 11434,
                system_prompt: "你是一个友好、乐于助人的AI助手，使用中文回答问题。".to_string(),
                candle_model_id: Some("microsoft/DialoGPT-medium".to_string()),
                candle_revision: Some("main".to_string()),
                candle_use_flash_attn: false,
            },
            voice: VoiceConfig {
                enabled: false,
                model_path: "model/vosk-model-small-cn-0.22".to_string(),
                timeout_seconds: 15,
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
                model_path: "models/live2d/hiyori/hiyori_free_en.model3.json".to_string(),
                model_name: "Hiyori".to_string(),
                scale: 1.0,
                position_x: 0.0,
                position_y: 0.0,
                auto_blink: true,
                auto_breath: true,
                check_model_on_startup: true,
                fallback_to_simple_character: true,
            },
        }
    }
}

#[allow(dead_code)]
impl AppConfig {
    pub fn new(config_path: PathBuf) -> Self {
        Self {
            config_path,
            ..Self::default()
        }
    }

    pub fn load_config(self) -> AppConfig {
        // 尝试从配置文件加载配置
        match self.clone().get_config_file_path() {
            Some(config_path) => {
                if config_path.exists() {
                    match fs::read_to_string(&config_path) {
                        Ok(yaml_str) => match serde_yaml::from_str(&yaml_str) {
                            Ok(config) => {
                                info!("配置已从 {:?} 加载", config_path);
                                return config;
                            }
                            Err(e) => {
                                error!("解析配置文件失败: {}", e);
                            }
                        },
                        Err(e) => {
                            error!("读取配置文件失败: {}", e);
                        }
                    }
                }

                // 文件不存在，创建默认配置文件
                let default_config = AppConfig::default();
                self.save_config(&default_config, &config_path);
                default_config
            }
            None => {
                error!("无法确定配置文件路径");
                AppConfig::default()
            }
        }
    }

    pub fn save_config(&self, config: &AppConfig, path: &PathBuf) {
        // 确保目录存在
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                if let Err(e) = fs::create_dir_all(parent) {
                    error!("创建配置目录失败: {}", e);
                    return;
                }
            }
        }

        // 写入配置文件
        match serde_yaml::to_string(config) {
            Ok(yaml_str) => match fs::write(path, yaml_str) {
                Ok(_) => {
                    info!("配置已保存到 {:?}", path);
                }
                Err(e) => {
                    error!("写入配置文件失败: {}", e);
                }
            },
            Err(e) => {
                error!("序列化配置失败: {}", e);
            }
        }
    }

    pub fn get_config_file_path(self) -> Option<PathBuf> {
        let config_path = AppConfig::default().config_path.clone();
        Some(config_path)
    }
}

// // 内部配置管理方法
// pub fn load_app_config(config_path: PathBuf) -> AppConfig {
//     let default_config = AppConfig::new(config_path.clone());
//     default_config.load_config()
// }
