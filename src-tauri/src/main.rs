// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod services;
mod models;
mod state;
mod utils;

use chrono::Utc;
use services::agent::ollama::OllamaAgent;
use services::asr::vosk_python::VoskASR;
use log::{error, info};
use models::{Conversation, Message};
use state::AppState;
use utils::logger::init_logger;

// 导入所有命令
use commands::*;

#[tokio::main]
async fn main() {
    init_logger(); // 初始化日志记录器

    // 创建OllamaAgent实例
    let ollama_agent = OllamaAgent::new("qwen2.5:0.5b", "http://localhost", &11434)
        .with_system_prompt("你是一个友好、乐于助人的AI助手，使用中文回答问题。");

    info!("OllamaAgent initialized");

    // 创建Vosk ASR实例
    let vosk_asr = match VoskASR::new(Some("model/vosk-model-small-cn-0.22")) {
        Ok(asr) => asr,
        Err(e) => {
            error!("VoskASR initialization failed: {}", e);
            return;
        }
    };

    let default_conversation_id = 1;

    // 初始化应用状态
    let conversations = vec![Conversation {
        id: default_conversation_id,
        title: "新对话".to_string(),
        last_message: "你好!".to_string(),
        timestamp: Utc::now().timestamp_millis() as u64,
    }];

    let messages = vec![Message {
        id: 1,
        content: "欢迎使用聊天应用!".to_string(),
        sender: "bot".to_string(),
        timestamp: Utc::now().timestamp_millis() as u64,
        conversation_id: default_conversation_id,
    }];

    let state = AppState::new(conversations, messages, ollama_agent, vosk_asr);

    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            // 对话相关命令
            get_conversations,
            get_conversation_messages,
            create_conversation,
            delete_conversation,
            // 消息相关命令
            send_user_message,
            // AI相关命令
            generate_ai_response,
            // 语音相关命令
            voice_input,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
