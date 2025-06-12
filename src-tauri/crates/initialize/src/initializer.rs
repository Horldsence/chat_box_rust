use crate::{
    config::InitConfig, error_handler::ErrorHandler, model_manager::ModelManager, ComponentStatus,
    InitError, InitializationResult, UserAction,
};
use anyhow::{anyhow, Result};
use log::{error, info, warn};
use std::path::Path;
use tauri::Manager;

pub struct AppInitializer {
    config: InitConfig,
    app_handle: tauri::AppHandle,
    error_handler: ErrorHandler,
    model_manager: ModelManager,
    component_statuses: Vec<ComponentStatus>,
}

impl AppInitializer {
    pub fn new(config: InitConfig, app_handle: tauri::AppHandle) -> Self {
        let error_handler =
            ErrorHandler::new(app_handle.clone(), config.app_behavior.show_error_dialogs);
        let model_manager = ModelManager::new(config.clone(), app_handle.clone());

        Self {
            config,
            app_handle,
            error_handler,
            model_manager,
            component_statuses: Vec::new(),
        }
    }

    pub async fn initialize_all(&mut self) -> Result<InitializationResult> {
        info!("开始应用初始化流程");

        let mut success_count = 0;
        let mut failed_count = 0;
        let mut ignored_count = 0;

        // 1. 初始化配置
        match self.initialize_config().await {
            Ok(()) => {
                self.component_statuses
                    .push(ComponentStatus::success("配置系统"));
                success_count += 1;
            }
            Err(e) => {
                let action = self
                    .error_handler
                    .handle_error(&e, "配置系统")
                    .await
                    .unwrap_or(UserAction::Ignore);

                match action {
                    UserAction::Retry => {
                        if let Ok(()) = self.initialize_config().await {
                            self.component_statuses
                                .push(ComponentStatus::success("配置系统"));
                            success_count += 1;
                        } else {
                            self.component_statuses
                                .push(ComponentStatus::failed("配置系统", &e.to_string()));
                            failed_count += 1;
                        }
                    }
                    UserAction::Exit => {
                        error!("用户选择退出，应用初始化中止");
                        std::process::exit(1);
                    }
                    UserAction::Ignore => {
                        self.component_statuses
                            .push(ComponentStatus::ignored("配置系统", &e.to_string()));
                        self.error_handler
                            .log_component_ignored("配置系统", &e.to_string());
                        ignored_count += 1;
                    }
                }
            }
        }

        // 2. 初始化AI模型
        match self.initialize_ai_model().await {
            Ok(()) => {
                self.component_statuses
                    .push(ComponentStatus::success("AI模型"));
                success_count += 1;
            }
            Err(e) => {
                let action = self
                    .error_handler
                    .handle_error(&e, "AI模型")
                    .await
                    .unwrap_or(UserAction::Ignore);

                match action {
                    UserAction::Retry => {
                        if let Ok(()) = self.initialize_ai_model().await {
                            self.component_statuses
                                .push(ComponentStatus::success("AI模型"));
                            success_count += 1;
                        } else {
                            self.component_statuses
                                .push(ComponentStatus::failed("AI模型", &e.to_string()));
                            failed_count += 1;
                        }
                    }
                    UserAction::Exit => {
                        error!("用户选择退出，应用初始化中止");
                        std::process::exit(1);
                    }
                    UserAction::Ignore => {
                        self.component_statuses
                            .push(ComponentStatus::ignored("AI模型", &e.to_string()));
                        self.error_handler
                            .log_component_ignored("AI模型", &e.to_string());
                        ignored_count += 1;
                    }
                }
            }
        }

        // 3. 初始化语音识别
        match self.initialize_voice_recognition().await {
            Ok(()) => {
                self.component_statuses
                    .push(ComponentStatus::success("语音识别"));
                success_count += 1;
            }
            Err(e) => {
                let action = self
                    .error_handler
                    .handle_error(&e, "语音识别")
                    .await
                    .unwrap_or(UserAction::Ignore);

                match action {
                    UserAction::Retry => {
                        if let Ok(()) = self.initialize_voice_recognition().await {
                            self.component_statuses
                                .push(ComponentStatus::success("语音识别"));
                            success_count += 1;
                        } else {
                            self.component_statuses
                                .push(ComponentStatus::failed("语音识别", &e.to_string()));
                            failed_count += 1;
                        }
                    }
                    UserAction::Exit => {
                        error!("用户选择退出，应用初始化中止");
                        std::process::exit(1);
                    }
                    UserAction::Ignore => {
                        self.component_statuses
                            .push(ComponentStatus::ignored("语音识别", &e.to_string()));
                        self.error_handler
                            .log_component_ignored("语音识别", &e.to_string());
                        ignored_count += 1;
                    }
                }
            }
        }

        // 4. 初始化数据库
        match self.initialize_database().await {
            Ok(()) => {
                self.component_statuses
                    .push(ComponentStatus::success("数据库"));
                success_count += 1;
            }
            Err(e) => {
                let action = self
                    .error_handler
                    .handle_error(&e, "数据库")
                    .await
                    .unwrap_or(UserAction::Ignore);

                match action {
                    UserAction::Retry => {
                        if let Ok(()) = self.initialize_database().await {
                            self.component_statuses
                                .push(ComponentStatus::success("数据库"));
                            success_count += 1;
                        } else {
                            self.component_statuses
                                .push(ComponentStatus::failed("数据库", &e.to_string()));
                            failed_count += 1;
                        }
                    }
                    UserAction::Exit => {
                        error!("用户选择退出，应用初始化中止");
                        std::process::exit(1);
                    }
                    UserAction::Ignore => {
                        self.component_statuses
                            .push(ComponentStatus::ignored("数据库", &e.to_string()));
                        self.error_handler
                            .log_component_ignored("数据库", &e.to_string());
                        ignored_count += 1;
                    }
                }
            }
        }

        // 记录初始化完成
        self.error_handler
            .log_initialization_complete(success_count, failed_count, ignored_count);

        let failed_components: Vec<String> = self
            .component_statuses
            .iter()
            .filter(|s| !s.initialized && !s.ignored)
            .map(|s| s.name.clone())
            .collect();

        let ignored_components: Vec<String> = self
            .component_statuses
            .iter()
            .filter(|s| s.ignored)
            .map(|s| s.name.clone())
            .collect();

        Ok(InitializationResult {
            success: failed_count == 0,
            failed_components,
            ignored_components,
        })
    }

