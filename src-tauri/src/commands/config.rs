use crate::state::AppState;
use crate::utils::config::{AppConfig, Live2DConfig};
use log::{debug, info, warn};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use tauri::{AppHandle, Manager, State};

/// 获取应用配置
#[tauri::command]
pub async fn get_app_config_full(app_state: State<'_, AppState>) -> Result<AppConfig, String> {
    let config = app_state.config.lock().unwrap();
    Ok(config.clone())
}

/// 获取Live2D配置
#[tauri::command]
pub async fn get_live2d_config_from_file(app_handle: AppHandle) -> Result<Live2DConfig, String> {
    // 获取配置文件路径
    let config_path = app_handle
        .path()
        .resolve("config.yaml", tauri::path::BaseDirectory::Resource)
        .map_err(|e| format!("无法解析配置文件路径: {}", e))?;

    // 读取配置文件
    if config_path.exists() {
        let yaml_str =
            fs::read_to_string(&config_path).map_err(|e| format!("读取配置文件失败: {}", e))?;

        let config: AppConfig =
            serde_yaml::from_str(&yaml_str).map_err(|e| format!("解析配置文件失败: {}", e))?;

        info!("从配置文件加载Live2D配置: {:?}", config.live2d);
        Ok(config.live2d)
    } else {
        // 返回默认配置
        warn!("配置文件不存在，使用默认Live2D配置");
        Ok(AppConfig::default().live2d)
    }
}

/// 检查Live2D模型文件
#[tauri::command]
pub async fn check_live2d_model(
    model_path: String,
    app_handle: AppHandle,
) -> Result<HashMap<String, Value>, String> {
    let mut result = HashMap::new();

    debug!("检查Live2D模型: {}", model_path);

    // 解析模型路径
    let full_path = if Path::new(&model_path).is_absolute() {
        model_path.clone()
    } else {
        app_handle
            .path()
            .resolve(&model_path, tauri::path::BaseDirectory::Resource)
            .map_err(|e| format!("解析模型路径失败: {}", e))?
            .to_string_lossy()
            .to_string()
    };

    let model_file = Path::new(&full_path);
    result.insert("model_path".to_string(), Value::String(full_path.clone()));

    // 检查模型文件是否存在
    if !model_file.exists() {
        result.insert("exists".to_string(), Value::Bool(false));
        result.insert(
            "error".to_string(),
            Value::String("模型文件不存在".to_string()),
        );
        result.insert("can_fallback".to_string(), Value::Bool(true));
        return Ok(result);
    }

    result.insert("exists".to_string(), Value::Bool(true));

    // 检查文件扩展名
    let extension = model_file
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");

    let is_valid_extension = matches!(extension, "model3.json" | "json");
    result.insert(
        "valid_extension".to_string(),
        Value::Bool(is_valid_extension),
    );

    if !is_valid_extension {
        result.insert(
            "error".to_string(),
            Value::String(format!("不支持的模型文件格式: {}", extension)),
        );
        result.insert("can_fallback".to_string(), Value::Bool(true));
        return Ok(result);
    }

    // 尝试读取和解析模型文件
    match fs::read_to_string(&model_file) {
        Ok(content) => {
            result.insert("readable".to_string(), Value::Bool(true));

            // 尝试解析JSON
            match serde_json::from_str::<Value>(&content) {
                Ok(json) => {
                    result.insert("valid_json".to_string(), Value::Bool(true));

                    // 检查Live2D模型基本结构
                    let has_version = json.get("Version").is_some();
                    let has_model = json
                        .get("FileReferences")
                        .and_then(|refs| refs.get("Moc"))
                        .is_some()
                        || json.get("model").is_some();
                    let has_textures = json
                        .get("FileReferences")
                        .and_then(|refs| refs.get("Textures"))
                        .is_some()
                        || json.get("textures").is_some();

                    result.insert("has_version".to_string(), Value::Bool(has_version));
                    result.insert("has_model".to_string(), Value::Bool(has_model));
                    result.insert("has_textures".to_string(), Value::Bool(has_textures));

                    let is_valid_model = has_model && has_textures;
                    result.insert("valid_model".to_string(), Value::Bool(is_valid_model));

                    if is_valid_model {
                        // 检查相关文件是否存在
                        if let Some(model_dir) = model_file.parent() {
                            let mut missing_files = Vec::new();

                            // 检查.moc3文件
                            if let Some(moc_file) = json
                                .get("FileReferences")
                                .and_then(|refs| refs.get("Moc"))
                                .and_then(|moc| moc.as_str())
                                .or_else(|| json.get("model").and_then(|m| m.as_str()))
                            {
                                let moc_path = model_dir.join(moc_file);
                                if !moc_path.exists() {
                                    missing_files.push(moc_file.to_string());
                                }
                            }

                            // 检查纹理文件
                            if let Some(textures) = json
                                .get("FileReferences")
                                .and_then(|refs| refs.get("Textures"))
                                .and_then(|tex| tex.as_array())
                                .or_else(|| json.get("textures").and_then(|t| t.as_array()))
                            {
                                for texture in textures {
                                    if let Some(texture_file) = texture.as_str() {
                                        let texture_path = model_dir.join(texture_file);
                                        if !texture_path.exists() {
                                            missing_files.push(texture_file.to_string());
                                        }
                                    }
                                }
                            }

                            result.insert(
                                "missing_files".to_string(),
                                Value::Array(
                                    missing_files
                                        .iter()
                                        .map(|f| Value::String(f.clone()))
                                        .collect(),
                                ),
                            );
                            result.insert(
                                "all_files_exist".to_string(),
                                Value::Bool(missing_files.is_empty()),
                            );

                            if missing_files.is_empty() {
                                result.insert(
                                    "status".to_string(),
                                    Value::String("valid".to_string()),
                                );
                                info!("Live2D模型检查通过: {}", model_path);
                            } else {
                                result.insert(
                                    "status".to_string(),
                                    Value::String("incomplete".to_string()),
                                );
                                result.insert(
                                    "error".to_string(),
                                    Value::String(format!(
                                        "缺少文件: {}",
                                        missing_files.join(", ")
                                    )),
                                );
                                warn!("Live2D模型文件不完整: {}", model_path);
                            }
                        }
                    } else {
                        result.insert(
                            "status".to_string(),
                            Value::String("invalid_structure".to_string()),
                        );
                        result.insert(
                            "error".to_string(),
                            Value::String("模型文件结构无效".to_string()),
                        );
                    }
                }
                Err(_) => {
                    result.insert("valid_json".to_string(), Value::Bool(false));
                    result.insert(
                        "error".to_string(),
                        Value::String("模型文件不是有效的JSON格式".to_string()),
                    );
                }
            }
        }
        Err(e) => {
            result.insert("readable".to_string(), Value::Bool(false));
            result.insert(
                "error".to_string(),
                Value::String(format!("无法读取模型文件: {}", e)),
            );
        }
    }

    result.insert("can_fallback".to_string(), Value::Bool(true));
    Ok(result)
}

