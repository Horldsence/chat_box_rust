use base64::Engine;
use futures::Stream;
use log::{debug, error};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use tokio::sync::mpsc as tokio_mpsc;
use tokio::time::interval;

use crate::traits::{AsrConfig, AsrProvider, AsrStatus};

// 百度ASR配置常量
const BAIDU_ASR_URL: &str = "https://vop.baidu.com/server_api";
const ACCESS_TOKEN: &str =
    "24.419b810b2613e51f88cdd73b69613bf2.2592000.1756451380.282335-119645483";

#[derive(Debug)]
pub struct BaiduASR {
    client: Client,
    access_token: String,
    is_recording: bool,
    config: BaiduAsrConfig,
}

#[derive(Debug, Clone)]
pub struct BaiduAsrConfig {
    pub format: String,
    pub rate: i32,
    pub channel: i32,
    pub cuid: String,
    pub dev_pid: Option<i32>,
    pub chunk_duration: Duration,
}

impl Default for BaiduAsrConfig {
    fn default() -> Self {
        Self {
            format: "pcm".to_string(),
            rate: 16000,
            channel: 1,
            cuid: "rust_asr_client".to_string(),
            dev_pid: Some(1537),                         // 普通话输入法模型
            chunk_duration: Duration::from_millis(3000), // 3秒分块
        }
    }
}

#[derive(Debug, Serialize)]
struct BaiduAsrRequest {
    format: String,
    rate: i32,
    channel: i32,
    cuid: String,
    token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    dev_pid: Option<i32>,
    speech: String, // base64编码的音频数据
    len: usize,     // 音频数据字节长度
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct BaiduAsrResponse {
    err_no: i32,
    err_msg: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    sn: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    corpus_no: Option<String>,
}

#[derive(Debug)]
pub struct BaiduAsrError(String);

impl std::fmt::Display for BaiduAsrError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Baidu ASR error: {}", self.0)
    }
}

impl std::error::Error for BaiduAsrError {}

impl From<reqwest::Error> for BaiduAsrError {
    fn from(err: reqwest::Error) -> Self {
        BaiduAsrError(format!("HTTP request error: {}", err))
    }
}

impl From<serde_json::Error> for BaiduAsrError {
    fn from(err: serde_json::Error) -> Self {
        BaiduAsrError(format!("JSON parsing error: {}", err))
    }
}

impl From<std::io::Error> for BaiduAsrError {
    fn from(err: std::io::Error) -> Self {
        BaiduAsrError(format!("IO error: {}", err))
    }
}

impl AsrProvider for BaiduASR {
    type Error = BaiduAsrError;
    type ResultStream = BaiduAsrStream;

    fn new(_config: AsrConfig) -> Result<Self, Self::Error> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| BaiduAsrError(format!("Failed to create HTTP client: {}", e)))?;

        let baidu_config = BaiduAsrConfig {
            cuid: format!("rust_asr_{}", std::process::id()),
            ..Default::default()
        };

        debug!("Creating BaiduASR with config: {:?}", baidu_config);

        Ok(Self {
            client,
            access_token: ACCESS_TOKEN.to_string(),
            is_recording: false,
            config: baidu_config,
        })
    }

    fn start_recording(&mut self) -> Result<(), Self::Error> {
        debug!("Starting Baidu ASR recording");
        self.is_recording = true;
        Ok(())
    }

    fn stop_recording(&mut self) -> Result<(), Self::Error> {
        debug!("Stopping Baidu ASR recording");
        self.is_recording = false;
        Ok(())
    }

    fn release_resources(&mut self) -> Result<(), Self::Error> {
        debug!("Releasing Baidu ASR resources");
        self.is_recording = false;
        Ok(())
    }

    fn is_recording(&self) -> bool {
        self.is_recording
    }

    fn listen_and_transcribe(
        &mut self,
        timeout: Option<Duration>,
        silence_threshold: Option<Duration>,
    ) -> Result<Self::ResultStream, Self::Error> {
        debug!("Starting Baidu ASR transcription");

        self.start_recording()?;

        let timeout_duration = timeout.unwrap_or(Duration::from_secs(30));
        let silence_duration = silence_threshold.unwrap_or(Duration::from_secs(2));

        let (sender, receiver) = tokio_mpsc::channel(100);
        let client = self.client.clone();
        let access_token = self.access_token.clone();
        let config = self.config.clone();

        // 启动音频捕获和识别任务
        tokio::spawn(async move {
            if let Err(e) = Self::audio_capture_task(
                client,
                access_token,
                config,
                sender,
                timeout_duration,
                silence_duration,
            )
            .await
            {
                error!("Audio capture task failed: {}", e);
            }
        });

        Ok(BaiduAsrStream {
            receiver,
            is_active: true,
        })
    }
}

