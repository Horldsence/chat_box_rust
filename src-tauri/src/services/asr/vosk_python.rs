use futures::Stream;
use log::{debug, error, info};
use pyo3::prelude::*;
use pyo3::types::PyList;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use tokio::sync::mpsc as tokio_mpsc;

#[derive(Debug)]
pub struct VoskASR {
    model_path: Option<String>,
    instance: Option<Py<PyAny>>, // 只有在需要时才初始化
}

#[allow(dead_code)]
impl VoskASR {
    pub fn new(model_path: Option<&str>) -> PyResult<Self> {
        debug!(
            "Creating VoskASR with path {}",
            model_path.unwrap_or("None")
        );

        // 只保存模型路径，不立即加载模型
        let model_path = model_path.map(String::from);

        Ok(Self {
            model_path,
            instance: None,
        })
    }

    // 初始化模型和Python实例（但不启动麦克风）
    fn ensure_initialized(&mut self) -> PyResult<()> {
        if self.instance.is_some() {
            return Ok(());
        }

        #[allow(deprecated)]
        Python::with_gil(|py| {
            let sys = py.import("sys")?;
            let path = sys.getattr("path")?.downcast_into::<PyList>()?;

            // 虚拟环境路径
            #[cfg(windows)]
            path.insert(0, r".venv\Lib\site-packages")?;
            #[cfg(not(windows))]
            path.insert(0, "chat_box/lib/python3.12/site-packages")?;

            path.insert(0, "src/python")?;

            debug!("Python path: {:?}", path);

            // 导入Python模块
            let module = py.import("voskASR")?;
            let class = module.getattr("VoskRecognizer")?;

            debug!("VoskASR class: {:?}", class);

            // 创建实例
            let instance: Py<PyAny> = match &self.model_path {
                Some(p) => class.call1((p,))?.into_py(py),
                None => class.call0()?.into_py(py),
            };

            debug!("VoskASR instance created: {:?}", instance);
            self.instance = Some(instance);

            Ok(())
        })
    }

    // 显式开始录音
    pub fn start_recording(&mut self) -> PyResult<()> {
        self.ensure_initialized()?;

        Python::with_gil(|py| {
            if let Some(instance) = &self.instance {
                instance.bind(py).call_method0("start_stream")?;
                debug!("Started audio stream");
            }
            Ok(())
        })
    }

    // 显式停止录音
    pub fn stop_recording(&mut self) -> PyResult<()> {
        if let Some(instance) = &self.instance {
            Python::with_gil(|py| {
                instance.bind(py).call_method0("stop_stream")?;
                debug!("Stopped audio stream");
                Ok(())
            })
        } else {
            Ok(())
        }
    }

    // 完全释放资源
    pub fn release_resources(&mut self) {
        if let Some(_instance) = &self.instance {
            self.stop_recording().ok();
            self.instance = None;
            debug!("Released VoskASR resources");
        }
    }

    #[allow(deprecated)]
    pub async fn listen_and_transcribe(&mut self, timeout_ms: Option<u64>) -> PyResult<VoskStream> {
        self.ensure_initialized()?;
        self.start_recording()?; // 确保在开始转录前启动录音

        let (sender, receiver) = tokio_mpsc::channel(32);

        let vosk_instance = Python::with_gil(|py| {
            let instance = self.instance.as_ref().unwrap().clone_ref(py);

            // Start recognition with the timeout parameter and set end_on_silence=false
            let timeout_py = match timeout_ms {
                Some(ms) => ms.into_py(py),
                None => py.None(),
            };

            // 传递第二个参数false，表示检测到静默时不要自动结束录音
            match instance.call_method1(py, "start_recognition", (timeout_py, false)) {
                Ok(_) => debug!("语音识别启动成功，超时设置: {:?}ms", timeout_ms),
                Err(e) => error!("启动语音识别失败: {:?}", e),
            }

            Ok::<_, PyErr>(instance)
        })?;

        // 创建一个任务来轮询Python队列中的结果
        let vosk_instance_clone = Python::with_gil(|py| vosk_instance.clone_ref(py));
        let sender_clone = sender.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(100));

            loop {
                interval.tick().await;

                let result = Python::with_gil(|py| {
                    match vosk_instance_clone.call_method0(py, "get_result") {
                        Ok(result) => {
                            if result.is_none(py) {
                                return Ok(None);
                            }

                            let text: String = match result.extract(py) {
                                Ok(text) => text,
                                Err(e) => {
                                    error!("提取结果失败: {:?}", e);
                                    return Ok(None);
                                }
                            };
                            if text == "[end]" {
                                debug!("收到[end]标记，结束轮询");
                                return Ok(None);
                            }
                            Ok(Some(text))
                        }
                        Err(e) => Err(format!("获取结果失败: {:?}", e)),
                    }
                });

                match result {
                    Ok(Some(text)) => {
                        if sender_clone.send(text).await.is_err() {
                            debug!("接收方已关闭，停止轮询");
                            break;
                        }
                    }
                    Ok(None) => break,
                    Err(e) => {
                        let _ = sender_clone.send(format!("[error] {}", e)).await;
                        break;
                    }
                }
            }
        });

        Ok(VoskStream {
            receiver,
            vosk_instance,
            is_active: true,
        })
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

        // 在GIL保护下获取Python对象引用
        let instance = Python::with_gil(|py| self.vosk_instance.clone_ref(py));

        // 使用AssertUnwindSafe包裹可能不安全的操作
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            Python::with_gil(|py| {
                match instance.call_method0(py, "stop_recognition") {
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
    type Item = Result<String, String>;

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
                    Poll::Ready(Some(Ok(text)))
                } else if text == "[silence detected]" {
                    // 检测到静默时，只记录日志但不停止流
                    info!("Silence detected, but continuing to listen.");
                    Poll::Ready(Some(Ok(text)))
                } else if text.starts_with("[error]") {
                    error!("Error detected: {}", &text[8..]);
                    self.cleanup_resources(); // 使用统一的清理方法
                    Poll::Ready(Some(Err(text[8..].to_string())))
                } else {
                    debug!("Received text: {}", text);
                    Poll::Ready(Some(Ok(text)))
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
        let mut vosk_asr = VoskASR::new(Some("C:/Users/18511/Documents/AppCode/Rust/chat_box/src-tauri/model/vosk-model-small-cn-0.22")).unwrap();
        vosk_asr.start_recording().unwrap();
        vosk_asr.stop_recording().unwrap();
        vosk_asr.release_resources();
    }

    #[tokio::test]
    async fn test_vosk_stream() {
        let mut vosk_asr = VoskASR::new(Some("C:/Users/18511/Documents/AppCode/Rust/chat_box/src-tauri/model/vosk-model-small-cn-0.22")).unwrap();
        let mut stream = vosk_asr.listen_and_transcribe(None).await.unwrap();

        while let Some(result) = stream.next().await {
            match result {
                Ok(text) => println!("Recognized: {}", text),
                Err(e) => println!("Error: {}", e),
            }
        }
    }
}
