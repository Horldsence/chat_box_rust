use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Host, Sample, SampleFormat, SampleRate, Stream, StreamConfig};
use log::{debug, error, info, warn};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::mpsc as tokio_mpsc;

#[derive(Debug)]
pub struct AudioCaptureError(String);

impl std::fmt::Display for AudioCaptureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Audio capture error: {}", self.0)
    }
}

impl std::error::Error for AudioCaptureError {}

impl From<cpal::BuildStreamError> for AudioCaptureError {
    fn from(err: cpal::BuildStreamError) -> Self {
        AudioCaptureError(format!("Failed to build audio stream: {}", err))
    }
}

impl From<cpal::PlayStreamError> for AudioCaptureError {
    fn from(err: cpal::PlayStreamError) -> Self {
        AudioCaptureError(format!("Failed to play audio stream: {}", err))
    }
}

impl From<cpal::DefaultStreamConfigError> for AudioCaptureError {
    fn from(err: cpal::DefaultStreamConfigError) -> Self {
        AudioCaptureError(format!("Failed to get default stream config: {}", err))
    }
}

#[derive(Debug, Clone)]
pub struct AudioConfig {
    pub sample_rate: u32,
    pub channels: u16,
    pub buffer_size: usize,
    pub chunk_duration: Duration,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            sample_rate: 16000,
            channels: 1,
            buffer_size: 4096,
            chunk_duration: Duration::from_millis(500), // 500ms chunks
        }
    }
}

pub struct AudioCapture {
    _host: Host,
    device: Device,
    config: AudioConfig,
    stream: Option<Stream>,
    is_recording: bool,
}

impl AudioCapture {
    pub fn new(config: AudioConfig) -> Result<Self, AudioCaptureError> {
        let host = cpal::default_host();

        let device = host
            .default_input_device()
            .ok_or_else(|| AudioCaptureError("No input device available".to_string()))?;

        info!(
            "Using audio input device: {}",
            device.name().unwrap_or_else(|_| "Unknown".to_string())
        );

        Ok(Self {
            _host: host,
            device,
            config,
            stream: None,
            is_recording: false,
        })
    }

    pub fn start_capture(
        &mut self,
        sender: tokio_mpsc::Sender<Vec<u8>>,
    ) -> Result<(), AudioCaptureError> {
        if self.is_recording {
            warn!("Audio capture is already running");
            return Ok(());
        }

        let supported_configs = self
            .device
            .supported_input_configs()
            .map_err(|e| AudioCaptureError(format!("Failed to get supported configs: {}", e)))?;

        // 寻找匹配的配置
        let mut target_config = None;
        for config in supported_configs {
            if config.channels() <= self.config.channels
                && config.min_sample_rate() <= SampleRate(self.config.sample_rate)
                && config.max_sample_rate() >= SampleRate(self.config.sample_rate)
            {
                target_config = Some(config.with_sample_rate(SampleRate(self.config.sample_rate)));
                break;
            }
        }

        let config = target_config
            .or_else(|| {
                // 如果没有找到匹配的配置，尝试使用默认配置
                self.device.default_input_config().ok()
            })
            .ok_or_else(|| {
                AudioCaptureError("No suitable audio configuration found".to_string())
            })?;

        info!("Using audio config: {:?}", config);

        // 创建音频缓冲区
        let buffer = Arc::new(Mutex::new(Vec::new()));
        let buffer_clone = buffer.clone();
        let chunk_size =
            (self.config.sample_rate as f32 * self.config.chunk_duration.as_secs_f32()) as usize;
        let target_channels = self.config.channels;

        // 创建同步通道用于线程间通信
        let (sync_sender, sync_receiver) = mpsc::channel();

        // 根据样本格式创建不同的流
        let stream = match config.sample_format() {
            SampleFormat::F32 => self.create_stream::<f32>(
                config.into(),
                buffer_clone,
                chunk_size,
                target_channels,
                sync_sender,
            )?,
            SampleFormat::I16 => self.create_stream::<i16>(
                config.into(),
                buffer_clone,
                chunk_size,
                target_channels,
                sync_sender,
            )?,
            SampleFormat::U16 => self.create_stream::<u16>(
                config.into(),
                buffer_clone,
                chunk_size,
                target_channels,
                sync_sender,
            )?,
            sample_format => {
                return Err(AudioCaptureError(format!(
                    "Unsupported sample format: {:?}",
                    sample_format
                )));
            }
        };

        // 启动流
        stream.play()?;
        self.stream = Some(stream);
        self.is_recording = true;

        // 启动后台任务处理音频数据
        tokio::spawn(async move {
            while let Ok(audio_chunk) = sync_receiver.recv() {
                if sender.send(audio_chunk).await.is_err() {
                    debug!("Audio capture sender channel closed");
                    break;
                }
            }
        });

        info!("Audio capture started successfully");
        Ok(())
    }