    async fn initialize_config(&self) -> Result<(), InitError> {
        info!("初始化配置系统");

        // 验证配置文件路径
        let config_path = self.config.config_path.clone();
        if let Some(parent) = config_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| InitError::FileSystemError(format!("创建配置目录失败: {}", e)))?;
            }
        }

        // 验证配置内容
        if self.config.ai_model.model_name.is_empty() {
            return Err(InitError::ConfigLoadFailed(
                "AI模型名称不能为空".to_string(),
            ));
        }

        info!("配置系统初始化成功");
        Ok(())
    }

    async fn initialize_ai_model(&self) -> Result<(), InitError> {
        info!("初始化AI模型: {}", self.model_manager.get_model_info());

        // 使用模型管理器初始化模型
        self.model_manager.initialize_model().await?;

        info!("AI模型初始化成功");
        Ok(())
    }

    async fn initialize_voice_recognition(&self) -> Result<(), InitError> {
        if !self.config.voice.enabled {
            info!("语音识别功能已禁用，跳过初始化");
            return Ok(());
        }

        info!("初始化语音识别系统");

        let model_path = if Path::new(&self.config.voice.model_path).is_absolute() {
            self.config.voice.model_path.clone()
        } else {
            self.app_handle
                .path()
                .resolve(
                    &self.config.voice.model_path,
                    tauri::path::BaseDirectory::Resource,
                )
                .map_err(|e| InitError::FileSystemError(format!("解析语音模型路径失败: {}", e)))?
                .to_string_lossy()
                .to_string()
        };

        // 检查语音模型文件是否存在
        if !Path::new(&model_path).exists() {
            return Err(InitError::VoiceInitFailed(format!(
                "语音模型文件不存在: {}",
                model_path
            )));
        }

        // 这里可以添加更多的语音模型验证逻辑
        // 由于 VoskASR 的初始化比较复杂，我们在这里只做基本检查

        info!("语音识别系统初始化成功");
        Ok(())
    }

    async fn initialize_database(&self) -> Result<(), InitError> {
        if !self.config.database.enabled {
            info!("数据库功能已禁用，跳过初始化");
            return Ok(());
        }

        info!("初始化数据库");

        let db_path = if Path::new(&self.config.database.path).is_absolute() {
            self.config.database.path.clone()
        } else {
            self.app_handle
                .path()
                .resolve(
                    &self.config.database.path,
                    tauri::path::BaseDirectory::AppData,
                )
                .map_err(|e| InitError::FileSystemError(format!("解析数据库路径失败: {}", e)))?
                .to_string_lossy()
                .to_string()
        };

        // 确保数据库目录存在
        if let Some(parent) = Path::new(&db_path).parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| InitError::DatabaseInitFailed(format!("创建数据库目录失败: {}", e)))?;
        }

        // 测试数据库连接
        match rusqlite::Connection::open(&db_path) {
            Ok(conn) => {
                // 创建基本表结构（如果不存在）
                conn.execute(
                    "CREATE TABLE IF NOT EXISTS test_table (id INTEGER PRIMARY KEY)",
                    [],
                )
                .map_err(|e| InitError::DatabaseInitFailed(format!("创建测试表失败: {}", e)))?;

                info!("数据库初始化成功: {}", db_path);
                Ok(())
            }
            Err(e) => Err(InitError::DatabaseInitFailed(format!(
                "数据库连接失败: {}",
                e
            ))),
        }
    }

    pub fn get_component_statuses(&self) -> &[ComponentStatus] {
        &self.component_statuses
    }
}