impl BaiduASR {
    async fn audio_capture_task(
        client: Client,
        access_token: String,
        config: BaiduAsrConfig,
        sender: tokio_mpsc::Sender<Result<AsrStatus, BaiduAsrError>>,
        timeout: Duration,
        silence_threshold: Duration,
    ) -> Result<(), BaiduAsrError> {
        debug!("Starting audio capture task");

        let start_time = std::time::Instant::now();
        let mut chunk_interval = interval(config.chunk_duration);
        let mut last_audio_time = std::time::Instant::now();
        let mut audio_buffer = Vec::new();

        // 模拟音频数据捕获（实际应用中需要从麦克风捕获）
        // 这里我们创建一个简单的模拟器
        let mut silence_detected = false;

        loop {
            tokio::select! {
                _ = chunk_interval.tick() => {
                    // 检查超时
                    if start_time.elapsed() > timeout {
                        debug!("Timeout reached, stopping transcription");
                        let _ = sender.send(Ok(AsrStatus::Timeout)).await;
                        break;
                    }

                    // 模拟音频数据收集
                    let audio_chunk = Self::simulate_audio_chunk();

                    if audio_chunk.is_empty() {
                        // 检测到静音
                        if last_audio_time.elapsed() > silence_threshold && !silence_detected {
                            silence_detected = true;
                            debug!("Silence detected");
                            let _ = sender.send(Ok(AsrStatus::SilenceDetected)).await;
                            continue;
                        }
                    } else {
                        last_audio_time = std::time::Instant::now();
                        silence_detected = false;
                        audio_buffer.extend_from_slice(&audio_chunk);
                    }

                    // 当缓冲区有足够数据时，发送到百度ASR
                    if audio_buffer.len() >= 16000 * 2 { // 1秒的16KHz PCM数据
                        match Self::recognize_audio(&client, &access_token, &config, &audio_buffer).await {
                            Ok(text) => {
                                if !text.is_empty() {
                                    debug!("Recognition result: {}", text);
                                    let _ = sender.send(Ok(AsrStatus::PartialResult(text))).await;
                                }
                            }
                            Err(e) => {
                                error!("Recognition failed: {}", e);
                                let _ = sender.send(Err(e)).await;
                            }
                        }
                        audio_buffer.clear();
                    }
                }
            }
        }

        debug!("Audio capture task completed");
        Ok(())
    }

    async fn recognize_audio(
        client: &Client,
        access_token: &str,
        config: &BaiduAsrConfig,
        audio_data: &[u8],
    ) -> Result<String, BaiduAsrError> {
        // 将音频数据转换为base64
        let audio_base64 = base64::engine::general_purpose::STANDARD.encode(audio_data);

        let request = BaiduAsrRequest {
            format: config.format.clone(),
            rate: config.rate,
            channel: config.channel,
            cuid: config.cuid.clone(),
            token: access_token.to_string(),
            dev_pid: config.dev_pid,
            speech: audio_base64,
            len: audio_data.len(),
        };

        debug!("Sending recognition request to Baidu ASR");

        let response = client
            .post(BAIDU_ASR_URL)
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        let response_text = response.text().await?;
        debug!("Baidu ASR response: {}", response_text);

        let baidu_response: BaiduAsrResponse = serde_json::from_str(&response_text)?;

        if baidu_response.err_no != 0 {
            return Err(BaiduAsrError(format!(
                "Baidu ASR API error {}: {}",
                baidu_response.err_no, baidu_response.err_msg
            )));
        }

        // 提取识别结果
        if let Some(results) = baidu_response.result {
            if let Some(first_result) = results.first() {
                return Ok(first_result.clone());
            }
        }

        Ok(String::new())
    }

