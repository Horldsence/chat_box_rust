use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::error::Error;
use tokio_stream::Stream;

pub mod candle;
pub mod ollama;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatRequest {
    pub messages: Vec<ChatMessage>,
    pub model: String,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub stream: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub content: String,
    pub done: bool,
    pub model: String,
}

#[async_trait]
pub trait LLMProvider: Send + Sync {
    /// 发送聊天请求并获取响应
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, Box<dyn Error + Send + Sync>>;
    
    /// 发送聊天请求并获取流式响应
    async fn chat_stream(&self, request: ChatRequest) -> Result<Box<dyn Stream<Item = Result<ChatResponse, Box<dyn Error + Send + Sync>>> + Send + Unpin>, Box<dyn Error + Send + Sync>>;
    
    /// 获取可用模型列表
    async fn list_models(&self) -> Result<Vec<String>, Box<dyn Error + Send + Sync>>;
    
    /// 检查模型是否可用
    async fn is_model_available(&self, model: &str) -> Result<bool, Box<dyn Error + Send + Sync>>;
    
    /// 获取提供者名称
    fn provider_name(&self) -> &'static str;
}