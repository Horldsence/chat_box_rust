use chrono::{DateTime, Utc};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::Mutex;
use tauri::{Emitter, Manager, State};

// 数据结构定义
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Conversation {
    pub id: String,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub id: String,
    pub conversation_id: String,
    pub content: String,
    pub sender: String, // "user" or "assistant"
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<ApiError>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    pub details: Option<String>,
    pub timestamp: DateTime<Utc>,
}

// 应用状态
pub struct AppState {
    pub db: Arc<Mutex<Connection>>,
    pub current_conversation: Arc<Mutex<Option<String>>>,
}

impl AppState {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let db = Connection::open("chat_box.db")?;

        // 创建表
        db.execute(
            "CREATE TABLE IF NOT EXISTS conversations (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;

        db.execute(
            "CREATE TABLE IF NOT EXISTS messages (
                id TEXT PRIMARY KEY,
                conversation_id TEXT NOT NULL,
                content TEXT NOT NULL,
                sender TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                FOREIGN KEY (conversation_id) REFERENCES conversations (id)
            )",
            [],
        )?;

        Ok(AppState {
            db: Arc::new(Mutex::new(db)),
            current_conversation: Arc::new(Mutex::new(None)),
        })
    }
}

// 生成UUID
fn generate_id() -> String {
    format!("{}", uuid::Uuid::new_v4())
}

// Tauri 命令实现
#[tauri::command]
async fn get_conversations(
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<Conversation>>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let mut stmt = db
        .prepare(
            "SELECT id, title, created_at, updated_at FROM conversations ORDER BY updated_at DESC",
        )
        .map_err(|e| e.to_string())?;

    let conversation_iter = stmt
        .query_map([], |row| {
            Ok(Conversation {
                id: row.get(0)?,
                title: row.get(1)?,
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(2)?)
                    .map_err(|_| {
                        rusqlite::Error::InvalidColumnType(
                            2,
                            "created_at".to_string(),
                            rusqlite::types::Type::Text,
                        )
                    })?
                    .with_timezone(&Utc),
                updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                    .map_err(|_| {
                        rusqlite::Error::InvalidColumnType(
                            3,
                            "updated_at".to_string(),
                            rusqlite::types::Type::Text,
                        )
                    })?
                    .with_timezone(&Utc),
            })
        })
        .map_err(|e| e.to_string())?;

    let mut conversations = Vec::new();
    for conversation in conversation_iter {
        conversations.push(conversation.map_err(|e| e.to_string())?);
    }

    Ok(ApiResponse {
        success: true,
        data: Some(conversations),
        error: None,
    })
}

#[tauri::command]
async fn create_conversation(
    title: String,
    state: State<'_, AppState>,
) -> Result<ApiResponse<Conversation>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let now = Utc::now();
    let conversation = Conversation {
        id: generate_id(),
        title,
        created_at: now,
        updated_at: now,
    };

    db.execute(
        "INSERT INTO conversations (id, title, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
        [
            &conversation.id,
            &conversation.title,
            &conversation.created_at.to_rfc3339(),
            &conversation.updated_at.to_rfc3339(),
        ],
    )
    .map_err(|e| e.to_string())?;

    // 设置为当前对话
    *state
        .current_conversation
        .lock()
        .map_err(|e| e.to_string())? = Some(conversation.id.clone());

    Ok(ApiResponse {
        success: true,
        data: Some(conversation),
        error: None,
    })
}

#[tauri::command]
async fn get_conversation_messages(
    conversation_id: String,
    state: State<'_, AppState>,
) -> Result<ApiResponse<Vec<Message>>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let mut stmt = db.prepare("SELECT id, conversation_id, content, sender, timestamp FROM messages WHERE conversation_id = ?1 ORDER BY timestamp ASC")
        .map_err(|e| e.to_string())?;

    let message_iter = stmt
        .query_map([&conversation_id], |row| {
            Ok(Message {
                id: row.get(0)?,
                conversation_id: row.get(1)?,
                content: row.get(2)?,
                sender: row.get(3)?,
                timestamp: DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                    .map_err(|_| {
                        rusqlite::Error::InvalidColumnType(
                            4,
                            "timestamp".to_string(),
                            rusqlite::types::Type::Text,
                        )
                    })?
                    .with_timezone(&Utc),
            })
        })
        .map_err(|e| e.to_string())?;

    let mut messages = Vec::new();
    for message in message_iter {
        messages.push(message.map_err(|e| e.to_string())?);
    }

    Ok(ApiResponse {
        success: true,
        data: Some(messages),
        error: None,
    })
}

