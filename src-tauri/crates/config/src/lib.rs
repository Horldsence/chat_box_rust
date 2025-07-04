use log::{error, info};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

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