    fn create_stream<T>(
        &self,
        config: StreamConfig,
        buffer: Arc<Mutex<Vec<u8>>>,
        chunk_size: usize,
        target_channels: u16,
        sender: mpsc::Sender<Vec<u8>>,
    ) -> Result<Stream, AudioCaptureError>
    where
        T: Sample + cpal::SizedSample + Send + Sync + 'static,
        f32: From<T>,
    {
        let stream = self.device.build_input_stream(
            &config,
            move |data: &[T], _: &cpal::InputCallbackInfo| {
                // 将音频数据转换为PCM格式
                let pcm_data = Self::convert_to_pcm(data, config.channels, target_channels);

                // 添加到缓冲区
                if let Ok(mut buffer_guard) = buffer.lock() {
                    buffer_guard.extend_from_slice(&pcm_data);

                    // 如果缓冲区达到块大小，发送数据
                    if buffer_guard.len() >= chunk_size * 2 {
                        // 16位PCM，每个样本2字节
                        let chunk = buffer_guard.drain(..chunk_size * 2).collect::<Vec<u8>>();
                        if let Err(e) = sender.send(chunk) {
                            error!("Failed to send audio chunk: {}", e);
                        }
                    }
                }
            },
            |err| {
                error!("Audio stream error: {}", err);
            },
            None,
        )?;

        Ok(stream)
    }

    fn convert_to_pcm<T>(data: &[T], input_channels: u16, target_channels: u16) -> Vec<u8>
    where
        T: Sample,
        f32: From<T>,
    {
        let mut pcm_data = Vec::new();

        for chunk in data.chunks(input_channels as usize) {
            // 如果输入是立体声但我们需要单声道，取平均值
            let sample = if input_channels > target_channels && target_channels == 1 {
                let sum: f32 = chunk.iter().map(|&s| f32::from(s)).sum();
                sum / input_channels as f32
            } else {
                // 否则取第一个通道
                f32::from(chunk[0])
            };

            // 转换为16位PCM
            let pcm_sample = (sample.clamp(-1.0, 1.0) * 32767.0) as i16;
            pcm_data.extend_from_slice(&pcm_sample.to_le_bytes());
        }

        pcm_data
    }

    pub fn stop_capture(&mut self) -> Result<(), AudioCaptureError> {
        if !self.is_recording {
            return Ok(());
        }

        if let Some(stream) = self.stream.take() {
            drop(stream);
        }

        self.is_recording = false;
        info!("Audio capture stopped");
        Ok(())
    }

    pub fn is_recording(&self) -> bool {
        self.is_recording
    }

    pub fn get_device_info(&self) -> String {
        self.device
            .name()
            .unwrap_or_else(|_| "Unknown device".to_string())
    }
}

impl Drop for AudioCapture {
    fn drop(&mut self) {
        if let Err(e) = self.stop_capture() {
            error!("Error stopping audio capture in drop: {}", e);
        }
    }
}

// 音频处理工具函数
pub struct AudioProcessor;

impl AudioProcessor {
    /// 检测音频中的静音
    pub fn detect_silence(audio_data: &[u8], threshold: i16, min_duration: Duration) -> bool {
        let samples: Vec<i16> = audio_data
            .chunks_exact(2)
            .map(|chunk| i16::from_le_bytes([chunk[0], chunk[1]]))
            .collect();

        if samples.is_empty() {
            return true;
        }

        // 计算音频能量
        let energy: f64 = samples
            .iter()
            .map(|&sample| (sample as f64).powi(2))
            .sum::<f64>()
            / samples.len() as f64;

        let rms = energy.sqrt();

        // 如果RMS低于阈值，认为是静音
        rms < threshold as f64
    }

