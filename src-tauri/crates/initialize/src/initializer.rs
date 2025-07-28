use crate::{
    error_handler::ErrorHandler, model_manager::ModelManager, ComponentStatus, InitError,
    InitializationResult, UserAction,
};
use anyhow::{anyhow, Result};
use cb_config::config::InitConfig;
use log::{error, info, warn};
use serde_yaml;
use std::path::Path;
use std::time::{Duration, Instant};
use tauri::{Emitter, Manager};
use tauri_plugin_dialog::{DialogExt, MessageDialogKind};

#[derive(Debug, Clone, serde::Serialize)]
pub struct InitProgress {
    pub current_step: String,
    pub step_index: usize,
    pub total_steps: usize,
    pub progress_percent: f32,
    pub elapsed_time: Duration,
    pub estimated_remaining: Option<Duration>,
}

impl InitProgress {
    pub fn new(
        current_step: String,
        step_index: usize,
        total_steps: usize,
        elapsed_time: Duration,
    ) -> Self {
        let progress_percent = (step_index as f32 / total_steps as f32) * 100.0;
        Self {
            current_step,
            step_index,
            total_steps,
            progress_percent,
            elapsed_time,
            estimated_remaining: None,
        }
    }

    pub fn with_estimated_remaining(mut self, estimated: Duration) -> Self {
        self.estimated_remaining = Some(estimated);
        self
    }
}

pub struct AppInitializer {
    config: InitConfig,
    app_handle: tauri::AppHandle,
    error_handler: ErrorHandler,
    model_manager: ModelManager,
    component_statuses: Vec<ComponentStatus>,
    start_time: Instant,
    step_durations: Vec<Duration>,
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
            start_time: Instant::now(),
            step_durations: Vec::new(),
        }
    }

    fn emit_progress(&self, progress: InitProgress) {
        info!(
            "初始化进度: {:.1}% - {}",
            progress.progress_percent, progress.current_step
        );

        // 发送进度事件到前端
        if let Err(e) = self.app_handle.emit("init_progress", &progress) {
            warn!("发送进度事件失败: {}", e);
        }
    }

    // fn show_progress_dialog(&self, message: &str, progress_percent: f32) {
    //     let title = format!("初始化进度 - {:.0}%", progress_percent);
    //     let _ = self
    //         .app_handle
    //         .dialog()
    //         .message(message)
    //         .title(&title)
    //         .kind(MessageDialogKind::Info);
    // }

    fn calculate_estimated_time(
        &self,
        current_step: usize,
        total_steps: usize,
    ) -> Option<Duration> {
        if current_step == 0 || self.step_durations.is_empty() {
            return None;
        }

        let avg_duration: Duration =
            self.step_durations.iter().sum::<Duration>() / self.step_durations.len() as u32;
        let remaining_steps = total_steps - current_step;
        Some(avg_duration * remaining_steps as u32)
    }

    pub async fn initialize_all(&mut self) -> Result<InitializationResult> {
        info!("开始应用初始化流程");
        self.start_time = Instant::now();

        let total_steps = 4;
        let mut success_count = 0;
        let mut failed_count = 0;
        let mut ignored_count = 0;

        // 显示初始化开始消息
        if self.config.app_behavior.show_error_dialogs {
            let _ = self
                .app_handle
                .dialog()
                .message("正在初始化应用组件，请稍候...")
                .title("Chat Box 初始化")
                .kind(MessageDialogKind::Info);
        }

        // 1. 初始化配置
        let step_start = Instant::now();
        let progress = InitProgress::new(
            "正在初始化配置系统...".to_string(),
            1,
            total_steps,
            self.start_time.elapsed(),
        );
        self.emit_progress(progress);

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

        self.step_durations.push(step_start.elapsed());

        // 2. 初始化AI模型
        let step_start = Instant::now();
        let estimated = self.calculate_estimated_time(2, total_steps);
        let progress = InitProgress::new(
            "正在初始化AI模型...".to_string(),
            2,
            total_steps,
            self.start_time.elapsed(),
        );
        let progress = if let Some(est) = estimated {
            progress.with_estimated_remaining(est)
        } else {
            progress
        };
        self.emit_progress(progress);

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

        self.step_durations.push(step_start.elapsed());

        // 3. 初始化语音识别
        let step_start = Instant::now();
        let estimated = self.calculate_estimated_time(3, total_steps);
        let progress = InitProgress::new(
            "正在初始化语音识别系统...".to_string(),
            3,
            total_steps,
            self.start_time.elapsed(),
        );
        let progress = if let Some(est) = estimated {
            progress.with_estimated_remaining(est)
        } else {
            progress
        };
        self.emit_progress(progress);

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

        self.step_durations.push(step_start.elapsed());

        // 4. 初始化数据库
        let step_start = Instant::now();
        let estimated = self.calculate_estimated_time(4, total_steps);
        let progress = InitProgress::new(
            "正在初始化数据库系统...".to_string(),
            4,
            total_steps,
            self.start_time.elapsed(),
        );
        let progress = if let Some(est) = estimated {
            progress.with_estimated_remaining(est)
        } else {
            progress
        };
        self.emit_progress(progress);

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

        self.step_durations.push(step_start.elapsed());

        // 发送完成进度
        let final_progress = InitProgress::new(
            "初始化完成".to_string(),
            total_steps,
            total_steps,
            self.start_time.elapsed(),
        );
        self.emit_progress(final_progress);

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

        // 显示初始化结果
        let total_time = self.start_time.elapsed();
        let result_message = if failed_count == 0 {
            format!(
                "应用初始化成功完成！\n总用时: {:.2}秒\n成功组件: {}",
                total_time.as_secs_f32(),
                success_count
            )
        } else {
            format!(
                "应用初始化部分完成\n总用时: {:.2}秒\n成功: {} | 失败: {} | 忽略: {}",
                total_time.as_secs_f32(),
                success_count,
                failed_count,
                ignored_count
            )
        };

        if self.config.app_behavior.show_error_dialogs {
            let dialog_kind = if failed_count == 0 {
                MessageDialogKind::Info
            } else {
                MessageDialogKind::Warning
            };

            let _ = self
                .app_handle
                .dialog()
                .message(&result_message)
                .title("初始化完成")
                .kind(dialog_kind);
        }

        info!("{}", result_message);

        Ok(InitializationResult {
            success: failed_count == 0,
            failed_components,
            ignored_components,
        })
    }

    async fn initialize_config(&self) -> Result<(), InitError> {
        info!("初始化配置系统");

        // 详细进度报告
        self.emit_progress(InitProgress::new(
            "验证配置文件路径...".to_string(),
            1,
            4,
            self.start_time.elapsed(),
        ));

        // 验证配置文件路径
        let config_path = self.config.config_path.clone();
        if let Some(parent) = config_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| InitError::FileSystemError(format!("创建配置目录失败: {}", e)))?;
            }
        }

        // 验证配置内容
        self.emit_progress(InitProgress::new(
            "验证配置内容...".to_string(),
            1,
            4,
            self.start_time.elapsed(),
        ));

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

        self.emit_progress(InitProgress::new(
            format!("加载 {} 模型...", self.config.ai_model.model_type),
            2,
            4,
            self.start_time.elapsed(),
        ));

        // 使用模型管理器初始化模型
        self.model_manager.initialize_model().await?;

        info!("AI模型初始化成功");
        Ok(())
    }

    async fn initialize_voice_recognition(&self) -> Result<(), InitError> {
        if !self.config.voice.enabled {
            info!("语音识别功能已禁用，跳过初始化");
            self.emit_progress(InitProgress::new(
                "语音识别功能已禁用，跳过...".to_string(),
                3,
                4,
                self.start_time.elapsed(),
            ));
            return Ok(());
        }

        info!("初始化语音识别系统");

        self.emit_progress(InitProgress::new(
            "检查语音模型文件...".to_string(),
            3,
            4,
            self.start_time.elapsed(),
        ));

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
            self.emit_progress(InitProgress::new(
                "数据库功能已禁用，跳过...".to_string(),
                4,
                4,
                self.start_time.elapsed(),
            ));
            return Ok(());
        }

        info!("初始化数据库");

        self.emit_progress(InitProgress::new(
            "创建数据库目录...".to_string(),
            4,
            4,
            self.start_time.elapsed(),
        ));

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
        self.emit_progress(InitProgress::new(
            "测试数据库连接...".to_string(),
            4,
            4,
            self.start_time.elapsed(),
        ));

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

