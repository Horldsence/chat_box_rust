use crate::services::live2d::{Live2DAction, Live2DActionType, Live2DConfig, Live2DState};
use crate::state::AppState;
use log::{debug, info};
use serde_json::Value;
use std::collections::HashMap;
use tauri::State;

/// 获取Live2D配置
#[tauri::command]
pub async fn get_live2d_config(app_state: State<'_, AppState>) -> Result<Live2DConfig, String> {
    let live2d_service = app_state.live2d_service.lock().await;
    Ok(live2d_service.get_config().await)
}

/// 更新Live2D配置
#[tauri::command]
pub async fn update_live2d_config(
    config: Live2DConfig,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    let live2d_service = app_state.live2d_service.lock().await;
    live2d_service.update_config(config).await?;
    info!("Live2D配置已更新");
    Ok(())
}

/// 执行Live2D动作
#[tauri::command]
pub async fn execute_live2d_action(
    action: Live2DAction,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    let live2d_service = app_state.live2d_service.lock().await;
    live2d_service.execute_action(action).await?;
    Ok(())
}

/// 通过动作类型执行动作
#[tauri::command]
pub async fn execute_live2d_action_by_type(
    action_type: String,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    let live2d_service = app_state.live2d_service.lock().await;

    let action_enum = match action_type.as_str() {
        "speaking" => Live2DActionType::Speaking,
        "thinking" => Live2DActionType::Thinking,
        "happy" => Live2DActionType::Happy,
        "confused" => Live2DActionType::Confused,
        "surprised" => Live2DActionType::Surprised,
        "greeting" => Live2DActionType::Greeting,
        "farewell" => Live2DActionType::Farewell,
        "idle" => Live2DActionType::Idle,
        "typing" => Live2DActionType::Typing,
        "listening" => Live2DActionType::Listening,
        custom => Live2DActionType::Custom(custom.to_string()),
    };

    // 创建默认动作配置
    let action = match action_enum {
        Live2DActionType::Speaking => Live2DAction {
            action_type: Live2DActionType::Speaking,
            motion_group: "TapBody".to_string(),
            motion_index: Some(0),
            expression: Some("happy".to_string()),
            duration: Some(2000),
            priority: 5,
        },
        Live2DActionType::Thinking => Live2DAction {
            action_type: Live2DActionType::Thinking,
            motion_group: "Idle".to_string(),
            motion_index: Some(1),
            expression: Some("thinking".to_string()),
            duration: Some(3000),
            priority: 3,
        },
        Live2DActionType::Happy => Live2DAction {
            action_type: Live2DActionType::Happy,
            motion_group: "TapBody".to_string(),
            motion_index: Some(1),
            expression: Some("happy".to_string()),
            duration: Some(2000),
            priority: 4,
        },
        Live2DActionType::Confused => Live2DAction {
            action_type: Live2DActionType::Confused,
            motion_group: "Shake".to_string(),
            motion_index: Some(0),
            expression: Some("confused".to_string()),
            duration: Some(2000),
            priority: 4,
        },
        Live2DActionType::Surprised => Live2DAction {
            action_type: Live2DActionType::Surprised,
            motion_group: "TapBody".to_string(),
            motion_index: Some(2),
            expression: Some("surprised".to_string()),
            duration: Some(2000),
            priority: 4,
        },
        Live2DActionType::Greeting => Live2DAction {
            action_type: Live2DActionType::Greeting,
            motion_group: "TapBody".to_string(),
            motion_index: Some(0),
            expression: Some("happy".to_string()),
            duration: Some(3000),
            priority: 6,
        },
        Live2DActionType::Farewell => Live2DAction {
            action_type: Live2DActionType::Farewell,
            motion_group: "TapBody".to_string(),
            motion_index: Some(1),
            expression: Some("happy".to_string()),
            duration: Some(3000),
            priority: 6,
        },
        Live2DActionType::Idle => Live2DAction {
            action_type: Live2DActionType::Idle,
            motion_group: "Idle".to_string(),
            motion_index: Some(0),
            expression: None,
            duration: None,
            priority: 0,
        },
        Live2DActionType::Typing => Live2DAction {
            action_type: Live2DActionType::Typing,
            motion_group: "Idle".to_string(),
            motion_index: Some(2),
            expression: Some("focused".to_string()),
            duration: None,
            priority: 2,
        },
        Live2DActionType::Listening => Live2DAction {
            action_type: Live2DActionType::Listening,
            motion_group: "Idle".to_string(),
            motion_index: Some(0),
            expression: Some("attentive".to_string()),
            duration: None,
            priority: 2,
        },
        Live2DActionType::Custom(name) => Live2DAction {
            action_type: Live2DActionType::Custom(name.clone()),
            motion_group: "Custom".to_string(),
            motion_index: Some(0),
            expression: Some(name),
            duration: Some(2000),
            priority: 3,
        },
    };

    live2d_service.execute_action(action).await?;
    Ok(())
}

/// 开始Live2D说话状态
#[tauri::command]
pub async fn start_live2d_speaking(app_state: State<'_, AppState>) -> Result<(), String> {
    let live2d_service = app_state.live2d_service.lock().await;
    live2d_service.start_speaking().await?;
    debug!("Live2D开始说话状态");
    Ok(())
}

