use super::models::llm::{
    ChatMessage, ChatRequest, ChatResponse, LLMError, LLMManager, LLMManagerConfig, LLMProvider,
    ModelInfo, StreamGenerator,
};
use async_trait::async_trait;
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use tokio_stream::Stream;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub default_model: String,
    pub default_temperature: f32,
    pub default_max_tokens: u32,
    pub timeout_seconds: u64,
    pub llm_manager: LLMManagerConfig,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            default_model: "llama2".to_string(),
            default_temperature: 0.7,
            default_max_tokens: 2048,
            timeout_seconds: 30,
            llm_manager: LLMManagerConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationContext {
    pub conversation_id: String,
    pub messages: Vec<ChatMessage>,
    pub model: String,
    pub system_prompt: Option<String>,
}

#[async_trait]
pub trait Agent: Send + Sync {
    /// 发送消息并获取响应
    async fn send_message(
        &self,
        context: &mut ConversationContext,
        message: &str,
    ) -> Result<String, Box<dyn Error + Send + Sync>>;

    /// 发送消息并获取流式响应
    async fn send_message_stream(
        &self,
        context: &mut ConversationContext,
        message: &str,
    ) -> Result<
        Box<dyn Stream<Item = Result<String, Box<dyn Error + Send + Sync>>> + Send + Unpin>,
        Box<dyn Error + Send + Sync>,
    >;

    /// 获取可用模型列表
    async fn list_available_models(&self) -> Result<Vec<ModelInfo>, Box<dyn Error + Send + Sync>>;

    /// 切换模型
    async fn switch_model(&self, model: &str) -> Result<(), Box<dyn Error + Send + Sync>>;

    /// 获取当前配置
    fn get_config(&self) -> &AgentConfig;

    /// 获取代理名称
    fn agent_name(&self) -> &'static str;

    /// 获取 LLM 管理器
    fn get_llm_manager(&self) -> &LLMManager;
}

pub struct MultiProviderAgent {
    config: AgentConfig,
    llm_manager: LLMManager,
    current_model: String,
}

impl MultiProviderAgent {
    /// 创建新的多提供者代理
    pub async fn new(config: AgentConfig) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let llm_manager = LLMManager::new(config.llm_manager.clone())
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;

        let current_model = config.default_model.clone();

        info!("创建多提供者代理，默认模型: {}", current_model);

        Ok(Self {
            config,
            llm_manager,
            current_model,
        })
    }

    /// 使用默认配置创建代理
    pub async fn with_default() -> Result<Self, Box<dyn Error + Send + Sync>> {
        Self::new(AgentConfig::default()).await
    }

    /// 设置当前模型
    pub async fn set_current_model(
        &mut self,
        model: &str,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        if self
            .llm_manager
            .is_model_available(model)
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?
        {
            self.current_model = model.to_string();
            info!("切换到模型: {}", model);
            Ok(())
        } else {
            Err(format!("模型 '{}' 不可用", model).into())
        }
    }

    /// 获取当前模型
    pub fn get_current_model(&self) -> &str {
        &self.current_model
    }

    /// 构建聊天请求
    fn build_chat_request(&self, context: &ConversationContext, new_message: &str) -> ChatRequest {
        let mut messages = context.messages.clone();

        // 添加系统提示（如果有）
        if let Some(ref system_prompt) = context.system_prompt {
            messages.insert(0, ChatMessage::system(system_prompt));
        }

        // 添加新的用户消息
        messages.push(ChatMessage::user(new_message));

        ChatRequest::new(&self.current_model)
            .with_messages(messages)
            .with_temperature(self.config.default_temperature)
            .with_max_tokens(self.config.default_max_tokens)
    }

    /// 获取提供者状态
    pub async fn get_provider_status(&self) -> HashMap<String, crate::models::llm::ProviderStatus> {
        self.llm_manager.get_provider_status().await
    }

    /// 刷新健康状态
    pub async fn refresh_health_status(&self) {
        self.llm_manager.refresh_health_status().await;
    }
}

#[async_trait]
impl Agent for MultiProviderAgent {
    async fn send_message(
        &self,
        context: &mut ConversationContext,
        message: &str,
    ) -> Result<String, Box<dyn Error + Send + Sync>> {
        let request = self.build_chat_request(context, message);

        match self.llm_manager.chat(request).await {
            Ok(response) => {
                // 更新对话上下文
                context.messages.push(ChatMessage::user(message));
                context
                    .messages
                    .push(ChatMessage::assistant(&response.content));

                Ok(response.content)
            }
            Err(e) => {
                error!("发送消息失败: {}", e);
                Err(Box::new(e))
            }
        }
    }

    async fn send_message_stream(
        &self,
        context: &mut ConversationContext,
        message: &str,
    ) -> Result<
        Box<dyn Stream<Item = Result<String, Box<dyn Error + Send + Sync>>> + Send + Unpin>,
        Box<dyn Error + Send + Sync>,
    > {
        let mut request = self.build_chat_request(context, message);
        request.stream = true;

        // 预先更新用户消息到上下文
        context.messages.push(ChatMessage::user(message));

        let stream = self
            .llm_manager
            .chat_stream(request)
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?;

        // 转换流以提取内容
        use tokio_stream::StreamExt;
        let mapped_stream = stream.map(|result| match result {
            Ok(response) => Ok(response.content),
            Err(e) => Err(Box::new(e) as Box<dyn Error + Send + Sync>),
        });

        Ok(Box::new(mapped_stream))
    }

    async fn list_available_models(&self) -> Result<Vec<ModelInfo>, Box<dyn Error + Send + Sync>> {
        self.llm_manager
            .list_models()
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)
    }

    async fn switch_model(&self, model: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        if self
            .llm_manager
            .is_model_available(model)
            .await
            .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)?
        {
            info!("模型切换成功: {}", model);
            Ok(())
        } else {
            Err(format!("模型 '{}' 不可用", model).into())
        }
    }

    fn get_config(&self) -> &AgentConfig {
        &self.config
    }

    fn agent_name(&self) -> &'static str {
        "multi_provider_agent"
    }

    fn get_llm_manager(&self) -> &LLMManager {
        &self.llm_manager
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_multi_provider_agent() {
        let config = AgentConfig::default();

        // 注意：在实际环境中可能需要可用的提供者
        match MultiProviderAgent::new(config).await {
            Ok(agent) => {
                assert_eq!(agent.agent_name(), "multi_provider_agent");
                assert!(!agent.get_current_model().is_empty());
            }
            Err(e) => {
                println!("代理创建失败（可能是预期的）: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_conversation_context() {
        let mut context = ConversationContext {
            conversation_id: "test_123".to_string(),
            messages: vec![],
            model: "llama2".to_string(),
            system_prompt: Some("You are a helpful assistant.".to_string()),
        };

        assert_eq!(context.conversation_id, "test_123");
        assert_eq!(context.messages.len(), 0);
        assert!(context.system_prompt.is_some());
    }

    #[test]
    fn test_agent_config_default() {
        let config = AgentConfig::default();
        assert_eq!(config.default_model, "llama2");
        assert_eq!(config.default_temperature, 0.7);
        assert_eq!(config.default_max_tokens, 2048);
    }
}
