use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::{Mutex, RwLock};

/// Live2D 动作类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Live2DActionType {
    /// 说话动作
    Speaking,
    /// 思考动作
    Thinking,
    /// 高兴表情
    Happy,
    /// 困惑表情
    Confused,
    /// 惊讶表情
    Surprised,
    /// 问候动作
    Greeting,
    /// 告别动作
    Farewell,
    /// 静默状态
    Idle,
    /// 打字动作
    Typing,
    /// 聆听动作
    Listening,
    /// 自定义动作
    Custom(String),
}

/// Live2D 动作配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Live2DAction {
    pub action_type: Live2DActionType,
    pub motion_group: String,
    pub motion_index: Option<u32>,
    pub expression: Option<String>,
    pub duration: Option<u32>, // 持续时间（毫秒）
    pub priority: u8,          // 优先级 (0-255)
}

/// Live2D 事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Live2DEvent {
    pub event_type: String,
    pub action: Live2DAction,
    pub timestamp: u64,
    pub metadata: HashMap<String, String>,
}

/// Live2D 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Live2DConfig {
    pub model_path: String,
    pub scale: f32,
    pub position: (f32, f32),
    pub auto_blink: bool,
    pub auto_breath: bool,
    pub default_actions: HashMap<String, Live2DAction>,
    pub text_triggers: HashMap<String, Live2DActionType>,
}

impl Default for Live2DConfig {
    fn default() -> Self {
        let mut default_actions = HashMap::new();
        let mut text_triggers = HashMap::new();

        // 默认动作配置
        default_actions.insert(
            "idle".to_string(),
            Live2DAction {
                action_type: Live2DActionType::Idle,
                motion_group: "Idle".to_string(),
                motion_index: Some(0),
                expression: None,
                duration: None,
                priority: 0,
            },
        );

        default_actions.insert(
            "speaking".to_string(),
            Live2DAction {
                action_type: Live2DActionType::Speaking,
                motion_group: "TapBody".to_string(),
                motion_index: Some(0),
                expression: Some("happy".to_string()),
                duration: Some(2000),
                priority: 5,
            },
        );

        default_actions.insert(
            "thinking".to_string(),
            Live2DAction {
                action_type: Live2DActionType::Thinking,
                motion_group: "Idle".to_string(),
                motion_index: Some(1),
                expression: Some("thinking".to_string()),
                duration: Some(3000),
                priority: 3,
            },
        );

        // 文本触发器配置
        text_triggers.insert("你好".to_string(), Live2DActionType::Greeting);
        text_triggers.insert("再见".to_string(), Live2DActionType::Farewell);
        text_triggers.insert("谢谢".to_string(), Live2DActionType::Happy);
        text_triggers.insert("什么".to_string(), Live2DActionType::Confused);
        text_triggers.insert("哇".to_string(), Live2DActionType::Surprised);
        text_triggers.insert("嗯".to_string(), Live2DActionType::Thinking);

        Self {
            model_path: "/models/live2d/default/model.json".to_string(),
            scale: 1.0,
            position: (0.0, 0.0),
            auto_blink: true,
            auto_breath: true,
            default_actions,
            text_triggers,
        }
    }
}

/// Live2D 服务状态
#[derive(Debug, Clone, Serialize)]
pub struct Live2DState {
    pub current_action: Option<Live2DAction>,
    pub action_queue: Vec<Live2DAction>,
    pub is_speaking: bool,
    pub last_action_time: u64,
}

impl Default for Live2DState {
    fn default() -> Self {
        Self {
            current_action: None,
            action_queue: Vec::new(),
            is_speaking: false,
            last_action_time: 0,
        }
    }
}

/// Live2D 服务
pub struct Live2DService {
    app_handle: AppHandle,
    config: Arc<RwLock<Live2DConfig>>,
    state: Arc<Mutex<Live2DState>>,
    text_buffer: Arc<Mutex<String>>,
}

