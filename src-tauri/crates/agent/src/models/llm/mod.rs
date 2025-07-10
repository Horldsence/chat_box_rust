use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::error::Error;
use tokio_stream::Stream;

// 重新导出主要类型
#[cfg(feature = "candle")]
pub use candle::{
    CandleConfig, CandleProvider, QwenCandleGenerator, QwenInferenceParams, WhichModel,
};
pub use manager::{LLMManager, LLMManagerConfig, ProviderConfig, ProviderHealth, ProviderStatus};
pub use ollama::provider::{OllamaConfig, OllamaProvider};
// 兼容性接口
pub use ollama::ollama::OllamaAgent;

#[cfg(feature = "candle")]
pub mod candle;
pub mod manager;
pub mod ollama;

/// 聊天消息结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String, // "system", "user", "assistant"
    pub content: String,
}

impl ChatMessage {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".to_string(),
            content: content.into(),
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".to_string(),
            content: content.into(),
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.into(),
        }
    }
}

/// 聊天请求结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub messages: Vec<ChatMessage>,
    pub model: String,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub stream: bool,
}

impl ChatRequest {
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            messages: vec![],
            model: model.into(),
            temperature: None,
            max_tokens: None,
            stream: false,
        }
    }

    pub fn with_messages(mut self, messages: Vec<ChatMessage>) -> Self {
        self.messages = messages;
        self
    }

    pub fn with_system_message(mut self, content: impl Into<String>) -> Self {
        self.messages.insert(0, ChatMessage::system(content));
        self
    }

    pub fn with_user_message(mut self, content: impl Into<String>) -> Self {
        self.messages.push(ChatMessage::user(content));
        self
    }

    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    pub fn with_stream(mut self, stream: bool) -> Self {
        self.stream = stream;
        self
    }
}

/// 聊天响应结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub content: String,
    pub done: bool,
    pub model: String,
    pub finish_reason: Option<String>,
}

impl ChatResponse {
    pub fn new(content: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            done: false,
            model: model.into(),
            finish_reason: None,
        }
    }

    pub fn with_done(mut self, done: bool) -> Self {
        self.done = done;
        self
    }

    pub fn with_finish_reason(mut self, reason: impl Into<String>) -> Self {
        self.finish_reason = Some(reason.into());
        self
    }
}

/// 模型信息结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub description: Option<String>,
    pub size: Option<String>,
    pub family: Option<String>,
    pub parameter_size: Option<String>,
}

impl ModelInfo {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            size: None,
            family: None,
            parameter_size: None,
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn with_size(mut self, size: impl Into<String>) -> Self {
        self.size = Some(size.into());
        self
    }

    pub fn with_family(mut self, family: impl Into<String>) -> Self {
        self.family = Some(family.into());
        self
    }

    pub fn with_parameter_size(mut self, parameter_size: impl Into<String>) -> Self {
        self.parameter_size = Some(parameter_size.into());
        self
    }
}

/// LLM 错误类型
#[derive(Debug, thiserror::Error)]
pub enum LLMError {
    #[error("网络错误: {0}")]
    Network(String),

    #[error("模型不可用: {0}")]
    ModelNotAvailable(String),

    #[error("请求格式错误: {0}")]
    InvalidRequest(String),

    #[error("API 错误: {0}")]
    ApiError(String),

    #[error("流式响应错误: {0}")]
    StreamError(String),

    #[error("配置错误: {0}")]
    ConfigError(String),

    #[error("内部错误: {0}")]
    Internal(String),
}

/// LLM 提供者统一接口
#[async_trait]
pub trait LLMProvider: Send + Sync {
    /// 发送聊天请求并获取响应
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, LLMError>;

    /// 发送聊天请求并获取流式响应
    async fn chat_stream(
        &self,
        request: ChatRequest,
    ) -> Result<Box<dyn Stream<Item = Result<ChatResponse, LLMError>> + Send + Unpin>, LLMError>;

    /// 获取可用模型列表
    async fn list_models(&self) -> Result<Vec<ModelInfo>, LLMError>;

    /// 检查模型是否可用
    async fn is_model_available(&self, model: &str) -> Result<bool, LLMError>;

    /// 获取提供者名称
    fn provider_name(&self) -> &'static str;

    /// 获取默认模型
    fn default_model(&self) -> &str;

    /// 健康检查
    async fn health_check(&self) -> Result<bool, LLMError>;
}

/// 简化的流式生成接口（兼容现有代码）
#[async_trait]
pub trait StreamGenerator: Send + Sync {
    /// 生成文本流
    async fn generate_stream(
        &self,
        prompt: &str,
    ) -> Result<Box<dyn Stream<Item = String> + Send + Unpin>, Box<dyn Error + Send + Sync>>;

    /// 生成完整响应
    async fn generate_response(&self, prompt: &str)
        -> Result<String, Box<dyn Error + Send + Sync>>;
}

/// 为实现了 LLMProvider 的类型自动实现 StreamGenerator
#[async_trait]
impl<T: LLMProvider> StreamGenerator for T {
    async fn generate_stream(
        &self,
        prompt: &str,
    ) -> Result<Box<dyn Stream<Item = String> + Send + Unpin>, Box<dyn Error + Send + Sync>> {
        let request = ChatRequest::new(self.default_model())
            .with_user_message(prompt)
            .with_stream(true);

        let stream = self
            .chat_stream(request)
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;

        use tokio_stream::StreamExt;
        let content_stream = stream.map(|result| match result {
            Ok(response) => response.content,
            Err(_) => String::new(),
        });

        Ok(Box::new(content_stream))
    }

    async fn generate_response(
        &self,
        prompt: &str,
    ) -> Result<String, Box<dyn Error + Send + Sync>> {
        let request = ChatRequest::new(self.default_model()).with_user_message(prompt);

        let response = self
            .chat(request)
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;

        Ok(response.content)
    }
}
