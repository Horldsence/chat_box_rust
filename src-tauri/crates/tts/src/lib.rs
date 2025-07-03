// pub mod kokoro;
pub mod kokoro_tts;
// pub mod natural_tts;

#[cfg(test)]
mod tests {
    use super::kokoro_tts::TtsEngine;
    use kokoro_tts::Voice;

    #[tokio::test]
    #[ignore = "requires model files - run with `cargo test -- --ignored`"]
    async fn test_tts_engine() {
        // Initialize engine
        let engine = TtsEngine::new("kokoro-v1.1-zh.onnx", "voices-v1.1-zh.bin")
            .await
            .expect("Failed to initialize TTS engine");

        // Test basic synthesis
        engine.speak("Hello world").await.expect("Synthesis failed");

        // Wait for playback to complete
        engine.wait_until_end();
    }
}
