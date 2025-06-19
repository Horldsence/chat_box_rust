use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::{Mutex, RwLock};

/// Agent 角色类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentRole {
    /// 助手角色
    Assistant,
    /// 朋友角色
    Friend,
    /// 导师角色
    Mentor,
    /// 专家角色
    Expert(String), // 专业领域
    /// 娱乐角色
    Entertainment,
    /// 自定义角色
    Custom(String),
}

/// Agent 个性特征
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPersonality {
    /// 友好程度 (0-10)
    pub friendliness: u8,
    /// 专业程度 (0-10)
    pub professionalism: u8,
    /// 幽默感 (0-10)
    pub humor: u8,
    /// 耐心程度 (0-10)
    pub patience: u8,
    /// 创造力 (0-10)
    pub creativity: u8,
    /// 表达风格
    pub expression_style: String,
    /// 语言偏好
    pub language_preference: String,
}

impl Default for AgentPersonality {
    fn default() -> Self {
        Self {
            friendliness: 8,
            professionalism: 7,
            humor: 5,
            patience: 8,
            creativity: 6,
            expression_style: "warm_and_helpful".to_string(),
            language_preference: "zh-CN".to_string(),
        }
    }
}

/// Agent 响应行为配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentBehavior {
    /// 回复长度偏好
    pub response_length: ResponseLength,
    /// 使用表情符号
    pub use_emojis: bool,
    /// 主动提问
    pub ask_questions: bool,
    /// 提供建议
    pub offer_suggestions: bool,
    /// 记住上下文
    pub remember_context: bool,
    /// 个性化回复
    pub personalized_responses: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResponseLength {
    Brief,
    Moderate,
    Detailed,
    Adaptive,
}

impl Default for AgentBehavior {
    fn default() -> Self {
        Self {
            response_length: ResponseLength::Moderate,
            use_emojis: true,
            ask_questions: true,
            offer_suggestions: true,
            remember_context: true,
            personalized_responses: true,
        }
    }
}

/// Agent Live2D 集成配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentLive2DIntegration {
    /// 启用Live2D集成
    pub enabled: bool,
    /// 情感表达映射
    pub emotion_mapping: HashMap<String, String>,
    /// 动作触发关键词
    pub action_triggers: HashMap<String, String>,
    /// 自动表情切换
    pub auto_expression: bool,
    /// 说话时动作
    pub speaking_actions: Vec<String>,
    /// 思考时动作
    pub thinking_actions: Vec<String>,
}

impl Default for AgentLive2DIntegration {
    fn default() -> Self {
        let mut emotion_mapping = HashMap::new();
        emotion_mapping.insert("happy".to_string(), "smile".to_string());
        emotion_mapping.insert("sad".to_string(), "sad".to_string());
        emotion_mapping.insert("excited".to_string(), "excited".to_string());
        emotion_mapping.insert("confused".to_string(), "confused".to_string());
        emotion_mapping.insert("surprised".to_string(), "surprised".to_string());

        let mut action_triggers = HashMap::new();
        action_triggers.insert("你好".to_string(), "greeting".to_string());
        action_triggers.insert("再见".to_string(), "farewell".to_string());
        action_triggers.insert("谢谢".to_string(), "happy".to_string());
        action_triggers.insert("加油".to_string(), "encouraging".to_string());

        Self {
            enabled: true,
            emotion_mapping,
            action_triggers,
            auto_expression: true,
            speaking_actions: vec!["speaking".to_string(), "gesture".to_string()],
            thinking_actions: vec!["thinking".to_string(), "pondering".to_string()],
        }
    }
}

/// Agent 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Agent名称
    pub name: String,
    /// Agent角色
    pub role: AgentRole,
    /// 系统提示词
    pub system_prompt: String,
    /// 个性特征
    pub personality: AgentPersonality,
    /// 行为配置
    pub behavior: AgentBehavior,
    /// Live2D集成
    pub live2d_integration: AgentLive2DIntegration,
    /// 预设回复
    pub preset_responses: HashMap<String, String>,
    /// 知识领域
    pub knowledge_domains: Vec<String>,
    /// 创建时间
    pub created_at: u64,
    /// 最后更新时间
    pub updated_at: u64,
}

