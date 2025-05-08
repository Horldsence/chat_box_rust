use crate::state::AppState;
use log::{debug, error, info};
use tauri::{Emitter, State, Window};
use tokio_stream::StreamExt;

#[tauri::command]
pub async fn voice_input(
    window: Window,
    conversation_id: u64,
    state: State<'_, AppState>,
) -> Result<String, String> {
    info!("开始语音输入，对话ID: {}", conversation_id);

    // 通知前端录音开始
    window
        .emit("voice_status", "recording")
        .map_err(|e| e.to_string())?;
    debug!("已通知前端录音");

    // 使用oneshot通道获取结果
    let (tx, rx) = tokio::sync::oneshot::channel();
    let vosk_asr_clone = state.vosk_asr.clone();
    let window_clone1 = window.clone();
    let window_clone2 = window.clone();

    // 在单独的任务中处理语音识别，避免跨线程共享VoskASR实例
    tokio::spawn(async move {
        let mut final_transcript = String::new();

        // 在单独作用域中锁定vosk_asr，确保锁被及时释放
        {
            let _ = window_clone1.emit("voice_partial", "[booting]");
            let mut vosk_asr = match vosk_asr_clone.lock().await {
                vosk => vosk,
            };

            // 直接调用，不使用catch_unwind
            let stream_result = match vosk_asr.listen_and_transcribe(Some(15)).await {
                Ok(stream) => Ok(stream),
                Err(e) => {
                    error!("创建流失败: {:?}", e);
                    Err(e)
                }
            };

            // 使用新的流式API
            match stream_result {
                Ok(mut stream) => {
                    debug!("成功创建语音输入流");

                    while let Some(result) = stream.next().await {
                        match result {
                            Ok(text) => {
                                // 特殊标记处理
                                if text == "[timeout reached]" || text == "[silence detected]" {
                                    info!("录音事件: {}", text);
                                    debug!("特殊标记: {}", text);
                                    let _ = window_clone1.emit("voice_partial", &text);

                                    // 如果是超时，则结束流程
                                    if text == "[timeout reached]" {
                                        info!("因超时结束录音");
                                        break;
                                    }
                                    continue;
                                }

                                // 发送部分结果到前端
                                let _ = window_clone1.emit("voice_partial", &text);

                                // 更新最终文本（仅当不为空时）
                                if !text.trim().is_empty() {
                                    final_transcript = text;
                                }
                            }
                            Err(e) => {
                                error!("语音识别错误: {}", e);
                                let _ = window_clone1.emit("voice_status", "error");
                                let _ = tx.send(Err(format!("语音识别出错: {}", e)));
                                return;
                            }
                        }
                    }

                    // 显式停止录音
                    debug!("流处理完成，显式停止录音");
                    if let Err(e) = vosk_asr.stop_recording() {
                        error!("停止录音失败: {:?}", e);
                    }
                }
                Err(e) => {
                    error!("创建语音输入流失败: {}", e);
                    let _ = window_clone1.emit("voice_status", "error");
                    let _ = tx.send(Err(format!("创建语音输入流失败: {}", e)));
                    return;
                }
            }
        }

        debug!("语音识别任务完成");
        // 通知前端录音完成
        let _ = window_clone1.emit("voice_status", "completed");

        // 发送最终结果
        let _ = tx.send(Ok(final_transcript));
    });

    // 等待任务完成，获取结果
    match rx.await {
        Ok(result) => {
            debug!("语音输入完成，返回结果");
            result
        }
        Err(e) => {
            error!("等待语音识别结果失败: {}", e);
            window_clone2
                .emit("voice_status", "error")
                .map_err(|e| e.to_string())?;
            Err(format!("等待语音识别结果失败: {}", e))
        }
    }
}