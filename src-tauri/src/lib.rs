// Prevents additional console window on Windows in release, DO NOT REMOVE!!
// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod commands;
mod services;
mod state;
mod utils;

use log::{error, info, warn};
use tauri::Manager;
use tauri_plugin_log::{Target, TargetKind};

use commands::*;
use utils::initializer::init_app_state;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 配置日志插件
    let log_plugin = tauri_plugin_log::Builder::new()
        .targets([
            // 输出到控制台/终端
            Target::new(TargetKind::Stdout),
            // 在 debug 模式下输出到文件
            #[cfg(debug_assertions)]
            Target::new(TargetKind::LogDir {
                file_name: Some("chat-box".to_string()),
            }),
            // 输出到 webview 控制台
            Target::new(TargetKind::Webview),
        ])
        .level(log::LevelFilter::Info)
        // 为不同模块设置不同日志级别
        .level_for("chat_box", log::LevelFilter::Debug)
        .level_for("agent", log::LevelFilter::Debug)
        .level_for("initialize", log::LevelFilter::Debug)
        // 过滤一些噪音日志
        .filter(|metadata| {
            !metadata.target().starts_with("hyper") && !metadata.target().starts_with("reqwest")
        })
        // 自定义日志格式
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}][{}][{}] {}",
                chrono::Utc::now().format("%Y-%m-%d][%H:%M:%S"),
                record.target(),
                record.level(),
                message
            ))
        })
        .build();

    tauri::Builder::default()
        .plugin(log_plugin)
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Debug)
                .targets([
                    Target::new(TargetKind::Stdout),
                    Target::new(TargetKind::LogDir {
                        file_name: Some("logs".to_string()),
                    }),
                    Target::new(TargetKind::Webview),
                ])
                .build()
        )
        .setup(|app| {
            let handle = app.handle().clone();

            info!("开始应用初始化...");

            // 同步初始化应用状态以确保在任何命令调用前完成
            let rt = tokio::runtime::Runtime::new().map_err(|e| {
                error!("无法创建 Tokio 运行时: {}", e);
                e
            })?;

            rt.block_on(async move {
                info!("开始应用状态初始化");
                match init_app_state(handle.clone()).await {
                    Ok(app_state) => {
                        handle.manage(app_state);
                        info!("应用状态初始化成功");
                    }
                    Err(e) => {
                        error!("应用状态初始化失败: {}", e);
                        // 创建一个最小状态以防止程序崩溃
                        warn!("创建最小应用状态作为备用");
                        match create_minimal_state(handle.clone()).await {
                            Ok(minimal_state) => {
                                handle.manage(minimal_state);
                                warn!("最小应用状态创建成功");
                            }
                            Err(minimal_err) => {
                                error!("创建最小应用状态也失败了: {}", minimal_err);
                                return Err(format!(
                                    "应用初始化完全失败: 主要错误: {}, 备用错误: {}",
                                    e, minimal_err
                                ));
                            }
                        }
                    }
                }
                Ok(())
            })?;

            info!("应用设置完成");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // 对话相关命令
            get_conversations,
            get_conversation_messages,
            create_conversation,
            delete_conversation,
            // 消息相关命令
            send_user_message,
            // AI相关命令
            generate_ai_response,
            // 语音相关命令
            voice_input,
            // 配置相关命令
            get_app_config,
            save_app_config,
            reset_app_config,
            // 系统相关命令
            get_system_info,
            ping,
            show_notification,
            get_health_status,
            // 日志相关命令
            log_error,
            log_warning,
            log_info,
            // 数据库管理命令
            get_database_conversations,
            delete_database_conversation,
            // 对话框相关命令
            show_info_dialog,
            show_warning_dialog,
            show_error_dialog,
            show_confirm_dialog,
            show_ask_dialog,
            open_file_dialog,
            open_folder_dialog,
            save_file_dialog,
            import_config_file,
            export_config_file,
            export_chat_history,
            select_voice_model_folder,
            select_database_file,
            create_database_file,
            // Candle测试命令
            test_candle_model,
            test_candle_stream,
            get_candle_models,
            check_candle_health,
            test_candle_with_manager,
            // Live2D相关命令
            get_live2d_config,
            update_live2d_config,
            execute_live2d_action,
            execute_live2d_action_by_type,
            start_live2d_speaking,
            stop_live2d_speaking,
            start_live2d_thinking,
            set_live2d_expression,
            process_ai_text_for_live2d,
            clear_live2d_text_buffer,
            get_live2d_state,
            process_live2d_action_queue,
            add_live2d_text_triggers,
            remove_live2d_text_trigger,
            reset_live2d_config,
            test_live2d_connection,
            // Agent相关命令
            get_agent_config,
            update_agent_config,
            get_agent_templates,
            apply_agent_template,
            add_agent_template,
            remove_agent_template,
            build_system_prompt,
            process_user_message_for_agent,
            get_agent_preset_response,
            add_agent_preset_response,
            get_agent_state,
            reset_agent_session,
            export_agent_config,
            import_agent_config,
            update_agent_personality,
            update_agent_behavior,
            update_agent_live2d_integration,
            update_agent_preset_responses,
            remove_agent_preset_response,
            update_agent_knowledge_domains,
            test_agent_config,
            create_custom_agent_template,
            clone_agent_template,
            // 配置相关命令
            get_app_config_full,
            get_live2d_config_from_file,
            check_live2d_model,
            get_live2d_model_status,
            update_live2d_config_in_file,
            disable_live2d,
            enable_live2d,
            check_live2d_environment,
            // 调试命令
            debug_database_status,
            debug_memory_state,
            debug_clear_database,
            debug_test_database_connection,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// 创建最小应用状态作为备用方案
async fn create_minimal_state(
    handle: tauri::AppHandle,
) -> Result<state::AppState, Box<dyn std::error::Error>> {
    use cb_config::AppConfig;
    use chrono::Utc;
    use db::models::{Conversation, Message};
    use services::asr::vosk_python::VoskASR;

    warn!("创建最小配置状态");

    // 使用默认配置
    let config = AppConfig::default();

    // 创建基本的对话和消息
    let conversations = vec![Conversation {
        id: 1,
        title: "默认对话".to_string(),
        last_message: "应用正在启动中...".to_string(),
        timestamp: Utc::now().timestamp_millis() as u64,
    }];

    let messages = vec![Message {
        id: 1,
        content: "应用已启动，但初始化过程中遇到了一些问题。请检查配置或重启应用。".to_string(),
        sender: "system".to_string(),
        timestamp: Utc::now().timestamp_millis() as u64,
        conversation_id: 1,
    }];

    // 尝试创建简化的语音识别服务
    let vosk_asr = VoskASR::new(None).unwrap_or_else(|e| {
        warn!("无法初始化语音识别服务: {}", e);
        // 这里应该返回一个虚拟的 VoskASR 实例
        // 为了编译通过，我们暂时使用默认路径再试一次
        VoskASR::new(Some("models/vosk-model-small-cn-0.22")).unwrap_or_else(|_| {
            // 如果还是失败，我们需要一个处理方案
            panic!("无法创建 VoskASR 实例")
        })
    });

    let tts_engine =  None;

    state::AppState::new(
        config,
        conversations,
        messages,
        vosk_asr,
        handle,
        tts_engine,
    ).await
}
