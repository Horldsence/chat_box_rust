use super::{
    ollama::provider::{OllamaConfig, OllamaProvider},
    ChatRequest, ChatResponse, LLMError, LLMProvider, ModelInfo,
};

#[cfg(feature = "candle")]
use super::candle::{CandleConfig, CandleProvider};
use async_trait::async_trait;
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio_stream::Stream;

/// LLM 管理器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMManagerConfig {
    pub default_provider: String,
    pub fallback_providers: Vec<String>,
    pub auto_fallback: bool,
    pub health_check_interval_seconds: u64,
    pub providers: HashMap<String, ProviderConfig>,
}

/// 提供者配置枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ProviderConfig {
    Ollama(OllamaConfig),
    #[cfg(feature = "candle")]
    Candle(CandleConfig),
}

impl Default for LLMManagerConfig {
    fn default() -> Self {
        let mut providers = HashMap::new();

        // 默认 Ollama 配置
        providers.insert(
            "ollama".to_string(),
            ProviderConfig::Ollama(OllamaConfig::default()),
        );

        // 默认 Candle 配置
        #[cfg(feature = "candle")]
        providers.insert(
            "candle".to_string(),
            ProviderConfig::Candle(CandleConfig::default()),
        );

        Self {
            default_provider: "ollama".to_string(),
            #[cfg(feature = "candle")]
            fallback_providers: vec!["candle".to_string()],
            #[cfg(not(feature = "candle"))]
            fallback_providers: vec![],
            auto_fallback: true,
            health_check_interval_seconds: 300, // 5分钟
            providers,
        }
    }
}

/// LLM 提供者的健康状态
#[derive(Debug, Clone, PartialEq)]
pub enum ProviderHealth {
    Healthy,
    Unhealthy,
    Unknown,
}

/// 提供者状态信息
#[derive(Debug, Clone)]
pub struct ProviderStatus {
    pub name: String,
    pub health: ProviderHealth,
    pub last_check: Option<std::time::Instant>,
    pub error_count: usize,
    pub available_models: Vec<ModelInfo>,
}

impl ProviderStatus {
    pub fn new(name: String) -> Self {
        Self {
            name,
            health: ProviderHealth::Unknown,
            last_check: None,
            error_count: 0,
            available_models: Vec::new(),
        }
    }
}

/// 统一的 LLM 管理器
pub struct LLMManager {
    config: LLMManagerConfig,
    providers: HashMap<String, Arc<dyn LLMProvider>>,
    provider_status: Arc<tokio::sync::RwLock<HashMap<String, ProviderStatus>>>,
    health_check_task: Option<tokio::task::JoinHandle<()>>,
}

impl LLMManager {
    /// 创建新的 LLM 管理器
    pub async fn new(config: LLMManagerConfig) -> Result<Self, LLMError> {
        let mut manager = Self {
            config,
            providers: HashMap::new(),
            provider_status: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            health_check_task: None,
        };

        manager.initialize_providers().await?;
        manager.start_health_monitoring();

        Ok(manager)
    }

    /// 使用默认配置创建管理器
    pub async fn with_default() -> Result<Self, LLMError> {
        Self::new(LLMManagerConfig::default()).await
    }

    /// 初始化所有配置的提供者
    async fn initialize_providers(&mut self) -> Result<(), LLMError> {
        info!("初始化 LLM 提供者...");

        for (name, provider_config) in &self.config.providers {
            match self.create_provider(provider_config).await {
                Ok(provider) => {
                    info!("成功初始化提供者: {}", name);
                    self.providers.insert(name.clone(), provider);

                    let mut status_map = self.provider_status.write().await;
                    status_map.insert(name.clone(), ProviderStatus::new(name.clone()));
                }
                Err(e) => {
                    error!("初始化提供者 '{}' 失败: {}", name, e);
                    // 不阻止其他提供者的初始化
                }
            }
        }

        if self.providers.is_empty() {
            return Err(LLMError::ConfigError("没有可用的 LLM 提供者".to_string()));
        }

        info!("初始化了 {} 个 LLM 提供者", self.providers.len());
        Ok(())
    }

