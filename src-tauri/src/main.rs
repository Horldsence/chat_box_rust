// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod models;
mod services;
mod state;
mod utils;

use std::path::Path;
use std::sync::Arc;

use chrono::Utc;
use log::{error, info};
use models::{Conversation, Message};
use services::agent::ollama::OllamaAgent;
use services::asr::vosk_python::VoskASR;
use state::AppState;
use utils::logger::init_logger; // 导入配置相关函数
use utils::config::{get_app_config, load_config, save_app_config};

// 导入所有命令
use commands::*;

#[tokio::main]
async fn main() {
    // 加载配置
    let config = load_config();

    // 设置日志级别（从配置中读取）
    init_logger();
    info!("应用启动，配置加载完成");

    // 创建OllamaAgent实例（使用配置中的值）
    let ollama_agent = OllamaAgent::new(
        &config.ai_model.model_name,
        &config.ai_model.server_url,
        &config.ai_model.server_port,
    )
    .with_system_prompt(&config.ai_model.system_prompt);

    info!("OllamaAgent initialized");

    // 创建Vosk ASR实例（使用配置中的值）
    let vosk_asr = if config.voice.enabled {
        match VoskASR::new(Some(&config.voice.model_path)) {
            Ok(asr) => asr,
            Err(e) => {
                error!("VoskASR initialization failed: {}", e);
                return;
            }
        }
    } else {
        match VoskASR::new(None) {
            Ok(asr) => asr,
            Err(e) => {
                error!("VoskASR initialization failed: {}", e);
                return;
            }
        }
    };

    let default_conversation_id = 1;

    // 初始化应用状态（使用配置中的值）
    let conversations = vec![Conversation {
        id: default_conversation_id,
        title: config.app_behavior.default_conversation_title.clone(),
        last_message: "你好!".to_string(),
        timestamp: Utc::now().timestamp_millis() as u64,
    }];

    let messages = vec![Message {
        id: 1,
        content: config.app_behavior.welcome_message.clone(),
        sender: "bot".to_string(),
        timestamp: Utc::now().timestamp_millis() as u64,
        conversation_id: default_conversation_id,
    }];

    let state = AppState::new(conversations, messages, ollama_agent, vosk_asr);
    let config_clone = Arc::new(config); // 将配置共享到状态中

    // 初始化数据库
    let db_path = "database/chat_database.db".to_string();
    if let Some(parent) = Path::new(&db_path).parent() {
        std::fs::create_dir_all(parent).expect("无法创建数据库目录");
    }
    if let Err(e) = state.init_database(&db_path) {
        error!("数据库初始化失败: {}", e);
    } else {
        // 从数据库加载对话和消息
        if let Err(e) = state.load_from_database() {
            error!("从数据库加载数据失败: {}", e);
        }
    }

    tauri::Builder::default()
        .manage(state)
        .manage(config_clone) // 将配置添加到应用状态
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
            // 配置相关命令
            get_app_config,
            save_app_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
