// Prevents additional console window on Windows in release, DO NOT REMOVE!!
// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod commands;
mod models;
mod services;
mod state;
mod utils;

use initialize::conditional_initialize_app;
use log::{error, info};
use tauri::Manager;

use commands::*;
use utils::initializer::init_app_state;

fn main() {
    tauri::Builder::default()
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