impl Live2DService {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            app_handle,
            config: Arc::new(RwLock::new(Live2DConfig::default())),
            state: Arc::new(Mutex::new(Live2DState::default())),
            text_buffer: Arc::new(Mutex::new(String::new())),
        }
    }

    /// 更新配置
    pub async fn update_config(&self, config: Live2DConfig) -> Result<(), String> {
        let mut current_config = self.config.write().await;
        *current_config = config;
        info!("Live2D 配置已更新");
        Ok(())
    }

    /// 获取配置
    pub async fn get_config(&self) -> Live2DConfig {
        self.config.read().await.clone()
    }

    /// 执行动作
    pub async fn execute_action(&self, action: Live2DAction) -> Result<(), String> {
        let mut state = self.state.lock().await;

        // 检查优先级
        if let Some(current) = &state.current_action {
            if current.priority > action.priority {
                debug!("当前动作优先级更高，添加到队列");
                state.action_queue.push(action);
                return Ok(());
            }
        }

        // 执行动作
        state.current_action = Some(action.clone());
        state.last_action_time = chrono::Utc::now().timestamp_millis() as u64;

        // 发送事件到前端
        let event = Live2DEvent {
            event_type: "action_change".to_string(),
            action: action.clone(),
            timestamp: state.last_action_time,
            metadata: HashMap::new(),
        };

        if let Err(e) = self.app_handle.emit("live2d_action", &event) {
            error!("发送Live2D事件失败: {}", e);
            return Err(format!("发送事件失败: {}", e));
        }

        info!("执行Live2D动作: {:?}", action.action_type);
        Ok(())
    }

    /// 处理流式文本输入
    pub async fn process_streaming_text(&self, text_chunk: &str) -> Result<(), String> {
        let mut buffer = self.text_buffer.lock().await;
        buffer.push_str(text_chunk);

        // 检查文本触发器
        let config = self.config.read().await;
        for (trigger, action_type) in &config.text_triggers {
            if buffer.contains(trigger) {
                if let Some(action) = self.get_action_for_type(action_type.clone()).await {
                    self.execute_action(action).await?;
                }
                buffer.clear(); // 清空缓冲区避免重复触发
                break;
            }
        }

        // 安全截断缓冲区：确保在字符边界处截断
        const MAX_BYTES: usize = 1000;
        if buffer.len() > MAX_BYTES {
            // 找到有效的UTF-8边界
            let mut split_index = buffer.len() - MAX_BYTES;
            while !buffer.is_char_boundary(split_index) {
                split_index += 1;
                // 防止越界（理论上不会发生，但安全起见）
                if split_index >= buffer.len() {
                    split_index = buffer.len();
                    break;
                }
            }
            *buffer = buffer[split_index..].to_string();
        }

        Ok(())
    }

    /// 开始说话状态
    pub async fn start_speaking(&self) -> Result<(), String> {
        let mut state = self.state.lock().await;
        state.is_speaking = true;

        let speaking_action = Live2DAction {
            action_type: Live2DActionType::Speaking,
            motion_group: "TapBody".to_string(),
            motion_index: Some(0),
            expression: Some("happy".to_string()),
            duration: None,
            priority: 5,
        };

        drop(state); // 释放锁
        self.execute_action(speaking_action).await
    }

    /// 结束说话状态
    pub async fn stop_speaking(&self) -> Result<(), String> {
        let mut state = self.state.lock().await;
        state.is_speaking = false;

        let idle_action = Live2DAction {
            action_type: Live2DActionType::Idle,
            motion_group: "Idle".to_string(),
            motion_index: Some(0),
            expression: None,
            duration: None,
            priority: 0,
        };

        drop(state); // 释放锁
        self.execute_action(idle_action).await
    }

    /// 开始思考状态
    pub async fn start_thinking(&self) -> Result<(), String> {
        let thinking_action = Live2DAction {
            action_type: Live2DActionType::Thinking,
            motion_group: "Idle".to_string(),
            motion_index: Some(1),
            expression: Some("thinking".to_string()),
            duration: Some(5000),
            priority: 3,
        };

        self.execute_action(thinking_action).await
    }

    /// 设置表情
    pub async fn set_expression(&self, expression: &str) -> Result<(), String> {
        let event = Live2DEvent {
            event_type: "expression_change".to_string(),
            action: Live2DAction {
                action_type: Live2DActionType::Custom(expression.to_string()),
                motion_group: "".to_string(),
                motion_index: None,
                expression: Some(expression.to_string()),
                duration: None,
                priority: 1,
            },
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
            metadata: HashMap::new(),
        };

        if let Err(e) = self.app_handle.emit("live2d_expression", &event) {
            error!("发送Live2D表情事件失败: {}", e);
            return Err(format!("发送表情事件失败: {}", e));
        }

        info!("设置Live2D表情: {}", expression);
        Ok(())
    }

    /// 获取指定类型的动作
    async fn get_action_for_type(&self, action_type: Live2DActionType) -> Option<Live2DAction> {
        let config = self.config.read().await;

        match action_type {
            Live2DActionType::Speaking => config.default_actions.get("speaking").cloned(),
            Live2DActionType::Thinking => config.default_actions.get("thinking").cloned(),
            Live2DActionType::Idle => config.default_actions.get("idle").cloned(),
            Live2DActionType::Happy => Some(Live2DAction {
                action_type: Live2DActionType::Happy,
                motion_group: "TapBody".to_string(),
                motion_index: Some(1),
                expression: Some("happy".to_string()),
                duration: Some(2000),
                priority: 4,
            }),
            Live2DActionType::Confused => Some(Live2DAction {
                action_type: Live2DActionType::Confused,
                motion_group: "Shake".to_string(),
                motion_index: Some(0),
                expression: Some("confused".to_string()),
                duration: Some(2000),
                priority: 4,
            }),
            Live2DActionType::Surprised => Some(Live2DAction {
                action_type: Live2DActionType::Surprised,
                motion_group: "TapBody".to_string(),
                motion_index: Some(2),
                expression: Some("surprised".to_string()),
                duration: Some(2000),
                priority: 4,
            }),
            Live2DActionType::Greeting => Some(Live2DAction {
                action_type: Live2DActionType::Greeting,
                motion_group: "TapBody".to_string(),
                motion_index: Some(0),
                expression: Some("happy".to_string()),
                duration: Some(3000),
                priority: 6,
            }),
            Live2DActionType::Farewell => Some(Live2DAction {
                action_type: Live2DActionType::Farewell,
                motion_group: "TapBody".to_string(),
                motion_index: Some(1),
                expression: Some("happy".to_string()),
                duration: Some(3000),
                priority: 6,
            }),
            Live2DActionType::Typing => Some(Live2DAction {
                action_type: Live2DActionType::Typing,
                motion_group: "Idle".to_string(),
                motion_index: Some(2),
                expression: Some("focused".to_string()),
                duration: None,
                priority: 2,
            }),
            Live2DActionType::Listening => Some(Live2DAction {
                action_type: Live2DActionType::Listening,
                motion_group: "Idle".to_string(),
                motion_index: Some(0),
                expression: Some("attentive".to_string()),
                duration: None,
                priority: 2,
            }),
            Live2DActionType::Custom(name) => {
                // 查找自定义动作
                config.default_actions.get(&name).cloned()
            }
        }
    }

    /// 处理动作队列
    pub async fn process_action_queue(&self) -> Result<(), String> {
        let mut state = self.state.lock().await;

        // 检查当前动作是否结束
        let current_time = chrono::Utc::now().timestamp_millis() as u64;
        let should_process_queue = if let Some(current) = &state.current_action {
            if let Some(duration) = current.duration {
                current_time - state.last_action_time > duration as u64
            } else {
                false
            }
        } else {
            true
        };

        if should_process_queue && !state.action_queue.is_empty() {
            let next_action = state.action_queue.remove(0);
            drop(state); // 释放锁
            self.execute_action(next_action).await?;
        }

        Ok(())
    }

    /// 清空文本缓冲区
    pub async fn clear_text_buffer(&self) {
        let mut buffer = self.text_buffer.lock().await;
        buffer.clear();
    }

    /// 获取当前状态
    pub async fn get_state(&self) -> Live2DState {
        let state = self.state.lock().await;
        state.clone()
    }
}