    /// 应用简单的降噪滤波器
    pub fn apply_noise_reduction(audio_data: &mut [u8]) {
        let mut samples: Vec<i16> = audio_data
            .chunks_exact(2)
            .map(|chunk| i16::from_le_bytes([chunk[0], chunk[1]]))
            .collect();

        // 简单的移动平均滤波器
        if samples.len() > 2 {
            for i in 1..samples.len() - 1 {
                samples[i] = ((samples[i - 1] as i32 + samples[i] as i32 + samples[i + 1] as i32)
                    / 3) as i16;
            }
        }

        // 将处理后的样本写回
        for (i, &sample) in samples.iter().enumerate() {
            let bytes = sample.to_le_bytes();
            audio_data[i * 2] = bytes[0];
            audio_data[i * 2 + 1] = bytes[1];
        }
    }

    /// 音频增益调整
    pub fn adjust_gain(audio_data: &mut [u8], gain: f32) {
        let mut samples: Vec<i16> = audio_data
            .chunks_exact(2)
            .map(|chunk| i16::from_le_bytes([chunk[0], chunk[1]]))
            .collect();

        for sample in &mut samples {
            let adjusted = (*sample as f32 * gain).clamp(-32768.0, 32767.0) as i16;
            *sample = adjusted;
        }

        // 将处理后的样本写回
        for (i, &sample) in samples.iter().enumerate() {
            let bytes = sample.to_le_bytes();
            audio_data[i * 2] = bytes[0];
            audio_data[i * 2 + 1] = bytes[1];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_audio_capture_creation() {
        let config = AudioConfig::default();
        let result = AudioCapture::new(config);

        // 在某些CI环境中可能没有音频设备
        match result {
            Ok(capture) => {
                assert!(!capture.is_recording());
                println!("Audio device: {}", capture.get_device_info());
            }
            Err(e) => {
                println!("No audio device available: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_audio_capture_start_stop() {
        let config = AudioConfig::default();

        if let Ok(mut capture) = AudioCapture::new(config) {
            let (sender, mut receiver) = tokio_mpsc::channel(100);

            // 启动捕获
            if capture.start_capture(sender).is_ok() {
                assert!(capture.is_recording());

                // 等待一小段时间
                sleep(Duration::from_millis(100)).await;

                // 停止捕获
                capture.stop_capture().unwrap();
                assert!(!capture.is_recording());

                // 检查是否收到音频数据
                let mut received_chunks = 0;
                while let Ok(chunk) = receiver.try_recv() {
                    received_chunks += 1;
                    assert!(!chunk.is_empty());
                    if received_chunks >= 5 {
                        break;
                    }
                }

                println!("Received {} audio chunks", received_chunks);
            }
        }
    }

    #[test]
    fn test_silence_detection() {
        // 创建静音音频数据
        let silent_audio = vec![0u8; 1000];
        assert!(AudioProcessor::detect_silence(
            &silent_audio,
            100,
            Duration::from_millis(100)
        ));

        // 创建有声音的音频数据
        let mut loud_audio = Vec::new();
        for i in 0..500 {
            let sample = (i as i16 * 100) % 32767;
            loud_audio.extend_from_slice(&sample.to_le_bytes());
        }
        assert!(!AudioProcessor::detect_silence(
            &loud_audio,
            100,
            Duration::from_millis(100)
        ));
    }

    #[test]
    fn test_audio_processing() {
        let mut audio_data = Vec::new();

        // 生成测试音频数据
        for i in 0..1000 {
            let sample = (i as i16 * 100) % 10000;
            audio_data.extend_from_slice(&sample.to_le_bytes());
        }

        let original_data = audio_data.clone();

        // 测试降噪
        AudioProcessor::apply_noise_reduction(&mut audio_data);
        assert_eq!(audio_data.len(), original_data.len());

        // 测试增益调整
        AudioProcessor::adjust_gain(&mut audio_data, 1.5);
        assert_eq!(audio_data.len(), original_data.len());
    }
}