    fn simulate_audio_chunk() -> Vec<u8> {
        // 模拟音频数据，实际应用中应该从麦克风捕获
        // 这里返回一些模拟的PCM数据或空数据（表示静音）

        use std::time::{SystemTime, UNIX_EPOCH};

        // 使用时间戳来决定是否返回音频数据
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();

        // 80% 的概率返回空数据（静音），20% 返回模拟音频
        if timestamp % 5 != 0 {
            Vec::new() // 静音
        } else {
            // 生成一些模拟的PCM数据（实际中应该是真实的音频数据）
            (0..1600u16) // 100ms worth of 16KHz mono PCM
                .map(|i| ((i as f32 * 0.1).sin() * 127.0 + 128.0) as u8)
                .collect()
        }
    }
}

pub struct BaiduAsrStream {
    receiver: tokio_mpsc::Receiver<Result<AsrStatus, BaiduAsrError>>,
    is_active: bool,
}

impl Stream for BaiduAsrStream {
    type Item = Result<AsrStatus, BaiduAsrError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if !self.is_active {
            return Poll::Ready(None);
        }

        match Pin::new(&mut self.receiver).poll_recv(cx) {
            Poll::Ready(Some(result)) => {
                match &result {
                    Ok(AsrStatus::Timeout) | Ok(AsrStatus::Finished) => {
                        self.is_active = false;
                    }
                    Err(_) => {
                        self.is_active = false;
                    }
                    _ => {}
                }
                Poll::Ready(Some(result))
            }
            Poll::Ready(None) => {
                self.is_active = false;
                Poll::Ready(None)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

impl Drop for BaiduAsrStream {
    fn drop(&mut self) {
        if self.is_active {
            debug!("BaiduAsrStream dropped while active");
            self.is_active = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_stream::StreamExt;

    #[tokio::test]
    async fn test_baidu_asr_creation() {
        let asr = BaiduASR::new(AsrConfig::default()).unwrap();
        assert!(!asr.is_recording());
        assert_eq!(asr.access_token, ACCESS_TOKEN);
    }

    #[tokio::test]
    async fn test_baidu_asr_recording_control() {
        let mut asr = BaiduASR::new(AsrConfig::default()).unwrap();

        // 测试开始录音
        asr.start_recording().unwrap();
        assert!(asr.is_recording());

        // 测试停止录音
        asr.stop_recording().unwrap();
        assert!(!asr.is_recording());

        // 测试释放资源
        asr.release_resources().unwrap();
        assert!(!asr.is_recording());
    }

    #[tokio::test]
    async fn test_baidu_asr_stream() {
        let mut asr = BaiduASR::new(AsrConfig::default()).unwrap();

        let stream = asr
            .listen_and_transcribe(
                Some(Duration::from_secs(5)),
                Some(Duration::from_millis(1000)),
            )
            .unwrap();

        // 测试流的基本功能
        let mut count = 0;
        let mut stream = Box::pin(stream);

        while let Some(result) = stream.next().await {
            match result {
                Ok(status) => {
                    debug!("Received status: {:?}", status);
                    count += 1;

                    // 限制测试时间
                    if count > 3 {
                        break;
                    }
                }
                Err(e) => {
                    debug!("Received error: {}", e);
                    break;
                }
            }
        }

        assert!(count > 0);
    }

    #[test]
    fn test_baidu_asr_config() {
        let config = BaiduAsrConfig::default();
        assert_eq!(config.format, "pcm");
        assert_eq!(config.rate, 16000);
        assert_eq!(config.channel, 1);
        assert_eq!(config.dev_pid, Some(1537));
    }

    #[test]
    fn test_audio_chunk_simulation() {
        let chunk = BaiduASR::simulate_audio_chunk();
        // 模拟的音频块可能为空（静音）或有数据
        debug!("Simulated chunk length: {}", chunk.len());
    }
}
