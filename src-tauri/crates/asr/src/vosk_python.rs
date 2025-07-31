use futures::Stream;
use log::{debug, error, info};
use pyo3::prelude::*;
use pyo3::types::PyList;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use tokio::sync::mpsc as tokio_mpsc;

use crate::traits::{AsrConfig, AsrProvider, AsrStatus};

#[derive(Debug)]
pub struct VoskASR {
    model_path: Option<String>,
    instance: Option<Py<PyAny>>, // 只有在需要时才初始化
    is_recording: bool,
}

// 定义错误类型
#[derive(Debug)]
pub struct VoskPythonError(String);

impl std::fmt::Display for VoskPythonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "VoskPython error: {}", self.0)
    }
}

impl std::error::Error for VoskPythonError {}

impl From<PyErr> for VoskPythonError {
    fn from(err: PyErr) -> Self {
        VoskPythonError(format!("Python error: {:?}", err))
    }
}

impl From<pyo3::DowncastError<'_, '_>> for VoskPythonError {
    fn from(err: pyo3::DowncastError<'_, '_>) -> Self {
        VoskPythonError(format!("Python downcast error: {}", err))
    }
}

#[allow(dead_code)]
impl AsrProvider for VoskASR {
    type Error = VoskPythonError;
    type ResultStream = VoskStream;

    fn new(config: AsrConfig) -> Result<Self, Self::Error> {
        debug!(
            "Creating VoskASR with path {}",
            config.model_path.as_deref().unwrap_or("None")
        );

        // 只保存模型路径，不立即加载模型
        let model_path = config.model_path.clone();

        Ok(Self {
            model_path,
            instance: None,
            is_recording: false,
        })
    }

    fn start_recording(&mut self) -> Result<(), Self::Error> {
        self.ensure_initialized()?;

        Python::with_gil(|py| {
            if let Some(instance) = &self.instance {
                instance.bind(py).call_method0("start_stream")?;
                debug!("Started audio stream");
                self.is_recording = true;
            }
            Ok(())
        })
    }

    fn stop_recording(&mut self) -> Result<(), Self::Error> {
        if let Some(instance) = &self.instance {
            Python::with_gil(|py| {
                instance.bind(py).call_method0("stop_stream")?;
                debug!("Stopped audio stream");
                self.is_recording = false;
                Ok(())
            })
        } else {
            self.is_recording = false;
            Ok(())
        }
    }

    fn release_resources(&mut self) -> Result<(), Self::Error> {
        if let Some(instance) = &self.instance {
            Python::with_gil(|py| {
                instance.bind(py).call_method0("release_resources")?;
                debug!("Released resources");
                Ok(())
            })
        } else {
            Ok(())
        }
    }

    fn is_recording(&self) -> bool {
        self.is_recording
    }

    fn listen_and_transcribe(
        &mut self,
        timeout: Option<Duration>,
        silence_threshold: Option<Duration>,
    ) -> Result<Self::ResultStream, Self::Error> {
        self.ensure_initialized()?;

        // 启动录音
        self.start_recording()?;

        // 设置超时
        let timeout_ms = timeout.map(|t| t.as_millis() as u64).unwrap_or(30000);
        let silence_threshold_ms = silence_threshold
            .map(|t| t.as_millis() as u64)
            .unwrap_or(2000);

        // 创建通道用于接收识别结果
        let (sender, receiver) = tokio_mpsc::channel(100);

        // 在 GIL 下获取 Python 实例并设置参数，同时创建用于跨线程的克隆
        let vosk_instance = Python::with_gil(|py| {
            let instance = self.instance.as_ref().unwrap().bind(py);
            instance.call_method1("set_timeout", (timeout_ms,))?;
            instance.call_method1("set_silence_threshold", (silence_threshold_ms,))?;
            // 使用 clone_ref 在 GIL 下安全克隆 Python 对象引用
            Ok::<Py<PyAny>, PyErr>(self.instance.as_ref().unwrap().clone_ref(py))
        })
        .map_err(VoskPythonError::from)?;

        // 启动一个线程来轮询 Python 队列
        let vosk_clone = Python::with_gil(|py| vosk_instance.clone_ref(py));
        std::thread::spawn(move || {
            Python::with_gil(|py| {
                let instance = vosk_clone.bind(py);

                loop {
                    // 检查是否有新的识别结果
                    let result: PyResult<Option<String>> = instance
                        .call_method0("get_result")
                        .and_then(|res| res.extract());

                    match result {
                        Ok(Some(text)) => {
                            // 发送识别结果到通道
                            if let Err(e) = sender.blocking_send(text) {
                                error!("Failed to send recognition result: {}", e);
                                break;
                            }
                        }
                        Ok(None) => {
                            // 没有新的结果，短暂休眠后继续
                            std::thread::sleep(Duration::from_millis(10));
                        }
                        Err(e) => {
                            // 发生错误，发送错误消息并退出
                            error!("Error polling recognition results: {:?}", e);
                            let _ = sender.blocking_send(format!("[error] {:?}", e));
                            break;
                        }
                    }
                }
            });
        });

        // 返回流
        Ok(VoskStream {
            receiver,
            vosk_instance,
            is_active: true,
        })
    }
}

