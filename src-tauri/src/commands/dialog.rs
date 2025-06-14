use crate::state::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(message: &str) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message.to_string()),
        }
    }
}

/// 显示信息对话框
#[tauri::command]
pub async fn show_info_dialog(
    app_handle: tauri::AppHandle,
    title: String,
    message: String,
) -> Result<ApiResponse<bool>, String> {
    let result = app_handle
        .dialog()
        .message(message)
        .title(title)
        .kind(MessageDialogKind::Info)
        .buttons(MessageDialogButtons::Ok)
        .blocking_show();

    Ok(ApiResponse::success(result))
}

/// 显示警告对话框
#[tauri::command]
pub async fn show_warning_dialog(
    app_handle: tauri::AppHandle,
    title: String,
    message: String,
) -> Result<ApiResponse<bool>, String> {
    let result = app_handle
        .dialog()
        .message(message)
        .title(title)
        .kind(MessageDialogKind::Warning)
        .buttons(MessageDialogButtons::Ok)
        .blocking_show();

    Ok(ApiResponse::success(result))
}

/// 显示错误对话框
#[tauri::command]
pub async fn show_error_dialog(
    app_handle: tauri::AppHandle,
    title: String,
    message: String,
) -> Result<ApiResponse<bool>, String> {
    let result = app_handle
        .dialog()
        .message(message)
        .title(title)
        .kind(MessageDialogKind::Error)
        .buttons(MessageDialogButtons::Ok)
        .blocking_show();

    Ok(ApiResponse::success(result))
}

/// 显示确认对话框
#[tauri::command]
pub async fn show_confirm_dialog(
    app_handle: tauri::AppHandle,
    title: String,
    message: String,
) -> Result<ApiResponse<bool>, String> {
    let result = app_handle
        .dialog()
        .message(message)
        .title(title)
        .kind(MessageDialogKind::Info)
        .buttons(MessageDialogButtons::OkCancel)
        .blocking_show();

    Ok(ApiResponse::success(result))
}

/// 显示是/否对话框
#[tauri::command]
pub async fn show_ask_dialog(
    app_handle: tauri::AppHandle,
    title: String,
    message: String,
) -> Result<ApiResponse<bool>, String> {
    let result = app_handle
        .dialog()
        .message(message)
        .title(title)
        .kind(MessageDialogKind::Info)
        .buttons(MessageDialogButtons::YesNo)
        .blocking_show();

    Ok(ApiResponse::success(result))
}

/// 打开文件选择对话框
#[tauri::command]
pub async fn open_file_dialog(
    app_handle: tauri::AppHandle,
    title: Option<String>,
    filters: Option<Vec<(String, Vec<String>)>>,
    multiple: Option<bool>,
) -> Result<ApiResponse<Option<Vec<String>>>, String> {
    let mut dialog = app_handle.dialog().file();

    if let Some(title) = title {
        dialog = dialog.set_title(&title);
    }

    if let Some(filters) = filters {
        for (name, extensions) in filters {
            let ext_refs: Vec<&str> = extensions.iter().map(|s| s.as_str()).collect();
            dialog = dialog.add_filter(&name, &ext_refs);
        }
    }

    if multiple.unwrap_or(false) {
        match dialog.blocking_pick_files() {
            Some(files) => {
                let paths: Vec<String> = files
                    .into_iter()
                    .filter_map(|p| p.as_path().map(|path| path.to_string_lossy().to_string()))
                    .collect();
                Ok(ApiResponse::success(Some(paths)))
            }
            None => Ok(ApiResponse::success(None)),
        }
    } else {
        match dialog.blocking_pick_file() {
            Some(file) => {
                if let Some(path) = file.as_path() {
                    let path_str = path.to_string_lossy().to_string();
                    Ok(ApiResponse::success(Some(vec![path_str])))
                } else {
                    Ok(ApiResponse::success(None))
                }
            }
            None => Ok(ApiResponse::success(None)),
        }
    }
}

