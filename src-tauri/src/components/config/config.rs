use serde::{Deserialize, Serialize};
use std::fs;
use log::debug;
use anyhow::{Result, Context};
use config::{Config, Environment};

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct AppConfig {
    pub font: String,
    pub log_level: String,
    pub ollama_setting: Vec<OllamaSetting>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct OllamaSetting {
    pub port: u16,
    pub model: String,
    pub host: String,
    pub prompt: String,
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        debug!("加载配置文件");
        let config_path = "config.toml";
        let config = Config::builder()
            .add_source(config::File::with_name(config_path))
            .add_source(Environment::with_prefix("api-gate"))
            .build()
            .context("构建配置错误")?;
        let app_config: AppConfig = config.try_deserialize()
            .context("反序列化配置文件错误")?;

        Ok(app_config)
    }

    pub fn save(&self) -> Result<()> {
        debug!("保存配置文件");
        let config_path = "config.toml";
        let toml_string = toml::to_string_pretty(self)
            .context("序列化配置失败")?;

        fs::write(config_path, toml_string)
            .context("写入配置文件失败")?;

        Ok(())
    }

    pub fn update_from_ui(&mut self,
        log_level: String,
        ollama_url: String,
        ollama_host: String,
        ollama_model: String,
    ) {
        debug!("更新配置文件");
        self.log_level = log_level;
        if let Some(ollama_setting) = self.ollama_setting.get_mut(0) {
            ollama_setting.host = ollama_url;
            ollama_setting.port = ollama_host.parse().unwrap_or(11434);
            ollama_setting.model = ollama_model;
        }
    }
}