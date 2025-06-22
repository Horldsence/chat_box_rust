use crate::models::{Conversation, Message};
use crate::services::agent::AgentService;
use crate::services::asr::vosk_python::VoskASR;
use crate::services::database::ChatDatabase;
use crate::utils::config::AppConfig;
#[cfg(feature = "candle")]
use agent::models::llm::{CandleConfig, WhichModel};
use agent::{LLMManager, LLMManagerConfig, OllamaConfig, ProviderConfig};
use live2d::live2d::Live2DService;
use log::{error, info};
use std::collections::HashMap;

use std::sync::{Arc, Mutex};

pub struct AppState {
    pub config: Arc<Mutex<AppConfig>>,
    pub conversations: Arc<Mutex<Vec<Conversation>>>,
    pub messages: Arc<Mutex<Vec<Message>>>,
    pub llm_manager: Arc<LLMManager>,
    pub vosk_asr: Arc<tokio::sync::Mutex<VoskASR>>,
    pub db: Arc<Mutex<Option<ChatDatabase>>>, // 添加数据库支持
    pub live2d_service: Arc<tokio::sync::Mutex<Live2DService>>, // 添加Live2D服务
    pub agent_service: Arc<tokio::sync::Mutex<AgentService>>, // 添加Agent服务
}

#[allow(dead_code)]
impl AppState {
    pub async fn new(
        config: AppConfig,
        conversations: Vec<Conversation>,
        messages: Vec<Message>,
        vosk_asr: VoskASR,
        app_handle: tauri::AppHandle,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // 创建 LLM 管理器配置
        let mut providers = HashMap::new();

        let llm_manager = if config.ai_model.model_type == "candle" {
            #[cfg(feature = "candle")]
            {
                let candle_config = CandleConfig {
                    default_model: WhichModel::W0_5b, // 使用默认的小模型
                    default_temperature: Some(0.7),
                    default_max_tokens: Some(2048),
                    cpu_only: true,
                    repeat_penalty: 1.1,
                    repeat_last_n: 64,
                    seed: 299792458,
                    system_prompt: Some(config.ai_model.system_prompt.clone()),
                };

                providers.insert("candle".to_string(), ProviderConfig::Candle(candle_config));

                let llm_manager_config = LLMManagerConfig {
                    default_provider: "candle".to_string(),
                    fallback_providers: vec![],
                    auto_fallback: true,
                    health_check_interval_seconds: 300,
                    providers,
                };

                LLMManager::new(llm_manager_config).await?
            }
            #[cfg(not(feature = "candle"))]
            {
                return Err("Candle feature not enabled".into());
            }
        } else {
            // 从应用配置中获取 Ollama 配置
            let ollama_config = OllamaConfig {
                host: config.ai_model.server_url.clone(),
                port: config.ai_model.server_port,
                default_model: config.ai_model.model_name.clone(),
                default_temperature: Some(0.7),
                default_max_tokens: Some(2048),
                timeout_seconds: Some(30),
                system_prompt: Some(config.ai_model.system_prompt.clone()),
            };

            providers.insert("ollama".to_string(), ProviderConfig::Ollama(ollama_config));

            let llm_manager_config = LLMManagerConfig {
                default_provider: "ollama".to_string(),
                fallback_providers: vec![],
                auto_fallback: true,
                health_check_interval_seconds: 300,
                providers,
            };

            LLMManager::new(llm_manager_config).await?
        };

        // 创建Live2D服务
        let live2d_service = Live2DService::new(app_handle.clone());

        // 创建Agent服务
        let agent_service = AgentService::new(app_handle);

        Ok(AppState {
            config: Arc::new(Mutex::new(config)),
            conversations: Arc::new(Mutex::new(conversations)),
            messages: Arc::new(Mutex::new(messages)),
            llm_manager: Arc::new(llm_manager),
            vosk_asr: Arc::new(tokio::sync::Mutex::new(vosk_asr)),
            db: Arc::new(Mutex::new(None)), // 初始时数据库为None
            live2d_service: Arc::new(tokio::sync::Mutex::new(live2d_service)),
            agent_service: Arc::new(tokio::sync::Mutex::new(agent_service)),
        })
    }

    // 初始化数据库
    pub fn init_database(&self, db_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        match ChatDatabase::new(db_path) {
            Ok(db) => {
                let mut db_guard = self.db.lock().unwrap();
                *db_guard = Some(db);
                info!("Database initialized at: {}", db_path);
                Ok(())
            }
            Err(e) => {
                error!("Failed to initialize database: {}", e);
                Err(Box::new(e))
            }
        }
    }

    // 从数据库加载所有对话和消息
    pub fn load_from_database(&self) -> Result<(), Box<dyn std::error::Error>> {
        let db_guard = self.db.lock().unwrap();
        if let Some(ref db) = *db_guard {
            // 加载所有对话
            let conversations = db.get_all_conversations()?;
            {
                let mut conv_guard = self.conversations.lock().unwrap();
                *conv_guard = conversations.clone();
            }

            // 加载所有对话的消息
            let mut all_messages = Vec::new();
            for conv in &conversations {
                let msgs = db.get_conversation_messages(conv.id)?;
                all_messages.extend(msgs);
            }

            {
                let mut msg_guard = self.messages.lock().unwrap();
                *msg_guard = all_messages;
            }

            info!("加载了{}个对话和相关消息", conversations.len());
            Ok(())
        } else {
            Err("Database not initialized".into())
        }
    }

    // 保存所有对话和消息到数据库
    pub fn save_to_database(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut db_guard = self.db.lock().unwrap();
        if let Some(ref mut db) = *db_guard {
            // 保存所有对话
            {
                let conv_guard = self.conversations.lock().unwrap();
                for conv in conv_guard.iter() {
                    db.save_conversation(conv)?;
                }
                info!("保存了{}个对话", conv_guard.len());
            }

            // 保存所有消息
            {
                let msg_guard = self.messages.lock().unwrap();
                db.save_messages(&msg_guard)?;
                info!("保存了{}条消息", msg_guard.len());
            }

            Ok(())
        } else {
            Err("Database not initialized".into())
        }
    }

    // 获取特定对话的历史记录
    pub fn get_conversation_history(&self, conversation_id: u64) -> Vec<Message> {
        let msg_guard = self.messages.lock().unwrap();
        msg_guard
            .iter()
            .filter(|msg| msg.conversation_id == conversation_id)
            .cloned()
            .collect()
    }
}
