use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use log::{error, info, warn}; // 移除未使用的debug导入
use num_traits::ToPrimitive;
use tokio_stream::Stream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::time::Duration;
use vosk::{Model, Recognizer};
use std::pin::Pin;
use std::task::{Context as TaskContext, Poll};
use std::fmt;
use futures;

pub struct VoskASR {
    model: Model,
    is_recording: Arc<AtomicBool>,
}


#[derive(Debug)]
struct VoskError(String);

// 实现 Display 和 Error
impl fmt::Display for VoskError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for VoskError {}

// VoskError 已经可以通过标准库自动转换为 Box<dyn std::error::Error + Send + Sync>
// 因为它已经实现了 std::error::Error

impl VoskASR {
    pub fn new(model_path: Option<&str>) -> Result<Self> {
        // 使用提供的路径或默认路径
        let path = model_path.unwrap_or("model");
        info!("Loading Vosk model from path: {}", path);

        let model = Model::new(path)
            .with_context(|| format!("Failed to load Vosk model from path: {}", path))?;

        Ok(Self {
            model,
            is_recording: Arc::new(AtomicBool::new(false)),
        })
    }

    /// 检查是否正在录音
    pub fn is_recording(&self) -> bool {
        self.is_recording.load(Ordering::SeqCst)
    }

    /// 手动停止正在进行的录音
    pub fn stop_recording(&self) {
        if self.is_recording() {
            info!("Manually stopping voice recording");
            self.is_recording.store(false, Ordering::SeqCst);
        }
    }

    /// 创建音频录音流
    fn create_stream<T>(
        &self,
        device: &cpal::Device,
        config: &cpal::StreamConfig,
        tx: mpsc::Sender<Vec<i16>>, // 修改通道类型为样本类型（如 i16）
    ) -> Result<cpal::Stream>
    where
        T: cpal::Sample + cpal::SizedSample + ToPrimitive + Send + 'static,
    {
        let is_recording = self.is_recording.clone();

        let err_fn = |err| error!("Recording error: {}", err);

        let stream = device.build_input_stream(
            config,
            move |data: &[T], _: &_| {
                // 检查是否应该继续录音
                if !is_recording.load(Ordering::SeqCst) {
                    return;
                }

                // 将音频数据转换为i16格式
                let samples: Vec<i16> = data
                    .iter()
                    .filter_map(|&sample| {
                        let value = sample.to_f32()?;
                        Some((value * 32767.0) as i16)
                    })
                    .collect();

                // 只有当有数据且通道未满时才发送
                if !samples.is_empty() {
                    if tx.try_send(samples).is_err() {
                        warn!("Audio buffer full, sample dropped");
                    }
                }
            },
            err_fn,
            None, // 延迟
        )?;

        Ok(stream)
    }

    // // 提供listen_and_transcribe方法，无需确保与main.rs匹配
    // pub async fn listen_and_transcribe(
    //     &mut self,
    //     max_duration_secs: Option<f32>,
    // ) -> Result<
    //     impl tokio_stream::Stream<Item = Result<String, Box<dyn std::error::Error + Send + Sync>>> + use<'_>,
    //     Box<dyn std::error::Error + Send + Sync>,
    // > {
    //     // 验证录音状态
    //     if self.is_recording() {
    //         return Err(Box::new(VoskError("Recording session already in progress".into())));
    //     }

    //     // 设置录音状态原子标志
    //     self.is_recording.store(true, Ordering::SeqCst);

    //     // 音频设备初始化
    //     let host = cpal::default_host();
    //     let device = host
    //         .default_input_device()
    //         .ok_or_else(|| Box::new(VoskError("No default input device".into())) as Box<dyn std::error::Error + Send + Sync>)?;
    //     let config = device.default_input_config()?;
    //     let sample_rate = config.sample_rate().0 as f32;

    //     // 创建识别器
    //     let recognizer = Arc::new(tokio::sync::Mutex::new(Recognizer::new(
    //         &self.model,
    //         sample_rate,
    //     )));

    //     // 创建音频数据通道
    //     let (audio_tx, audio_rx) = mpsc::channel::<Vec<i16>>(32);

    //     // 启动音频流
    //     let stream = self.create_stream::<i16>(&device, &config.config(), audio_tx)?;
    //     stream.play()?;

    //     // 创建结果流
    //     Ok(stream! {
    //         let mut audio_rx = audio_rx;
    //         let is_recording_flag = self.is_recording.clone();
    //         let _guard = scopeguard::guard(is_recording_flag, |flag| {
    //             flag.store(false, Ordering::SeqCst);
    //         });

    //         let start = tokio::time::Instant::now();
    //         let max_duration = max_duration_secs.map(|s| Duration::from_secs_f32(s));
    //         let mut silence_counter = 0;
    //         const SILENCE_THRESHOLD: usize = 30;
    //         const SOUND_THRESHOLD: i16 = 500;

    //         loop {
    //             // 计算剩余时间
    //             let remaining = match max_duration {
    //                 Some(dur) => {
    //                     let elapsed = start.elapsed();
    //                     if elapsed >= dur {
    //                         // 已超时，退出循环
    //                         yield Ok("[timeout reached]".to_string());
    //                         break;
    //                     }
    //                     Some(dur - elapsed)
    //                 }
    //                 None => None,
    //             };

    //             // 接收数据（带超时）
    //             let data = match remaining {
    //                 Some(rem) => match timeout(rem, audio_rx.recv()).await {
    //                     Ok(Some(d)) => d,
    //                     Ok(None) => break, // 通道关闭
    //                     Err(_) => {
    //                         yield Ok("[timeout reached]".to_string());
    //                         break;
    //                     }
    //                 },
    //                 None => match audio_rx.recv().await {
    //                     Some(d) => d,
    //                     None => break,
    //                 },
    //             };

