use crate::services::agent::{AgentConfig, AgentState, AgentTemplate};
use crate::state::AppState;
use log::info;
use serde_json::Value;
use std::collections::HashMap;
use tauri::State;

/// 获取Agent配置
#[tauri::command]
pub async fn get_agent_config(app_state: State<'_, AppState>) -> Result<AgentConfig, String> {
    let agent_service = app_state.agent_service.lock().await;
    Ok(agent_service.get_config().await)
}

/// 更新Agent配置
#[tauri::command]
pub async fn update_agent_config(
    config: AgentConfig,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    let agent_service = app_state.agent_service.lock().await;
    agent_service.update_config(config).await?;
    info!("Agent配置已更新");
    Ok(())
}

/// 获取所有Agent模板
#[tauri::command]
pub async fn get_agent_templates(
    app_state: State<'_, AppState>,
) -> Result<Vec<AgentTemplate>, String> {
    let agent_service = app_state.agent_service.lock().await;
    Ok(agent_service.get_templates().await)
}

/// 应用Agent模板
#[tauri::command]
pub async fn apply_agent_template(
    template_id: String,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    let agent_service = app_state.agent_service.lock().await;
    agent_service.apply_template(&template_id).await?;
    info!("应用Agent模板: {}", template_id);
    Ok(())
}

/// 添加自定义Agent模板
#[tauri::command]
pub async fn add_agent_template(
    template: AgentTemplate,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    let agent_service = app_state.agent_service.lock().await;
    agent_service.add_template(template).await?;
    info!("添加新的Agent模板");
    Ok(())
}

/// 删除Agent模板
#[tauri::command]
pub async fn remove_agent_template(
    template_id: String,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    let agent_service = app_state.agent_service.lock().await;
    agent_service.remove_template(&template_id).await?;
    info!("删除Agent模板: {}", template_id);
    Ok(())
}

/// 构建系统提示词
#[tauri::command]
pub async fn build_system_prompt(
    additional_context: Option<String>,
    app_state: State<'_, AppState>,
) -> Result<String, String> {
    let agent_service = app_state.agent_service.lock().await;
    let prompt = agent_service
        .build_system_prompt(additional_context.as_deref())
        .await;
    Ok(prompt)
}

/// 处理用户消息
#[tauri::command]
pub async fn process_user_message_for_agent(
    message: String,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    let agent_service = app_state.agent_service.lock().await;
    agent_service.process_user_message(&message).await?;
    Ok(())
}

/// 获取预设回复
#[tauri::command]
pub async fn get_agent_preset_response(
    key: String,
    app_state: State<'_, AppState>,
) -> Result<Option<String>, String> {
    let agent_service = app_state.agent_service.lock().await;
    Ok(agent_service.get_preset_response(&key).await)
}

/// 添加预设回复
#[tauri::command]
pub async fn add_agent_preset_response(
    key: String,
    response: String,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    let agent_service = app_state.agent_service.lock().await;
    agent_service.add_preset_response(key, response).await?;
    info!("添加Agent预设回复");
    Ok(())
}

/// 获取Agent状态
#[tauri::command]
pub async fn get_agent_state(app_state: State<'_, AppState>) -> Result<AgentState, String> {
    let agent_service = app_state.agent_service.lock().await;
    let state = agent_service.get_state().await;
    Ok(state)
}

/// 重置Agent会话
#[tauri::command]
pub async fn reset_agent_session(app_state: State<'_, AppState>) -> Result<(), String> {
    let agent_service = app_state.agent_service.lock().await;
    agent_service.reset_session().await?;
    info!("Agent会话已重置");
    Ok(())
}

/// 导出Agent配置
#[tauri::command]
pub async fn export_agent_config(app_state: State<'_, AppState>) -> Result<String, String> {
    let agent_service = app_state.agent_service.lock().await;
    agent_service.export_config().await
}

/// 导入Agent配置
#[tauri::command]
pub async fn import_agent_config(
    config_json: String,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    let agent_service = app_state.agent_service.lock().await;
    agent_service.import_config(&config_json).await?;
    info!("Agent配置导入成功");
    Ok(())
}

/// 更新Agent个性特征
#[tauri::command]
pub async fn update_agent_personality(
    friendliness: u8,
    professionalism: u8,
    humor: u8,
    patience: u8,
    creativity: u8,
    expression_style: String,
    language_preference: String,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    let agent_service = app_state.agent_service.lock().await;
    let mut config = agent_service.get_config().await;

    // 验证数值范围
    if friendliness > 10 || professionalism > 10 || humor > 10 || patience > 10 || creativity > 10 {
        return Err("个性特征值必须在0-10之间".to_string());
    }

    config.personality.friendliness = friendliness;
    config.personality.professionalism = professionalism;
    config.personality.humor = humor;
    config.personality.patience = patience;
    config.personality.creativity = creativity;
    config.personality.expression_style = expression_style;
    config.personality.language_preference = language_preference;

    agent_service.update_config(config).await?;
    info!("Agent个性特征已更新");
    Ok(())
}

