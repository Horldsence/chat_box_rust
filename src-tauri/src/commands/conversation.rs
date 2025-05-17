use crate::models::{Conversation, Message};
use crate::state::AppState;
use chrono::Utc;
use log::{error, info};
use tauri::State;

#[tauri::command]
pub fn get_conversations(state: State<AppState>) -> Vec<Conversation> {
    state.conversations.lock().unwrap().clone()
}

#[tauri::command]
pub fn get_conversation_messages(conversation_id: u64, state: State<AppState>) -> Vec<Message> {
    let messages = state.get_conversation_history(conversation_id);
    messages
        .iter()
        .filter(|m| m.conversation_id == conversation_id)
        .cloned()
        .collect()
}

#[tauri::command]
pub fn create_conversation(title: String, state: State<AppState>) -> Result<Conversation, String> {
    let mut conversations = state.conversations.lock().unwrap();

    // 生成新ID
    let new_id = conversations.iter().map(|c| c.id).max().unwrap_or(0) + 1;

    // 创建新对话
    let new_conversation = Conversation {
        id: new_id,
        title,
        last_message: "开始新的对话".to_string(),
        timestamp: Utc::now().timestamp_millis() as u64,
    };

    // 创建对话后尝试保存到数据库
    if let Ok(mut db_guard) = state.db.lock() {
        if let Some(ref mut db) = *db_guard {
            if let Err(e) = db.save_conversation(&new_conversation) {
                error!("保存新对话到数据库失败: {}", e);
            }
        }
    }

    // 添加到对话列表
    conversations.push(new_conversation.clone());

    info!("创建了新对话: {:?}", new_conversation);
    Ok(new_conversation)
}

#[tauri::command]
pub fn delete_conversation(conversation_id: u64, state: State<AppState>) -> Result<(), String> {
    // 删除对话
    {
        let mut conversations = state.conversations.lock().unwrap();
        let position = conversations
            .iter()
            .position(|c| c.id == conversation_id)
            .ok_or_else(|| format!("对话 {} 不存在", conversation_id))?;

        conversations.remove(position);
        info!("删除了对话 {}", conversation_id);
    }

    // 删除关联的消息
    {
        let mut messages = state.messages.lock().unwrap();
        messages.retain(|m| m.conversation_id != conversation_id);
        info!("删除了对话 {} 相关的所有消息", conversation_id);
    }

    Ok(())
}
