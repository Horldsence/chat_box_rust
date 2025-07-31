use asr::{AsrConfig, AsrProvider, AsrStatus, BaiduASR};
use log::{error, info};
use std::time::Duration;
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    env_logger::init();

    info!("Starting Baidu ASR example");

    // 创建ASR配置
    let config = AsrConfig::default();

    // 创建百度ASR实例
    let mut baidu_asr = BaiduASR::new(config)?;

    info!("Created Baidu ASR instance");

    // 开始语音识别，设置30秒超时和2秒静音检测
    let mut stream = baidu_asr
        .listen_and_transcribe(Some(Duration::from_secs(30)), Some(Duration::from_secs(2)))?;

    info!("Started transcription stream, listening for speech...");

    // 处理识别结果
    while let Some(result) = stream.next().await {
        match result {
            Ok(status) => match status {
                AsrStatus::PartialResult(text) => {
                    info!("🎤 识别结果: {}", text);
                }
                AsrStatus::FinalResult(text) => {
                    info!("✅ 最终结果: {}", text);
                }
                AsrStatus::SilenceDetected => {
                    info!("🔇 检测到静音");
                }
                AsrStatus::Timeout => {
                    info!("⏰ 识别超时");
                    break;
                }
                AsrStatus::Finished => {
                    info!("🏁 识别完成");
                    break;
                }
            },
            Err(e) => {
                error!("❌ 识别错误: {}", e);
                break;
            }
        }
    }

    // 释放资源
    baidu_asr.release_resources()?;
    info!("Released ASR resources");

    Ok(())
}
