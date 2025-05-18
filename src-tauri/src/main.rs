// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod models;
mod services;
mod state;
mod utils;

use chrono::Utc;
use log::{error, info};
use models::{Conversation, Message};
use services::agent::ollama::OllamaAgent;
use services::asr::vosk_python::VoskASR;
use state::AppState;
use std::path::Path;
use tauri::path::BaseDirectory;
use tauri::Manager;
use utils::config::{get_app_config, save_app_config, AppConfig};
use utils::logger::init_logger; // 导入配置相关函数

// 导入所有命令
use commands::*;

#[tokio::main]
async fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let app_state = match init_config(app.handle().clone()) {
                Ok(state) => state,
                Err(e) => {
                    error!("初始化配置失败: {}", e);
                    return Err(e.into());
                }
            };
            app.manage(app_state);
            Ok(())
        })
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
            // 数据库管理命令
            get_database_conversations,
            delete_database_conversation,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn init_config(handle: tauri::AppHandle) -> Result<AppState, std::io::Error> {
    // TODO:添加资源检查
    // check_resource(handle.path().resource_dir())
    //     .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
    // 获取设置目录
    let config_path = handle
        .path()
        .resolve("config.yaml", BaseDirectory::Resource)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
    // 加载配置
    let config = AppConfig::new(config_path.clone()).load_config();

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
        let model_path = if Path::new(&config.voice.model_path).is_absolute() {
            config.voice.model_path.clone()
        } else {
            // 使用相对路径，根据应用资源目录解析
            handle
                .path()
                .resolve(&config.voice.model_path, BaseDirectory::Resource)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?
                .to_string_lossy()
                .to_string()
        };

        match VoskASR::new(Some(&model_path)) {
            Ok(asr) => asr,
            Err(e) => {
                error!("VoskASR initialization failed: {}", e);
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    e.to_string(),
                ));
            }
        }
    } else {
        let vosk_model_path = handle
            .path()
            .resolve("model/vosk-model-small-cn-0.22", BaseDirectory::Resource)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?
            .to_string_lossy()
            .to_string();
        info!("Vosk model path: {:?}", vosk_model_path);
        match VoskASR::new(Some(&vosk_model_path)) {
            Ok(asr) => asr,
            Err(e) => {
                error!("VoskASR initialization failed: {}", e);
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    e.to_string(),
                ));
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

    let state = AppState::new(
        config.clone(),
        conversations,
        messages,
        ollama_agent,
        vosk_asr,
    );

    // 初始化数据库
    if config.database.enabled {
        let db_path = if Path::new(&config.database.path).is_absolute() {
            config.database.path.clone()
        } else {
            handle
                .path()
                .resolve(&config.database.path, BaseDirectory::Resource)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?
                .to_string_lossy()
                .to_string()
        };
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
    }

    Ok(state)
}