/// 更新Agent行为配置
#[tauri::command]
pub async fn update_agent_behavior(
    response_length: String,
    use_emojis: bool,
    ask_questions: bool,
    offer_suggestions: bool,
    remember_context: bool,
    personalized_responses: bool,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    let agent_service = app_state.agent_service.lock().await;
    let mut config = agent_service.get_config().await;

    use crate::services::agent::ResponseLength;
    config.behavior.response_length = match response_length.as_str() {
        "Brief" => ResponseLength::Brief,
        "Moderate" => ResponseLength::Moderate,
        "Detailed" => ResponseLength::Detailed,
        "Adaptive" => ResponseLength::Adaptive,
        _ => return Err("无效的回复长度设置".to_string()),
    };

    config.behavior.use_emojis = use_emojis;
    config.behavior.ask_questions = ask_questions;
    config.behavior.offer_suggestions = offer_suggestions;
    config.behavior.remember_context = remember_context;
    config.behavior.personalized_responses = personalized_responses;

    agent_service.update_config(config).await?;
    info!("Agent行为配置已更新");
    Ok(())
}

/// 更新Agent Live2D集成配置
#[tauri::command]
pub async fn update_agent_live2d_integration(
    enabled: bool,
    emotion_mapping: HashMap<String, String>,
    action_triggers: HashMap<String, String>,
    auto_expression: bool,
    speaking_actions: Vec<String>,
    thinking_actions: Vec<String>,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    let agent_service = app_state.agent_service.lock().await;
    let mut config = agent_service.get_config().await;

    config.live2d_integration.enabled = enabled;
    config.live2d_integration.emotion_mapping = emotion_mapping;
    config.live2d_integration.action_triggers = action_triggers;
    config.live2d_integration.auto_expression = auto_expression;
    config.live2d_integration.speaking_actions = speaking_actions;
    config.live2d_integration.thinking_actions = thinking_actions;

    agent_service.update_config(config).await?;
    info!("Agent Live2D集成配置已更新");
    Ok(())
}

/// 批量更新预设回复
#[tauri::command]
pub async fn update_agent_preset_responses(
    responses: HashMap<String, String>,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    let agent_service = app_state.agent_service.lock().await;
    let mut config = agent_service.get_config().await;

    config.preset_responses = responses;
    agent_service.update_config(config).await?;
    info!("Agent预设回复已批量更新");
    Ok(())
}

/// 删除预设回复
#[tauri::command]
pub async fn remove_agent_preset_response(
    key: String,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    let agent_service = app_state.agent_service.lock().await;
    let mut config = agent_service.get_config().await;

    if config.preset_responses.remove(&key).is_some() {
        agent_service.update_config(config).await?;
        info!("删除Agent预设回复: {}", key);
        Ok(())
    } else {
        Err(format!("预设回复不存在: {}", key))
    }
}

/// 更新Agent知识领域
#[tauri::command]
pub async fn update_agent_knowledge_domains(
    domains: Vec<String>,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    let agent_service = app_state.agent_service.lock().await;
    let mut config = agent_service.get_config().await;

    config.knowledge_domains = domains;
    agent_service.update_config(config).await?;
    info!("Agent知识领域已更新");
    Ok(())
}

/// 测试Agent配置
#[tauri::command]
pub async fn test_agent_config(
    app_state: State<'_, AppState>,
) -> Result<HashMap<String, Value>, String> {
    let agent_service = app_state.agent_service.lock().await;

    let mut result = HashMap::new();

    // 测试基本功能
    result.insert("service_available".to_string(), Value::Bool(true));

    // 获取当前配置
    let config = agent_service.get_config().await;
    result.insert(
        "config".to_string(),
        serde_json::to_value(config).unwrap_or(Value::Null),
    );

    // 获取状态信息
    let state = agent_service.get_state().await;
    result.insert(
        "state".to_string(),
        serde_json::to_value(state).unwrap_or(Value::Null),
    );

    // 测试系统提示词构建
    let prompt = agent_service.build_system_prompt(Some("测试上下文")).await;
    result.insert("system_prompt".to_string(), Value::String(prompt));

    // 获取模板信息
    let templates = agent_service.get_templates().await;
    result.insert(
        "templates_count".to_string(),
        Value::Number(templates.len().into()),
    );

    // 测试预设回复
    let greeting = agent_service.get_preset_response("greeting").await;
    result.insert(
        "preset_response_test".to_string(),
        Value::Bool(greeting.is_some()),
    );

    info!("Agent配置测试完成");
    Ok(result)
}

/// 创建自定义Agent模板
#[tauri::command]
pub async fn create_custom_agent_template(
    id: String,
    name: String,
    description: String,
    config: AgentConfig,
    tags: Vec<String>,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    let agent_service = app_state.agent_service.lock().await;

    let template = AgentTemplate {
        id,
        name,
        description,
        config,
        preview_image: None,
        tags,
    };

    agent_service.add_template(template).await?;
    info!("创建自定义Agent模板成功");
    Ok(())
}

/// 克隆现有模板
#[tauri::command]
pub async fn clone_agent_template(
    source_template_id: String,
    new_id: String,
    new_name: String,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    let agent_service = app_state.agent_service.lock().await;
    let templates = agent_service.get_templates().await;

    let source_template = templates
        .iter()
        .find(|t| t.id == source_template_id)
        .ok_or_else(|| format!("源模板不存在: {}", source_template_id))?;

    let mut new_template = source_template.clone();
    new_template.id = new_id;
    new_template.name = new_name;
    new_template.description = format!("基于 {} 的副本", source_template.name);

    agent_service.add_template(new_template).await?;
    info!("克隆Agent模板成功");
    Ok(())
}
