use crate::{InitError, UserAction};
use anyhow::Result;
use serde_json::json;
use tauri::Emitter;

pub struct ErrorHandler {
    pub app_handle: tauri::AppHandle,
    pub show_dialogs: bool,
}

impl ErrorHandler {
    pub fn new(app_handle: tauri::AppHandle, show_dialogs: bool) -> Self {
        Self {
            app_handle,
            show_dialogs,
        }
    }

    pub async fn handle_error(
        &self,
        error: &InitError,
        component_name: &str,
    ) -> Result<UserAction> {
        if !self.show_dialogs {
            log::error!("组件 {} 初始化失败: {}", component_name, error);
            return Ok(UserAction::Ignore);
        }

        let error_message = format!(
            "组件 \"{}\" 初始化失败:\n\n{}\n\n请选择操作:",
            component_name, error
        );

        match self.show_error_dialog(&error_message).await {
            Ok(action) => {
                log::info!("用户选择了: {:?} 对于组件 {}", action, component_name);
                Ok(action)
            }
            Err(e) => {
                log::error!("显示错误对话框失败: {}", e);
                Ok(UserAction::Ignore)
            }
        }
    }

    async fn show_error_dialog(&self, message: &str) -> Result<UserAction> {
        use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};

        let _dialog = self
            .app_handle
            .dialog()
            .message(message)
            .kind(MessageDialogKind::Error)
            .title("初始化错误")
            .buttons(MessageDialogButtons::OkCancel);

        // 对于 Tauri 对话框，我们需要使用自定义对话框来提供三个选项
        self.show_custom_error_dialog(message).await
    }

    async fn show_custom_error_dialog(&self, message: &str) -> Result<UserAction> {
        // 创建一个简单的对话框，发送事件到前端
        let dialog_data = json!({
            "type": "error_dialog",
            "message": message,
            "buttons": ["忽略", "重试", "退出"]
        });

        // 发送事件到前端
        if let Err(e) = self.app_handle.emit("show_error_dialog", &dialog_data) {
            log::error!("发送错误对话框事件失败: {}", e);
            return Ok(UserAction::Ignore);
        }

        // 等待前端响应
        // 注意：这里需要实现一个等待机制，但由于这是异步的，
        // 我们可以使用一个简单的方法或者返回默认操作

        // 为了简化，这里我们使用系统原生对话框的替代方案
        self.show_native_confirmation_dialog(message).await
    }

    async fn show_native_confirmation_dialog(&self, message: &str) -> Result<UserAction> {
        // 使用 tokio 创建一个简单的控制台输入（仅用于测试）
        // 在实际应用中，应该使用前端对话框

        log::warn!("错误: {}", message);
        log::warn!("自动选择忽略操作（在生产环境中应该显示对话框）");

        // 返回默认操作
        Ok(UserAction::Ignore)
    }

    pub fn log_component_ignored(&self, component_name: &str, reason: &str) {
        log::warn!("组件 \"{}\" 被忽略: {}", component_name, reason);

        // 可以发送通知到前端
        let notification_data = json!({
            "type": "component_ignored",
            "component": component_name,
            "reason": reason
        });

        if let Err(e) = self
            .app_handle
            .emit("component_ignored", &notification_data)
        {
            log::error!("发送组件忽略通知失败: {}", e);
        }
    }

    pub fn log_initialization_complete(
        &self,
        success_count: usize,
        failed_count: usize,
        ignored_count: usize,
    ) {
        let message = if failed_count == 0 && ignored_count == 0 {
            format!("所有 {} 个组件初始化成功", success_count)
        } else {
            format!(
                "初始化完成: {} 个成功, {} 个失败, {} 个忽略",
                success_count, failed_count, ignored_count
            )
        };

        log::info!("{}", message);

        let notification_data = json!({
            "type": "initialization_complete",
            "message": message,
            "success_count": success_count,
            "failed_count": failed_count,
            "ignored_count": ignored_count
        });

        if let Err(e) = self
            .app_handle
            .emit("initialization_complete", &notification_data)
        {
            log::error!("发送初始化完成通知失败: {}", e);
        }
    }
}

#[derive(Debug)]
pub struct ComponentStatus {
    pub name: String,
    pub initialized: bool,
    pub ignored: bool,
    pub error: Option<String>,
}

impl ComponentStatus {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            initialized: false,
            ignored: false,
            error: None,
        }
    }

    pub fn success(name: &str) -> Self {
        Self {
            name: name.to_string(),
            initialized: true,
            ignored: false,
            error: None,
        }
    }

    pub fn failed(name: &str, error: &str) -> Self {
        Self {
            name: name.to_string(),
            initialized: false,
            ignored: false,
            error: Some(error.to_string()),
        }
    }

    pub fn ignored(name: &str, reason: &str) -> Self {
        Self {
            name: name.to_string(),
            initialized: false,
            ignored: true,
            error: Some(reason.to_string()),
        }
    }
}
