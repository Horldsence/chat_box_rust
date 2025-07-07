// pub mod kokoro;
pub mod kokoro_tts;
// pub mod natural_tts;

#[cfg(test)]
mod tests {
    use super::kokoro_tts::TtsEngine;

    #[tokio::test]
    #[ignore = "requires model files - run with `cargo test -- --ignored`"]
    async fn test_tts_engine() {
        // Initialize engine
        let engine = TtsEngine::new("kokoro-v1.1-zh.onnx".to_string(), "voices-v1.1-zh.bin".to_string())
            .expect("Failed to initialize TTS engine");

        // Test basic synthesis
        engine.speak("Hello world").expect("Synthesis failed");
    }
}
