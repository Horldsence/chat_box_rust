#[cfg(test)]
mod tests {
    use super::*;
    use crate::{InitError, UserAction};
    use cb_config::config::InitConfig;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn test_init_config_default() {
        let config = InitConfig::default();
        assert_eq!(config.ai_model.model_type, "ollama");
        assert_eq!(config.ai_model.model_name, "qwen2.5:0.5b");
        assert_eq!(config.ai_model.server_port, 11434);
        assert!(!config.voice.enabled);
        assert!(config.database.enabled);
        assert!(config.app_behavior.show_error_dialogs);
    }

    #[test]
    fn test_init_config_new() {
        let config_path = PathBuf::from("test_config.yaml");
        let config = InitConfig::new(config_path.clone());
        assert_eq!(config.config_path, config_path);
    }

    #[test]
    fn test_model_type_from_string() {
        use crate::model_manager::ModelType;

        let ollama_type = ModelType::from("ollama");
        assert!(matches!(ollama_type, ModelType::Ollama));

        let candle_type = ModelType::from("candle");
        assert!(matches!(candle_type, ModelType::Candle));

        let default_type = ModelType::from("unknown");
        assert!(matches!(default_type, ModelType::Ollama));
    }

    #[test]
    fn test_init_error_display() {
        let error = InitError::ModelUnavailable("Test model".to_string());
        let error_str = format!("{}", error);
        assert!(error_str.contains("模型不可用"));
        assert!(error_str.contains("Test model"));

        let error = InitError::ConfigLoadFailed("Invalid config".to_string());
        let error_str = format!("{}", error);
        assert!(error_str.contains("配置加载失败"));
        assert!(error_str.contains("Invalid config"));
    }

    #[test]
    fn test_component_status() {
        use crate::error_handler::ComponentStatus;

        let status = ComponentStatus::new("test_component");
        assert_eq!(status.name, "test_component");
        assert!(!status.initialized);
        assert!(!status.ignored);
        assert!(status.error.is_none());

        let success_status = ComponentStatus::success("success_component");
        assert!(success_status.initialized);
        assert!(!success_status.ignored);

        let failed_status = ComponentStatus::failed("failed_component", "Test error");
        assert!(!failed_status.initialized);
        assert!(!failed_status.ignored);
        assert_eq!(failed_status.error.as_ref().unwrap(), "Test error");

        let ignored_status = ComponentStatus::ignored("ignored_component", "Test reason");
        assert!(!ignored_status.initialized);
        assert!(ignored_status.ignored);
        assert_eq!(ignored_status.error.as_ref().unwrap(), "Test reason");
    }

    #[test]
    fn test_user_action_serialization() {
        use serde_json;

        let ignore_action = UserAction::Ignore;
        let serialized = serde_json::to_string(&ignore_action).unwrap();
        let deserialized: UserAction = serde_json::from_str(&serialized).unwrap();
        assert!(matches!(deserialized, UserAction::Ignore));

        let retry_action = UserAction::Retry;
        let serialized = serde_json::to_string(&retry_action).unwrap();
        let deserialized: UserAction = serde_json::from_str(&serialized).unwrap();
        assert!(matches!(deserialized, UserAction::Retry));

        let exit_action = UserAction::Exit;
        let serialized = serde_json::to_string(&exit_action).unwrap();
        let deserialized: UserAction = serde_json::from_str(&serialized).unwrap();
        assert!(matches!(deserialized, UserAction::Exit));
    }

    #[test]
    fn test_initialization_result() {
        use crate::InitializationResult;

        let result = InitializationResult {
            success: true,
            failed_components: vec![],
            ignored_components: vec![],
        };
        assert!(result.success);
        assert!(result.failed_components.is_empty());
        assert!(result.ignored_components.is_empty());

        let result_with_failures = InitializationResult {
            success: false,
            failed_components: vec!["AI模型".to_string(), "数据库".to_string()],
            ignored_components: vec!["语音识别".to_string()],
        };
        assert!(!result_with_failures.success);
        assert_eq!(result_with_failures.failed_components.len(), 2);
        assert_eq!(result_with_failures.ignored_components.len(), 1);
    }

    #[tokio::test]
    async fn test_config_load_and_save() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test_config.yaml");

        let config = InitConfig::new(config_path.clone());
        let default_config = config.clone().load_config();

        // Test saving
        default_config.save_config(&default_config, &config_path);
        assert!(config_path.exists());

        // Test loading
        let loaded_config = InitConfig::new(config_path).load_config();
        assert_eq!(
            loaded_config.ai_model.model_name,
            default_config.ai_model.model_name
        );
        assert_eq!(
            loaded_config.ai_model.server_port,
            default_config.ai_model.server_port
        );
    }

    #[test]
    fn test_model_manager_model_info() {
        use crate::model_manager::ModelManager;

        // Create a mock app handle for testing (this would need proper mocking in real tests)
        // For now, just test the basic model info generation logic
        let mut config = InitConfig::default();

        // Test Ollama model info
        config.ai_model.model_type = "ollama".to_string();
        config.ai_model.model_name = "test-model".to_string();
        config.ai_model.server_url = "http://test".to_string();
        config.ai_model.server_port = 8080;

        // Note: We can't easily test ModelManager without a real AppHandle
        // This would require dependency injection or mocking

        // Test Candle model info
        config.ai_model.model_type = "candle".to_string();
        config.ai_model.candle_model_id = Some("test/model".to_string());
    }

    #[test]
    fn test_error_types() {
        use std::error::Error;

        let error = InitError::ModelUnavailable("test".to_string());
        assert!(error.source().is_none());

        let error = InitError::NetworkError("connection failed".to_string());
        let error_str = format!("{}", error);
        assert!(error_str.contains("网络错误"));
        assert!(error_str.contains("connection failed"));
    }

    // Integration test helpers
    #[cfg(feature = "test-integration")]
    mod integration_tests {
        use super::*;

        // These tests would require a full Tauri environment
        // and are marked with a feature flag for optional testing

        #[tokio::test]
        #[ignore] // Requires full app context
        async fn test_full_initialization() {
            // This would test the complete initialization flow
            // with a real or mocked Tauri app handle
        }

        #[tokio::test]
        #[ignore] // Requires Ollama service
        async fn test_ollama_connectivity() {
            // This would test actual Ollama connectivity
            // when an Ollama service is available
        }

        #[tokio::test]
        #[ignore] // Requires network access
        async fn test_candle_model_download() {
            // This would test actual model downloading
            // from Hugging Face
        }
    }
}
