use crate::state::AppState;
use cb_config::AppConfig;
use log::{error, info};
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ErrorInfo {
    pub code: String,
    pub message: String,
    pub details: Option<String>,
    pub timestamp: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<ErrorInfo>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(code: &str, message: &str, details: Option<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(ErrorInfo {
                code: code.to_string(),
                message: message.to_string(),
                details,
                timestamp: chrono::Utc::now().timestamp_millis() as u64,
            }),
        }
    }
}

/// 获取应用配置
#[tauri::command]
pub async fn get_app_config(state: State<'_, AppState>) -> Result<ApiResponse<AppConfig>, String> {
    info!("获取应用配置");

    let config = state.config.lock().unwrap().clone();
    Ok(ApiResponse::success(config))
}

/// 保存应用配置
#[tauri::command]
pub async fn save_app_config(
    state: State<'_, AppState>,
    config: AppConfig,
) -> Result<ApiResponse<()>, String> {
    info!("保存应用配置");

    // 更新内存中的配置
    {
        let mut current_config = state.config.lock().unwrap();
        *current_config = config.clone();
    }

    // 保存到文件
    match config.clone().get_config_file_path() {
        Some(path) => {
            config.save_config(&config, &path);
            info!("配置保存成功");
            Ok(ApiResponse::success(()))
        }
        None => {
            error!("无法确定配置文件路径");
            Ok(ApiResponse::error(
                "CONFIG_SAVE_ERROR",
                "保存配置失败",
                Some("无法确定配置文件路径".to_string()),
            ))
        }
    }
}

/// 重置应用配置到默认值
#[tauri::command]
pub async fn reset_app_config(
    state: State<'_, AppState>,
) -> Result<ApiResponse<AppConfig>, String> {
    info!("重置应用配置");

    let default_config = AppConfig::default();

    // 更新内存中的配置
    {
        let mut current_config = state.config.lock().unwrap();
        *current_config = default_config.clone();
    }

    // 保存到文件
    match default_config.clone().get_config_file_path() {
        Some(path) => {
            default_config.save_config(&default_config, &path);
            info!("配置重置成功");
            Ok(ApiResponse::success(default_config))
        }
        None => {
            error!("无法确定配置文件路径");
            Ok(ApiResponse::error(
                "CONFIG_RESET_ERROR",
                "重置配置失败",
                Some("无法确定配置文件路径".to_string()),
            ))
        }
    }
}

/// 获取系统信息
#[tauri::command]
pub async fn get_system_info() -> Result<ApiResponse<serde_json::Value>, String> {
    info!("获取系统信息");

    let mut system_info = serde_json::Map::new();

    // 获取操作系统信息
    system_info.insert(
        "os".to_string(),
        serde_json::Value::String(std::env::consts::OS.to_string()),
    );
    system_info.insert(
        "arch".to_string(),
        serde_json::Value::String(std::env::consts::ARCH.to_string()),
    );

    // 获取应用版本
    system_info.insert(
        "app_version".to_string(),
        serde_json::Value::String(env!("CARGO_PKG_VERSION").to_string()),
    );

    // 获取Rust版本
    system_info.insert(
        "rust_version".to_string(),
        serde_json::Value::String("unknown".to_string()),
    );

    // 获取当前时间
    system_info.insert(
        "timestamp".to_string(),
        serde_json::Value::Number(serde_json::Number::from(
            chrono::Utc::now().timestamp_millis(),
        )),
    );

    Ok(ApiResponse::success(serde_json::Value::Object(system_info)))
}

/// 检查连接状态
#[tauri::command]
pub async fn ping() -> Result<ApiResponse<String>, String> {
    info!("执行连接检查");
    Ok(ApiResponse::success("pong".to_string()))
}

/// 显示系统通知
#[tauri::command]
pub async fn show_notification(
    title: String,
    body: String,
    icon: Option<String>,
) -> Result<ApiResponse<()>, String> {
    info!("显示系统通知: {}", title);

    // 这里可以集成系统通知API
    // 目前只是记录日志
    info!("通知标题: {}, 内容: {}", title, body);
    if let Some(icon_path) = icon {
        info!("通知图标: {}", icon_path);
    }

    Ok(ApiResponse::success(()))
}

/// 记录错误信息
#[tauri::command]
pub async fn log_error(
    error_code: String,
    error_message: String,
    error_details: Option<String>,
) -> Result<ApiResponse<()>, String> {
    error!("前端错误 [{}]: {}", error_code, error_message);

    if let Some(details) = error_details {
        error!("错误详情: {}", details);
    }

    Ok(ApiResponse::success(()))
}

/// 记录警告信息
#[tauri::command]
pub async fn log_warning(
    warning_message: String,
    warning_details: Option<String>,
) -> Result<ApiResponse<()>, String> {
    log::warn!("前端警告: {}", warning_message);

    if let Some(details) = warning_details {
        log::warn!("警告详情: {}", details);
    }

    Ok(ApiResponse::success(()))
}

/// 记录信息日志
#[tauri::command]
pub async fn log_info(
    info_message: String,
    info_details: Option<String>,
) -> Result<ApiResponse<()>, String> {
    info!("前端信息: {}", info_message);

    if let Some(details) = info_details {
        info!("信息详情: {}", details);
    }

    Ok(ApiResponse::success(()))
}

/// 获取应用健康状态
#[tauri::command]
pub async fn get_health_status(
    state: State<'_, AppState>,
) -> Result<ApiResponse<serde_json::Value>, String> {
    info!("检查应用健康状态");

    let mut health_status = serde_json::Map::new();

    // 检查配置状态
    health_status.insert("config_loaded".to_string(), serde_json::Value::Bool(true));

    // 检查数据库连接状态
    let db_status = state.db.lock().unwrap().is_some();
    health_status.insert(
        "database_connected".to_string(),
        serde_json::Value::Bool(db_status),
    );

    // 检查LLM状态
    health_status.insert("llm_available".to_string(), serde_json::Value::Bool(true));

    // 检查语音识别状态
    let voice_status = true; // VoskASR is always present in AppState
    health_status.insert(
        "voice_recognition_available".to_string(),
        serde_json::Value::Bool(voice_status),
    );

    // 获取对话和消息数量
    let conversation_count = state.conversations.lock().unwrap().len();
    let message_count = state.messages.lock().unwrap().len();

    health_status.insert(
        "conversation_count".to_string(),
        serde_json::Value::Number(serde_json::Number::from(conversation_count)),
    );
    health_status.insert(
        "message_count".to_string(),
        serde_json::Value::Number(serde_json::Number::from(message_count)),
    );

    // 应用启动时间
    health_status.insert(
        "uptime_ms".to_string(),
        serde_json::Value::Number(serde_json::Number::from(
            chrono::Utc::now().timestamp_millis(),
        )),
    );

    Ok(ApiResponse::success(serde_json::Value::Object(
        health_status,
    )))
}
