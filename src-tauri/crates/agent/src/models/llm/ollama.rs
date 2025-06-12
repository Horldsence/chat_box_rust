use super::{ChatMessage, ChatRequest, ChatResponse, LLMError, LLMProvider, ModelInfo};
use async_trait::async_trait;
use log::{debug, error, info, warn};
use ollama_rs::{
    generation::chat::{ChatMessage as OllamaChatMessage, MessageRole},
    generation::completion::request::GenerationRequest,
    Ollama,
};
use serde::{Deserialize, Serialize};
use tokio_stream::{Stream, StreamExt};

/// Ollama 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaConfig {
    pub host: String,
    pub port: u16,
    pub default_model: String,
    pub default_temperature: Option<f32>,
    pub default_max_tokens: Option<u32>,
    pub timeout_seconds: Option<u64>,
    pub system_prompt: Option<String>,
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 11434,
            default_model: "llama2".to_string(),
            default_temperature: Some(0.7),
            default_max_tokens: Some(2048),
            timeout_seconds: Some(30),
            system_prompt: Some("你是一个使用中文作为主要语言的问答助手。".to_string()),
        }
    }
}

/// Ollama LLM 提供者
pub struct OllamaProvider {
    config: OllamaConfig,
    client: Ollama,
    available_models: tokio::sync::RwLock<Option<Vec<ModelInfo>>>,
}

impl OllamaProvider {
    /// 创建新的 Ollama 提供者
    pub fn new(config: OllamaConfig) -> Self {
        let client = Ollama::new(config.host.clone(), config.port);

        Self {
            config,
            client,
            available_models: tokio::sync::RwLock::new(None),
        }
    }

    /// 使用默认配置创建 Ollama 提供者
    pub fn with_default() -> Self {
        Self::new(OllamaConfig::default())
    }

    /// 设置主机和端口
    pub fn with_host_port(mut self, host: impl Into<String>, port: u16) -> Self {
        self.config.host = host.into();
        self.config.port = port;
        self.client = Ollama::new(self.config.host.clone(), self.config.port);
        self
    }

    /// 设置默认模型
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.config.default_model = model.into();
        self
    }

    /// 设置系统提示
    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.config.system_prompt = Some(prompt.into());
        self
    }

    /// 转换聊天消息格式
    fn convert_messages(&self, messages: &[ChatMessage]) -> Vec<OllamaChatMessage> {
        messages
            .iter()
            .map(|msg| {
                let role = match msg.role.as_str() {
                    "user" => MessageRole::User,
                    "assistant" => MessageRole::Assistant,
                    "system" => MessageRole::System,
                    _ => MessageRole::User,
                };
                OllamaChatMessage::new(role, msg.content.clone())
            })
            .collect()
    }

    /// 构建完整的消息列表（包含系统提示）
    fn build_messages(&self, mut messages: Vec<ChatMessage>) -> Vec<ChatMessage> {
        // 如果有系统提示且消息列表中没有系统消息，添加系统提示
        if let Some(ref system_prompt) = self.config.system_prompt {
            let has_system = messages.iter().any(|msg| msg.role == "system");
            if !has_system {
                messages.insert(0, ChatMessage::system(system_prompt));
            }
        }
        messages
    }

    /// 刷新可用模型列表
    async fn refresh_models(&self) -> Result<Vec<ModelInfo>, LLMError> {
        debug!("刷新 Ollama 模型列表");

        match self.client.list_local_models().await {
            Ok(models) => {
                let model_infos: Vec<ModelInfo> = models
                    .into_iter()
                    .map(|model| {
                        ModelInfo::new(model.name.clone())
                            .with_description(format!("Ollama 本地模型: {}", model.name))
                            .with_size(format!("{} bytes", model.size))
                            .with_family("ollama".to_string())
                    })
                    .collect();

                // 缓存模型列表
                {
                    let mut cache = self.available_models.write().await;
                    *cache = Some(model_infos.clone());
                }

                info!("获取到 {} 个 Ollama 模型", model_infos.len());
                Ok(model_infos)
            }
            Err(e) => {
                error!("获取 Ollama 模型列表失败: {}", e);
                Err(LLMError::ApiError(format!("获取模型列表失败: {}", e)))
            }
        }
    }
}

