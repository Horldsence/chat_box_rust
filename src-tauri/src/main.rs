// Prevents additional console window on Windows in release, DO NOT REMOVE!!
// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod commands;
mod models;
mod services;
mod state;
mod utils;

use log::{error, info};
use tauri::Manager;

use commands::*;
use utils::initializer::init_app_state;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let handle = app.handle().clone();

            // 简化初始化过程 - 在新线程中异步初始化状态
            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async move {
                    info!("开始应用状态初始化");
                    match init_app_state(handle.clone()).await {
                        Ok(app_state) => {
                            handle.manage(app_state);
                            info!("App state initialized successfully");
                        }
                        Err(e) => {
                            error!("应用状态初始化失败: {}", e);
                        }
                    }
                });
            });

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
        .plugin(tauri_plugin_log::Builder::new().build())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
