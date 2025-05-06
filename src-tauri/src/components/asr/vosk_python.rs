use log::debug;
use pyo3::prelude::*;
use pyo3::types::PyList;
use std::sync::mpsc;
use std::thread;

#[derive(Debug)]
pub struct VoskASR {
    receiver: mpsc::Receiver<String>,
    _instance: Py<PyAny>,
}

impl VoskASR {
    pub fn new(model_path: Option<&str>) -> PyResult<Self> {
        debug!("Initializing VoskASR with path {}", model_path.unwrap_or("None"));
        Python::with_gil(|py| {
            let sys = py.import("sys")?;
            let path = sys.getattr("path")?.downcast_into::<PyList>()?;

            // 虚拟环境路径
            #[cfg(windows)]
            path.insert(0, r".venv\Lib\site-packages")?;
            #[cfg(not(windows))]
            path.insert(0, "voice-assitent/lib/python3.12/site-packages")?;

            path.insert(0, "src/python")?;

            debug!("Python path: {:?}", path);

            // 导入Python模块
            let module = py.import("voskASR")?;
            let class = module.getattr("VoskRecognizer")?;

            debug!("VoskASR class: {:?}", class);
            // 创建实例（这里需要显式处理Result）
            let instance: Py<PyAny> = match model_path {
                Some(p) => class.call1((p,))?.into_py(py),
                None => class.call0()?.into_py(py),
            };

            debug!("VoskASR instance created: {:?}", instance);
            // 启动音频流（需要绑定Python实例）
            instance.bind(py).call_method0("start_stream")?;

            let (sender, receiver) = mpsc::channel();
            let thread_instance = instance.clone_ref(py); // 正确使用clone_ref

            thread::spawn(move || {
                Python::with_gil(|py| {
                    let generator = thread_instance
                        .bind(py)
                        .call_method0("recognize_loop")
                        .expect("Failed to get generator");

                    loop {
                        let next_result = match generator.call_method0("__next__") {
                            Ok(res) => res,
                            Err(e) if e.is_instance_of::<pyo3::exceptions::PyStopIteration>(py) => break,
                            Err(e) => panic!("Generator error: {:?}", e),
                        };

                        let text: String = next_result
                            .extract()
                            .expect("Failed to parse result");
                        sender.send(text).expect("Channel disconnected");
                    }
                });
            });

            Ok(Self { receiver, _instance: instance })
        })
    }

    pub fn try_get(&self) -> Option<String> {
        self.receiver.try_recv().ok()
    }

    /// 阻塞式获取结果
    pub fn get(&self) -> String {
        self.receiver.recv().unwrap()
    }
}

impl Drop for VoskASR {
    fn drop(&mut self) {
        Python::with_gil(|py| {
            self._instance.call_method0(py, "stop_stream")
                .expect("Failed to stop stream");
        });
    }
}
