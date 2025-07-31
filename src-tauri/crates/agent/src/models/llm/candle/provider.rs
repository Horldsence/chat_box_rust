use super::super::{ChatMessage, ChatRequest, ChatResponse, LLMError, LLMProvider, ModelInfo};
use super::candle::{QwenCandleGenerator, QwenInferenceParams, WhichModel};
use async_trait::async_trait;
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio_stream::Stream;

/// Candle 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandleConfig {
    pub default_model: WhichModel,
    pub default_temperature: Option<f64>,
    pub default_max_tokens: Option<usize>,
    pub cpu_only: bool,
    pub repeat_penalty: f32,
    pub repeat_last_n: usize,
    pub seed: u64,
    pub system_prompt: Option<String>,
}

impl Default for CandleConfig {
    fn default() -> Self {
        Self {
            default_model: WhichModel::W0_5b, // 使用最小的模型作为默认
            default_temperature: Some(0.7),
            default_max_tokens: Some(2048),
            cpu_only: true, // 默认使用 CPU，更好的兼容性
            repeat_penalty: 1.1,
            repeat_last_n: 64,
            seed: 299792458,
            system_prompt: Some("你是一个使用中文作为主要语言的问答助手。".to_string()),
        }
    }
}

/// Candle LLM 提供者
pub struct CandleProvider {
    config: CandleConfig,
    generator: Arc<Mutex<Option<QwenCandleGenerator>>>,
    supported_models: HashMap<String, WhichModel>,
}

impl CandleProvider {
    /// 创建新的 Candle 提供者
    pub fn new(config: CandleConfig) -> Result<Self, LLMError> {
        let supported_models = Self::build_supported_models();

        Ok(Self {
            config,
            generator: Arc::new(Mutex::new(None)),
            supported_models,
        })
    }

    /// 使用默认配置创建 Candle 提供者
    pub fn with_default() -> Result<Self, LLMError> {
        Self::new(CandleConfig::default())
    }

    /// 设置模型
    pub fn with_model(mut self, model: WhichModel) -> Self {
        self.config.default_model = model;
        self
    }