    /// 根据配置创建提供者实例
    async fn create_provider(
        &self,
        config: &ProviderConfig,
    ) -> Result<Arc<dyn LLMProvider>, LLMError> {
        match config {
            ProviderConfig::Ollama(ollama_config) => {
                let provider = OllamaProvider::new(ollama_config.clone());
                Ok(Arc::new(provider))
            }
            #[cfg(feature = "candle")]
            ProviderConfig::Candle(candle_config) => {
                let provider = CandleProvider::new(candle_config.clone())?;
                Ok(Arc::new(provider))
            }
        }
    }

    /// 启动健康监控
    fn start_health_monitoring(&mut self) {
        if self.config.health_check_interval_seconds > 0 {
            let providers = self.providers.clone();
            let status_rwlock = Arc::clone(&self.provider_status);
            let interval =
                std::time::Duration::from_secs(self.config.health_check_interval_seconds);

            let task = tokio::spawn(async move {
                let mut interval_timer = tokio::time::interval(interval);
                loop {
                    interval_timer.tick().await;
                    Self::perform_health_checks(&providers, &status_rwlock).await;
                }
            });

            self.health_check_task = Some(task);
            info!(
                "启动健康监控，间隔: {}秒",
                self.config.health_check_interval_seconds
            );
        }
    }

    /// 执行健康检查
    async fn perform_health_checks(
        providers: &HashMap<String, Arc<dyn LLMProvider>>,
        status_rwlock: &Arc<tokio::sync::RwLock<HashMap<String, ProviderStatus>>>,
    ) {
        debug!("执行定期健康检查...");

        for (name, provider) in providers {
            let health = match provider.health_check().await {
                Ok(true) => ProviderHealth::Healthy,
                Ok(false) => ProviderHealth::Unhealthy,
                Err(_) => ProviderHealth::Unhealthy,
            };

            let models = provider.list_models().await.unwrap_or_default();

            {
                let mut status_map = status_rwlock.write().await;
                if let Some(status) = status_map.get_mut(name) {
                    status.health = health.clone();
                    status.last_check = Some(std::time::Instant::now());
                    status.available_models = models;

                    if health == ProviderHealth::Unhealthy {
                        status.error_count += 1;
                    } else {
                        status.error_count = 0; // 重置错误计数
                    }
                }
            }

            debug!("提供者 '{}' 健康状态: {:?}", name, health);
        }
    }

    /// 获取健康的提供者
    #[allow(dead_code)]
    async fn get_healthy_provider(&self, preferred: Option<&str>) -> Option<Arc<dyn LLMProvider>> {
        let status_map = self.provider_status.read().await;

        // 首先尝试首选提供者
        if let Some(preferred_name) = preferred {
            if let Some(status) = status_map.get(preferred_name) {
                if status.health == ProviderHealth::Healthy
                    || status.health == ProviderHealth::Unknown
                {
                    if let Some(provider) = self.providers.get(preferred_name) {
                        return Some(provider.clone());
                    }
                }
            }
        }

        // 尝试默认提供者
        if let Some(status) = status_map.get(&self.config.default_provider) {
            if status.health == ProviderHealth::Healthy || status.health == ProviderHealth::Unknown
            {
                if let Some(provider) = self.providers.get(&self.config.default_provider) {
                    return Some(provider.clone());
                }
            }
        }

        // 尝试后备提供者
        if self.config.auto_fallback {
            for fallback_name in &self.config.fallback_providers {
                if let Some(status) = status_map.get(fallback_name) {
                    if status.health == ProviderHealth::Healthy
                        || status.health == ProviderHealth::Unknown
                    {
                        if let Some(provider) = self.providers.get(fallback_name) {
                            warn!("使用后备提供者: {}", fallback_name);
                            return Some(provider.clone());
                        }
                    }
                }
            }
        }

        // 最后，尝试任何可用的提供者
        for (name, provider) in &self.providers {
            if let Some(status) = status_map.get(name) {
                if status.health != ProviderHealth::Unhealthy {
                    warn!("使用最后可用的提供者: {}", name);
                    return Some(provider.clone());
                }
            }
        }

        None
    }

    /// 获取所有提供者状态
    pub async fn get_provider_status(&self) -> HashMap<String, ProviderStatus> {
        self.provider_status.read().await.clone()
    }

    /// 获取特定提供者
    pub fn get_provider(&self, name: &str) -> Option<Arc<dyn LLMProvider>> {
        self.providers.get(name).cloned()
    }