impl Default for AgentConfig {
    fn default() -> Self {
        let now = chrono::Utc::now().timestamp_millis() as u64;

        let mut preset_responses = HashMap::new();
        preset_responses.insert(
            "greeting".to_string(),
            "你好！我是你的AI助手，很高兴为你服务！😊".to_string(),
        );
        preset_responses.insert(
            "farewell".to_string(),
            "再见！祝你有美好的一天！👋".to_string(),
        );
        preset_responses.insert(
            "thanks".to_string(),
            "不客气！能帮到你我很开心！😊".to_string(),
        );
        preset_responses.insert(
            "help".to_string(),
            "当然！我很乐意帮助你。请告诉我你需要什么帮助？".to_string(),
        );

        Self {
            name: "AI助手".to_string(),
            role: AgentRole::Assistant,
            system_prompt: "你是一个友好、乐于助人的AI助手。你的目标是为用户提供准确、有用的信息和建议。请保持礼貌、耐心，并尽力理解用户的需求。".to_string(),
            personality: AgentPersonality::default(),
            behavior: AgentBehavior::default(),
            live2d_integration: AgentLive2DIntegration::default(),
            preset_responses,
            knowledge_domains: vec![
                "通用问答".to_string(),
                "技术支持".to_string(),
                "日常对话".to_string(),
            ],
            created_at: now,
            updated_at: now,
        }
    }
}

/// Agent 预设模板
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub config: AgentConfig,
    pub preview_image: Option<String>,
    pub tags: Vec<String>,
}

/// Agent 服务状态
#[derive(Debug, Clone, Serialize)]
pub struct AgentState {
    pub current_config: AgentConfig,
    pub conversation_context: Vec<String>,
    pub user_preferences: HashMap<String, String>,
    pub session_start_time: u64,
    pub interaction_count: u64,
}

impl Default for AgentState {
    fn default() -> Self {
        Self {
            current_config: AgentConfig::default(),
            conversation_context: Vec::new(),
            user_preferences: HashMap::new(),
            session_start_time: chrono::Utc::now().timestamp_millis() as u64,
            interaction_count: 0,
        }
    }
}

/// Agent 服务
pub struct AgentService {
    app_handle: AppHandle,
    config: Arc<RwLock<AgentConfig>>,
    state: Arc<Mutex<AgentState>>,
    templates: Arc<RwLock<Vec<AgentTemplate>>>,
}

impl AgentService {
    pub fn new(app_handle: AppHandle) -> Self {
        let mut templates = Vec::new();
        templates.extend(Self::create_default_templates());

        Self {
            app_handle,
            config: Arc::new(RwLock::new(AgentConfig::default())),
            state: Arc::new(Mutex::new(AgentState::default())),
            templates: Arc::new(RwLock::new(templates)),
        }
    }

