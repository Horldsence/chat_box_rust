use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub id: u64,
    pub content: String,
    pub sender: String,
    pub timestamp: u64,
    pub conversation_id: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Conversation {
    pub id: u64,
    pub title: String,
    pub last_message: String,
    pub timestamp: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MessageChunk {
    pub conversation_id: u64,
    pub content: String,
    pub is_complete: bool,
}