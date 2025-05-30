use super::{ChatMessage, ChatRequest, ChatResponse, LLMProvider};
use async_trait::async_trait;
use candle_core::{Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::llama::{Llama, LlamaConfig};
use hf_hub::api::tokio::Api;
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_stream::{wrappers::ReceiverStream, Stream, StreamExt};
use tokenizers::Tokenizer;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandleConfig {
    pub model_path: Option<PathBuf>,
    pub model_repo: String,
    pub model_file: String,
    pub tokenizer_file: String,
    pub max_seq_len: usize,
    pub temperature: f32,
    pub top_p: f32,
    pub repeat_penalty: f32,
    pub repeat_last_n: usize,
    pub device: String, // "cpu" or "cuda" or "metal"
}

impl Default for CandleConfig {
    fn default() -> Self {
        Self {
            model_path: None,
            model_repo: "microsoft/DialoGPT-medium".to_string(),
            model_file: "pytorch_model.bin".to_string(),
            tokenizer_file: "tokenizer.json".to_string(),
            max_seq_len: 2048,
            temperature: 0.7,
            top_p: 0.9,
            repeat_penalty: 1.1,
            repeat_last_n: 64,
            device: "cpu".to_string(),
        }
    }
}

pub struct CandleLLM {
    config: CandleConfig,
    model: Arc<Mutex<Option<Llama>>>,
    tokenizer: Arc<Mutex<Option<Tokenizer>>>,
    device: Device,
    available_models: HashMap<String, String>,
}

impl CandleLLM {
    pub fn new(config: CandleConfig) -> Result<Self, Box<dyn Error + Send + Sync>> {
        let device = match config.device.as_str() {
            "cuda" => Device::new_cuda(0)?,
            "metal" => Device::new_metal(0)?,
            _ => Device::Cpu,
        };

        let mut available_models = HashMap::new();
        available_models.insert("llama2-7b".to_string(), "microsoft/DialoGPT-medium".to_string());
        available_models.insert("llama2-13b".to_string(), "microsoft/DialoGPT-large".to_string());

        Ok(Self {
            config,
            model: Arc::new(Mutex::new(None)),
            tokenizer: Arc::new(Mutex::new(None)),
            device,
            available_models,
        })
    }

    /// 初始化模型和分词器
    async fn initialize_model(&self, model_name: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut model_guard = self.model.lock().await;
        let mut tokenizer_guard = self.tokenizer.lock().await;

        if model_guard.is_some() && tokenizer_guard.is_some() {
            return Ok(()); // 已经初始化
        }

        info!("正在初始化 Candle 模型: {}", model_name);

        // 从 Hugging Face Hub 下载模型
        let api = Api::new()?;
        let repo = api.model(self.config.model_repo.clone());

        // 下载分词器
        let tokenizer_path = repo.get(&self.config.tokenizer_file).await?;
        let tokenizer = Tokenizer::from_file(tokenizer_path)?;

        // 下载模型配置和权重
        let config_path = repo.get("config.json").await?;
        let model_config: LlamaConfig = serde_json::from_slice(&std::fs::read(config_path)?)?;

        let model_path = repo.get(&self.config.model_file).await?;
        let vb = unsafe { VarBuilder::from_mmaped_safetensors(&[model_path], candle_core::DType::F32, &self.device)? };

        // 创建模型
        let model = Llama::load(&vb, &model_config)?;

        *model_guard = Some(model);
        *tokenizer_guard = Some(tokenizer);

        info!("Candle 模型初始化完成");
        Ok(())
    }

    /// 生成文本
    async fn generate_text(&self, prompt: &str, stream: bool) -> Result<String, Box<dyn Error + Send + Sync>> {
        self.initialize_model("default").await?;

        let model_guard = self.model.lock().await;
        let tokenizer_guard = self.tokenizer.lock().await;

        let model = model_guard.as_ref().ok_or("模型未初始化")?;
        let tokenizer = tokenizer_guard.as_ref().ok_or("分词器未初始化")?;

        // 编码输入
        let tokens = tokenizer.encode(prompt, false)
            .map_err(|e| format!("分词失败: {}", e))?;
        let input_ids = tokens.get_ids();

        // 转换为张量
        let input_tensor = Tensor::new(input_ids, &self.device)?
            .unsqueeze(0)?;

        // 生成文本
        let mut generated_tokens = input_ids.to_vec();
        let mut output_text = String::new();

        // 初始化KV缓存
        let mut kv_cache = None;
        
        for _ in 0..100 { // 最大生成100个token
            let input_len = generated_tokens.len();
            let current_input = Tensor::new(&generated_tokens[..], &self.device)?
                .unsqueeze(0)?;

            // 前向传播，提供KV缓存和起始位置
            let start_pos = if kv_cache.is_some() { input_len - 1 } else { 0 };
            let logits = model.forward(&current_input, kv_cache.as_mut(), start_pos)?;
            let logits = logits.squeeze(0)?;
            
            // 获取最后一个位置的logits
            let last_logits = logits.get(input_len - 1)?;
            
            // 应用温度采样
            let next_token = self.sample_token(&last_logits)?;
            
            // 检查是否为结束token
            if next_token == tokenizer.token_to_id("<|endoftext|>").unwrap_or(0) {
                break;
            }

            generated_tokens.push(next_token);
            
            // 解码新生成的token
            if let Ok(decoded) = tokenizer.decode(&[next_token], false) {
                output_text.push_str(&decoded);
                
                if stream {
                    // 在流式模式下，这里可以发送中间结果
                    // 但由于接口限制，我们只能在最后返回完整结果
                }
            }
        }

        Ok(output_text)
    }

    /// 采样下一个token
    fn sample_token(&self, logits: &Tensor) -> Result<u32, Box<dyn Error + Send + Sync>> {
        // 应用温度
        let logits = (logits / self.config.temperature as f64)?;
        
        // 简单的贪婪采样 - 选择概率最高的token
        let probabilities = candle_nn::ops::softmax(&logits, 0)?;
        let probabilities_vec = probabilities.to_vec1::<f32>()?;
        
        let max_idx = probabilities_vec
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(idx, _)| idx)
            .unwrap_or(0);

        Ok(max_idx as u32)
    }

    /// 将聊天消息转换为提示文本
    fn messages_to_prompt(&self, messages: &[ChatMessage]) -> String {
        let mut prompt = String::new();
        for message in messages {
            match message.role.as_str() {
                "system" => prompt.push_str(&format!("System: {}\n", message.content)),
                "user" => prompt.push_str(&format!("Human: {}\n", message.content)),
                "assistant" => prompt.push_str(&format!("Assistant: {}\n", message.content)),
                _ => prompt.push_str(&format!("{}: {}\n", message.role, message.content)),
            }
        }
        prompt.push_str("Assistant: ");
        prompt
    }
}