    //             // 静音检测
    //             let has_sound = data.iter().any(|&s| s.abs() > SOUND_THRESHOLD);
    //             silence_counter = if has_sound { 0 } else { silence_counter + 1 };

    //             if silence_counter > SILENCE_THRESHOLD {
    //                 yield Ok("[silence detected]".to_string());
    //                 break;
    //             }

    //             // 语音识别处理
    //             let mut recognizer_lock = recognizer.lock().await;
    //             let rec = recognizer_lock.as_mut().unwrap();
    //             if let Err(e) = rec.accept_waveform(&data) {
    //                 yield Err(VoskError(format!("Recognition error: {}", e)).into());
    //                 continue;
    //             }

    //             // 发送部分结果
    //             let partial = rec.partial_result().partial;
    //             if !partial.is_empty() {
    //                 yield Ok(partial.to_string());
    //             }
    //             drop(recognizer_lock);
    //         };

    //         // 发送最终结果
    //         let mut recognizer_lock = recognizer.lock().await;
    //         let rec = recognizer_lock.as_mut().unwrap();
    //         let final_result = rec.final_result().single().unwrap().text;
    //         if !final_result.is_empty() {
    //             yield Ok(final_result.to_string());
    //         }
    //         drop(recognizer_lock);
    //     })
    // }

    pub async fn listen_and_transcribe(
        &mut self,
        max_duration_secs: Option<f32>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String, Box<dyn std::error::Error + Send + Sync>>> + Send>>> {
        // 初始化音频设备
        let host = cpal::default_host();
        let device = host.default_input_device()
            .ok_or_else(|| Box::new(VoskError("No default input device".into())))?;
        let config = device.default_input_config()?;
        let sample_rate = config.sample_rate().0 as f32;

        // 创建识别器
        let recognizer = Arc::new(tokio::sync::Mutex::new(
            Recognizer::new(&self.model, sample_rate)
                .ok_or_else(|| VoskError("Failed to create recognizer".into()))?
        ));

        // 创建音频通道
        let (audio_tx, audio_rx) = mpsc::channel(32);

        // 创建音频流
        let stream = self.create_stream::<i16>(&device, &config.config(), audio_tx)?;
        stream.play()?;

        // 构建 VoskStream
        let vosk_stream = VoskStream {
            is_recording: self.is_recording.clone(),
            audio_rx,
            recognizer,
            max_duration: max_duration_secs.map(|s| Duration::from_secs_f32(s)),
            start_time: Some(tokio::time::Instant::now()),
            silence_counter: 0,
        };

        Ok(Box::pin(vosk_stream))
    }
}

impl Drop for VoskASR {
    fn drop(&mut self) {
        // 确保在实例被销毁时停止任何录音
        self.stop_recording();
    }
}

pub struct VoskStream {
    is_recording: Arc<AtomicBool>,
    audio_rx: mpsc::Receiver<Vec<i16>>,
    recognizer: Arc<tokio::sync::Mutex<Recognizer>>,
    max_duration: Option<Duration>,
    start_time: Option<tokio::time::Instant>,
    silence_counter: usize,
}

impl Stream for VoskStream {
    type Item = Result<String, Box<dyn std::error::Error + Send + Sync>>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut TaskContext<'_>) -> Poll<Option<Self::Item>> {
        loop {
            // 处理超时逻辑
            if let Some(max_duration) = self.max_duration {
                let elapsed = self.start_time.unwrap().elapsed();
                if elapsed >= max_duration {
                    return Poll::Ready(Some(Ok("[timeout reached]".to_string())));
                }
            }

            // 接收音频数据
            match self.audio_rx.poll_recv(cx) {
                Poll::Ready(Some(data)) => {
                    // 静音检测逻辑
                    const SOUND_THRESHOLD: i16 = 500;
                    let has_sound = data.iter().any(|&s| s.abs() > SOUND_THRESHOLD);
                    self.silence_counter = if has_sound { 0 } else { self.silence_counter + 1 };

                    if self.silence_counter > 30 {
                        return Poll::Ready(Some(Ok("[silence detected]".to_string())));
                    }

                    // 语音识别处理
                    let recognizer = self.recognizer.clone();
                    // Use try_lock instead of blocking
                    let mut recognizer = match recognizer.try_lock() {
                        Ok(guard) => guard,
                        Err(e) => return Poll::Pending, // Return pending if lock is unavailable
                    };

                    if let Err(e) = recognizer.accept_waveform(&data) {
                        return Poll::Ready(Some(Err(Box::new(VoskError(format!("Recognition error: {}", e))))));
                    }

                    // 返回部分结果
                    let partial = recognizer.partial_result().partial;
                    if !partial.is_empty() {
                        return Poll::Ready(Some(Ok(partial.to_string())));
                    }

                    // // 处理最终结果
                    // let recognizer = self.recognizer.clone();
                    // let guard_result = futures::executor::block_on(recognizer.lock());
                    // let mut recognizer = match guard_result {
                    //     Ok(guard) => guard,
                    //     Err(e) => return Poll::Ready(Some(Err(Box::new(e)))),
                    // };

                    // let final_result = recognizer.final_result().single().unwrap().text;
                    // if !final_result.is_empty() {
                    //     return Poll::Ready(Some(Ok(final_result.to_string())));
                    // }
                    return Poll::Ready(None);
                }
                Poll::Pending => return Poll::Pending,
                Poll::Ready(None) => {
                    // 通道关闭，返回 None
                    return Poll::Ready(None);
                }
            }
        }
    }
}