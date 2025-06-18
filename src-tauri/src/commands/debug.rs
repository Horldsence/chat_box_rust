use crate::state::AppState;
use log::{debug, error, info};
use rusqlite::params;
use serde::Serialize;
use tauri::State;

#[derive(Serialize, Debug)]
pub struct DatabaseStats {
    pub total_conversations: i64,
    pub total_messages: i64,
    pub conversations: Vec<ConversationDebugInfo>,
}

#[derive(Serialize, Debug)]
pub struct ConversationDebugInfo {
    pub id: u64,
    pub title: String,
    pub last_message: String,
    pub timestamp: u64,
    pub message_count: i64,
    pub messages: Vec<MessageDebugInfo>,
}

#[derive(Serialize, Debug)]
pub struct MessageDebugInfo {
    pub id: u64,
    pub conversation_id: u64,
    pub content: String,
    pub sender: String,
    pub timestamp: u64,
    pub content_length: usize,
}

#[tauri::command]
pub fn debug_database_status(state: State<AppState>) -> Result<DatabaseStats, String> {
    info!("开始调试数据库状态");

    if let Ok(db_guard) = state.db.lock() {
        if let Some(ref db) = *db_guard {
            // 获取总统计信息
            let total_conversations: i64 = db
                .get_all_conversations()
                .map_err(|e| format!("查询对话失败: {}", e))?
                .len() as i64;

            // 直接使用数据库服务的方法
            let all_conversations = db
                .get_all_conversations()
                .map_err(|e| format!("获取所有对话失败: {}", e))?;

            let mut total_messages = 0i64;
            let mut conversations = Vec::new();

            for conv in all_conversations {
                let messages_result = db.get_conversation_messages(conv.id);
                let messages = match messages_result {
                    Ok(msgs) => msgs,
                    Err(e) => {
                        error!("获取对话 {} 的消息失败: {}", conv.id, e);
                        Vec::new()
                    }
                };

                total_messages += messages.len() as i64;

                let debug_messages: Vec<MessageDebugInfo> = messages
                    .iter()
                    .map(|msg| MessageDebugInfo {
                        id: msg.id,
                        conversation_id: msg.conversation_id,
                        content: msg.content.clone(),
                        sender: msg.sender.clone(),
                        timestamp: msg.timestamp,
                        content_length: msg.content.len(),
                    })
                    .collect();

                debug!(
                    "对话 {}: '{}', {} 条消息",
                    conv.id,
                    conv.title,
                    debug_messages.len()
                );

                conversations.push(ConversationDebugInfo {
                    id: conv.id,
                    title: conv.title,
                    last_message: conv.last_message,
                    timestamp: conv.timestamp,
                    message_count: debug_messages.len() as i64,
                    messages: debug_messages,
                });
            }

            debug!(
                "数据库统计: {} 个对话, {} 条消息",
                total_conversations, total_messages
            );

            let stats = DatabaseStats {
                total_conversations,
                total_messages,
                conversations,
            };

            info!(
                "数据库调试完成: {} 个对话, {} 条消息",
                total_conversations, total_messages
            );
            Ok(stats)
        } else {
            Err("数据库连接不可用".to_string())
        }
    } else {
        Err("无法获取数据库锁".to_string())
    }
}

#[tauri::command]
pub fn debug_memory_state(state: State<AppState>) -> Result<String, String> {
    info!("开始调试内存状态");

    let conversations = state.conversations.lock().unwrap();
    let messages = state.messages.lock().unwrap();

    let mut result = String::new();

    result.push_str(&format!("=== 内存状态调试 ===\n"));
    result.push_str(&format!("对话数量: {}\n", conversations.len()));
    result.push_str(&format!("消息数量: {}\n", messages.len()));
    result.push_str(&format!("\n=== 对话详情 ===\n"));

    for conv in conversations.iter() {
        result.push_str(&format!(
            "对话 {}: '{}' (最后消息: '{}', 时间戳: {})\n",
            conv.id, conv.title, conv.last_message, conv.timestamp
        ));
    }

    result.push_str(&format!("\n=== 消息详情 ===\n"));
    for msg in messages.iter() {
        result.push_str(&format!(
            "消息 {} (对话{}): {} - '{}' ({}字符) [{}]\n",
            msg.id,
            msg.conversation_id,
            msg.sender,
            if msg.content.len() > 50 {
                format!("{}...", &msg.content[..50])
            } else {
                msg.content.clone()
            },
            msg.content.len(),
            msg.timestamp
        ));
    }

    info!("内存状态调试完成");
    Ok(result)
}

#[tauri::command]
pub fn debug_clear_database(state: State<AppState>) -> Result<String, String> {
    info!("开始清空数据库");

    // 清空内存状态
    {
        let mut conversations = state.conversations.lock().unwrap();
        let mut messages = state.messages.lock().unwrap();
        conversations.clear();
        messages.clear();
    }

    // 清空数据库
    if let Ok(mut db_guard) = state.db.lock() {
        if let Some(ref mut db) = *db_guard {
            // 首先获取所有对话
            let conversations = db
                .get_all_conversations()
                .map_err(|e| format!("获取对话列表失败: {}", e))?;

            // 删除每个对话（这会级联删除消息）
            for conv in conversations {
                if let Err(e) = db.delete_conversation(conv.id) {
                    error!("删除对话 {} 失败: {}", conv.id, e);
                }
            }

            info!("数据库清空完成");
            Ok("数据库已清空".to_string())
        } else {
            Err("数据库连接不可用".to_string())
        }
    } else {
        Err("无法获取数据库锁".to_string())
    }
}

#[tauri::command]
pub fn debug_test_database_connection(state: State<AppState>) -> Result<String, String> {
    info!("测试数据库连接");

    if let Ok(db_guard) = state.db.lock() {
        if let Some(ref db) = *db_guard {
            // 测试通过获取对话来验证连接
            match db.get_all_conversations() {
                Ok(conversations) => {
                    info!("数据库连接测试成功，找到 {} 个对话", conversations.len());
                    Ok(format!(
                        "数据库连接正常，当前有 {} 个对话",
                        conversations.len()
                    ))
                }
                Err(e) => Err(format!("数据库查询测试失败: {}", e)),
            }
        } else {
            Err("数据库连接不可用".to_string())
        }
    } else {
        Err("无法获取数据库锁".to_string())
    }
}
