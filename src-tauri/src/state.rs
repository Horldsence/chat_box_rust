use crate::models::{Conversation, Message};
use crate::services::agent::ollama::OllamaAgent;
use crate::services::asr::vosk_python::VoskASR;
use crate::services::database::ChatDatabase;
use log::{error, info};
use std::sync::{Arc, Mutex};

pub struct AppState {
    pub conversations: Arc<Mutex<Vec<Conversation>>>,
    pub messages: Arc<Mutex<Vec<Message>>>,
    pub ollama_agent: Arc<OllamaAgent>,
    pub vosk_asr: Arc<tokio::sync::Mutex<VoskASR>>,
    pub db: Arc<Mutex<Option<ChatDatabase>>>, // 添加数据库支持
}

impl AppState {
    pub fn new(
        conversations: Vec<Conversation>,
        messages: Vec<Message>,
        ollama_agent: OllamaAgent,
        vosk_asr: VoskASR,
    ) -> Self {
        AppState {
            conversations: Arc::new(Mutex::new(conversations)),
            messages: Arc::new(Mutex::new(messages)),
            ollama_agent: Arc::new(ollama_agent),
            vosk_asr: Arc::new(tokio::sync::Mutex::new(vosk_asr)),
            db: Arc::new(Mutex::new(None)), // 初始时数据库为None
        }
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