    /// 设置系统提示
    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.config.system_prompt = Some(prompt.into());
        self
    }

    /// 设置是否仅使用 CPU
    pub fn with_cpu_only(mut self, cpu_only: bool) -> Self {
        self.config.cpu_only = cpu_only;
        self
    }

    /// 构建支持的模型映射
    fn build_supported_models() -> HashMap<String, WhichModel> {
        let mut models = HashMap::new();

        // Qwen 1.5 系列
        models.insert("qwen-0.5b".to_string(), WhichModel::W0_5b);
        models.insert("qwen-1.8b".to_string(), WhichModel::W1_8b);
        models.insert("qwen-4b".to_string(), WhichModel::W4b);
        models.insert("qwen-7b".to_string(), WhichModel::W7b);
        models.insert("qwen-14b".to_string(), WhichModel::W14b);
        models.insert("qwen-72b".to_string(), WhichModel::W72b);
        models.insert("qwen-moe-a2.7b".to_string(), WhichModel::MoeA27b);

        // Qwen 2 系列
        models.insert("qwen2-0.5b".to_string(), WhichModel::W2_0_5b);
        models.insert("qwen2-1.5b".to_string(), WhichModel::W2_1_5b);
        models.insert("qwen2-7b".to_string(), WhichModel::W2_7b);
        models.insert("qwen2-72b".to_string(), WhichModel::W2_72b);

        // Qwen 3 系列 (如果启用)
        models.insert("qwen3-0.6b".to_string(), WhichModel::W3_0_6b);
        models.insert("qwen3-1.7b".to_string(), WhichModel::W3_1_7b);
        models.insert("qwen3-4b".to_string(), WhichModel::W3_4b);
        models.insert("qwen3-8b".to_string(), WhichModel::W3_8b);

        models
    }

    /// 获取或初始化生成器
    async fn get_generator(&self) -> Result<(), LLMError> {
        let mut generator_guard = self
            .generator
            .lock()
            .map_err(|e| LLMError::Internal(format!("无法获取生成器锁: {}", e)))?;

        if generator_guard.is_none() {
            info!(
                "初始化 Candle 生成器，模型: {:?}",
                self.config.default_model
            );

            let params = QwenInferenceParams {
                model: self.config.default_model,
                cpu: self.config.cpu_only,
                temperature: self.config.default_temperature,
                repeat_penalty: self.config.repeat_penalty,
                repeat_last_n: self.config.repeat_last_n,
                seed: self.config.seed,
                sample_len: self.config.default_max_tokens.unwrap_or(2048),
                ..Default::default()
            };

            let generator = QwenCandleGenerator::new(params).map_err(|e| {
                error!("初始化 Candle 生成器失败: {}", e);
                LLMError::ConfigError(format!("初始化生成器失败: {}", e))
            })?;

            *generator_guard = Some(generator);
            info!("Candle 生成器初始化成功");
        }

        Ok(())
    }

    /// 构建完整的提示词（包含系统提示和对话历史）
    fn build_prompt(&self, messages: &[ChatMessage]) -> String {
        let mut prompt = String::new();

        // 添加系统提示
        if let Some(ref system_prompt) = self.config.system_prompt {
            prompt.push_str(&format!("System: {}\n\n", system_prompt));
        }

        // 添加消息历史
        for message in messages {
            match message.role.as_str() {
                "system" => prompt.push_str(&format!("System: {}\n\n", message.content)),
                "user" => prompt.push_str(&format!("User: {}\n\n", message.content)),
                "assistant" => prompt.push_str(&format!("Assistant: {}\n\n", message.content)),
                _ => prompt.push_str(&format!("{}: {}\n\n", message.role, message.content)),
            }
        }

        // 添加 Assistant 前缀以引导回复
        prompt.push_str("Assistant: ");

        prompt
    }

    /// 获取模型显示名称
    fn get_model_display_name(&self, which_model: &WhichModel) -> String {
        match which_model {
            WhichModel::W0_5b => "Qwen 1.5 0.5B".to_string(),
            WhichModel::W1_8b => "Qwen 1.5 1.8B".to_string(),
            WhichModel::W4b => "Qwen 1.5 4B".to_string(),
            WhichModel::W7b => "Qwen 1.5 7B".to_string(),
            WhichModel::W14b => "Qwen 1.5 14B".to_string(),
            WhichModel::W72b => "Qwen 1.5 72B".to_string(),
            WhichModel::MoeA27b => "Qwen 1.5 MoE A2.7B".to_string(),
            WhichModel::W2_0_5b => "Qwen 2 0.5B".to_string(),
            WhichModel::W2_1_5b => "Qwen 2 1.5B".to_string(),
            WhichModel::W2_7b => "Qwen 2 7B".to_string(),
            WhichModel::W2_72b => "Qwen 2 72B".to_string(),
            WhichModel::W3_0_6b => "Qwen 3 0.6B".to_string(),
            WhichModel::W3_1_7b => "Qwen 3 1.7B".to_string(),
            WhichModel::W3_4b => "Qwen 3 4B".to_string(),
            WhichModel::W3_8b => "Qwen 3 8B".to_string(),
        }
    }
}

#[async_trait]
impl LLMProvider for CandleProvider {
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, LLMError> {
        debug!("发送 Candle 聊天请求: {:?}", request);

        self.get_generator().await?;

        let prompt = self.build_prompt(&request.messages);
        let max_tokens = request
            .max_tokens
            .map(|t| t as usize)
            .unwrap_or(self.config.default_max_tokens.unwrap_or(2048));

        debug!("生成提示词长度: {} 字符", prompt.len());

        let mut generator_guard = self
            .generator
            .lock()
            .map_err(|e| LLMError::Internal(format!("无法获取生成器: {}", e)))?;

        let generator = generator_guard
            .as_mut()
            .ok_or_else(|| LLMError::Internal("生成器未初始化".to_string()))?;

        match generator.generate_string(&prompt, max_tokens) {
            Ok(content) => {
                debug!("Candle 生成响应长度: {} 字符", content.len());
                Ok(ChatResponse::new(content.trim(), &request.model)
                    .with_done(true)
                    .with_finish_reason("stop"))
            }
            Err(e) => {
                error!("Candle 生成失败: {}", e);
                Err(LLMError::ApiError(format!("生成失败: {}", e)))
            }
        }
    }