    /// 创建默认模板
    fn create_default_templates() -> Vec<AgentTemplate> {
        vec![
            // 友好助手模板
            AgentTemplate {
                id: "friendly_assistant".to_string(),
                name: "友好助手".to_string(),
                description: "一个友好、乐于助人的AI助手，适合日常对话和一般性问题".to_string(),
                config: AgentConfig {
                    name: "小助手".to_string(),
                    role: AgentRole::Assistant,
                    system_prompt: "你是一个非常友好和乐于助人的AI助手。你总是保持积极乐观的态度，用温暖的语调与用户交流。你会记住用户的偏好，并尝试提供个性化的帮助。在回答问题时，你会尽量详细但不冗长，并在适当的时候使用表情符号让对话更生动。".to_string(),
                    personality: AgentPersonality {
                        friendliness: 9,
                        professionalism: 7,
                        humor: 6,
                        patience: 9,
                        creativity: 6,
                        expression_style: "warm_and_caring".to_string(),
                        language_preference: "zh-CN".to_string(),
                    },
                    ..AgentConfig::default()
                },
                preview_image: None,
                tags: vec!["助手".to_string(), "友好".to_string(), "日常".to_string()],
            },

            // 专业导师模板
            AgentTemplate {
                id: "professional_mentor".to_string(),
                name: "专业导师".to_string(),
                description: "专业、严谨的导师角色，适合学习和专业问题讨论".to_string(),
                config: AgentConfig {
                    name: "导师".to_string(),
                    role: AgentRole::Mentor,
                    system_prompt: "你是一位经验丰富的专业导师。你擅长引导用户思考，提供深入的分析和建议。你的回答总是结构清晰、逻辑严密，并且会主动提问来帮助用户更好地理解问题。你注重培养用户的独立思考能力，而不仅仅是给出答案。".to_string(),
                    personality: AgentPersonality {
                        friendliness: 7,
                        professionalism: 10,
                        humor: 3,
                        patience: 10,
                        creativity: 8,
                        expression_style: "structured_and_analytical".to_string(),
                        language_preference: "zh-CN".to_string(),
                    },
                    behavior: AgentBehavior {
                        response_length: ResponseLength::Detailed,
                        use_emojis: false,
                        ask_questions: true,
                        offer_suggestions: true,
                        remember_context: true,
                        personalized_responses: true,
                    },
                    ..AgentConfig::default()
                },
                preview_image: None,
                tags: vec!["导师".to_string(), "专业".to_string(), "学习".to_string()],
            },

            // 创意伙伴模板
            AgentTemplate {
                id: "creative_companion".to_string(),
                name: "创意伙伴".to_string(),
                description: "富有创造力和想象力的伙伴，适合创意讨论和头脑风暴".to_string(),
                config: AgentConfig {
                    name: "创意家".to_string(),
                    role: AgentRole::Friend,
                    system_prompt: "你是一个充满创造力和想象力的创意伙伴。你喜欢从不同角度思考问题，提出新颖有趣的想法。你善于激发用户的创造力，经常使用比喻、类比和生动的描述。你的回答富有想象力，并且总是鼓励用户跳出常规思维。".to_string(),
                    personality: AgentPersonality {
                        friendliness: 8,
                        professionalism: 6,
                        humor: 8,
                        patience: 7,
                        creativity: 10,
                        expression_style: "imaginative_and_inspiring".to_string(),
                        language_preference: "zh-CN".to_string(),
                    },
                    behavior: AgentBehavior {
                        response_length: ResponseLength::Moderate,
                        use_emojis: true,
                        ask_questions: true,
                        offer_suggestions: true,
                        remember_context: true,
                        personalized_responses: true,
                    },
                    ..AgentConfig::default()
                },
                preview_image: None,
                tags: vec!["创意".to_string(), "想象力".to_string(), "头脑风暴".to_string()],
            },

            // 技术专家模板
            AgentTemplate {
                id: "tech_expert".to_string(),
                name: "技术专家".to_string(),
                description: "专业的技术专家，适合编程、技术问题和解决方案讨论".to_string(),
                config: AgentConfig {
                    name: "技术专家".to_string(),
                    role: AgentRole::Expert("技术".to_string()),
                    system_prompt: "你是一位资深的技术专家，精通多种编程语言和技术栈。你能够提供准确的技术建议，解释复杂的技术概念，并帮助用户解决编程问题。你的回答通常包含具体的代码示例和最佳实践建议。你注重代码质量和性能优化。".to_string(),
                    personality: AgentPersonality {
                        friendliness: 6,
                        professionalism: 10,
                        humor: 4,
                        patience: 8,
                        creativity: 7,
                        expression_style: "technical_and_precise".to_string(),
                        language_preference: "zh-CN".to_string(),
                    },
                    behavior: AgentBehavior {
                        response_length: ResponseLength::Detailed,
                        use_emojis: false,
                        ask_questions: true,
                        offer_suggestions: true,
                        remember_context: true,
                        personalized_responses: false,
                    },
                    knowledge_domains: vec![
                        "编程".to_string(),
                        "软件架构".to_string(),
                        "数据库".to_string(),
                        "网络".to_string(),
                        "DevOps".to_string(),
                    ],
                    ..AgentConfig::default()
                },
                preview_image: None,
                tags: vec!["技术".to_string(), "编程".to_string(), "专家".to_string()],
            },
        ]
    }

    /// 获取当前配置
    pub async fn get_config(&self) -> AgentConfig {
        self.config.read().await.clone()
    }

    /// 更新配置
    pub async fn update_config(&self, mut config: AgentConfig) -> Result<(), String> {
        config.updated_at = chrono::Utc::now().timestamp_millis() as u64;

        let mut current_config = self.config.write().await;
        *current_config = config.clone();

        // 更新状态中的配置
        let mut state = self.state.lock().await;
        state.current_config = config;

        info!("Agent配置已更新");
        Ok(())
    }

    /// 应用模板
    pub async fn apply_template(&self, template_id: &str) -> Result<(), String> {
        let templates = self.templates.read().await;
        let template = templates
            .iter()
            .find(|t| t.id == template_id)
            .ok_or_else(|| format!("模板不存在: {}", template_id))?;

        self.update_config(template.config.clone()).await?;
        info!("应用Agent模板: {}", template.name);
        Ok(())
    }

    /// 获取所有模板
    pub async fn get_templates(&self) -> Vec<AgentTemplate> {
        self.templates.read().await.clone()
    }

    /// 添加自定义模板
    pub async fn add_template(&self, template: AgentTemplate) -> Result<(), String> {
        let mut templates = self.templates.write().await;

        // 检查ID是否已存在
        if templates.iter().any(|t| t.id == template.id) {
            return Err(format!("模板ID已存在: {}", template.id));
        }

        templates.push(template);
        info!("添加新的Agent模板");
        Ok(())
    }

