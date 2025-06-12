pub mod agent;
pub mod models;

// 重新导出主要类型
pub use agent::{Agent, AgentConfig, ConversationContext, MultiProviderAgent};
pub use models::llm::{
    ChatMessage, ChatRequest, ChatResponse, LLMError, LLMManager, LLMManagerConfig, LLMProvider,
    ModelInfo, OllamaAgent, OllamaConfig, OllamaProvider, ProviderConfig, ProviderHealth,
    ProviderStatus, StreamGenerator,
};

#[cfg(feature = "candle")]
pub use models::llm::{CandleConfig, CandleProvider};