#[async_trait]
impl LLMProvider for OllamaProvider {
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, LLMError> {
        debug!("发送 Ollama 聊天请求: {:?}", request);

        let messages = self.build_messages(request.messages);

        // 构建完整的提示词，因为简化的 API 可能不支持复杂的消息结构
        let mut prompt = String::new();
        for message in &messages {
            match message.role.as_str() {
                "system" => prompt.push_str(&format!("System: {}\n\n", message.content)),
                "user" => prompt.push_str(&format!("User: {}\n\n", message.content)),
                "assistant" => prompt.push_str(&format!("Assistant: {}\n\n", message.content)),
                _ => prompt.push_str(&format!("{}: {}\n\n", message.role, message.content)),
            }
        }
        prompt.push_str("Assistant: ");

        let generation_request = GenerationRequest::new(request.model.clone(), prompt);

        match self.client.generate(generation_request).await {
            Ok(response) => {
                debug!("收到 Ollama 响应");
                Ok(ChatResponse::new(response.response, request.model).with_done(true))
            }
            Err(e) => {
                error!("Ollama 聊天请求失败: {}", e);
                Err(LLMError::ApiError(format!("聊天请求失败: {}", e)))
            }
        }
    }

    async fn chat_stream(
        &self,
        request: ChatRequest,
    ) -> Result<Box<dyn Stream<Item = Result<ChatResponse, LLMError>> + Send + Unpin>, LLMError>
    {
        debug!("发送 Ollama 流式聊天请求: {:?}", request);

        let messages = self.build_messages(request.messages);

        // 构建完整的提示词
        let mut prompt = String::new();
        for message in &messages {
            match message.role.as_str() {
                "system" => prompt.push_str(&format!("System: {}\n\n", message.content)),
                "user" => prompt.push_str(&format!("User: {}\n\n", message.content)),
                "assistant" => prompt.push_str(&format!("Assistant: {}\n\n", message.content)),
                _ => prompt.push_str(&format!("{}: {}\n\n", message.role, message.content)),
            }
        }
        prompt.push_str("Assistant: ");

        let generation_request = GenerationRequest::new(request.model.clone(), prompt);

        match self.client.generate_stream(generation_request).await {
            Ok(stream) => {
                let model = request.model.clone();
                use tokio_stream::StreamExt;
                let mapped_stream = stream.map(move |result| match result {
                    Ok(responses) => {
                        let content = responses
                            .into_iter()
                            .map(|r| r.response)
                            .collect::<String>();
                        Ok(ChatResponse::new(content, model.clone()).with_done(false))
                    }
                    Err(e) => {
                        error!("流式响应错误: {:?}", e);
                        Err(LLMError::StreamError(format!("流式响应错误: {:?}", e)))
                    }
                });
                Ok(Box::new(mapped_stream))
            }
            Err(e) => {
                error!("创建 Ollama 流式请求失败: {:?}", e);
                Err(LLMError::ApiError(format!("创建流式请求失败: {:?}", e)))
            }
        }
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>, LLMError> {
        // 先检查缓存
        {
            let cache = self.available_models.read().await;
            if let Some(ref models) = *cache {
                if !models.is_empty() {
                    debug!("使用缓存的模型列表，共 {} 个模型", models.len());
                    return Ok(models.clone());
                }
            }
        }

        // 缓存为空或不存在，刷新模型列表
        self.refresh_models().await
    }

    async fn is_model_available(&self, model: &str) -> Result<bool, LLMError> {
        let models = self.list_models().await?;
        let available = models.iter().any(|m| m.name == model);
        debug!("检查模型 '{}' 可用性: {}", model, available);
        Ok(available)
    }

    fn provider_name(&self) -> &'static str {
        "ollama"
    }

