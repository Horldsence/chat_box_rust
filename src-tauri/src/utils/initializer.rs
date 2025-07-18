use crate::services;
use crate::state;
use db::models;

use cb_config::AppConfig;
use chrono::Utc;
use initialize::InitConfig;
use log::{error, info};
use models::{Conversation, Message};
use services::asr::vosk_python::VoskASR;
use state::AppState;
use std::path::Path;
use tauri::path::BaseDirectory;
use tauri::Manager;

pub async fn init_app_state(
    handle: tauri::AppHandle,
) -> Result<AppState, Box<dyn std::error::Error>> {
    // 获取配置文件路径
    let config_path = handle
        .path()
        .resolve("config.yaml", BaseDirectory::Resource)
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

    // 使用新的初始化系统加载配置
    let init_config = InitConfig::new(config_path.clone()).load_config();

    info!("应用启动，开始新的初始化流程");

    // 注：条件初始化已在 conditional_initialize_app 中完成
    // 这里只需要加载配置和创建应用状态

    // 转换为旧的 AppConfig 格式以保持兼容性
    let config = convert_init_config_to_app_config(init_config);

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

    #[cfg(feature = "tts")]
    // 创建 TtsEngine 实例
    let tts_engine = tts::kokoro_tts::TtsEngine::new(
            config
                .tts_engine
                .model_path
                .to_str()
                .ok_or("Invalid model path")?.to_string(),
            config
                .tts_engine
                .voice_path
                .to_str()
                .ok_or("Invalid voice path")?.to_string(),
        )?;

    let state = AppState::new(
        config.clone(),
        conversations,
        messages,
        vosk_asr,
        handle.clone(),
        #[cfg(feature = "tts")]
        Some(tts_engine),
    )
    .await?;

    // 初始化数据库
    if config.database.enabled {
        let db_path = resolve_database_path(&config, &handle)?;
        initialize_database(&state, &db_path)?;
    }

    Ok(state)
}

pub fn convert_init_config_to_app_config(init_config: InitConfig) -> AppConfig {
    AppConfig {
        config_path: init_config.config_path,
        ai_model: cb_config::AiModelConfig {
            model_type: init_config.ai_model.model_type,
            model_name: init_config.ai_model.model_name,
            server_url: init_config.ai_model.server_url,
            server_port: init_config.ai_model.server_port,
            system_prompt: init_config.ai_model.system_prompt,
            candle_model_id: init_config.ai_model.candle_model_id,
            candle_revision: init_config.ai_model.candle_revision,
            candle_use_flash_attn: init_config.ai_model.candle_use_flash_attn,
        },
        voice: cb_config::VoiceConfig {
            enabled: init_config.voice.enabled,
            model_path: init_config.voice.model_path,
            timeout_seconds: init_config.voice.timeout_seconds,
        },
        tts_engine: cb_config::TtsConfig {
            enabled: init_config.tts_engine.enabled,
            model_path: init_config.tts_engine.model_path,
            voice_path: init_config.tts_engine.voice_path,
        },
        ui: cb_config::UiConfig {
            theme: init_config.ui.theme,
            language: init_config.ui.language,
        },
        database: cb_config::DatabaseConfig {
            enabled: init_config.database.enabled,
            path: init_config.database.path,
        },
        app_behavior: cb_config::AppBehaviorConfig {
            log_level: init_config.app_behavior.log_level,
            default_conversation_title: init_config.app_behavior.default_conversation_title,
            welcome_message: init_config.app_behavior.welcome_message,
            message_chunk_buffer_size: init_config.app_behavior.message_chunk_buffer_size,
            message_chunk_send_interval_ms: init_config.app_behavior.message_chunk_send_interval_ms,
            show_error_dialogs: init_config.app_behavior.show_error_dialogs,
            auto_retry_failed_init: init_config.app_behavior.auto_retry_failed_init,
        },
        live2d: cb_config::Live2DConfig {
            enabled: true,
            model_path: "models/live2d/hiyori/hiyori_free_en.model3.json".to_string(),
            model_name: "Hiyori".to_string(),
            scale: 1.0,
            position_x: 0.0,
            position_y: 0.0,
            auto_blink: true,
            auto_breath: true,
            check_model_on_startup: true,
            fallback_to_simple_character: true,
        },
    }
}

pub fn create_vosk_asr(
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

pub fn resolve_database_path(
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

pub fn initialize_database(
    state: &AppState,
    db_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
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
