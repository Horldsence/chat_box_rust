use super::models::llm::{ChatMessage, ChatRequest, ChatResponse, LLMProvider};
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
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            default_model: "llama2-7b".to_string(),
            default_temperature: 0.7,
            default_max_tokens: 2048,
            timeout_seconds: 30,
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
    async fn send_message(&self, context: &mut ConversationContext, message: &str) -> Result<String, Box<dyn Error + Send + Sync>>;
    
    /// 发送消息并获取流式响应
    async fn send_message_stream(&self, context: &mut ConversationContext, message: &str) -> Result<Box<dyn Stream<Item = Result<String, Box<dyn Error + Send + Sync>>> + Send + Unpin>, Box<dyn Error + Send + Sync>>;
    
    /// 获取可用模型列表
    async fn list_available_models(&self) -> Result<Vec<String>, Box<dyn Error + Send + Sync>>;
    
    /// 切换模型
    async fn switch_model(&self, model: &str) -> Result<(), Box<dyn Error + Send + Sync>>;
    
    /// 获取当前配置
    fn get_config(&self) -> &AgentConfig;
    
    /// 获取代理名称
    fn agent_name(&self) -> &'static str;
}

pub struct MultiProviderAgent {
    config: AgentConfig,
    providers: HashMap<String, Arc<dyn LLMProvider>>,
    current_provider: String,
}

impl MultiProviderAgent {
    pub fn new(config: AgentConfig) -> Self {
        Self {
            config,
            providers: HashMap::new(),
            current_provider: "ollama".to_string(), // 默认使用 ollama
        }
    }

    /// 添加 LLM 提供者
    pub fn add_provider(&mut self, name: String, provider: Arc<dyn LLMProvider>) {
        info!("添加 LLM 提供者: {}", name);
        self.providers.insert(name, provider);
    }

    /// 设置当前提供者
    pub fn set_current_provider(&mut self, provider_name: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        if self.providers.contains_key(provider_name) {
            self.current_provider = provider_name.to_string();
            info!("切换到 LLM 提供者: {}", provider_name);
            Ok(())
        } else {
            Err(format!("提供者 '{}' 不存在", provider_name).into())
        }
    }

    /// 获取当前提供者
    fn get_current_provider(&self) -> Result<&Arc<dyn LLMProvider>, Box<dyn Error + Send + Sync>> {
        self.providers.get(&self.current_provider)
            .ok_or_else(|| format!("当前提供者 '{}' 不可用", self.current_provider).into())
    }

    /// 构建聊天请求
    fn build_chat_request(&self, context: &ConversationContext, new_message: &str) -> ChatRequest {
        let mut messages = context.messages.clone();
        
        // 添加系统提示（如果有）
        if let Some(ref system_prompt) = context.system_prompt {
            messages.insert(0, ChatMessage {
                role: "system".to_string(),
                content: system_prompt.clone(),
            });
        }
        
        // 添加新的用户消息
        messages.push(ChatMessage {
            role: "user".to_string(),
            content: new_message.to_string(),
        });

        ChatRequest {
            messages,
            model: context.model.clone(),
            temperature: Some(self.config.default_temperature),
            max_tokens: Some(self.config.default_max_tokens),
            stream: false,
        }
    }
}

#[async_trait]
impl Agent for MultiProviderAgent {
    async fn send_message(&self, context: &mut ConversationContext, message: &str) -> Result<String, Box<dyn Error + Send + Sync>> {
        let provider = self.get_current_provider()?;
        let request = self.build_chat_request(context, message);

        match provider.chat(request).await {
            Ok(response) => {
                // 更新对话上下文
                context.messages.push(ChatMessage {
                    role: "user".to_string(),
                    content: message.to_string(),
                });
                context.messages.push(ChatMessage {
                    role: "assistant".to_string(),
                    content: response.content.clone(),
                });

                Ok(response.content)
            }
            Err(e) => {
                error!("发送消息失败: {}", e);
                Err(e)
            }
        }
    }

    async fn send_message_stream(&self, context: &mut ConversationContext, message: &str) -> Result<Box<dyn Stream<Item = Result<String, Box<dyn Error + Send + Sync>>> + Send + Unpin>, Box<dyn Error + Send + Sync>> {
        let provider = self.get_current_provider()?;
        let mut request = self.build_chat_request(context, message);
        request.stream = true;

        // 预先更新用户消息到上下文
        context.messages.push(ChatMessage {
            role: "user".to_string(),
            content: message.to_string(),
        });

        let stream = provider.chat_stream(request).await?;
        
        // 转换流以提取内容并更新上下文
        use tokio_stream::StreamExt;
        let mapped_stream = stream.map(|result| {
            match result {
                Ok(response) => Ok(response.content),
                Err(e) => Err(e),
            }
        });

        Ok(Box::new(mapped_stream))
    }

    async fn list_available_models(&self) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
        let provider = self.get_current_provider()?;
        provider.list_models().await
    }

    async fn switch_model(&self, model: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        let provider = self.get_current_provider()?;
        
        if provider.is_model_available(model).await? {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::agent::models::llm::candle::{CandleConfig, CandleLLM};

    #[tokio::test]
    async fn test_multi_provider_agent() {
        let config = AgentConfig::default();
        let mut agent = MultiProviderAgent::new(config);

        // 添加 Candle 提供者
        let candle_config = CandleConfig::default();
        let candle_provider = Arc::new(CandleLLM::new(candle_config).unwrap());
        agent.add_provider("candle".to_string(), candle_provider);

        // 测试切换提供者
        assert!(agent.set_current_provider("candle").is_ok());
        
        // 测试获取模型列表
        let models = agent.list_available_models().await;
        assert!(models.is_ok());
    }

    #[tokio::test]
    async fn test_conversation_context() {
        let mut context = ConversationContext {
            conversation_id: "test_123".to_string(),
            messages: vec![],
            model: "llama2-7b".to_string(),
            system_prompt: Some("You are a helpful assistant.".to_string()),
        };

        assert_eq!(context.conversation_id, "test_123");
        assert_eq!(context.messages.len(), 0);
        assert!(context.system_prompt.is_some());
    }
}