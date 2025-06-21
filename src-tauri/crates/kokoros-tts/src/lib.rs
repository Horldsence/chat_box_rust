use anyhow::Result;
use futures::{Stream, StreamExt};
use kokoro_tts::{KokoroTts, Voice};
use std::pin::Pin;
use std::sync::Arc;

/// Main TTS engine struct, wrapping KokoroTts and providing batch/streaming APIs.
pub struct TtsEngine {
    tts: Arc<KokoroTts>,
    default_voice: Voice,
}

impl TtsEngine {
    /// Create a new TtsEngine with the given model and voice files, and a default voice.
    pub async fn new<M: AsRef<str>, V: AsRef<str>>(
        model_path: M,
        voices_path: V,
        default_voice: Voice,
    ) -> Result<Self> {
        let tts = KokoroTts::new(model_path.as_ref(), voices_path.as_ref()).await?;
        Ok(Self {
            tts: Arc::new(tts),
            default_voice,
        })
    }

    /// Synthesize a batch of texts (Vec<String>) into a Vec of audio buffers (Vec<Vec<f32>>).
    /// Each buffer corresponds to the input text at the same index.
    pub async fn synthesize_batch(
        &self,
        texts: Vec<String>,
        voice: Option<Voice>,
    ) -> Result<Vec<Vec<f32>>> {
        let voice = voice.unwrap_or(self.default_voice);
        let (mut sink, mut stream) = self.tts.stream(voice);

        // Queue all texts for synthesis
        for text in &texts {
            sink.synth(text).await?;
        }

        // Collect audio buffers as they are produced
        let mut results = Vec::with_capacity(texts.len());
        for _ in 0..texts.len() {
            if let Some((audio, _took)) = stream.next().await {
                results.push(audio);
            }
        }
        Ok(results)
    }

    /// Synthesize a stream of texts, returning a stream of audio buffers as they are ready.
    /// The returned stream yields (audio_buffer, synth_time_ms).
    pub fn synthesize_stream<'a, S>(
        &'a self,
        texts: S,
        voice: Option<Voice>,
    ) -> Pin<Box<dyn Stream<Item = Result<(Vec<f32>, u128)>> + Send + 'a>>
    where
        S: Stream<Item = String> + Send + 'a,
    {
        let voice = voice.unwrap_or(self.default_voice);
        let tts = self.tts.clone();

        Box::pin(async_stream::try_stream! {
            let (mut sink, mut stream) = tts.stream(voice);

            // Spawn a task to feed the sink with texts as they arrive
            let mut input_stream = Box::pin(texts);
            tokio::spawn(async move {
                while let Some(text) = input_stream.next().await {
                    // Ignore errors for now; could be handled more gracefully
                    let _ = sink.synth(&text).await;
                }
            });

            // Yield audio buffers as they are produced
            while let Some((audio, took)) = stream.next().await {
                yield (audio, took.as_millis());
            }
        })
    }
}

// Re-export Voice for convenience
pub use kokoro_tts::Voice;
