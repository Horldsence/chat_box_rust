use chrono::Utc;
use log::{debug, error, info};
use rusqlite::{params, Connection, Result};
use std::fs;
use std::path::Path;

use crate::models::{Conversation, Message};

pub struct ChatDatabase {
    conn: Connection,
}

impl ChatDatabase {
    pub fn new(db_path: &str) -> Result<Self> {
        info!("开始创建数据库");
        // 确保目录存在
        if let Some(parent) = Path::new(db_path).parent() {
            fs::create_dir_all(parent).map_err(|_e| rusqlite::Error::ExecuteReturnedResults)?;
        }

        info!("Opening database at: {}", db_path);
        let conn = Connection::open(db_path)?;

        // 创建表结构
        conn.execute(
            "CREATE TABLE IF NOT EXISTS conversations (
                id INTEGER PRIMARY KEY,
                title TEXT NOT NULL,
                last_message TEXT NOT NULL,
                timestamp INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS messages (
                id INTEGER PRIMARY KEY,
                conversation_id INTEGER NOT NULL,
                content TEXT NOT NULL,
                sender TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                FOREIGN KEY (conversation_id) REFERENCES conversations (id) ON DELETE CASCADE
            )",
            [],
        )?;

        // 启用外键约束
        conn.execute("PRAGMA foreign_keys = ON", [])?;

        Ok(ChatDatabase { conn })
    }

    // 保存对话
    pub fn save_conversation(&mut self, conversation: &Conversation) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO conversations (id, title, last_message, timestamp) VALUES (?, ?, ?, ?)",
            params![
                conversation.id,
                conversation.title,
                conversation.last_message,
                conversation.timestamp
            ],
        )?;

        debug!("保存对话: {}", conversation.id);
        Ok(())
    }

    // 保存消息
    pub fn save_message(&mut self, message: &Message) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO messages (id, conversation_id, content, sender, timestamp) 
             VALUES (?, ?, ?, ?, ?)",
            params![
                message.id,
                message.conversation_id,
                message.content,
                message.sender,
                message.timestamp
            ],
        )?;

        debug!(
            "保存消息: {} 到对话: {}",
            message.id, message.conversation_id
        );
        Ok(())
    }

    // 保存多条消息
    pub fn save_messages(&mut self, messages: &[Message]) -> Result<()> {
        let tx = self.conn.transaction()?;

        for message in messages {
            tx.execute(
                "INSERT OR REPLACE INTO messages (id, conversation_id, content, sender, timestamp) 
                 VALUES (?, ?, ?, ?, ?)",
                params![
                    message.id,
                    message.conversation_id,
                    message.content,
                    message.sender,
                    message.timestamp
                ],
            )?;
        }

        tx.commit()?;
        debug!("批量保存 {} 条消息", messages.len());
        Ok(())
    }

    // 获取所有对话
    pub fn get_all_conversations(&self) -> Result<Vec<Conversation>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, last_message, timestamp FROM conversations ORDER BY timestamp DESC",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(Conversation {
                id: row.get(0)?,
                title: row.get(1)?,
                last_message: row.get(2)?,
                timestamp: row.get(3)?,
            })
        })?;

        let mut conversations = Vec::new();
        for row in rows {
            conversations.push(row?);
        }

        info!("加载了 {} 个对话", conversations.len());
        Ok(conversations)
    }

    // 获取特定对话的所有消息
    pub fn get_conversation_messages(&self, conversation_id: u64) -> Result<Vec<Message>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, conversation_id, content, sender, timestamp FROM messages WHERE conversation_id = ? ORDER BY timestamp ASC"
        )?;

        let rows = stmt.query_map(params![conversation_id], |row| {
            Ok(Message {
                id: row.get(0)?,
                conversation_id: row.get(1)?,
                content: row.get(2)?,
                sender: row.get(3)?,
                timestamp: row.get(4)?,
            })
        })?;

        let mut messages = Vec::new();
        for row in rows {
            messages.push(row?);
        }

        info!(
            "加载了对话 {} 的 {} 条消息",
            conversation_id,
            messages.len()
        );
        Ok(messages)
    }
}
