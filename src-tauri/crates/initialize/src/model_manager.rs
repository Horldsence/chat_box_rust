use crate::InitError;
use anyhow::{anyhow, Result};
use cb_config::config::InitConfig;
use hf_hub::api::tokio::Api;
use log::{info, warn};
use ollama_rs::Ollama;
use std::path::{Path, PathBuf};
use tauri::Manager;
use tokio::fs;

pub enum ModelType {
    Ollama,
    Candle,
}

impl From<&str> for ModelType {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "candle" => ModelType::Candle,
            _ => ModelType::Ollama,
        }
    }
}

pub struct ModelManager {
    config: InitConfig,
    app_handle: tauri::AppHandle,
}

impl ModelManager {
    pub fn new(config: InitConfig, app_handle: tauri::AppHandle) -> Self {
        Self { config, app_handle }
    }

    pub async fn initialize_model(&self) -> Result<(), InitError> {
        let model_type = ModelType::from(self.config.ai_model.model_type.as_str());

        match model_type {
            ModelType::Ollama => self.initialize_ollama_model().await,
            ModelType::Candle => self.initialize_candle_model().await,
        }
    }

    async fn initialize_ollama_model(&self) -> Result<(), InitError> {
        info!("初始化 Ollama 模型: {}", self.config.ai_model.model_name);

        let ollama_url = format!(
            "{}:{}",
            self.config.ai_model.server_url, self.config.ai_model.server_port
        );

        let ollama = Ollama::new(ollama_url.clone(), self.config.ai_model.server_port);

        // 检查 Ollama 服务是否可用
        match self.check_ollama_availability(&ollama).await {
            Ok(()) => {
                info!("Ollama 服务连接成功");
            }
            Err(e) => {
                return Err(InitError::NetworkError(format!(
                    "无法连接到 Ollama 服务 {}: {}",
                    ollama_url, e
                )));
            }
        }

        // 检查指定模型是否可用
        match self.check_model_availability(&ollama).await {
            Ok(available) => {
                if !available {
                    return Err(InitError::ModelUnavailable(format!(
                        "模型 '{}' 在 Ollama 服务中不可用。请先下载该模型。",
                        self.config.ai_model.model_name
                    )));
                }
                info!("模型 '{}' 验证成功", self.config.ai_model.model_name);
            }
            Err(e) => {
                return Err(InitError::ModelUnavailable(format!(
                    "检查模型可用性失败: {}",
                    e
                )));
            }
        }

        Ok(())
    }

    async fn check_ollama_availability(&self, ollama: &Ollama) -> Result<()> {
        // 尝试获取模型列表来验证连接
        match ollama.list_local_models().await {
            Ok(_) => Ok(()),
            Err(e) => Err(anyhow!("Ollama 服务不可用: {}", e)),
        }
    }

    async fn check_model_availability(&self, ollama: &Ollama) -> Result<bool> {
        let models = ollama.list_local_models().await?;

        let model_exists = models
            .iter()
            .any(|model| model.name == self.config.ai_model.model_name);

        if !model_exists {
            // 尝试拉取模型（可选）
            warn!(
                "模型 '{}' 不存在，建议手动拉取: ollama pull {}",
                self.config.ai_model.model_name, self.config.ai_model.model_name
            );
        }

        Ok(model_exists)
    }

    async fn initialize_candle_model(&self) -> Result<(), InitError> {
        info!("初始化 Candle 模型");

        let model_id = self
            .config
            .ai_model
            .candle_model_id
            .as_ref()
            .ok_or_else(|| {
                InitError::ConfigLoadFailed("Candle 模型配置中缺少 model_id".to_string())
            })?;

        let default_revision = "main".to_string();
        let revision = self
            .config
            .ai_model
            .candle_revision
            .as_ref()
            .unwrap_or(&default_revision);

        info!(
            "正在初始化 Candle 模型: {} (revision: {})",
            model_id, revision
        );

        // 检查和下载模型
        match self.ensure_candle_model_available(model_id, revision).await {
            Ok(model_path) => {
                info!("Candle 模型路径: {:?}", model_path);

                // 验证模型文件
                if let Err(e) = self.validate_candle_model(&model_path).await {
                    return Err(InitError::ModelUnavailable(format!(
                        "Candle 模型验证失败: {}",
                        e
                    )));
                }

                info!("Candle 模型初始化成功");
                Ok(())
            }
            Err(e) => Err(InitError::ModelUnavailable(format!(
                "Candle 模型下载/初始化失败: {}",
                e
            ))),
        }
    }