/// 打开文件夹选择对话框
#[tauri::command]
pub async fn open_folder_dialog(
    app_handle: tauri::AppHandle,
    title: Option<String>,
) -> Result<ApiResponse<Option<String>>, String> {
    let mut dialog = app_handle.dialog().file();

    if let Some(title) = title {
        dialog = dialog.set_title(&title);
    }

    let result = dialog.blocking_pick_folder();

    match result {
        Some(folder) => {
            if let Some(path) = folder.as_path() {
                Ok(ApiResponse::success(Some(
                    path.to_string_lossy().to_string(),
                )))
            } else {
                Ok(ApiResponse::success(None))
            }
        }
        None => Ok(ApiResponse::success(None)),
    }
}

/// 打开保存文件对话框
#[tauri::command]
pub async fn save_file_dialog(
    app_handle: tauri::AppHandle,
    title: Option<String>,
    default_name: Option<String>,
    filters: Option<Vec<(String, Vec<String>)>>,
) -> Result<ApiResponse<Option<String>>, String> {
    let mut dialog = app_handle.dialog().file();

    if let Some(title) = title {
        dialog = dialog.set_title(&title);
    }

    if let Some(name) = default_name {
        dialog = dialog.set_file_name(&name);
    }

    if let Some(filters) = filters {
        for (name, extensions) in filters {
            let ext_refs: Vec<&str> = extensions.iter().map(|s| s.as_str()).collect();
            dialog = dialog.add_filter(&name, &ext_refs);
        }
    }

    let result = dialog.blocking_save_file();

    match result {
        Some(file) => {
            if let Some(path) = file.as_path() {
                Ok(ApiResponse::success(Some(
                    path.to_string_lossy().to_string(),
                )))
            } else {
                Ok(ApiResponse::success(None))
            }
        }
        None => Ok(ApiResponse::success(None)),
    }
}

/// 选择配置文件导入
#[tauri::command]
pub async fn import_config_file(
    app_handle: tauri::AppHandle,
    _state: State<'_, AppState>,
) -> Result<ApiResponse<Option<String>>, String> {
    let result = app_handle
        .dialog()
        .file()
        .set_title("选择配置文件")
        .add_filter("配置文件", &["yaml", "yml", "json"])
        .blocking_pick_file();

    match result {
        Some(file) => {
            if let Some(path) = file.as_path() {
                let path_str = path.to_string_lossy().to_string();

                // 读取文件内容
                match std::fs::read_to_string(path) {
                    Ok(content) => {
                        // 验证文件格式
                        let is_valid = if path_str.ends_with(".json") {
                            serde_json::from_str::<serde_json::Value>(&content).is_ok()
                        } else {
                            serde_yaml::from_str::<serde_yaml::Value>(&content).is_ok()
                        };

                        if is_valid {
                            Ok(ApiResponse::success(Some(content)))
                        } else {
                            Ok(ApiResponse::error("配置文件格式无效"))
                        }
                    }
                    Err(e) => Ok(ApiResponse::error(&format!("无法读取文件: {}", e))),
                }
            } else {
                Ok(ApiResponse::error("无法获取文件路径"))
            }
        }
        None => Ok(ApiResponse::success(None)),
    }
}

/// 导出配置文件
#[tauri::command]
pub async fn export_config_file(
    app_handle: tauri::AppHandle,
    content: String,
    format: String, // "json" or "yaml"
) -> Result<ApiResponse<bool>, String> {
    let extension = if format == "json" { "json" } else { "yaml" };
    let default_name = format!("chatbox-config.{}", extension);

    let result = app_handle
        .dialog()
        .file()
        .set_title("保存配置文件")
        .set_file_name(&default_name)
        .add_filter("配置文件", &[extension])
        .blocking_save_file();

    match result {
        Some(file) => {
            if let Some(path) = file.as_path() {
                match std::fs::write(path, content) {
                    Ok(_) => Ok(ApiResponse::success(true)),
                    Err(e) => Ok(ApiResponse::error(&format!("保存文件失败: {}", e))),
                }
            } else {
                Ok(ApiResponse::error("无法获取文件路径"))
            }
        }
        None => Ok(ApiResponse::success(false)),
    }
}

