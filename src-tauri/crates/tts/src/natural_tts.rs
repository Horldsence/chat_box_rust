use pyo3::prelude::*;
use pyo3::types::PyList;

use log::{info, debug};

#[derive(Debug)]
pub struct TTSHandler {
    instance: Py<PyAny>,
}

impl TTSHandler {
    pub fn new() -> PyResult<Self> {
        info!("Initializing TTSHandler");
        Python::with_gil(|py| {
            let sys = py.import("sys")?;
            let path = sys.getattr("path")?.downcast_into::<PyList>()?;

            // 虚拟环境路径
            #[cfg(windows)]
            path.insert(0, r".venv\Lib\site-packages")?;
            #[cfg(not(windows))]
            path.insert(0, "voice-assitant/lib/python3.12/site-packages")?;

            path.insert(0, "src/python")?;
            debug!("Python path: {:?}", path);

            // 实例创建部分
            let edgetts = py.import("edgetts")?;
            let tts_class = edgetts.getattr("TextToSpeech")?;

            // 显式类型转换
            let instance: Py<PyAny> = tts_class.call0()?.unbind().into();

            Ok(Self { instance })
        })
    }

    pub fn with_voice(voice: &str) -> PyResult<Self> {
        Python::with_gil(|py| {
            let sys = py.import("sys")?;
            let path = sys.getattr("path")?.downcast_into::<PyList>()?;
            path.insert(0, ".")?;

            let edgetts = py.import("edgetts")?;
            let tts_class = edgetts.getattr("TextToSpeech")?;

            // 使用元组传递参数
            let instance: Py<PyAny> = tts_class
                .call1((voice,))?
                .unbind()
                .into();

            Ok(Self { instance })
        })
    }

    pub fn convert(&self, text: &str, output_path: &str) -> PyResult<()> {
        Python::with_gil(|py| {
            let instance = self.instance.bind(py);
            instance.call_method1(
                "text_to_speech_sync",
                (text, output_path)
            )?;
            Ok(())
        })
    }
}

// 单元测试模块
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_conversion() {
        let tts = TTSHandler::new().unwrap();
        let result = tts.convert(
            "测试文本转换",
            "test_output.mp3"
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_custom_voice() {
        let tts = TTSHandler::with_voice("zh-CN-YunyangNeural").unwrap();
        let result = tts.convert(
            "自定义语音测试",
            "custom_voice.mp3"
        );

        assert!(result.is_ok());
    }
}