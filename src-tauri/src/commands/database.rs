use crate::{models::Conversation, state::AppState};
use tauri::State;

#[tauri::command]
pub async fn get_database_conversations(
    state: State<'_, AppState>,
) -> Result<Vec<Conversation>, String> {
    let db_arc = state.db.clone();
    let guard = db_arc.lock().unwrap();
    let db = guard.as_ref().unwrap();
    db.get_all_conversations().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_database_conversation(
    state: State<'_, AppState>,
    conversation_id: u64,
) -> Result<(), String> {
    let db_arc = state.db.clone();
    let mut guard = db_arc.lock().unwrap();
    let db = guard.as_mut().unwrap();
    db.delete_conversation(conversation_id)
        .map_err(|e| e.to_string())
}