    async fn chat_stream(
        &self,
        request: ChatRequest,
    ) -> Result<Box<dyn Stream<Item = Result<ChatResponse, LLMError>> + Send + Unpin>, LLMError>
    {
        debug!("发送 Candle 流式聊天请求: {:?}", request);

        self.get_generator().await?;

        let prompt = self.build_prompt(&request.messages);
        let max_tokens = request
            .max_tokens
            .map(|t| t as usize)
            .unwrap_or(self.config.default_max_tokens.unwrap_or(2048));

        let model = request.model.clone();
        let generator_mutex = self.generator.clone();

        // 创建异步流
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        // 在后台任务中执行生成
        tokio::spawn(async move {
            // 获取生成器并启动真正的流式生成
            let stream_result = {
                let mut generator_guard = match generator_mutex.lock() {
                    Ok(g) => g,
                    Err(e) => {
                        let _ = tx.send(Err(LLMError::Internal(format!("无法获取生成器: {}", e))));
                        return;
                    }
                };

                let generator = match generator_guard.as_mut() {
                    Some(g) => g,
                    None => {
                        let _ = tx.send(Err(LLMError::Internal("生成器未初始化".to_string())));
                        return;
                    }
                };

                // 使用真正的流式生成方法
                generator.generate_stream(&prompt, max_tokens)
            }; // 锁在这里被释放

            match stream_result {
                Ok(mut stream) => {
                    use tokio_stream::StreamExt;

                    // 处理流式输出的每个片段
                    while let Some(chunk_result) = stream.next().await {
                        match chunk_result {
                            Ok(chunk) => {
                                if !chunk.is_empty() {
                                    let response =
                                        ChatResponse::new(chunk, &model).with_done(false);
                                    if tx.send(Ok(response)).is_err() {
                                        break; // 接收端已关闭
                                    }
                                }
                            }
                            Err(e) => {
                                let _ = tx
                                    .send(Err(LLMError::ApiError(format!("流式生成失败: {}", e))));
                                break;
                            }
                        }
                    }

                    // 发送完成信号
                    let final_response = ChatResponse::new("", &model)
                        .with_done(true)
                        .with_finish_reason("stop");
                    let _ = tx.send(Ok(final_response));
                }
                Err(e) => {
                    let _ = tx.send(Err(LLMError::ApiError(format!("创建流式生成失败: {}", e))));
                }
            }
        });

        // 将接收端转换为流
        let stream = tokio_stream::wrappers::UnboundedReceiverStream::new(rx);
        Ok(Box::new(stream))
    }

    async fn list_models(&self) -> Result<Vec<ModelInfo>, LLMError> {
        debug!("获取 Candle 支持的模型列表");

        let mut models = Vec::new();

        for (name, which_model) in &self.supported_models {
            let display_name = self.get_model_display_name(which_model);
            let model_info = ModelInfo::new(name.clone())
                .with_description(format!("本地 Candle 模型: {}", display_name))
                .with_family("Qwen".to_string())
                .with_parameter_size(
                    match which_model {
                        WhichModel::W0_5b | WhichModel::W2_0_5b => "0.5B",
                        WhichModel::W2_1_5b => "1.5B",
                        WhichModel::W1_8b => "1.8B",
                        WhichModel::W4b | WhichModel::W3_4b => "4B",
                        WhichModel::W7b | WhichModel::W2_7b => "7B",
                        WhichModel::W14b => "14B",
                        WhichModel::W72b | WhichModel::W2_72b => "72B",
                        WhichModel::MoeA27b => "2.7B (MoE)",
                        WhichModel::W3_0_6b => "0.6B",
                        WhichModel::W3_1_7b => "1.7B",
                        WhichModel::W3_8b => "8B",
                    }
                    .to_string(),
                );
            models.push(model_info);
        }

        info!("返回 {} 个 Candle 支持的模型", models.len());
        Ok(models)
    }

