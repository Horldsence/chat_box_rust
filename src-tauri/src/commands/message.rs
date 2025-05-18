use crate::models::Message;
use crate::state::AppState;
use chrono::Utc;
use log::{debug, error, info};
use tauri::State;

#[tauri::command]
pub fn send_user_message(
    content: String,
    conversation_id: u64,
    state: State<AppState>,
) -> Result<Message, String> {
    info!("接收用户消息，对话ID: {}", conversation_id);
    debug!("消息内容: {}", content);

    // 创建用户消息
    let user_message = Message {
        id: Utc::now().timestamp_millis() as u64,
        content: content.clone(),
        sender: "user".to_string(),
        timestamp: Utc::now().timestamp_millis() as u64,
        conversation_id,
    };

    debug!("创建的用户消息: {:?}", user_message);

    // 存储用户消息
    state.messages.lock().unwrap().push(user_message.clone());

    // 尝试保存到数据库
    if let Ok(mut db_guard) = state.db.lock() {
        if let Some(ref mut db) = *db_guard {
            if let Err(e) = db.save_message(&user_message) {
                error!("保存用户消息到数据库失败: {}", e);
            }
        }
    }

    // 更新对话的最后消息时间
    if let Some(conv) = state
        .conversations
        .lock()
        .unwrap()
        .iter_mut()
        .find(|c| c.id == conversation_id)
    {
        conv.last_message = content;
        debug!("更新对话 {} 的时间戳", conversation_id);
        conv.timestamp = user_message.timestamp;
        // 更新数据库中的对话
        if let Ok(mut db_guard) = state.db.lock() {
            if let Some(ref mut db) = *db_guard {
                if let Err(e) = db.save_conversation(conv) {
                    error!("更新对话到数据库失败: {}", e);
                }
            }
        }
    } else {
        info!("未找到对话ID: {}", conversation_id);
    }

    info!("用户消息处理完成");
    Ok(user_message)
}
