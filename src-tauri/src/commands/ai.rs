use crate::models::{Message, MessageChunk};
use crate::state::AppState;
use log::{debug, error, info};
use tauri::{Emitter, State, Window};

#[tauri::command]
pub async fn generate_ai_response(
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
            let should_emit =
                buffer.len() >= 2 || now.duration_since(last_emit_time).as_millis() >= 3;

            if should_emit && !buffer.is_empty() {
                match window.emit(
                    "message_chunk",
                    MessageChunk {
                        conversation_id,
                        content: chunk.to_owned(),
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