use crate::state::AppState;
use chrono::Utc;
use db::models::Message;
use log::{debug, error, info};
use serde::Deserialize;
use tauri::State;

#[derive(Deserialize)]
pub struct SendMessageRequest {
    pub content: String,
    pub conversation_id: u64,
}

#[tauri::command]
pub fn send_user_message(
    request: SendMessageRequest,
    state: State<AppState>,
) -> Result<Message, String> {
    let content = request.content;
    let conversation_id = request.conversation_id;

    debug!(
        "接收用户消息，对话ID: {}, 消息内容: '{}'",
        conversation_id, content
    );

    // 创建用户消息
    let user_message = Message {
        id: Utc::now().timestamp_millis() as u64,
        content: content.clone(),
        sender: "user".to_string(),
        timestamp: Utc::now().timestamp_millis() as u64,
        conversation_id,
    };

    debug!("创建的用户消息: {:?}", user_message);

    // 存储用户消息到内存
    state.messages.lock().unwrap().push(user_message.clone());
    debug!("用户消息已添加到内存状态");

    // 尝试保存到数据库
    if let Ok(mut db_guard) = state.db.lock() {
        if let Some(ref mut db) = *db_guard {
            if let Err(e) = db.save_message(&user_message) {
                error!("保存用户消息到数据库失败: {}", e);
            } else {
                debug!("用户消息成功保存到数据库: ID={}", user_message.id);
            }
        } else {
            error!("数据库连接不可用");
        }
    } else {
        error!("无法获取数据库锁");
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
                } else {
                    debug!("对话更新成功保存到数据库: ID={}", conv.id);
                }
            }
        }
    } else {
        info!("未找到对话ID: {}", conversation_id);
    }

    info!("用户消息处理完成");
    Ok(user_message)
}
