pub mod audio_capture;
pub mod baidu;
pub mod traits;
pub mod vosk_python;
// pub mod vosk;

// Re-export commonly used types
pub use audio_capture::{AudioCapture, AudioCaptureError, AudioConfig, AudioProcessor};
pub use baidu::{BaiduASR, BaiduAsrConfig, BaiduAsrError};
pub use traits::{AsrConfig, AsrProvider, AsrStatus};
pub use vosk_python::{VoskASR, VoskPythonError};
