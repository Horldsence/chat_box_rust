#[allow(unused_imports)]
use crate::{
    ChatMessage, ChatRequest, ChatResponse, LLMError, LLMProvider, ModelInfo,
};
use tokio_stream::{Stream, StreamExt};
#[allow(unused_imports)]
use ollama_rs::generation::chat::MessageRole;

use crate::models::llm::ollama::provider::{OllamaProvider, OllamaConfig};

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