/// 检查配置是否完整，只有在配置不完整时才需要初始化
pub async fn check_config_completeness(app_handle: &tauri::AppHandle) -> Result<bool> {
    info!("检查配置完整性");

    // 获取配置目录
    let config_dir = app_handle
        .path()
        .app_config_dir()
        .map_err(|e| anyhow!("获取配置目录失败: {}", e))?;

    let config_file = config_dir.join("config.yaml");

    // 如果配置文件不存在，需要初始化
    if !config_file.exists() {
        info!("配置文件不存在，需要初始化");
        return Ok(false);
    }

    // 尝试加载配置
    match tokio::fs::read_to_string(&config_file).await {
        Ok(yaml_str) => match serde_yaml::from_str::<crate::config::InitConfig>(&yaml_str) {
            Ok(config) => {
                // 检查关键配置项是否完整
                let ai_model_configured = !config.ai_model.model_name.is_empty()
                    && !config.ai_model.server_url.is_empty();

                let model_path_configured = if config.voice.enabled {
                    !config.voice.model_path.is_empty()
                        && std::path::Path::new(&config.voice.model_path).exists()
                } else {
                    true // 如果语音功能未启用，则认为已配置
                };

                let is_complete = ai_model_configured && model_path_configured;

                if !is_complete {
                    info!(
                        "配置不完整 - AI模型: {}, 语音模型: {}",
                        ai_model_configured, model_path_configured
                    );
                } else {
                    info!("配置完整，跳过初始化");
                }

                Ok(is_complete)
            }
            Err(e) => {
                warn!("配置文件损坏或无法解析: {}", e);
                Ok(false)
            }
        },
        Err(e) => {
            warn!("无法读取配置文件: {}", e);
            Ok(false)
        }
    }
}

/// 条件初始化：只在配置不完整时执行
pub async fn conditional_initialize_app(app_handle: tauri::AppHandle) -> Result<bool> {
    // 首先检查配置是否完整
    let config_complete = check_config_completeness(&app_handle).await?;

    if config_complete {
        info!("配置已完整，跳过初始化流程");
        return Ok(true);
    }

    info!("配置不完整，开始初始化流程");

    // 执行完整初始化
    let config = crate::config::InitConfig::default();
    let result = initialize_app(config, app_handle).await?;

    Ok(result.success)
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
