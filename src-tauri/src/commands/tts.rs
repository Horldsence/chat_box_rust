use log::{error, info};
use serde::Deserialize;

use crate::state::{self, AppState};

use tts::kokoro_tts::TtsEngine;

#[derive(Deserialize)]
pub struct TTSSpeakRequest {
    pub text: String,
}

#[tauri::command]
pub async fn tts_speak(request: TTSSpeakRequest, engine: &TtsEngine) -> Result<(), String> {
    let text = request.text;

    info!("TTS合成文本: '{}'", text);

    // 修复：直接使用 TtsEngine 的异步方法，无需加锁
    if let Err(e) = engine.speak(&text) {
        error!("TTS合成失败: {}", e);
        return Err(format!("TTS合成失败: {}", e));
    }

    Ok(())
}
