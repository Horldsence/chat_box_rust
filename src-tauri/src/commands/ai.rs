use crate::state::AppState;
use agent::StreamGenerator;
use chrono::Utc;
use db::models::{Message, MessageChunk};
use log::{debug, error, info};
use serde::Deserialize;
use tauri::{Emitter, State, Window};

#[derive(Deserialize)]
pub struct GenerateAIResponseRequest {
    pub user_message_content: String,
    pub conversation_id: u64,
}

#[tauri::command]
pub async fn generate_ai_response(
    window: Window,
    request: GenerateAIResponseRequest,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let user_message_content = request.user_message_content;
    let conversation_id = request.conversation_id;

    let user_message = Message {
        id: Utc::now().timestamp_millis() as u64,
        content: user_message_content.clone(),
        sender: "user".to_string(),
        timestamp: Utc::now().timestamp_millis() as u64,
        conversation_id,
    };
    debug!("收到用户消息: {:?}", user_message);
    if let Ok(mut db_guard) = state.db.lock() {
        if let Some(ref mut db) = *db_guard {
            if let Err(e) = db.save_message(&user_message) {
                error!("保存初始AI消息到数据库失败: {}", e);
            } else {
                debug!("初始AI消息已保存到数据库: {}", user_message.id);
            }
        }
    }

    info!("开始生成AI回复，对话ID: {}", conversation_id);

    // 触发Live2D思考状态
    {
        let live2d_service = state.live2d_service.lock().await;
        if let Err(e) = live2d_service.start_thinking().await {
            error!("启动Live2D思考状态失败: {}", e);
        }
    }

    // 处理用户消息以便Agent分析
    {
        let agent_service = state.agent_service.lock().await;
        if let Err(e) = agent_service
            .process_user_message(&user_message_content)
            .await
        {
            error!("Agent处理用户消息失败: {}", e);
        }
    }

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

    // 保存初始的空机器人消息到内存和数据库
    state.messages.lock().unwrap().push(bot_message.clone());

    // 尝试保存初始消息到数据库
    if let Ok(mut db_guard) = state.db.lock() {
        if let Some(ref mut db) = *db_guard {
            if let Err(e) = db.save_message(&bot_message) {
                error!("保存初始AI消息到数据库失败: {}", e);
            } else {
                debug!("初始AI消息已保存到数据库: {}", bot_message.id);
            }
        }
    }

    // 获取LLM管理器
    let llm_manager = state.llm_manager.clone();

    let history = state.get_conversation_history(conversation_id);

    // 构建Agent增强的系统提示词
    let agent_service = state.agent_service.lock().await;
    let system_prompt = agent_service
        .build_system_prompt(Some(&format!(
            "当前对话上下文：用户刚刚说了：{}",
            user_message_content
        )))
        .await;
    drop(agent_service);

    // 构建完整的对话历史提示
    let mut prompt = format!("{}\n\n", system_prompt);
    for msg in &history {
        match msg.sender.as_str() {
            "user" => prompt.push_str(&format!("User: {}\n", msg.content)),
            "bot" => prompt.push_str(&format!("Assistant: {}\n", msg.content)),
            _ => {}
        }
    }
    prompt.push_str(&format!("User: {}\nAssistant: ", user_message_content));

    debug!("构建的对话提示: {}", prompt);

    // 生成消息流
    debug!("调用LLM管理器生成响应流");
    let mut stream = match llm_manager.generate_stream(&prompt).await {
        Ok(stream) => {
            info!("成功创建LLM响应流");
            // 开始说话状态
            let live2d_service = state.live2d_service.lock().await;
            if let Err(e) = live2d_service.start_speaking().await {
                error!("启动Live2D说话状态失败: {}", e);
            }
            drop(live2d_service);
            stream
        }
        Err(e) => {
            error!("创建LLM响应流失败: {}", e);
            // 如果生成失败，恢复到idle状态
            let live2d_service = state.live2d_service.lock().await;
            if let Err(live2d_err) = live2d_service.stop_speaking().await {
                error!("恢复Live2D状态失败: {}", live2d_err);
            }
            return Err(format!("创建响应流失败: {}", e));
        }
    };

    // 完整的响应内容
    let mut full_response = String::new();

    let conv_arc = state.conversations.clone();
    let msg_arc = state.messages.clone();
    let config_arc = state.config.clone();
    let db_arc = state.db.clone();
    let live2d_service_arc = state.live2d_service.clone();
    let tts_engine_arc = state.tts_service.clone();
    let window_clone = window.clone();

    let config = config_arc.lock().unwrap();
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

        // 引入tts_speak命令
        use crate::commands::tts::tts_speak;
        use crate::commands::tts::TTSSpeakRequest;

        while let Some(chunk) = stream.next().await {
            // 将新的内容添加到完整响应中
            full_response.push_str(&chunk);
            buffer.push_str(&chunk);
            chunk_count += 1;

            debug!(
                "接收到响应块 {}: '{}', 当前总长度: {}",
                chunk_count,
                chunk,
                full_response.len()
            );

            // 处理流式文本以触发Live2D动作
            {
                let live2d_service = live2d_service_arc.lock().await;
                if let Err(e) = live2d_service.process_streaming_text(&chunk).await {
                    error!("Live2D处理流式文本失败: {}", e);
                }
            }

            // 实时TTS播报每个chunk（可根据实际需求调整为buffer）
            if !chunk.trim().is_empty() {
                let text = chunk.clone();
                let tts_engine_guard = tts_engine_arc
                    .lock()
                    .await;
                let tts_engine = tts_engine_guard.as_ref();
                match tts_engine {
                    Some(engine) => {
                        if let Err(e) = engine.speak(&text) {
                            error!("TTS合成失败: {}", e);
                        } else {
                            debug!("TTS合成成功: '{}'", chunk);
                        }
                    }
                    None => {
                        error!("TTS引擎未初始化，无法进行语音合成");
                    }
                }
            }

            // 使用缓冲策略: 从配置获取缓冲大小和发送间隔
            let now = std::time::Instant::now();
            let should_emit = buffer.len() >= buffer_size
                || now.duration_since(last_emit_time).as_millis() >= send_interval_ms as u128;

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
        debug!("完整AI响应内容: '{}'", full_response);

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
                msg.content = full_response.clone();
                debug!(
                    "更新AI消息内容: ID={}, 内容长度={}, 内容='{}'",
                    msg.id,
                    msg.content.len(),
                    msg.content
                );

                // 保存完整的AI响应到数据库
                if let Ok(mut db_guard) = db_arc.lock() {
                    if let Some(ref mut db) = *db_guard {
                        if let Err(e) = db.save_message(msg) {
                            error!("保存AI响应消息到数据库失败: {}", e);
                        } else {
                            debug!(
                                "AI响应消息已保存到数据库: ID={}, 内容='{}'",
                                msg.id, msg.content
                            );
                        }
                    }
                } else {
                    error!("无法获取数据库锁来保存AI响应");
                }
            } else {
                error!("未找到要更新的AI消息，ID: {}", bot_message_id);
            }
        }

        // 保存更新后的对话到数据库
        {
            let convs = conv_arc.lock().unwrap();
            if let Some(conv) = convs.iter().find(|c| c.id == conversation_id) {
                if let Ok(mut db_guard) = db_arc.lock() {
                    if let Some(ref mut db) = *db_guard {
                        if let Err(e) = db.save_conversation(conv) {
                            error!("保存更新的对话到数据库失败: {}", e);
                        } else {
                            debug!("对话已更新到数据库: {}", conv.id);
                        }
                    }
                }
            }
        }

        // 停止Live2D说话状态
        {
            let live2d_service = live2d_service_arc.lock().await;
            if let Err(e) = live2d_service.stop_speaking().await {
                error!("停止Live2D说话状态失败: {}", e);
            }
            // 清空文本缓冲区
            live2d_service.clear_text_buffer().await;
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
