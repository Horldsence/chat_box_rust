use tokio_stream::{Stream, StreamExt};
use kokoro_tts::{get_voice_names, load, start_synth_session};
use rodio::{buffer::SamplesBuffer, OutputStream, Sink};
use std::sync::Arc;
use tokio::time::{sleep, Duration};

/// 文本到语音合成引擎
pub struct KokoroTTSEngine {
    voice_names: Vec<String>,
    sink: kokoro_tts::SynthSink,
    _output_stream: OutputStream,
    player: Arc<Sink>,
}

impl KokoroTTSEngine {
    /// 创建新的TTS引擎实例
    pub async fn new(model_path: &str, voices_path: &str) -> anyhow::Result<Self> {
        // 加载模型和语音数据
        load(model_path, voices_path).await?;

        // 获取可用语音列表
        let voice_names = get_voice_names().await?;

        // 创建合成会话（默认使用第一个语音，速度1.0）
        let (sink, mut stream) = start_synth_session(
            voice_names.first().expect("至少需要一个语音").as_str(),
            1.0,
        );

        // 创建音频输出流
        let (_output_stream, handle) = OutputStream::try_default()?;
        let player = Arc::new(Sink::try_new(&handle)?);

        // 启动音频播放后台任务
        let player_clone = player.clone();
        tokio::spawn(async move {
            while let Some((audio, took)) = stream.next().await {
                player_clone.append(SamplesBuffer::new(1, 24000, audio));
                println!("合成耗时: {:?}", took);
            }
        });

        Ok(Self {
            voice_names,
            sink,
            _output_stream,
            player,
        })
    }

    /// 获取可用语音列表
    pub fn get_voice_names(&self) -> &[String] {
        &self.voice_names
    }

    /// 合成文本
    pub async fn synth(&mut self, text: &str) -> anyhow::Result<()> {
        self.sink.synth(text).await
    }

    /// 设置当前语音
    pub async fn set_voice(&mut self, voice_name: &str) -> anyhow::Result<()> {
        self.sink.set_voice(voice_name).await
    }

    /// 设置合成速度
    pub async fn set_speed(&mut self, speed: f32) -> anyhow::Result<()> {
        self.sink.set_speed(speed).await
    }

    /// 等待所有音频播放完成
    pub fn wait_until_finished(&self) {
        self.player.sleep_until_end();
    }

    /// 保持程序运行指定时间（演示用）
    pub async fn keep_alive(&self, seconds: u64) {
        sleep(Duration::from_secs(seconds)).await;
    }
}