    async fn ensure_candle_model_available(
        &self,
        model_id: &str,
        revision: &str,
    ) -> Result<PathBuf> {
        // 获取模型存储路径
        let models_dir = self
            .app_handle
            .path()
            .resolve("models", tauri::path::BaseDirectory::AppData)
            .map_err(|e| anyhow!("无法解析模型目录路径: {}", e))?;

        let model_dir = models_dir.join(model_id.replace('/', "_"));

        // 检查模型是否已存在
        if model_dir.exists() && self.validate_model_directory(&model_dir).await? {
            info!("本地模型已存在: {:?}", model_dir);
            return Ok(model_dir);
        }

        // 创建模型目录
        fs::create_dir_all(&model_dir)
            .await
            .map_err(|e| anyhow!("创建模型目录失败: {}", e))?;

        // 下载模型
        info!("正在从 Hugging Face 下载模型: {}", model_id);
        self.download_candle_model(model_id, revision, &model_dir)
            .await?;

        Ok(model_dir)
    }

    async fn download_candle_model(
        &self,
        model_id: &str,
        _revision: &str,
        model_dir: &Path,
    ) -> Result<()> {
        let api = Api::new()?;
        let repo = api.model(model_id.to_string());

        // 常见的模型文件
        let model_files = vec![
            "config.json",
            "tokenizer.json",
            "tokenizer_config.json",
            "pytorch_model.bin",
            "model.safetensors",
            "vocab.txt",
        ];

        let mut downloaded_files = 0;

        for file_name in model_files {
            match repo.get(file_name).await {
                Ok(file_path) => {
                    let dest_path = model_dir.join(file_name);
                    if let Err(e) = fs::copy(&file_path, &dest_path).await {
                        warn!("复制文件 {} 失败: {}", file_name, e);
                    } else {
                        downloaded_files += 1;
                        info!("已下载: {}", file_name);
                    }
                }
                Err(e) => {
                    warn!("下载文件 {} 失败: {}", file_name, e);
                }
            }
        }

        if downloaded_files == 0 {
            return Err(anyhow!("没有成功下载任何模型文件"));
        }

        info!("成功下载 {} 个模型文件", downloaded_files);
        Ok(())
    }

    async fn validate_model_directory(&self, model_dir: &Path) -> Result<bool> {
        if !model_dir.exists() {
            return Ok(false);
        }

        // 检查必需的文件是否存在
        let required_files = vec!["config.json"];

        for file_name in required_files {
            let file_path = model_dir.join(file_name);
            if !file_path.exists() {
                return Ok(false);
            }
        }

        Ok(true)
    }

    async fn validate_candle_model(&self, model_path: &Path) -> Result<()> {
        // 验证配置文件
        let config_path = model_path.join("config.json");
        if !config_path.exists() {
            return Err(anyhow!("模型配置文件不存在: {:?}", config_path));
        }

        // 尝试读取配置文件
        let config_content = fs::read_to_string(&config_path)
            .await
            .map_err(|e| anyhow!("读取模型配置文件失败: {}", e))?;

        // 验证 JSON 格式
        let _: serde_json::Value = serde_json::from_str(&config_content)
            .map_err(|e| anyhow!("模型配置文件格式无效: {}", e))?;

        // 检查模型权重文件
        let model_files = vec!["pytorch_model.bin", "model.safetensors"];
        let mut has_model_file = false;

        for file_name in model_files {
            let file_path = model_path.join(file_name);
            if file_path.exists() {
                has_model_file = true;
                break;
            }
        }

        if !has_model_file {
            return Err(anyhow!("找不到模型权重文件"));
        }

        Ok(())
    }

    pub fn get_model_type(&self) -> ModelType {
        ModelType::from(self.config.ai_model.model_type.as_str())
    }

    pub fn get_model_info(&self) -> String {
        match self.get_model_type() {
            ModelType::Ollama => format!(
                "Ollama 模型: {} ({}:{})",
                self.config.ai_model.model_name,
                self.config.ai_model.server_url,
                self.config.ai_model.server_port
            ),
            ModelType::Candle => format!(
                "Candle 模型: {}",
                self.config
                    .ai_model
                    .candle_model_id
                    .as_ref()
                    .unwrap_or(&"未配置".to_string())
            ),
        }
    }
}