/// 导出聊天记录
#[tauri::command]
pub async fn export_chat_history(
    app_handle: tauri::AppHandle,
    conversation_id: Option<i64>,
    format: String, // "json", "txt", "md"
) -> Result<ApiResponse<bool>, String> {
    let extension = match format.as_str() {
        "txt" => "txt",
        "md" => "md",
        _ => "json",
    };

    let default_name = if let Some(id) = conversation_id {
        format!("chat-conversation-{}.{}", id, extension)
    } else {
        format!("chat-history.{}", extension)
    };

    let result = app_handle
        .dialog()
        .file()
        .set_title("导出聊天记录")
        .set_file_name(&default_name)
        .add_filter("聊天记录", &[extension])
        .blocking_save_file();

    match result {
        Some(_file) => {
            // 这里应该从状态或数据库获取聊天记录
            // 暂时返回成功，实际实现需要根据 conversation_id 获取数据
            Ok(ApiResponse::success(true))
        }
        None => Ok(ApiResponse::success(false)),
    }
}

/// 选择语音模型文件夹
#[tauri::command]
pub async fn select_voice_model_folder(
    app_handle: tauri::AppHandle,
) -> Result<ApiResponse<Option<String>>, String> {
    let result = app_handle
        .dialog()
        .file()
        .set_title("选择语音模型文件夹")
        .blocking_pick_folder();

    match result {
        Some(folder) => {
            if let Some(path) = folder.as_path() {
                let path_str = path.to_string_lossy().to_string();

                // 验证是否是有效的 Vosk 模型文件夹
                let model_files = ["final.mdl", "conf/model.conf"];
                let is_valid_model = model_files.iter().any(|file| path.join(file).exists());

                if is_valid_model {
                    Ok(ApiResponse::success(Some(path_str)))
                } else {
                    Ok(ApiResponse::error("所选文件夹不是有效的 Vosk 模型文件夹"))
                }
            } else {
                Ok(ApiResponse::error("无法获取文件夹路径"))
            }
        }
        None => Ok(ApiResponse::success(None)),
    }
}

/// 选择数据库文件
#[tauri::command]
pub async fn select_database_file(
    app_handle: tauri::AppHandle,
) -> Result<ApiResponse<Option<String>>, String> {
    let result = app_handle
        .dialog()
        .file()
        .set_title("选择数据库文件")
        .add_filter("SQLite 数据库", &["db", "sqlite", "sqlite3"])
        .blocking_pick_file();

    match result {
        Some(file) => {
            if let Some(path) = file.as_path() {
                Ok(ApiResponse::success(Some(
                    path.to_string_lossy().to_string(),
                )))
            } else {
                Ok(ApiResponse::error("无法获取文件路径"))
            }
        }
        None => Ok(ApiResponse::success(None)),
    }
}

/// 创建新数据库文件
#[tauri::command]
pub async fn create_database_file(
    app_handle: tauri::AppHandle,
) -> Result<ApiResponse<Option<String>>, String> {
    let result = app_handle
        .dialog()
        .file()
        .set_title("创建新数据库")
        .set_file_name("chat_database.db")
        .add_filter("SQLite 数据库", &["db"])
        .blocking_save_file();

    match result {
        Some(file) => {
            if let Some(path) = file.as_path() {
                Ok(ApiResponse::success(Some(
                    path.to_string_lossy().to_string(),
                )))
            } else {
                Ok(ApiResponse::error("无法获取文件路径"))
            }
        }
        None => Ok(ApiResponse::success(None)),
    }
}
