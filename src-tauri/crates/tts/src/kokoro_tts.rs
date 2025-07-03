use futures::StreamExt;
use kokoro_tts::{KokoroTts, SynthSink, Voice};
use rodio::{OutputStream, Sink as RodioSink, buffer::SamplesBuffer};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::mpsc;

pub struct TtsEngine {
    text_tx: mpsc::Sender<String>, // 文本输入通道
    player: Arc<RodioSink>,
}

impl TtsEngine {
    pub async fn new(model_path: &str, voice_file: &str) -> anyhow::Result<Self> {
        let tts = KokoroTts::new(model_path, voice_file).await?;
        let (synth_sink, mut stream) = tts.stream(Voice::Zm098(1));
        let (_output_stream, handle) = OutputStream::try_default()?;
        let player = Arc::new(RodioSink::try_new(&handle)?);

        // 创建文本传输通道
        let (text_tx, text_rx) = mpsc::channel(100);

        // 共享的合成器，使用 Mutex 保证线程安全
        let sink = Arc::new(Mutex::new(synth_sink));
        let player_clone = player.clone();

        // 后台任务：接收文本并合成
        let sink_clone = sink.clone();
        tokio::spawn(async move {
            let mut receiver = text_rx;
            while let Some(text) = receiver.recv().await {
                let mut sink = sink_clone.lock().await;
                if let Err(e) = sink.synth(text).await {
                    eprintln!("合成失败: {}", e);
                }
            }
        });

        // 后台任务：处理音频流
        tokio::spawn(async move {
            while let Some((audio, took)) = stream.next().await {
                player_clone.append(SamplesBuffer::new(1, 24000, audio));
                println!("合成耗时: {:?}", took);
            }
        });

        Ok(Self { text_tx, player })
    }

    /// 添加文本进行合成（线程安全）
    pub async fn speak(&self, text: &str) -> anyhow::Result<()> {
        self.text_tx.send(text.to_string()).await?;
        Ok(())
    }

    /// 等待所有音频播放完成
    pub fn wait_until_end(&self) {
        self.player.sleep_until_end();
    }
}
// use futures::StreamExt;
// use kokoro_tts::{KokoroTts, SynthSink, Voice};
// use rodio::{OutputStream, Sink as RodioSink, buffer::SamplesBuffer};
// use std::sync::Arc;
// use tokio::sync::Mutex;

// pub struct TtsEngine {
//     sink: Arc<Mutex<SynthSink<String>>>, // 使用 String 代替 &str
//     _output_stream: OutputStream,
//     player: Arc<RodioSink>,
// }

// impl TtsEngine {
//     pub async fn new(model_path: &str, voice_file: &str) -> anyhow::Result<Self> {
//         let tts = KokoroTts::new(model_path, voice_file).await?;
//         // 使用 String 类型而不是 &str
//         let (synth_sink, mut stream) = tts.stream(Voice::Zm098(1));
//         let (output_stream, handle) = OutputStream::try_default()?;
//         let player = Arc::new(RodioSink::try_new(&handle)?);

//         let sink = Arc::new(Mutex::new(synth_sink));
//         let player_clone = player.clone();

//         tokio::spawn(async move {
//             while let Some((audio, took)) = stream.next().await {
//                 player_clone.append(SamplesBuffer::new(1, 24000, audio));
//                 println!("Synth took: {:?}", took);
//             }
//         });

//         Ok(Self {
//             sink,
//             _output_stream: output_stream,
//             player,
//         })
//     }

//     pub async fn speak(&self, text: &str) -> anyhow::Result<()> {
//         let mut sink = self.sink.lock().await;
//         // 将文本转为 String 存储
//         sink.synth(text.to_string()).await?;
//         Ok(())
//     }

//     pub fn wait_until_end(&self) {
//         self.player.sleep_until_end();
//     }
// }