    /// 获取所有可用模型
    pub async fn list_all_models(&self) -> Result<HashMap<String, Vec<ModelInfo>>, LLMError> {
        let mut all_models = HashMap::new();

        for (name, provider) in &self.providers {
            match provider.list_models().await {
                Ok(models) => {
                    all_models.insert(name.clone(), models);
                }
                Err(e) => {
                    warn!("获取提供者 '{}' 的模型列表失败: {}", name, e);
                    all_models.insert(name.clone(), Vec::new());
                }
            }
        }

        Ok(all_models)
    }

    /// 查找模型所在的提供者
    pub async fn find_provider_for_model(&self, model_name: &str) -> Option<Arc<dyn LLMProvider>> {
        for (name, provider) in &self.providers {
            match provider.is_model_available(model_name).await {
                Ok(true) => {
                    debug!("模型 '{}' 在提供者 '{}' 中找到", model_name, name);
                    return Some(provider.clone());
                }
                Ok(false) => continue,
                Err(e) => {
                    warn!("检查提供者 '{}' 模型 '{}' 时出错: {}", name, model_name, e);
                    continue;
                }
            }
        }
        None
    }

    /// 强制刷新健康状态
    pub async fn refresh_health_status(&self) {
        Self::perform_health_checks(&self.providers, &self.provider_status).await;
    }
}

#[async_trait]
impl LLMProvider for LLMManager {
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, LLMError> {
        let provider = self
            .find_provider_for_model(&request.model)
            .await
            .or_else(|| self.providers.get(&self.config.default_provider).cloned())
            .ok_or_else(|| {
                LLMError::ModelNotAvailable(format!("无法找到模型 '{}'", request.model))
            })?;

        provider.chat(request).await
    }

    async fn chat_stream(
        &self,
        request: ChatRequest,
    ) -> Result<Box<dyn Stream<Item = Result<ChatResponse, LLMError>> + Send + Unpin>, LLMError>
    {
        let provider = self
            .find_provider_for_model(&request.model)
            .await
            .or_else(|| self.providers.get(&self.config.default_provider).cloned())
            .ok_or_else(|| {
                LLMError::ModelNotAvailable(format!("无法找到模型 '{}'", request.model))
            })?;

        provider.chat_stream(request).await
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>, LLMError> {
        let mut all_models = Vec::new();

        for (name, provider) in &self.providers {
            match provider.list_models().await {
                Ok(models) => {
                    all_models.extend(models);
                }
                Err(e) => {
                    warn!("获取提供者 '{}' 的模型列表失败: {}", name, e);
                }
            }
        }

        Ok(all_models)
    }

    async fn is_model_available(&self, model: &str) -> Result<bool, LLMError> {
        for provider in self.providers.values() {
            match provider.is_model_available(model).await {
                Ok(true) => return Ok(true),
                Ok(false) => continue,
                Err(_) => continue,
            }
        }
        Ok(false)
    }

    fn provider_name(&self) -> &'static str {
        "llm_manager"
    }

    fn default_model(&self) -> &str {
        // 返回默认提供者的默认模型
        if let Some(provider) = self.providers.get(&self.config.default_provider) {
            provider.default_model()
        } else {
            "unknown"
        }
    }

    async fn health_check(&self) -> Result<bool, LLMError> {
        // 如果有任何一个提供者健康，管理器就是健康的
        for provider in self.providers.values() {
            if provider.health_check().await.unwrap_or(false) {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

impl Drop for LLMManager {
    fn drop(&mut self) {
        if let Some(task) = self.health_check_task.take() {
            task.abort();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_manager_creation() {
        let config = LLMManagerConfig::default();
        let result = LLMManager::new(config).await;

        // 注意：这个测试可能会失败，如果没有可用的提供者
        // 在实际环境中，你可能需要模拟提供者
        match result {
            Ok(manager) => {
                assert_eq!(manager.provider_name(), "llm_manager");
                assert!(!manager.providers.is_empty());
            }
            Err(e) => {
                println!("管理器创建失败（可能是预期的）: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_provider_status() {
        let status = ProviderStatus::new("test".to_string());
        assert_eq!(status.name, "test");
        assert_eq!(status.health, ProviderHealth::Unknown);
        assert_eq!(status.error_count, 0);
        assert!(status.available_models.is_empty());
    }

    #[test]
    fn test_config_serialization() {
        let config = LLMManagerConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: LLMManagerConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.default_provider, deserialized.default_provider);
        assert_eq!(config.auto_fallback, deserialized.auto_fallback);
    }
}
