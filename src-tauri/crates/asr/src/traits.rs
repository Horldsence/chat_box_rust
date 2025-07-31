use futures::Stream;
use std::time::Duration;

/// ASR (Automatic Speech Recognition) 的统一接口
///
/// 这个 trait 定义了所有 ASR 实现必须提供的功能
pub trait AsrProvider: Send + Sync {
    /// ASR 处理过程中可能发生的错误类型
    type Error: std::error::Error + Send + Sync + 'static;

    /// ASR 识别结果的流类型
    type ResultStream: Stream<Item = Result<AsrStatus, Self::Error>> + Send + 'static;

    /// 创建一个新的 ASR 提供者实例
    fn new(config: AsrConfig) -> Result<Self, Self::Error>
    where
        Self: Sized;

    /// 开始录音
    fn start_recording(&mut self) -> Result<(), Self::Error>;

    /// 停止录音
    fn stop_recording(&mut self) -> Result<(), Self::Error>;

    /// 释放资源
    fn release_resources(&mut self) -> Result<(), Self::Error>;

    /// 检查是否正在录音
    fn is_recording(&self) -> bool;

    /// 开始监听并转录语音
    ///
    /// `timeout`: 可选的超时时间，超过这个时间后自动停止录音
    /// `silence_threshold`: 可选的静音检测阈值，检测到静音后自动停止录音
    fn listen_and_transcribe(
        &mut self,
        timeout: Option<Duration>,
        silence_threshold: Option<Duration>,
    ) -> Result<Self::ResultStream, Self::Error>;
}

/// ASR 结果的状态枚举
#[derive(Debug, Clone, PartialEq)]
pub enum AsrStatus {
    /// 部分识别结果
    PartialResult(String),
    /// 最终识别结果
    FinalResult(String),
    /// 检测到静音
    SilenceDetected,
    /// 超时
    Timeout,
    /// 识别结束
    Finished,
}

/// ASR 配置选项
#[derive(Debug, Clone)]
pub struct AsrConfig {
    /// 模型路径
    pub model_path: Option<String>,
    /// 超时时间（毫秒）
    pub timeout_ms: Option<u64>,
    /// 静音检测阈值
    pub silence_threshold: Option<i16>,
    /// 静音持续帧数阈值
    pub silence_frames_threshold: Option<usize>,
    /// api_service
    pub api_service: Option<String>,
    /// token
    pub token: Option<String>,
}

impl Default for AsrConfig {
    fn default() -> Self {
        Self {
            model_path: None,
            timeout_ms: Some(15000), // 默认15秒超时
            silence_threshold: Some(500),
            silence_frames_threshold: Some(30),
            api_service: None,
            token: None,
        }
    }
}