/// 获取Live2D模型状态和建议
#[tauri::command]
pub async fn get_live2d_model_status(
    app_handle: AppHandle,
) -> Result<HashMap<String, Value>, String> {
    let mut status = HashMap::new();

    // 获取Live2D配置
    let live2d_config = get_live2d_config_from_file(app_handle.clone()).await?;

    status.insert(
        "config".to_string(),
        serde_json::to_value(&live2d_config).unwrap(),
    );
    status.insert("enabled".to_string(), Value::Bool(live2d_config.enabled));

    if !live2d_config.enabled {
        status.insert("status".to_string(), Value::String("disabled".to_string()));
        status.insert(
            "message".to_string(),
            Value::String("Live2D功能已禁用".to_string()),
        );
        return Ok(status);
    }

    // 检查模型文件
    let model_check =
        check_live2d_model(live2d_config.model_path.clone(), app_handle.clone()).await?;
    status.insert(
        "model_check".to_string(),
        serde_json::to_value(&model_check).unwrap(),
    );

    // 根据检查结果确定状态
    let model_exists = model_check
        .get("exists")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let model_valid = model_check
        .get("valid_model")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let all_files_exist = model_check
        .get("all_files_exist")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if !model_exists {
        status.insert(
            "status".to_string(),
            Value::String("model_not_found".to_string()),
        );
        status.insert(
            "message".to_string(),
            Value::String("Live2D模型文件未找到".to_string()),
        );
        status.insert(
            "action_required".to_string(),
            Value::String("download_model".to_string()),
        );

        // 提供下载建议
        let download_suggestions = get_model_download_suggestions();
        status.insert(
            "download_suggestions".to_string(),
            serde_json::to_value(&download_suggestions).unwrap(),
        );
    } else if !model_valid || !all_files_exist {
        status.insert(
            "status".to_string(),
            Value::String("model_invalid".to_string()),
        );
        status.insert(
            "message".to_string(),
            Value::String("Live2D模型文件无效或不完整".to_string()),
        );
        status.insert(
            "action_required".to_string(),
            Value::String("fix_or_replace_model".to_string()),
        );

        if let Some(error) = model_check.get("error") {
            status.insert("error_details".to_string(), error.clone());
        }
    } else {
        status.insert("status".to_string(), Value::String("ready".to_string()));
        status.insert(
            "message".to_string(),
            Value::String("Live2D模型准备就绪".to_string()),
        );
    }

    // 检查是否可以降级到简单角色
    status.insert(
        "can_fallback".to_string(),
        Value::Bool(live2d_config.fallback_to_simple_character),
    );

    Ok(status)
}

