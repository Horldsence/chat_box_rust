use crate::state::AppState;
use chrono::Utc;
use db::models::{Conversation, Message};
use log::{error, info};
use serde::Deserialize;
use tauri::State;

#[derive(Deserialize)]
pub struct DeleteConversationRequest {
    pub conversation_id: u64,
}

#[tauri::command]
pub fn get_conversations(state: State<AppState>) -> Result<Vec<Conversation>, String> {
    // 优先从数据库查询
    if let Ok(db_guard) = state.db.lock() {
        if let Some(ref db) = *db_guard {
            match db.get_all_conversations() {
                Ok(conversations) => {
                    info!("从数据库加载了{}个对话", conversations.len());
                    return Ok(conversations);
                }
                Err(e) => {
                    error!("从数据库查询对话失败: {}", e);
                    // fallback to memory
                }
            }
        }
    }

    // 如果数据库查询失败，从内存中获取
    let conversations = state.conversations.lock().unwrap().clone();
    info!("从内存加载了{}个对话", conversations.len());
    Ok(conversations)
}

#[tauri::command]
pub fn get_conversation_messages(
    conversation_id: u64,
    state: State<AppState>,
) -> Result<Vec<Message>, String> {
    // 优先从数据库查询
    if let Ok(db_guard) = state.db.lock() {
        if let Some(ref db) = *db_guard {
            match db.get_conversation_messages(conversation_id) {
                Ok(messages) => {
                    info!(
                        "从数据库加载了对话{}的{}条消息",
                        conversation_id,
                        messages.len()
                    );
                    return Ok(messages);
                }
                Err(e) => {
                    error!("从数据库查询消息失败: {}", e);
                    // fallback to memory
                }
            }
        }
    }

    // 如果数据库查询失败，从内存中获取
    let messages = state.get_conversation_history(conversation_id);
    let filtered_messages: Vec<Message> = messages
        .iter()
        .filter(|m| m.conversation_id == conversation_id)
        .cloned()
        .collect();

    info!(
        "从内存加载了对话{}的{}条消息",
        conversation_id,
        filtered_messages.len()
    );
    Ok(filtered_messages)
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
pub fn delete_conversation(
    request: DeleteConversationRequest,
    state: State<AppState>,
) -> Result<(), String> {
    let conversation_id = request.conversation_id;
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
