mod test {
    use agent::models::llm::ollama::provider::OllamaConfig;
    use agent::{LLMManager, LLMManagerConfig, ProviderConfig};

    #[tokio::test]
    async fn test_llm_manager_with_custom_config() {
        let mut providers = std::collections::HashMap::new();
        providers.insert(
            "ollama".to_string(),
            ProviderConfig::Ollama(OllamaConfig {
                host: "127.0.0.1".to_string(),
                default_model: "qwen2.5:0.5b".to_string(),
                ..Default::default()
            }),
        );

        let config = LLMManagerConfig {
            default_provider: "ollama".to_string(),
            fallback_providers: vec![],
            auto_fallback: false,
            health_check_interval_seconds: 60,
            providers,
        };

        let manager = LLMManager::new(config).await;
        assert!(manager.is_ok(), "LLMManager should initialize successfully");
    }
}
