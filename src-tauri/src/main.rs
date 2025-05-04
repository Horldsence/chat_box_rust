// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// 在文件顶部添加：
mod components {
    pub mod agent {
        pub mod ollama;
    }
    pub mod asr {
        pub mod vosk;
    }
}

use components::agent::ollama::OllamaAgent;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tauri::{Emitter, State, Window};

use log::{debug, error, info, warn};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Message {
    id: u64,
    content: String,
    sender: String,
    timestamp: u64,
    conversation_id: u64,  // 新增字段，关联消息与对话
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Conversation {
    id: u64,
    title: String,
    last_message: String,
    timestamp: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct MessageChunk {
    conversation_id: u64,
    content: String,
    is_complete: bool,
}

#[derive(Clone)]
struct AppState {
    conversations: Arc<Mutex<Vec<Conversation>>>, // 使用Arc包装Mutex
    messages: Arc<Mutex<Vec<Message>>>,           // 使用Arc包装Mutex
    ollama_agent: Arc<OllamaAgent>,
}

#[tauri::command]
fn get_conversations(state: State<AppState>) -> Vec<Conversation> {
    state.conversations.lock().unwrap().clone()
}

#[tauri::command]
fn get_messages(state: State<AppState>) -> Vec<Message> {
    state.messages.lock().unwrap().clone()
}

#[tauri::command]
fn send_message(content: String, state: State<AppState>) -> Result<Message, String> {
    let message = Message {
        id: chrono::Utc::now().timestamp_millis() as u64,
        content: content.clone(),
        sender: "user".to_string(),
        timestamp: chrono::Utc::now().timestamp_millis() as u64,
        conversation_id: 0, // 默认值
    };
    debug!("send_message User message: {:?}", message);

    // 存储用户消息
    state.messages.lock().unwrap().push(message.clone());

    // 模拟机器人回复
    let bot_message = Message {
        id: chrono::Utc::now().timestamp_millis() as u64,
        content: "这是一个模拟的机器人回复".to_string(),
        sender: "bot".to_string(),
        timestamp: chrono::Utc::now().timestamp_millis() as u64,
        conversation_id: 0, // 默认值
    };

    // 存储机器人消息
    state.messages.lock().unwrap().push(bot_message);

    Ok(message)
}

#[tauri::command]
fn send_user_message(
    content: String,
    conversation_id: u64,
    state: State<AppState>,
) -> Result<Message, String> {
    info!("接收用户消息，对话ID: {}", conversation_id);
    debug!("消息内容: {}", content);

    // 创建用户消息
    let user_message = Message {
        id: chrono::Utc::now().timestamp_millis() as u64,
        content: content.clone(),
        sender: "user".to_string(),
        timestamp: chrono::Utc::now().timestamp_millis() as u64,
        conversation_id,
    };

    debug!("创建的用户消息: {:?}", user_message);

    // 存储用户消息
    state.messages.lock().unwrap().push(user_message.clone());

    // 更新对话的最后消息时间
    if let Some(conv) = state
        .conversations
        .lock()
        .unwrap()
        .iter_mut()
        .find(|c| c.id == conversation_id)
    {
        debug!("更新对话 {} 的时间戳", conversation_id);
        conv.timestamp = user_message.timestamp;
    } else {
        warn!("未找到对话ID: {}", conversation_id);
    }

    let user_message = Message {
        id: chrono::Utc::now().timestamp_millis() as u64,
        content: content.clone(),
        sender: "user".to_string(),
        timestamp: chrono::Utc::now().timestamp_millis() as u64,
        conversation_id,  // 添加会话ID
    };

    info!("用户消息处理完成");
    Ok(user_message)
}

#[tauri::command]
async fn send_message_stream(
    window: Window,
    content: String,
    conversation_id: u64,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // 创建并保存用户消息
    let user_message = Message {
        id: chrono::Utc::now().timestamp_millis() as u64,
        content: content.clone(),
        sender: "user".to_string(),
        timestamp: chrono::Utc::now().timestamp_millis() as u64,
        conversation_id,
    };
    debug!("send_message_stream User message: {:?}", user_message);

    state.messages.lock().unwrap().push(user_message);

    // 创建机器人消息占位符
    let bot_message_id = chrono::Utc::now().timestamp_millis() as u64;
    let bot_message = Message {
        id: bot_message_id,
        content: String::new(), // 初始为空，将通过流式更新
        sender: "bot".to_string(),
        timestamp: chrono::Utc::now().timestamp_millis() as u64,
        conversation_id,
    };

    // 保存初始的空机器人消息
    state.messages.lock().unwrap().push(bot_message.clone());

    // 开始流式生成
    let agent = state.ollama_agent.clone();

    // 生成消息流
    let mut stream = match agent.generate_stream(&content).await {
        Ok(stream) => stream,
        Err(e) => return Err(format!("创建流失败: {}", e)),
    };
    debug!("Stream created successfully");

    // 完整的响应内容
    let mut full_response = String::new();

    // 获取需要的字段克隆以在异步任务中使用
    let mut conversations = state.conversations.lock().unwrap().clone();
    let mut messages = state.messages.lock().unwrap().clone();

    // 启动另一个任务处理流
    tokio::spawn(async move {
        use tokio_stream::StreamExt;

        while let Some(chunk) = stream.next().await {
            // 将新的内容添加到完整响应中
            full_response.push_str(&chunk);

            // 发送消息块到前端
            let _ = window.emit(
                "message_chunk",
                MessageChunk {
                    conversation_id,
                    content: chunk,
                    is_complete: false,
                },
            );
            debug!("Chunk sent to front-end");
        }

        // 更新消息存储中的内容
        if let Some(conv) = conversations.iter_mut().find(|c| c.id == conversation_id) {
            conv.last_message = full_response.clone();
            conv.timestamp = chrono::Utc::now().timestamp_millis() as u64;
            debug!("Updated conversation: {:?}", conv);
        }

        // 更新机器人消息的内容
        if let Some(msg) = messages.iter_mut().find(|m| m.id == bot_message_id) {
            msg.content = full_response.clone();
            debug!("Updated bot message: {:?}", msg);
        }

        // 发送完成信号
        let _ = window.emit(
            "message_chunk",
            MessageChunk {
                conversation_id,
                content: String::new(),
                is_complete: true,
            },
        );
    });

    Ok(())
}

#[tauri::command]
async fn generate_ai_response(
    window: Window,
    user_message_content: String,
    conversation_id: u64,
    state: State<'_, AppState>,
) -> Result<(), String> {
    info!("开始生成AI回复，对话ID: {}", conversation_id);

    // 创建机器人消息占位符
    let bot_message_id = chrono::Utc::now().timestamp_millis() as u64;
    let bot_message = Message {
        id: bot_message_id,
        content: String::new(), // 初始为空，将通过流式更新
        sender: "bot".to_string(),
        timestamp: chrono::Utc::now().timestamp_millis() as u64,
        conversation_id,
    };

    debug!("创建AI消息占位符: {:?}", bot_message);

    // 保存初始的空机器人消息
    state.messages.lock().unwrap().push(bot_message);

    // 开始流式生成
    let agent = state.ollama_agent.clone();

    // 生成消息流
    debug!("调用Ollama生成响应流");
    let mut stream = match agent.generate_stream(&user_message_content).await {
        Ok(stream) => {
            info!("成功创建Ollama响应流");
            stream
        }
        Err(e) => {
            error!("创建Ollama响应流失败: {}", e);
            return Err(format!("创建响应流失败: {}", e));
        }
    };

    // 完整的响应内容
    let mut full_response = String::new();

    let conv_arc = state.conversations.clone();
    let msg_arc = state.messages.clone();
    let window_clone = window.clone();

    // 启动另一个任务处理流
    debug!("启动异步任务处理响应流");
    tokio::spawn(async move {
        use tokio_stream::StreamExt;
        let mut chunk_count = 0;
        let mut buffer = String::new();
        let mut last_emit_time = std::time::Instant::now();

        while let Some(chunk) = stream.next().await {
            // 将新的内容添加到完整响应中
            full_response.push_str(&chunk);
            buffer.push_str(&chunk);
            chunk_count += 1;

            // 使用缓冲策略: 每收集一定数量的内容，或者经过一定时间后发送
            let now = std::time::Instant::now();
            let should_emit = buffer.len() >= 2 || now.duration_since(last_emit_time).as_millis() >= 3;

            if should_emit && !buffer.is_empty() {
                match window.emit(
                    "message_chunk",
                    MessageChunk {
                        conversation_id,
                        content: std::mem::take(&mut buffer), // 清空缓冲区并使用其内容
                        is_complete: false,
                    },
                ) {
                    Ok(_) => {
                        last_emit_time = now;
                    }
                    Err(e) => error!("发送消息块到前端失败: {}", e),
                }
            }

            // 更频繁地更新消息内容，避免长时间锁等待
            if chunk_count % 10 == 0 {
                let mut msgs = msg_arc.lock().unwrap();
                if let Some(msg) = msgs.iter_mut().find(|m| m.id == bot_message_id) {
                    msg.content = full_response.clone();
                }
            }
        }

        info!(
            "流式响应完成，共 {} 个响应块，总长度 {} 字符",
            chunk_count,
            full_response.len()
        );

        // 更新对话
        {
            let mut convs = conv_arc.lock().unwrap();
            if let Some(conv) = convs.iter_mut().find(|c| c.id == conversation_id) {
                conv.last_message = full_response.clone();
                conv.timestamp = chrono::Utc::now().timestamp_millis() as u64;
            }
        }

        // 更新消息
        {
            let mut msgs = msg_arc.lock().unwrap();
            if let Some(msg) = msgs.iter_mut().find(|m| m.id == bot_message_id) {
                msg.content = full_response;
            }
        }

        // 发送完成信号
        window_clone.emit(
            "message_chunk",
            MessageChunk {
                conversation_id,
                content: String::new(),
                is_complete: true,
            },
        ).unwrap();
    });

    Ok(())
}

#[tauri::command]
fn voice_input(state: State<AppState>) -> Result<String, String> {
    // 这里可以实现语音输入的逻辑
    // 例如，调用外部库进行语音识别等
    Ok("语音输入功能尚未实现".to_string())
}

#[tauri::command]
fn create_conversation(title: String, state: State<AppState>) -> Result<Conversation, String> {
    let mut conversations = state.conversations.lock().unwrap();

    // 生成新ID (简单地使用时间戳)
    let id = chrono::Utc::now().timestamp_millis() as u64;

    // 创建新对话
    let new_conversation = Conversation {
        id,
        title,
        last_message: String::from("新对话已创建"),
        timestamp: chrono::Utc::now().timestamp_millis() as u64,
    };

    // 添加到对话列表
    conversations.push(new_conversation.clone());

    info!("创建了新对话: {:?}", new_conversation);
    Ok(new_conversation)
}

#[tauri::command]
fn delete_conversation(conversation_id: u64, state: State<AppState>) -> Result<(), String> {
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

#[tauri::command]
fn get_conversation_messages(conversation_id: u64, state: State<AppState>) -> Vec<Message> {
    let messages = state.messages.lock().unwrap();
    // 只返回属于特定对话的消息
    messages
        .iter()
        .filter(|m| m.conversation_id == conversation_id)
        .cloned()
        .collect()
}

// 在main.rs中配置tokio运行时
#[tokio::main]
async fn main() {
    env_logger::builder()
        .filter(None, log::LevelFilter::Debug)
        .init();

    // 创建OllamaAgent实例
    let ollama_agent = Arc::new(
        OllamaAgent::new("qwen2.5:0.5b", "http://localhost", &11434)
            .with_system_prompt("你是一个友好、乐于助人的AI助手，使用中文回答问题。"),
    );
    info!("OllamaAgent initialized");

    let default_conversation_id = 1;

    let state = AppState {
        conversations: Arc::new(Mutex::new(vec![Conversation {
            id: default_conversation_id,
            title: "新对话".to_string(),
            last_message: "你好!".to_string(),
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
        }])),
        messages: Arc::new(Mutex::new(vec![
            Message {
                id: 1,
                content: "欢迎使用聊天应用!".to_string(),
                sender: "bot".to_string(),
                timestamp: chrono::Utc::now().timestamp_millis() as u64,
                conversation_id: default_conversation_id,
            }
        ])),
        ollama_agent,
    };

    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            get_conversations,
            get_messages,
            send_message,
            send_user_message,    // 添加新函数
            generate_ai_response, // 添加新函数
            send_message_stream,  // 保留原函数以保持兼容
            create_conversation,  // 添加新函数
            delete_conversation,  // 添加新函数
            get_conversation_messages, // 添加新函数
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
