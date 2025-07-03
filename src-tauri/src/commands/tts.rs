use log::{error, info};
use once_cell::sync::OnceCell;
use serde::Deserialize;
use std::sync::Arc;
use tauri::State;

use tts::kokoro_tts::TtsEngine;

// 修改为正确的全局变量类型
static GLOBAL_TTS_ENGINE: OnceCell<Arc<TtsEngine>> = OnceCell::new();

pub async fn init_tts_engine(model_path: &str, voices_path: &str) {
    let engine = TtsEngine::new(model_path, voices_path)
        .await
        .expect("TTS引擎初始化失败");

    // 修复：直接存储 Arc<TtsEngine> 类型
    GLOBAL_TTS_ENGINE.set(Arc::new(engine)).ok();
}

fn get_tts_engine() -> Arc<TtsEngine> {
    GLOBAL_TTS_ENGINE.get().expect("TTS引擎未初始化").clone()
}

#[derive(Deserialize)]
pub struct TTSSpeakRequest {
    pub text: String,
    pub voice: Option<String>,
}

#[tauri::command]
pub async fn tts_speak(request: TTSSpeakRequest) -> Result<(), String> {
    let text = request.text;

    let engine = get_tts_engine();

    info!("TTS合成文本: '{}'", text);

    // 修复：直接使用 TtsEngine 的异步方法，无需加锁
    if let Err(e) = engine.speak(&text).await {
        error!("TTS合成失败: {}", e);
        return Err(format!("TTS合成失败: {}", e));
    }

    // 修复：使用异步等待播放结束
    engine.wait_until_end();

    Ok(())
}
