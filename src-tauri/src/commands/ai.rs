use std::sync::Arc;
use crate::models::{Message, MessageChunk};
use crate::state::AppState;
use crate::utils::config::AppConfig;
use chrono::Utc;
use log::{debug, error, info};
use tauri::{Emitter, State, Window};

#[tauri::command]
pub async fn generate_ai_response(
    window: Window,
    user_message_content: String,
    conversation_id: u64,
    state: State<'_, AppState>,
    config: State<'_, Arc<AppConfig>>,
) -> Result<(), String> {
    info!("开始生成AI回复，对话ID: {}", conversation_id);

    // 创建机器人消息占位符
    let bot_message_id = Utc::now().timestamp_millis() as u64;
    let bot_message = Message {
        id: bot_message_id,
        content: String::new(),
        sender: "bot".to_string(),
        timestamp: Utc::now().timestamp_millis() as u64,
        conversation_id,
    };

    debug!("创建AI消息占位符: {:?}", bot_message);

    // 保存初始的空机器人消息
    state.messages.lock().unwrap().push(bot_message);

    // 获取Ollama代理
    let agent = state.ollama_agent.clone();

    let history = state.get_conversation_history(conversation_id);
    let user_messages = history.iter()
        .filter(|msg| msg.sender == "user")
        .map(|msg| msg.content.clone())
        .collect::<Vec<String>>()
        .join("\n\n");
    debug!("从database中加载: {}", user_messages);

    // 生成消息流
    debug!("调用Ollama生成响应流");
    let mut stream = match agent.generate_stream(&user_messages).await {
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

    // 从配置中获取缓冲设置
    let buffer_size = config.app_behavior.message_chunk_buffer_size;
    let send_interval_ms = config.app_behavior.message_chunk_send_interval_ms;

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

            // 使用缓冲策略: 从配置获取缓冲大小和发送间隔
            let now = std::time::Instant::now();
            let should_emit =
                buffer.len() >= buffer_size || now.duration_since(last_emit_time).as_millis() >= send_interval_ms as u128;

            if should_emit && !buffer.is_empty() {
                match window.emit(
                    "message_chunk",
                    MessageChunk {
                        conversation_id,
                        content: buffer.clone(),
                        is_complete: false,
                    },
                ) {
                    Ok(_) => {
                        buffer.clear();
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
                conv.timestamp = Utc::now().timestamp_millis() as u64;
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
        window_clone
            .emit(
                "message_chunk",
                MessageChunk {
                    conversation_id,
                    content: String::new(),
                    is_complete: true,
                },
            )
            .unwrap();
    });

    Ok(())
}