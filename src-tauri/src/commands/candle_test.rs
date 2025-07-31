use crate::state::AppState;
use agent::models::llm::{CandleConfig, CandleProvider, ChatRequest, LLMProvider, WhichModel};
use log::{error, info};
use tauri::State;

/// 测试 Candle 模型的基本功能
#[tauri::command]
pub async fn test_candle_model() -> Result<String, String> {
    info!("开始测试 Candle 模型...");

    // 创建 Candle 配置
    let config = CandleConfig {
        default_model: WhichModel::W0_5b, // 使用最小的模型进行测试
        default_temperature: Some(0.7),
        default_max_tokens: Some(100), // 限制输出长度以加快测试
        cpu_only: true,                // 使用 CPU 以确保兼容性
        repeat_penalty: 1.1,
        repeat_last_n: 64,
        seed: 299792458,
        system_prompt: Some("你是一个有用的AI助手。".to_string()),
    };

    // 创建 Candle 提供者
    let provider = match CandleProvider::new(config) {
        Ok(p) => p,
        Err(e) => {
            error!("创建 Candle 提供者失败: {}", e);
            return Err(format!("创建 Candle 提供者失败: {}", e));
        }
    };

    // 测试简单的聊天请求
    let request = ChatRequest::new("qwen-0.5b")
        .with_user_message("你好，请简单介绍一下自己。")
        .with_max_tokens(50);

    info!("发送测试请求...");

    match provider.chat(request).await {
        Ok(response) => {
            info!("Candle 模型测试成功");
            Ok(format!("测试成功！模型响应: {}", response.content))
        }
        Err(e) => {
            error!("Candle 模型测试失败: {}", e);
            Err(format!("测试失败: {}", e))
        }
    }
}

/// 测试 Candle 模型的流式生成
#[tauri::command]
pub async fn test_candle_stream() -> Result<String, String> {
    info!("开始测试 Candle 流式生成...");

    let config = CandleConfig {
        default_model: WhichModel::W0_5b,
        default_temperature: Some(0.7),
        default_max_tokens: Some(50),
        cpu_only: true,
        repeat_penalty: 1.1,
        repeat_last_n: 64,
        seed: 299792458,
        system_prompt: Some("你是一个有用的AI助手。".to_string()),
    };

    let provider = match CandleProvider::new(config) {
        Ok(p) => p,
        Err(e) => {
            error!("创建 Candle 提供者失败: {}", e);
            return Err(format!("创建 Candle 提供者失败: {}", e));
        }
    };

    let request = ChatRequest::new("qwen-0.5b")
        .with_user_message("计算 1+1 等于多少？")
        .with_max_tokens(30)
        .with_stream(true);

    info!("发送流式测试请求...");

    match provider.chat_stream(request).await {
        Ok(mut stream) => {
            use tokio_stream::StreamExt;
            let mut content = String::new();
            let mut chunk_count = 0;

            while let Some(result) = stream.next().await {
                match result {
                    Ok(response) => {
                        content.push_str(&response.content);
                        chunk_count += 1;
                        if response.done {
                            break;
                        }
                    }
                    Err(e) => {
                        error!("流式响应错误: {}", e);
                        return Err(format!("流式响应错误: {}", e));
                    }
                }

                // 防止无限循环
                if chunk_count > 100 {
                    break;
                }
            }

            info!("Candle 流式生成测试成功，收到 {} 个数据块", chunk_count);
            Ok(format!(
                "流式测试成功！收到 {} 个数据块，内容: {}",
                chunk_count,
                content.trim()
            ))
        }
        Err(e) => {
            error!("Candle 流式生成测试失败: {}", e);
            Err(format!("流式测试失败: {}", e))
        }
    }
}

/// 获取 Candle 支持的模型列表
#[tauri::command]
pub async fn get_candle_models() -> Result<Vec<String>, String> {
    info!("获取 Candle 支持的模型列表...");

    let config = CandleConfig::default();
    let provider = match CandleProvider::new(config) {
        Ok(p) => p,
        Err(e) => {
            error!("创建 Candle 提供者失败: {}", e);
            return Err(format!("创建 Candle 提供者失败: {}", e));
        }
    };

    match provider.list_models().await {
        Ok(models) => {
            let model_names: Vec<String> = models.into_iter().map(|m| m.name).collect();
            info!("获取到 {} 个 Candle 模型", model_names.len());
            Ok(model_names)
        }
        Err(e) => {
            error!("获取模型列表失败: {}", e);
            Err(format!("获取模型列表失败: {}", e))
        }
    }
}

/// 检查 Candle 提供者的健康状态
#[tauri::command]
pub async fn check_candle_health() -> Result<bool, String> {
    info!("检查 Candle 提供者健康状态...");

    let config = CandleConfig::default();
    let provider = match CandleProvider::new(config) {
        Ok(p) => p,
        Err(e) => {
            error!("创建 Candle 提供者失败: {}", e);
            return Err(format!("创建 Candle 提供者失败: {}", e));
        }
    };

    match provider.health_check().await {
        Ok(healthy) => {
            if healthy {
                info!("Candle 提供者健康状态: 正常");
            } else {
                info!("Candle 提供者健康状态: 异常");
            }
            Ok(healthy)
        }
        Err(e) => {
            error!("健康检查失败: {}", e);
            Err(format!("健康检查失败: {}", e))
        }
    }
}

/// 使用应用状态中的 LLM 管理器测试 Candle
#[tauri::command]
pub async fn test_candle_with_manager(state: State<'_, AppState>) -> Result<String, String> {
    info!("使用 LLM 管理器测试 Candle...");

    let request = ChatRequest::new("qwen-0.5b")
        .with_user_message("你好，这是一个测试消息。")
        .with_max_tokens(50);

    match state.llm_manager.chat(request).await {
        Ok(response) => {
            info!("通过 LLM 管理器的 Candle 测试成功");
            Ok(format!("测试成功！响应: {}", response.content))
        }
        Err(e) => {
            error!("通过 LLM 管理器的 Candle 测试失败: {}", e);
            Err(format!("测试失败: {}", e))
        }
    }
}