impl VoskASR {
    // 初始化模型和Python实例（但不启动麦克风）
    fn ensure_initialized(&mut self) -> Result<(), VoskPythonError> {
        if self.instance.is_some() {
            return Ok(());
        }

        Python::with_gil(|py| {
            let sys = py.import("sys")?;
            let path = sys.getattr("path")?;
            let path = path.downcast::<PyList>()?;

            // 虚拟环境路径
            #[cfg(windows)]
            path.insert(0, r".venv\Lib\site-packages")?;
            #[cfg(not(windows))]
            path.insert(0, ".venv/lib/python3.9/site-packages")?;

            path.insert(0, "src/python")?;

            debug!("Python path: {:?}", path);

            // 导入Python模块
            let module = py.import("voskASR")?;
            let class = module.getattr("VoskRecognizer")?;

            debug!("VoskASR class: {:?}", class);

            // 创建实例
            let instance: Py<PyAny> = match &self.model_path {
                Some(p) => class.call1((p,))?.into(),
                None => class.call0()?.into(),
            };

            debug!("VoskASR instance created: {:?}", instance);
            self.instance = Some(instance);

            Ok(())
        })
    }

    // 完全释放资源的方法，在 Drop 实现中调用
    fn release_resources_internal(&mut self) -> PyResult<()> {
        if let Some(instance) = &self.instance {
            Python::with_gil(|py| {
                instance.bind(py).call_method0("release_resources")?;
                debug!("Released resources");
                Ok(())
            })
        } else {
            Ok(())
        }
    }

    // Drop 实现，确保资源被释放
    pub fn drop_impl(&mut self) {
        if let Err(e) = self.release_resources_internal() {
            error!("Error releasing resources: {:?}", e);
        }
    }
}

// 实现 Drop trait 以确保资源被释放
impl Drop for VoskASR {
    fn drop(&mut self) {
        self.drop_impl();
    }
}

// 创建一个新的结构体以支持异步流
pub struct VoskStream {
    receiver: tokio_mpsc::Receiver<String>,
    vosk_instance: Py<PyAny>,
    is_active: bool,
}

// 为 VoskStream 实现安全的清理方法
impl VoskStream {
    fn cleanup_resources(&mut self) {
        if !self.is_active {
            return;
        }

        debug!("开始清理VoskStream资源");
        self.is_active = false;

        // 使用AssertUnwindSafe包裹可能不安全的操作
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            Python::with_gil(|py| {
                match self.vosk_instance.bind(py).call_method0("stop_recognition") {
                    Ok(_) => debug!("成功调用stop_recognition方法"),
                    Err(e) => error!("调用stop_recognition出错: {:?}", e),
                }
            })
        }));

        if let Err(e) = result {
            error!("清理资源时发生panic: {:?}", e);
        }

        debug!("VoskStream资源清理完成");
    }
}

// 修改 Drop 实现，确保安全地释放资源
impl Drop for VoskStream {
    fn drop(&mut self) {
        debug!("VoskStream被释放");
        self.cleanup_resources();
    }
}

// 修改 Stream 实现中处理流结束的代码
impl Stream for VoskStream {
    type Item = Result<AsrStatus, VoskPythonError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if !self.is_active {
            return Poll::Ready(None);
        }

        // 使用 tokio 的 Receiver 进行非阻塞轮询
        match Pin::new(&mut self.receiver).poll_recv(cx) {
            Poll::Ready(Some(text)) => {
                // 检查特殊标记
                if text == "[timeout reached]" {
                    info!("Timeout reached, stopping recognition.");
                    self.cleanup_resources(); // 使用统一的清理方法
                    Poll::Ready(Some(Ok(AsrStatus::Timeout)))
                } else if text == "[silence detected]" {
                    // 检测到静默时，只记录日志但不停止流
                    info!("Silence detected, but continuing to listen.");
                    Poll::Ready(Some(Ok(AsrStatus::SilenceDetected)))
                } else if text.starts_with("[error]") {
                    error!("Error detected: {}", &text[8..]);
                    self.cleanup_resources(); // 使用统一的清理方法
                    Poll::Ready(Some(Err(VoskPythonError(text[8..].to_string()))))
                } else {
                    debug!("Received text: {}", text);
                    Poll::Ready(Some(Ok(AsrStatus::PartialResult(text))))
                }
            }
            Poll::Ready(None) => {
                debug!("Stream closed, stopping recognition.");
                self.cleanup_resources(); // 使用统一的清理方法
                Poll::Ready(None)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

// 写一个测试用来验证 VoskASR 的功能
#[cfg(test)]
mod tests {
    use tokio_stream::StreamExt;

    use super::*;

    #[tokio::test]
    async fn test_vosk_asr() {
        let mut vosk_asr = VoskASR::new(AsrConfig {
            model_path: Some("C:/Users/18511/Documents/AppCode/Rust/chat_box/src-tauri/model/vosk-model-small-cn-0.22".to_string()),
            ..Default::default()
        }).unwrap();
        vosk_asr.start_recording().unwrap();
        vosk_asr.stop_recording().unwrap();
        vosk_asr.release_resources().unwrap();
    }

    #[tokio::test]
    async fn test_vosk_stream() {
        let mut vosk_asr = VoskASR::new(AsrConfig {
            model_path: Some("C:/Users/18511/Documents/AppCode/Rust/chat_box/src-tauri/model/vosk-model-small-cn-0.22".to_string()),
            ..Default::default()
        }).unwrap();
        let mut stream = vosk_asr.listen_and_transcribe(None, None).unwrap();

        while let Some(result) = stream.next().await {
            match result {
                Ok(status) => println!("Recognized: {:?}", status),
                Err(e) => println!("Error: {}", e),
            }
        }
    }
}