    fn default_model(&self) -> &str {
        &self.config.default_model
    }

    async fn health_check(&self) -> Result<bool, LLMError> {
        debug!("执行 Ollama 健康检查");

        // 尝试获取模型列表作为健康检查
        match self.client.list_local_models().await {
            Ok(_) => {
                info!("Ollama 健康检查通过");
                Ok(true)
            }
            Err(e) => {
                warn!("Ollama 健康检查失败: {}", e);
                Ok(false) // 不抛出错误，只返回不健康状态
            }
        }
    }
}

/// 兼容旧版本接口的 OllamaAgent
pub struct OllamaAgent {
    provider: OllamaProvider,
}

impl OllamaAgent {
    /// 创建新的 OllamaAgent（兼容接口）
    pub fn new(model: &str, host: &str, port: &u16) -> Self {
        let config = OllamaConfig {
            host: host.to_string(),
            port: *port,
            default_model: model.to_string(),
            ..Default::default()
        };

        Self {
            provider: OllamaProvider::new(config),
        }
    }

    /// 设置系统提示（兼容接口）
    pub fn with_system_prompt(mut self, prompt: &str) -> Self {
        self.provider = self.provider.with_system_prompt(prompt);
        self
    }

    /// 设置主机和端口（兼容接口）
    pub fn with_host_port(mut self, host: &str, port: u16) -> Self {
        self.provider = self.provider.with_host_port(host, port);
        self
    }

    /// 生成响应（兼容接口）
    pub async fn generate_response(
        &self,
        user_prompt: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let request =
            ChatRequest::new(self.provider.default_model()).with_user_message(user_prompt);

        match self.provider.chat(request).await {
            Ok(response) => Ok(response.content),
            Err(e) => Err(Box::new(e)),
        }
    }

    /// 生成流式响应（兼容接口）
    pub async fn generate_stream(
        &self,
        user_prompt: &str,
    ) -> Result<impl Stream<Item = String>, Box<dyn std::error::Error>> {
        let request = ChatRequest::new(self.provider.default_model())
            .with_user_message(user_prompt)
            .with_stream(true);

        match self.provider.chat_stream(request).await {
            Ok(stream) => {
                let content_stream = stream.map(|result| match result {
                    Ok(response) => response.content,
                    Err(_) => String::new(),
                });
                Ok(content_stream)
            }
            Err(e) => Err(Box::new(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ollama_provider_creation() {
        let config = OllamaConfig::default();
        let provider = OllamaProvider::new(config);
        assert_eq!(provider.provider_name(), "ollama");
        assert_eq!(provider.default_model(), "llama2");
    }

    #[tokio::test]
    async fn test_ollama_agent_compatibility() {
        let agent = OllamaAgent::new("llama2", "localhost", &11434);
        // 基本的创建测试，不需要实际连接 Ollama
        assert_eq!(agent.provider.default_model(), "llama2");
    }

    #[test]
    fn test_message_conversion() {
        let provider = OllamaProvider::with_default();
        let messages = vec![
            ChatMessage::user("Hello"),
            ChatMessage::assistant("Hi there!"),
        ];

        let converted = provider.convert_messages(&messages);
        assert_eq!(converted.len(), 2);
        assert_eq!(converted[0].role, MessageRole::User);
        assert_eq!(converted[1].role, MessageRole::Assistant);
    }

    #[test]
    fn test_build_messages_with_system_prompt() {
        let provider = OllamaProvider::with_default().with_system_prompt("You are helpful");
        let messages = vec![ChatMessage::user("Hello")];

        let built = provider.build_messages(messages);
        assert_eq!(built.len(), 2);
        assert_eq!(built[0].role, "system");
        assert_eq!(built[1].role, "user");
    }
}