/// 获取模型下载建议
fn get_model_download_suggestions() -> Vec<HashMap<String, String>> {
    vec![
        HashMap::from([
            ("name".to_string(), "Hiyori Free".to_string()),
            ("description".to_string(), "免费的Live2D模型".to_string()),
            (
                "url".to_string(),
                "https://www.live2d.com/download/sample-data/".to_string(),
            ),
            ("file_name".to_string(), "hiyori_free_en.zip".to_string()),
            ("recommended".to_string(), "true".to_string()),
        ]),
        HashMap::from([
            ("name".to_string(), "Live2D官方样例".to_string()),
            (
                "description".to_string(),
                "Live2D官方提供的样例模型".to_string(),
            ),
            (
                "url".to_string(),
                "https://github.com/Live2D/CubismWebSamples".to_string(),
            ),
            ("file_name".to_string(), "各种样例模型".to_string()),
            ("recommended".to_string(), "false".to_string()),
        ]),
    ]
}

/// 设置Live2D配置
#[tauri::command]
pub async fn update_live2d_config_in_file(
    live2d_config: Live2DConfig,
    app_handle: AppHandle,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    // 获取配置文件路径
    let config_path = app_handle
        .path()
        .resolve("config.yaml", tauri::path::BaseDirectory::Resource)
        .map_err(|e| format!("无法解析配置文件路径: {}", e))?;

    // 读取现有配置
    let mut app_config = if config_path.exists() {
        let yaml_str =
            fs::read_to_string(&config_path).map_err(|e| format!("读取配置文件失败: {}", e))?;
        serde_yaml::from_str::<AppConfig>(&yaml_str)
            .map_err(|e| format!("解析配置文件失败: {}", e))?
    } else {
        AppConfig::default()
    };

    // 更新Live2D配置
    app_config.live2d = live2d_config.clone();

    // 保存配置文件
    let yaml_str =
        serde_yaml::to_string(&app_config).map_err(|e| format!("序列化配置失败: {}", e))?;

    // 确保目录存在
    if let Some(parent) = config_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(|e| format!("创建配置目录失败: {}", e))?;
        }
    }

    fs::write(&config_path, yaml_str).map_err(|e| format!("写入配置文件失败: {}", e))?;

    // 更新内存中的配置
    {
        let mut config = app_state.config.lock().unwrap();
        config.live2d = live2d_config;
    }

    info!("Live2D配置已更新并保存到文件");
    Ok(())
}

/// 禁用Live2D功能
#[tauri::command]
pub async fn disable_live2d(
    app_handle: AppHandle,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    let mut live2d_config = get_live2d_config_from_file(app_handle.clone()).await?;
    live2d_config.enabled = false;

    update_live2d_config_in_file(live2d_config, app_handle, app_state).await?;
    info!("Live2D功能已禁用");
    Ok(())
}

/// 启用Live2D功能
#[tauri::command]
pub async fn enable_live2d(
    app_handle: AppHandle,
    app_state: State<'_, AppState>,
) -> Result<(), String> {
    let mut live2d_config = get_live2d_config_from_file(app_handle.clone()).await?;
    live2d_config.enabled = true;

    update_live2d_config_in_file(live2d_config, app_handle, app_state).await?;
    info!("Live2D功能已启用");
    Ok(())
}

/// 检查Live2D环境
#[tauri::command]
pub async fn check_live2d_environment(
    app_handle: AppHandle,
) -> Result<HashMap<String, Value>, String> {
    let mut result = HashMap::new();

    // 检查模型目录
    let models_dir = app_handle
        .path()
        .resolve("models/live2d", tauri::path::BaseDirectory::Resource)
        .map_err(|e| format!("解析模型目录失败: {}", e))?;

    result.insert(
        "models_dir".to_string(),
        Value::String(models_dir.to_string_lossy().to_string()),
    );
    result.insert(
        "models_dir_exists".to_string(),
        Value::Bool(models_dir.exists()),
    );

    // 扫描可用模型
    let mut available_models = Vec::new();
    if models_dir.exists() {
        if let Ok(entries) = fs::read_dir(&models_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    // 查找.model3.json文件
                    if let Ok(model_files) = fs::read_dir(&path) {
                        for model_file in model_files.flatten() {
                            let file_path = model_file.path();
                            if let Some(extension) = file_path.extension() {
                                if extension == "json"
                                    && file_path
                                        .file_name()
                                        .and_then(|name| name.to_str())
                                        .map(|name| name.contains("model"))
                                        .unwrap_or(false)
                                {
                                    let relative_path = file_path
                                        .strip_prefix(
                                            &app_handle
                                                .path()
                                                .resolve("", tauri::path::BaseDirectory::Resource)
                                                .unwrap(),
                                        )
                                        .unwrap_or(&file_path)
                                        .to_string_lossy()
                                        .to_string();

                                    available_models.push(HashMap::from([
                                        (
                                            "name".to_string(),
                                            path.file_name().unwrap().to_string_lossy().to_string(),
                                        ),
                                        ("path".to_string(), relative_path),
                                        (
                                            "full_path".to_string(),
                                            file_path.to_string_lossy().to_string(),
                                        ),
                                    ]));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    result.insert(
        "available_models".to_string(),
        serde_json::to_value(&available_models).unwrap(),
    );
    result.insert(
        "models_count".to_string(),
        Value::Number(available_models.len().into()),
    );

    // 检查WebGL支持（这个需要在前端检查）
    result.insert("webgl_check_required".to_string(), Value::Bool(true));

    Ok(result)
}
