use crate::services::agent::ollama::OllamaAgent;
use crate::services::asr::vosk_python::VoskASR;
use crate::models::{Conversation, Message};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct AppState {
    pub conversations: Arc<Mutex<Vec<Conversation>>>,
    pub messages: Arc<Mutex<Vec<Message>>>,
    pub ollama_agent: Arc<OllamaAgent>,
    pub vosk_asr: Arc<tokio::sync::Mutex<VoskASR>>,
}

impl AppState {
    pub fn new(
        conversations: Vec<Conversation>, 
        messages: Vec<Message>, 
        ollama_agent: OllamaAgent, 
        vosk_asr: VoskASR
    ) -> Self {
        Self {
            conversations: Arc::new(Mutex::new(conversations)),
            messages: Arc::new(Mutex::new(messages)),
            ollama_agent: Arc::new(ollama_agent),
            vosk_asr: Arc::new(tokio::sync::Mutex::new(vosk_asr)),
        }
    }
}