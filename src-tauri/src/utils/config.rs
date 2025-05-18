use log::{error, info};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::State;

use crate::state::AppState;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub enabled: bool,
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AiModelConfig {
    pub model_name: String,
    pub server_url: String,
    pub server_port: u16,
    pub system_prompt: String,
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
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub config_path: PathBuf,
    pub ai_model: AiModelConfig,
    pub voice: VoiceConfig,
    pub ui: UiConfig,
    pub database: DatabaseConfig,
    pub app_behavior: AppBehaviorConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            config_path: "config.yaml".into(),
            ai_model: AiModelConfig {
                model_name: "qwen2.5:0.5b".to_string(),
                server_url: "http://localhost".to_string(),
                server_port: 11434,
                system_prompt: "你是一个友好、乐于助人的AI助手，使用中文回答问题。".to_string(),
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
            },
            database: DatabaseConfig {
                enabled: true,
                path: "database/chat_database.db".to_string(),
            },
        }
    }
}

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

// 导出配置更改API用于前端调用
#[tauri::command]
pub fn get_app_config(state: State<'_, AppState>) -> Result<AppConfig, String> {
    let binding = state.config.clone();
    let config = binding.lock().expect("获取配置失败");
    Ok(config.clone().load_config())
}

#[tauri::command]
pub fn save_app_config(state: State<'_, AppState>, save_config: AppConfig) -> Result<(), String> {
    let binding = state.config.clone();
    let config = binding.lock().expect("获取配置失败");
    match config.clone().get_config_file_path() {
        Some(path) => {
            config.save_config(&save_config, &path);
            Ok(())
        }
        None => Err("无法确定配置文件路径".to_string()),
    }
}
