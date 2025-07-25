use anyhow::Result;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use tokio::runtime::Runtime;
use std::sync::Arc;
use once_cell::sync::Lazy;
use log::{info, debug};

pub struct KokoroTTS {
    py_tts: PyObject,
}

impl KokoroTTS {
    pub async fn new() -> Result<Self> {
        let py_tts = tokio::task::spawn_blocking(|| {
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
                let kokoro_module = py.import("kokoroTTS")?;
                let kokoro_class = kokoro_module.getattr("KokoroTTS")?;
                let instance = kokoro_class.call0()?;
                Ok::<PyObject, PyErr>(instance.into())
            })
        })
        .await??;

        Ok(Self { py_tts })
    }

    pub async fn generate_speech(&self, text: &str, voice: Option<&str>) -> Result<Vec<(i32, Vec<f32>)>> {
        let py_tts = tokio::task::spawn_blocking(|| {
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
                let kokoro_module = py.import("kokoroTTS")?;
                let kokoro_class = kokoro_module.getattr("KokoroTTS")?;
                let instance = kokoro_class.call0()?;
                Ok::<PyObject, PyErr>(instance.into())
            })
        })
        .await??;
        let text = text.to_string();
        let voice = voice.map(|v| v.to_string());

        let result = tokio::task::spawn_blocking(move || {
            Python::with_gil(|py| {
                let generate_speech = py_tts.getattr(py, "generate_speech")?;
                let args = PyDict::new(py);
                args.set_item("text", text)?;
                if let Some(v) = voice {
                    args.set_item("voice", v)?;
                }

                let generator = generate_speech.call(py, (), Some(&args))?;
                let mut results = Vec::new();

                let iter = generator.getattr(py, "__iter__")?.call0(py)?;
                let next = iter.getattr(py, "__next__")?;

                loop {
                    match next.call0(py) {
                        Ok(item) => {
                            let tuple: (i32, PyObject, PyObject, PyObject) = item.extract(py)?;
                            let audio: Vec<f32> = tuple.3.extract(py)?;
                            results.push((tuple.0, audio));
                        }
                        Err(e) if e.is_instance_of::<pyo3::exceptions::PyStopIteration>(py) => {
                            break;
                        }
                        Err(e) => return Err(e),
                    }
                }

                Ok::<Vec<(i32, Vec<f32>)>, PyErr>(results)
            })
        })
        .await??;

        Ok(result)
    }

    pub async fn save_audio(&self, audio: &[f32], filename: &str, sample_rate: Option<i32>) -> Result<()> {
        let py_tts = tokio::task::spawn_blocking(|| {
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
                let kokoro_module = py.import("kokoroTTS")?;
                let kokoro_class = kokoro_module.getattr("KokoroTTS")?;
                let instance = kokoro_class.call0()?;
                Ok::<PyObject, PyErr>(instance.into())
            })
        })
        .await??;
        let audio = audio.to_vec();
        let filename = filename.to_string();
        let sample_rate = sample_rate.unwrap_or(24000);

        tokio::task::spawn_blocking(move || {
            Python::with_gil(|py| {
                let save_audio = py_tts.getattr(py, "save_audio")?;
                let args = PyDict::new(py);
                args.set_item("audio", audio)?;
                args.set_item("filename", filename)?;
                args.set_item("sample_rate", sample_rate)?;

                save_audio.call(py, (), Some(&args))?;
                Ok::<(), PyErr>(())
            })
        })
        .await??;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_kokoro_tts() -> Result<()> {
        pyo3::prepare_freethreaded_python();
        let tts = KokoroTTS::new().await?;
        let text = "Hello, this is a test.";
        let results = tts.generate_speech(text, Some("af_heart")).await?;

        for (i, audio) in results {
            tts.save_audio(&audio, &format!("test_{}.wav", i), None).await?;
        }

        Ok(())
    }
}