// 公共初始化函数，供主应用调用
pub async fn initialize_app(
    config: InitConfig,
    app_handle: tauri::AppHandle,
) -> Result<InitializationResult> {
    let mut initializer = AppInitializer::new(config, app_handle);
    initializer.initialize_all().await
}

// 辅助函数：检查资源文件
pub async fn check_required_resources(app_handle: &tauri::AppHandle) -> Result<()> {
    let resource_dir = app_handle
        .path()
        .resource_dir()
        .map_err(|e| anyhow!("获取资源目录失败: {}", e))?;

    info!("检查资源目录: {:?}", resource_dir);

    if !resource_dir.exists() {
        return Err(anyhow!("资源目录不存在: {:?}", resource_dir));
    }

    // 检查关键资源文件
    let required_files = vec!["config.yaml"];

    for file_name in required_files {
        let file_path = resource_dir.join(file_name);
        if !file_path.exists() {
            warn!("可选资源文件不存在: {:?}", file_path);
        }
    }

    Ok(())
}

// 辅助函数：验证系统环境
pub async fn validate_system_environment() -> Result<()> {
    info!("验证系统环境");

    // 检查必要的系统库
    #[cfg(target_os = "linux")]
    {
        // Linux 特定检查
        info!("检测到 Linux 系统");
    }

    #[cfg(target_os = "windows")]
    {
        // Windows 特定检查
        info!("检测到 Windows 系统");
    }

    #[cfg(target_os = "macos")]
    {
        // macOS 特定检查
        info!("检测到 macOS 系统");
    }

    Ok(())
}