/// 结束Live2D说话状态
#[tauri::command]
pub async fn stop_live2d_speaking(app_state: State<'_, AppState>) -> Result<(), String> {
    let live2d_service = app_state.live2d_service.lock().await;
    live2d_service.stop_speaking().await?;
    debug!("Live2D结束说话状态");
    Ok(())
}

/// 开始Live2D思考状态
#[tauri::command]
pub async fn start_live2d_thinking(app_state: State<'_, AppState>) -> Result<(), String> {
    let live2d_service = app_state.live2d_service.lock().await;
    live2d_service.start_thinking().await?;
    debug!("Live2D开始思考状态");
    Ok(())
}

/// 设置Live2D表情
#[tauri::command]
pub async fn set_live2d_expression(
    expression: String,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    let live2d_service = app_state.live2d_service.lock().await;
    live2d_service.set_expression(&expression).await?;
    debug!("设置Live2D表情: {}", expression);
    Ok(())
}

/// 处理AI流式文本并触发Live2D动作
#[tauri::command]
pub async fn process_ai_text_for_live2d(
    text_chunk: String,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    let live2d_service = app_state.live2d_service.lock().await;
    live2d_service.process_streaming_text(&text_chunk).await?;
    Ok(())
}

/// 清空Live2D文本缓冲区
#[tauri::command]
pub async fn clear_live2d_text_buffer(app_state: State<'_, AppState>) -> Result<(), String> {
    let live2d_service = app_state.live2d_service.lock().await;
    live2d_service.clear_text_buffer().await;
    debug!("Live2D文本缓冲区已清空");
    Ok(())
}

/// 获取Live2D状态
#[tauri::command]
pub async fn get_live2d_state(app_state: State<'_, AppState>) -> Result<Live2DState, String> {
    let live2d_service = app_state.live2d_service.lock().await;
    let state = live2d_service.get_state().await;
    Ok(state)
}

/// 处理Live2D动作队列
#[tauri::command]
pub async fn process_live2d_action_queue(app_state: State<'_, AppState>) -> Result<(), String> {
    let live2d_service = app_state.live2d_service.lock().await;
    live2d_service.process_action_queue().await?;
    Ok(())
}

/// 批量添加文本触发器
#[tauri::command]
pub async fn add_live2d_text_triggers(
    triggers: HashMap<String, String>,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    let live2d_service = app_state.live2d_service.lock().await;
    let mut config = live2d_service.get_config().await;

    for (text, action_type) in triggers {
        let action_enum = match action_type.as_str() {
            "speaking" => Live2DActionType::Speaking,
            "thinking" => Live2DActionType::Thinking,
            "happy" => Live2DActionType::Happy,
            "confused" => Live2DActionType::Confused,
            "surprised" => Live2DActionType::Surprised,
            "greeting" => Live2DActionType::Greeting,
            "farewell" => Live2DActionType::Farewell,
            "idle" => Live2DActionType::Idle,
            "typing" => Live2DActionType::Typing,
            "listening" => Live2DActionType::Listening,
            custom => Live2DActionType::Custom(custom.to_string()),
        };
        config.text_triggers.insert(text, action_enum);
    }

    live2d_service.update_config(config).await?;
    info!("Live2D文本触发器已更新");
    Ok(())
}

/// 移除文本触发器
#[tauri::command]
pub async fn remove_live2d_text_trigger(
    text: String,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    let live2d_service = app_state.live2d_service.lock().await;
    let mut config = live2d_service.get_config().await;
    config.text_triggers.remove(&text);
    live2d_service.update_config(config).await?;
    info!("移除Live2D文本触发器: {}", text);
    Ok(())
}

/// 重置Live2D配置为默认值
#[tauri::command]
pub async fn reset_live2d_config(app_state: State<'_, AppState>) -> Result<(), String> {
    let live2d_service = app_state.live2d_service.lock().await;
    let default_config = Live2DConfig::default();
    live2d_service.update_config(default_config).await?;
    info!("Live2D配置已重置为默认值");
    Ok(())
}

/// 测试Live2D连接和功能
#[tauri::command]
pub async fn test_live2d_connection(
    app_state: State<'_, AppState>,
) -> Result<HashMap<String, Value>, String> {
    let live2d_service = app_state.live2d_service.lock().await;

    let mut result = HashMap::new();

    // 测试基本功能
    result.insert("service_available".to_string(), Value::Bool(true));

    // 获取当前状态
    let state = live2d_service.get_state().await;
    result.insert(
        "current_state".to_string(),
        serde_json::to_value(state).unwrap_or(Value::Null),
    );

    // 获取配置
    let config = live2d_service.get_config().await;
    result.insert(
        "config".to_string(),
        serde_json::to_value(config).unwrap_or(Value::Null),
    );

    // 测试动作执行
    let test_action = Live2DAction {
        action_type: Live2DActionType::Happy,
        motion_group: "Test".to_string(),
        motion_index: Some(0),
        expression: Some("test".to_string()),
        duration: Some(1000),
        priority: 1,
    };

    match live2d_service.execute_action(test_action).await {
        Ok(_) => {
            result.insert("action_test".to_string(), Value::Bool(true));
        }
        Err(e) => {
            result.insert("action_test".to_string(), Value::Bool(false));
            result.insert("action_test_error".to_string(), Value::String(e));
        }
    };

    info!("Live2D连接测试完成");
    Ok(result)
}
