// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod models;
mod services;
mod state;
mod utils;

use agent::OllamaAgent;
use chrono::Utc;
use initialize::{initialize_app, InitConfig};
use log::{error, info, warn};
use models::{Conversation, Message};
use services::asr::vosk_python::VoskASR;
use state::AppState;
use std::path::Path;
use tauri::path::BaseDirectory;
use tauri::Manager;
use utils::config::{get_app_config, save_app_config, AppConfig};
use utils::logger::init_logger; // 导入配置相关函数

// 导入所有命令
use commands::*;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let handle = app.handle().clone();

            // Initialize app using the new initialization system
            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async move {
                    match init_with_new_system(handle.clone()).await {
                        Ok(app_state) => {
                            handle.manage(app_state);
                            info!("App state initialized successfully");
                        }
                        Err(e) => {
                            error!("初始化失败: {}", e);
                            panic!("Failed to initialize app: {}", e);
                        }
                    }
                });
            });

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

async fn init_with_new_system(
    handle: tauri::AppHandle,
) -> Result<AppState, Box<dyn std::error::Error>> {
    // 获取配置文件路径
    let config_path = handle
        .path()
        .resolve("config.yaml", BaseDirectory::Resource)
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

    // 使用新的初始化系统加载配置
    let init_config = InitConfig::new(config_path.clone()).load_config();

    // 设置日志级别
    init_logger();
    info!("应用启动，开始新的初始化流程");

    // 使用新的初始化系统
    match initialize_app(init_config.clone(), handle.clone()).await {
        Ok(result) => {
            if !result.success {
                warn!("初始化完成，但有组件失败或被忽略");
                for component in &result.failed_components {
                    warn!("失败组件: {}", component);
                }
                for component in &result.ignored_components {
                    warn!("忽略组件: {}", component);
                }
            }
        }
        Err(e) => {
            error!("初始化系统执行失败: {}", e);
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            )));
        }
    }

    // 转换为旧的 AppConfig 格式以保持兼容性
    let config = convert_init_config_to_app_config(init_config);

    // 创建 OllamaAgent 实例
    let ollama_agent = OllamaAgent::new(
        &config.ai_model.model_name,
        &config.ai_model.server_url,
        &config.ai_model.server_port,
    )
    .with_system_prompt(&config.ai_model.system_prompt);

    info!("OllamaAgent initialized");

    // 创建 Vosk ASR 实例
    let vosk_asr = create_vosk_asr(&config, &handle)?;

    let default_conversation_id = 1;

    // 初始化应用状态
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

    let state = AppState::new(config.clone(), conversations, messages, vosk_asr).await?;

    // 初始化数据库
    if config.database.enabled {
        let db_path = resolve_database_path(&config, &handle)?;
        initialize_database(&state, &db_path)?;
    }

    Ok(state)
}

fn convert_init_config_to_app_config(init_config: InitConfig) -> AppConfig {
    AppConfig {
        config_path: init_config.config_path,
        ai_model: utils::config::AiModelConfig {
            model_type: init_config.ai_model.model_type,
            model_name: init_config.ai_model.model_name,
            server_url: init_config.ai_model.server_url,
            server_port: init_config.ai_model.server_port,
            system_prompt: init_config.ai_model.system_prompt,
            candle_model_id: init_config.ai_model.candle_model_id,
            candle_revision: init_config.ai_model.candle_revision,
            candle_use_flash_attn: init_config.ai_model.candle_use_flash_attn,
        },
        voice: utils::config::VoiceConfig {
            enabled: init_config.voice.enabled,
            model_path: init_config.voice.model_path,
            timeout_seconds: init_config.voice.timeout_seconds,
        },
        ui: utils::config::UiConfig {
            theme: init_config.ui.theme,
            language: init_config.ui.language,
        },
        database: utils::config::DatabaseConfig {
            enabled: init_config.database.enabled,
            path: init_config.database.path,
        },
        app_behavior: utils::config::AppBehaviorConfig {
            log_level: init_config.app_behavior.log_level,
            default_conversation_title: init_config.app_behavior.default_conversation_title,
            welcome_message: init_config.app_behavior.welcome_message,
            message_chunk_buffer_size: init_config.app_behavior.message_chunk_buffer_size,
            message_chunk_send_interval_ms: init_config.app_behavior.message_chunk_send_interval_ms,
            show_error_dialogs: init_config.app_behavior.show_error_dialogs,
            auto_retry_failed_init: init_config.app_behavior.auto_retry_failed_init,
        },
    }
}

fn create_vosk_asr(
    config: &AppConfig,
    handle: &tauri::AppHandle,
) -> Result<VoskASR, Box<dyn std::error::Error>> {
    let model_path = if config.voice.enabled {
        if Path::new(&config.voice.model_path).is_absolute() {
            config.voice.model_path.clone()
        } else {
            handle
                .path()
                .resolve(&config.voice.model_path, BaseDirectory::Resource)
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?
                .to_string_lossy()
                .to_string()
        }
    } else {
        handle
            .path()
            .resolve("model/vosk-model-small-cn-0.22", BaseDirectory::Resource)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?
            .to_string_lossy()
            .to_string()
    };

    info!("Vosk model path: {:?}", model_path);

    VoskASR::new(Some(&model_path)).map_err(|e| {
        error!("VoskASR initialization failed: {}", e);
        Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            e.to_string(),
        )) as Box<dyn std::error::Error>
    })
}

fn resolve_database_path(
    config: &AppConfig,
    handle: &tauri::AppHandle,
) -> Result<String, Box<dyn std::error::Error>> {
    if Path::new(&config.database.path).is_absolute() {
        Ok(config.database.path.clone())
    } else {
        Ok(handle
            .path()
            .resolve(&config.database.path, BaseDirectory::AppData)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?
            .to_string_lossy()
            .to_string())
    }
}

fn initialize_database(state: &AppState, db_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // 确保数据库目录存在
    if let Some(parent) = Path::new(db_path).parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("无法创建数据库目录: {}", e),
            )) as Box<dyn std::error::Error>
        })?;
    }

    // 初始化数据库
    if let Err(e) = state.init_database(db_path) {
        error!("数据库初始化失败: {}", e);
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("数据库初始化失败: {}", e),
        )));
    }

    // 从数据库加载数据
    if let Err(e) = state.load_from_database() {
        error!("从数据库加载数据失败: {}", e);
        // 这里不返回错误，因为数据库可能是空的
    }

    Ok(())
}
