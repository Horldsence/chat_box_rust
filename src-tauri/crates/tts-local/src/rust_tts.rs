use std::sync::{Arc, Mutex};
use std::thread;
use tts::{Error, Features, Tts, Voice};

/// 线程安全的 TTS 流式接口
#[derive(Clone)]
pub struct TtsStream {
    inner: Arc<Mutex<TtsStreamInner>>,
}

struct TtsStreamInner {
    tts: Tts,
    queue: Vec<String>,
    is_speaking: bool,
}

impl TtsStream {
    /// 创建新的 TTS 流实例
    pub fn new() -> Result<Self, Error> {
        let tts = Tts::default()?;
        let inner = TtsStreamInner {
            tts,
            queue: Vec::new(),
            is_speaking: false,
        };

        Ok(Self {
            inner: Arc::new(Mutex::new(inner)),
        })
    }

    /// 添加语音到队列（非阻塞）
    pub fn speak(&self, text: &str) -> Result<(), Error> {
        let mut inner = self.inner.lock().unwrap();
        inner.queue.push(text.to_string());

        // 如果没有正在说话，开始处理队列
        if !inner.is_speaking {
            inner.is_speaking = true;
            self.process_queue(inner)?;
        }

        Ok(())
    }

    /// 处理语音队列
    fn process_queue(
        &self,
        mut inner: std::sync::MutexGuard<'_, TtsStreamInner>,
    ) -> Result<(), Error> {
        if inner.queue.is_empty() {
            inner.is_speaking = false;
            return Ok(());
        }

        let text = inner.queue.remove(0);
        let tts_clone = inner.tts.clone();
        let self_clone = self.clone();

        // 使用新线程处理语音播放，避免阻塞主线程
        thread::spawn(move || {
            if let Err(e) = tts_clone.speak(&text, false) {
                eprintln!("TTS speak error: {}", e);
            }

            // 递归处理队列中的下一项
            let mut inner = self_clone.inner.lock().unwrap();
            if let Err(e) = self_clone.process_queue(inner) {
                eprintln!("TTS queue processing error: {}", e);
            }
        });

        Ok(())
    }

    /// 停止当前语音并清空队列
    pub fn stop(&self) -> Result<(), Error> {
        let mut inner = self.inner.lock().unwrap();
        inner.tts.stop()?;
        inner.queue.clear();
        inner.is_speaking = false;
        Ok(())
    }

    /// 设置语速 (0.0-1.0)
    pub fn set_rate(&self, rate: f32) -> Result<(), Error> {
        let mut inner = self.inner.lock().unwrap();
        inner.tts.set_rate(rate)
    }

    /// 设置音量 (0.0-1.0)
    pub fn set_volume(&self, volume: f32) -> Result<(), Error> {
        let mut inner = self.inner.lock().unwrap();
        inner.tts.set_volume(volume)
    }

    /// 设置音高 (0.0-1.0)
    pub fn set_pitch(&self, pitch: f32) -> Result<(), Error> {
        let mut inner = self.inner.lock().unwrap();
        inner.tts.set_pitch(pitch)
    }

    /// 设置语音
    pub fn set_voice(&self, voice: &Voice) -> Result<(), Error> {
        let mut inner = self.inner.lock().unwrap();
        inner.tts.set_voice(voice)
    }

    /// 获取支持的功能
    pub fn supported_features(&self) -> Features {
        let inner = self.inner.lock().unwrap();
        inner.tts.supported_features()
    }

    /// 检查是否正在说话
    pub fn is_speaking(&self) -> Result<bool, Error> {
        let inner = self.inner.lock().unwrap();
        inner.tts.is_speaking()
    }

    /// 获取可用语音列表
    pub fn voices(&self) -> Result<Vec<Voice>, Error> {
        let inner = self.inner.lock().unwrap();
        inner.tts.voices()
    }
}

// 为 TtsStream 实现 Send，使其可以安全跨线程
unsafe impl Send for TtsStream {}
