use futures::StreamExt;
use kokoro_tts::{KokoroTts, Voice};
use rodio::{OutputStream, Sink, buffer::SamplesBuffer};
use std::sync::Arc;
use tokio::time::{Duration, sleep};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let tts = KokoroTts::new(
        "model/kokoro/kokoro-v1.1-zh.onnx",
        "model/kokoro/voices-v1.1-zh.bin",
    )
    .await?;
    let (mut sink, mut stream) = tts.stream(Voice::Zm098(1));
    sink.synth("hello world.").await?;
    sink.synth("你好，我们是一群追逐梦想的人。").await?;
    sink.set_voice(Voice::Zf032(2));
    sink.synth("我正在使用qq。").await?;
    sink.set_voice(Voice::Zf090(3));
    sink.synth("今天天气如何？").await?;
    sink.set_voice(Voice::Zm045(1));
    sink.synth("你在使用Rust编程语言吗？").await?;
    sink.set_voice(Voice::Zf039(1));
    sink.synth(
        "你轻轻地走过那
在风雨花丛中
每一点一滴带走
是我醒来的梦
是在那天空上
最美丽的云朵
在那彩虹 最温柔的风",
    )
    .await?;
    sink.set_voice(Voice::Zf088(1));
    sink.synth(
        "你静静看着我们
最不舍的面容
像流星划过夜空
转瞬即逝的梦
是最深情的脸 在这一瞬间
在遥远天边
",
    )
    .await?;

    let (_stream, handle) = OutputStream::try_default().unwrap();
    let player = Arc::new(Sink::try_new(&handle)?);
    let player2 = player.clone();
    tokio::spawn(async move {
        while let Some((audio, took)) = stream.next().await {
            player.append(SamplesBuffer::new(1, 24000, audio));
            println!("Synth took: {:?}", took);
        }
    });

    sleep(Duration::from_secs(20)).await;
    player2.sleep_until_end();
    Ok(())
}