#[async_trait]
impl LLMProvider for CandleLLM {
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, Box<dyn Error + Send + Sync>> {
        let prompt = self.messages_to_prompt(&request.messages);
        
        match self.generate_text(&prompt, false).await {
            Ok(content) => Ok(ChatResponse {
                content,
                done: true,
                model: request.model,
            }),
            Err(e) => {
                error!("Candle 聊天生成失败: {}", e);
                Err(e)
            }
        }
    }

    async fn chat_stream(&self, request: ChatRequest) -> Result<Box<dyn Stream<Item = Result<ChatResponse, Box<dyn Error + Send + Sync>>> + Send + Unpin>, Box<dyn Error + Send + Sync>> {
        let prompt = self.messages_to_prompt(&request.messages);
        let model_name = request.model.clone();
        
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        
        // 在后台任务中生成流式响应
        let self_clone = self.clone();
        tokio::spawn(async move {
            match self_clone.generate_text(&prompt, true).await {
                Ok(content) => {
                    // 模拟流式输出，将完整内容分割发送
                    let words: Vec<&str> = content.split_whitespace().collect();
                    let mut current_content = String::new();
                    
                    for word in words {
                        current_content.push_str(word);
                        current_content.push(' ');
                        
                        let response = ChatResponse {
                            content: current_content.clone(),
                            done: false,
                            model: model_name.clone(),
                        };
                        
                        if tx.send(Ok(response)).await.is_err() {
                            break;
                        }
                        
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    }
                    
                    // 发送最终响应
                    let final_response = ChatResponse {
                        content: current_content,
                        done: true,
                        model: model_name,
                    };
                    let _ = tx.send(Ok(final_response)).await;
                }
                Err(e) => {
                    let _ = tx.send(Err(e)).await;
                }
            }
        });

        Ok(Box::new(ReceiverStream::new(rx)))
    }

    async fn list_models(&self) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
        Ok(self.available_models.keys().cloned().collect())
    }

    async fn is_model_available(&self, model: &str) -> Result<bool, Box<dyn Error + Send + Sync>> {
        Ok(self.available_models.contains_key(model))
    }

    fn provider_name(&self) -> &'static str {
        "candle"
    }
}

// 为了支持 clone，我们需要手动实现
impl Clone for CandleLLM {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            model: Arc::new(Mutex::new(None)), // 新实例需要重新初始化
            tokenizer: Arc::new(Mutex::new(None)),
            device: self.device.clone(),
            available_models: self.available_models.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_candle_llm_creation() {
        let config = CandleConfig::default();
        let llm = CandleLLM::new(config);
        assert!(llm.is_ok());
    }

    #[tokio::test]
    async fn test_list_models() {
        let config = CandleConfig::default();
        let llm = CandleLLM::new(config).unwrap();
        let models = llm.list_models().await.unwrap();
        assert!(!models.is_empty());
    }
}