    /// 删除模板
    pub async fn remove_template(&self, template_id: &str) -> Result<(), String> {
        let mut templates = self.templates.write().await;
        let index = templates
            .iter()
            .position(|t| t.id == template_id)
            .ok_or_else(|| format!("模板不存在: {}", template_id))?;

        templates.remove(index);
        info!("删除Agent模板: {}", template_id);
        Ok(())
    }

    /// 构建系统提示词
    pub async fn build_system_prompt(&self, additional_context: Option<&str>) -> String {
        let config = self.config.read().await;
        let state = self.state.lock().await;

        let mut prompt = config.system_prompt.clone();

        // 添加个性特征
        prompt.push_str(&format!(
            "\n\n你的个性特征：\n- 友好程度: {}/10\n- 专业程度: {}/10\n- 幽默感: {}/10\n- 耐心: {}/10\n- 创造力: {}/10",
            config.personality.friendliness,
            config.personality.professionalism,
            config.personality.humor,
            config.personality.patience,
            config.personality.creativity
        ));

        // 添加行为配置
        if config.behavior.use_emojis {
            prompt.push_str("\n- 在适当的时候使用表情符号让对话更生动");
        }
        if config.behavior.ask_questions {
            prompt.push_str("\n- 主动提问来更好地理解用户需求");
        }
        if config.behavior.offer_suggestions {
            prompt.push_str("\n- 提供有用的建议和解决方案");
        }

        // 添加知识领域
        if !config.knowledge_domains.is_empty() {
            prompt.push_str(&format!(
                "\n\n你的专业领域包括：{}",
                config.knowledge_domains.join("、")
            ));
        }

        // 添加上下文信息
        if config.behavior.remember_context && !state.conversation_context.is_empty() {
            let context = state.conversation_context.join(" ");
            prompt.push_str(&format!("\n\n对话上下文：{}", context));
        }

        // 添加额外上下文
        if let Some(context) = additional_context {
            prompt.push_str(&format!("\n\n额外信息：{}", context));
        }

        prompt
    }

    /// 处理用户消息并更新上下文
    pub async fn process_user_message(&self, message: &str) -> Result<(), String> {
        let mut state = self.state.lock().await;
        let config = self.config.read().await;

        // 增加交互计数
        state.interaction_count += 1;

        // 更新对话上下文
        if config.behavior.remember_context {
            state.conversation_context.push(message.to_string());

            // 限制上下文长度，保留最近的10条消息
            if state.conversation_context.len() > 10 {
                state.conversation_context.remove(0);
            }
        }

        // 检查是否需要触发Live2D动作
        if config.live2d_integration.enabled {
            for (trigger, action) in &config.live2d_integration.action_triggers {
                if message.contains(trigger) {
                    let event_data = serde_json::json!({
                        "action": action,
                        "trigger": trigger,
                        "message": message
                    });

                    if let Err(e) = self.app_handle.emit("agent_live2d_trigger", &event_data) {
                        warn!("发送Live2D触发事件失败: {}", e);
                    }
                    break;
                }
            }
        }

        Ok(())
    }

    /// 获取预设回复
    pub async fn get_preset_response(&self, key: &str) -> Option<String> {
        let config = self.config.read().await;
        config.preset_responses.get(key).cloned()
    }

    /// 添加预设回复
    pub async fn add_preset_response(&self, key: String, response: String) -> Result<(), String> {
        let mut config = self.config.write().await;
        config.preset_responses.insert(key, response);
        config.updated_at = chrono::Utc::now().timestamp_millis() as u64;
        info!("添加预设回复");
        Ok(())
    }

    /// 获取状态信息
    pub async fn get_state(&self) -> AgentState {
        let state = self.state.lock().await;
        state.clone()
    }

    /// 重置会话
    pub async fn reset_session(&self) -> Result<(), String> {
        let mut state = self.state.lock().await;
        state.conversation_context.clear();
        state.user_preferences.clear();
        state.session_start_time = chrono::Utc::now().timestamp_millis() as u64;
        state.interaction_count = 0;

        info!("Agent会话已重置");
        Ok(())
    }

    /// 导出配置
    pub async fn export_config(&self) -> Result<String, String> {
        let config = self.config.read().await;
        serde_json::to_string_pretty(&*config).map_err(|e| format!("序列化配置失败: {}", e))
    }

    /// 导入配置
    pub async fn import_config(&self, config_json: &str) -> Result<(), String> {
        let config: AgentConfig =
            serde_json::from_str(config_json).map_err(|e| format!("解析配置失败: {}", e))?;

        self.update_config(config).await?;
        info!("Agent配置导入成功");
        Ok(())
    }
}