    async fn is_model_available(&self, model: &str) -> Result<bool, LLMError> {
        let available = self.supported_models.contains_key(model);
        debug!("检查 Candle 模型 '{}' 可用性: {}", model, available);
        Ok(available)
    }

    fn provider_name(&self) -> &'static str {
        "candle"
    }

    fn default_model(&self) -> &str {
        // 返回默认模型的字符串表示
        match self.config.default_model {
            WhichModel::W0_5b => "qwen-0.5b",
            WhichModel::W1_8b => "qwen-1.8b",
            WhichModel::W4b => "qwen-4b",
            WhichModel::W7b => "qwen-7b",
            WhichModel::W14b => "qwen-14b",
            WhichModel::W72b => "qwen-72b",
            WhichModel::MoeA27b => "qwen-moe-a2.7b",
            WhichModel::W2_0_5b => "qwen2-0.5b",
            WhichModel::W2_1_5b => "qwen2-1.5b",
            WhichModel::W2_7b => "qwen2-7b",
            WhichModel::W2_72b => "qwen2-72b",
            WhichModel::W3_0_6b => "qwen3-0.6b",
            WhichModel::W3_1_7b => "qwen3-1.7b",
            WhichModel::W3_4b => "qwen3-4b",
            WhichModel::W3_8b => "qwen3-8b",
        }
    }

    async fn health_check(&self) -> Result<bool, LLMError> {
        debug!("执行 Candle 健康检查");

        // 尝试初始化生成器作为健康检查
        match self.get_generator().await {
            Ok(_) => {
                info!("Candle 健康检查通过");
                Ok(true)
            }
            Err(e) => {
                warn!("Candle 健康检查失败: {}", e);
                Ok(false) // 不抛出错误，只返回不健康状态
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_candle_provider_creation() {
        let config = CandleConfig::default();
        let result = CandleProvider::new(config);
        assert!(result.is_ok());

        let provider = result.unwrap();
        assert_eq!(provider.provider_name(), "candle");
        assert_eq!(provider.default_model(), "qwen-0.5b");
    }

    #[tokio::test]
    async fn test_model_availability() {
        let provider = CandleProvider::with_default().unwrap();

        // 测试支持的模型
        let available = provider.is_model_available("qwen-0.5b").await.unwrap();
        assert!(available);

        // 测试不支持的模型
        let unavailable = provider
            .is_model_available("unsupported-model")
            .await
            .unwrap();
        assert!(!unavailable);
    }

    #[tokio::test]
    async fn test_list_models() {
        let provider = CandleProvider::with_default().unwrap();
        let models = provider.list_models().await.unwrap();

        assert!(!models.is_empty());
        assert!(models.iter().any(|m| m.name == "qwen-0.5b"));
    }

    #[test]
    fn test_prompt_building() {
        let provider = CandleProvider::with_default().unwrap();
        let messages = vec![
            ChatMessage::user("Hello"),
            ChatMessage::assistant("Hi there!"),
            ChatMessage::user("How are you?"),
        ];

        let prompt = provider.build_prompt(&messages);
        assert!(prompt.contains("User: Hello"));
        assert!(prompt.contains("Assistant: Hi there!"));
        assert!(prompt.contains("User: How are you?"));
        assert!(prompt.ends_with("Assistant: "));
    }

    #[test]
    fn test_supported_models_mapping() {
        let models = CandleProvider::build_supported_models();
        assert!(models.contains_key("qwen-0.5b"));
        assert!(models.contains_key("qwen2-7b"));
        assert_eq!(models.get("qwen-0.5b"), Some(&WhichModel::W0_5b));
    }
}