#[tauri::command]
async fn send_message(
    conversation_id: String,
    content: String,
    sender: String,
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<ApiResponse<Message>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let now = Utc::now();
    let message = Message {
        id: generate_id(),
        conversation_id: conversation_id.clone(),
        content: content.clone(),
        sender: sender.clone(),
        timestamp: now,
    };

    // 保存用户消息
    db.execute(
        "INSERT INTO messages (id, conversation_id, content, sender, timestamp) VALUES (?1, ?2, ?3, ?4, ?5)",
        [
            &message.id,
            &message.conversation_id,
            &message.content,
            &message.sender,
            &message.timestamp.to_rfc3339(),
        ],
    ).map_err(|e| e.to_string())?;

    // 更新对话的最后更新时间
    db.execute(
        "UPDATE conversations SET updated_at = ?1 WHERE id = ?2",
        [&now.to_rfc3339(), &conversation_id],
    )
    .map_err(|e| e.to_string())?;

    // 如果是用户消息，生成AI回复
    if sender == "user" {
        tokio::spawn(async move {
            if let Err(e) = generate_ai_response(conversation_id, content, app_handle).await {
                eprintln!("Failed to generate AI response: {}", e);
            }
        });
    }

    Ok(ApiResponse {
        success: true,
        data: Some(message),
        error: None,
    })
}

#[tauri::command]
async fn clear_conversation(
    conversation_id: String,
    state: State<'_, AppState>,
) -> Result<ApiResponse<bool>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    db.execute(
        "DELETE FROM messages WHERE conversation_id = ?1",
        [&conversation_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(ApiResponse {
        success: true,
        data: Some(true),
        error: None,
    })
}

#[tauri::command]
async fn delete_conversation(
    conversation_id: String,
    state: State<'_, AppState>,
) -> Result<ApiResponse<bool>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    // 删除消息
    db.execute(
        "DELETE FROM messages WHERE conversation_id = ?1",
        [&conversation_id],
    )
    .map_err(|e| e.to_string())?;

    // 删除对话
    db.execute(
        "DELETE FROM conversations WHERE id = ?1",
        [&conversation_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(ApiResponse {
        success: true,
        data: Some(true),
        error: None,
    })
}

// 生成AI回复（简化版本，实际应该调用LLM服务）
async fn generate_ai_response(
    conversation_id: String,
    user_message: String,
    app_handle: tauri::AppHandle,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 模拟AI处理时间
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

    // 简单的响应生成逻辑（实际应该调用LLM）
    let ai_response = generate_simple_response(&user_message);

    let state = app_handle.state::<AppState>();
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let now = Utc::now();
    let response_message = Message {
        id: generate_id(),
        conversation_id: conversation_id.clone(),
        content: ai_response.clone(),
        sender: "assistant".to_string(),
        timestamp: now,
    };

    // 保存AI响应
    db.execute(
        "INSERT INTO messages (id, conversation_id, content, sender, timestamp) VALUES (?1, ?2, ?3, ?4, ?5)",
        [
            &response_message.id,
            &response_message.conversation_id,
            &response_message.content,
            &response_message.sender,
            &response_message.timestamp.to_rfc3339(),
        ],
    ).map_err(|e| e.to_string())?;

    // 发送事件到前端
    app_handle.emit("chat_response", &response_message)?;

    Ok(())
}

// 简单的响应生成（实际应该替换为LLM调用）
fn generate_simple_response(user_message: &str) -> String {
    let user_lower = user_message.to_lowercase();

    if user_lower.contains("hello") || user_lower.contains("hi") || user_lower.contains("你好") {
        "你好！我是你的AI助手，很高兴为你服务！有什么可以帮助你的吗？".to_string()
    } else if user_lower.contains("help") || user_lower.contains("帮助") {
        "我可以帮助你进行对话、回答问题、提供建议等。请告诉我你需要什么帮助！".to_string()
    } else if user_lower.contains("time") || user_lower.contains("时间") {
        format!("当前时间是：{}", Utc::now().format("%Y-%m-%d %H:%M:%S UTC"))
    } else if user_lower.contains("weather") || user_lower.contains("天气") {
        "抱歉，我暂时无法获取天气信息。你可以尝试询问其他问题！".to_string()
    } else {
        format!("我理解你说的是：\"{}\"\n\n作为你的AI助手，我会尽力帮助你。不过目前我的功能还比较简单，如果你有具体的问题或需要帮助的地方，请告诉我！", user_message)
    }
}

// 日志相关命令
#[tauri::command]
async fn log_error(code: String, message: String, details: Option<String>) -> Result<(), String> {
    eprintln!("[ERROR] {}: {} {:?}", code, message, details);
    Ok(())
}

#[tauri::command]
async fn log_warning(message: String, details: Option<String>) -> Result<(), String> {
    println!("[WARNING] {} {:?}", message, details);
    Ok(())
}

#[tauri::command]
async fn log_info(message: String, details: Option<String>) -> Result<(), String> {
    println!("[INFO] {} {:?}", message, details);
    Ok(())
}

// 原有的greet命令
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_state = AppState::new().expect("Failed to initialize app state");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            greet,
            get_conversations,
            create_conversation,
            get_conversation_messages,
            send_message,
            clear_conversation,
            delete_conversation,
            log_error,
            log_warning,
            log_info
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
