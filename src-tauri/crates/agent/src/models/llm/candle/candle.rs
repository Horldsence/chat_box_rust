#[cfg(feature = "mkl")]
extern crate intel_mkl_src;

#[cfg(feature = "accelerate")]
extern crate accelerate_src;

use anyhow::{Error as E, Result};
use clap::Parser;


use candle_transformers::models::qwen2::{Config as ConfigBase, ModelForCausalLM as ModelBase};
use candle_transformers::models::qwen2_moe::{Config as ConfigMoe, Model as ModelMoe};
// use candle_transformers::models::qwen3::{Config as Config3, ModelForCausalLM as Model3};

use candle_core::{DType, Device, Tensor};
use candle_examples::token_output_stream::TokenOutputStream;
use candle_nn::VarBuilder;
use candle_transformers::generation::LogitsProcessor;
use hf_hub::{api::sync::Api, Repo, RepoType};
use tokenizers::Tokenizer;

#[derive(Clone)]
pub enum Model {
    Base(ModelBase),
    Moe(ModelMoe),
    // Base3(Model3),
}

impl Model {
    fn forward(&mut self, xs: &Tensor, s: usize) -> candle_core::Result<Tensor> {
        match self {
            Self::Moe(ref mut m) => m.forward(xs, s),
            Self::Base(ref mut m) => m.forward(xs, s),
            // Self::Base3(ref mut m) => m.forward(xs, s),
        }
    }
}

pub struct TextGeneration {
    model: Model,
    device: Device,
    tokenizer: TokenOutputStream,
    logits_processor: LogitsProcessor,
    repeat_penalty: f32,
    repeat_last_n: usize,
}

impl TextGeneration {
    #[allow(clippy::too_many_arguments)]
    fn new(
        model: Model,
        tokenizer: Tokenizer,
        seed: u64,
        temp: Option<f64>,
        top_p: Option<f64>,
        repeat_penalty: f32,
        repeat_last_n: usize,
        device: &Device,
    ) -> Self {
        let logits_processor = LogitsProcessor::new(seed, temp, top_p);
        Self {
            model,
            tokenizer: TokenOutputStream::new(tokenizer),
            logits_processor,
            repeat_penalty,
            repeat_last_n,
            device: device.clone(),
        }
    }

    fn run(&mut self, prompt: &str, sample_len: usize) -> Result<()> {
        use std::io::Write;
        self.tokenizer.clear();
        let mut tokens = self
            .tokenizer
            .tokenizer()
            .encode(prompt, true)
            .map_err(E::msg)?
            .get_ids()
            .to_vec();
        for &t in tokens.iter() {
            if let Some(t) = self.tokenizer.next_token(t)? {
                print!("{t}")
            }
        }
        std::io::stdout().flush()?;

        let mut generated_tokens = 0usize;
        let eos_token = match self.tokenizer.get_token("<|endoftext|>") {
            Some(token) => token,
            None => anyhow::bail!("cannot find the <|endoftext|> token"),
        };
        let eos_token2 = match self.tokenizer.get_token("<|im_end|>") {
            Some(token) => token,
            None => anyhow::bail!("cannot find the <|im_end|> token"),
        };
        let start_gen = std::time::Instant::now();
        for index in 0..sample_len {
            let context_size = if index > 0 { 1 } else { tokens.len() };
            let start_pos = tokens.len().saturating_sub(context_size);
            let ctxt = &tokens[start_pos..];
            let input = Tensor::new(ctxt, &self.device)?.unsqueeze(0)?;
            let logits = self.model.forward(&input, start_pos)?;
            let logits = logits.squeeze(0)?.squeeze(0)?.to_dtype(DType::F32)?;
            let logits = if self.repeat_penalty == 1. {
                logits
            } else {
                let start_at = tokens.len().saturating_sub(self.repeat_last_n);
                candle_transformers::utils::apply_repeat_penalty(
                    &logits,
                    self.repeat_penalty,
                    &tokens[start_at..],
                )?
            };

            let next_token = self.logits_processor.sample(&logits)?;
            tokens.push(next_token);
            generated_tokens += 1;
            if next_token == eos_token || next_token == eos_token2 {
                break;
            }
            if let Some(t) = self.tokenizer.next_token(next_token)? {
                print!("{t}");
                std::io::stdout().flush()?;
            }
        }
        let dt = start_gen.elapsed();
        if let Some(rest) = self.tokenizer.decode_rest().map_err(E::msg)? {
            print!("{rest}");
        }
        std::io::stdout().flush()?;
        println!(
            "\n{generated_tokens} tokens generated ({:.2} token/s)",
            generated_tokens as f64 / dt.as_secs_f64(),
        );
        Ok(())
    }

    // Modify TextGeneration to support streaming via callback
    fn run_with_callback<F>(&mut self, prompt: &str, sample_len: usize, mut callback: F) -> Result<()>
    where
        F: FnMut(&str) -> Result<()>,
    {
        self.tokenizer.clear();
        let mut tokens = self
            .tokenizer
            .tokenizer()
            .encode(prompt, true)
            .map_err(E::msg)?
            .get_ids()
            .to_vec();

        // Process prompt tokens
        for &t in tokens.iter() {
            if let Some(t) = self.tokenizer.next_token(t)? {
                callback(&t)?;
            }
        }

        let mut generated_tokens = 0usize;
        let eos_token = match self.tokenizer.get_token("<|endoftext|>") {
            Some(token) => token,
            None => anyhow::bail!("cannot find the <|endoftext|> token"),
        };
        let eos_token2 = match self.tokenizer.get_token("<|im_end|>") {
            Some(token) => token,
            None => anyhow::bail!("cannot find the <|im_end|> token"),
        };

        let start_gen = std::time::Instant::now();
        for index in 0..sample_len {
            let context_size = if index > 0 { 1 } else { tokens.len() };
            let start_pos = tokens.len().saturating_sub(context_size);
            let ctxt = &tokens[start_pos..];
            let input = Tensor::new(ctxt, &self.device)?.unsqueeze(0)?;
            let logits = self.model.forward(&input, start_pos)?;
            let logits = logits.squeeze(0)?.squeeze(0)?.to_dtype(DType::F32)?;
            let logits = if self.repeat_penalty == 1. {
                logits
            } else {
                let start_at = tokens.len().saturating_sub(self.repeat_last_n);
                candle_transformers::utils::apply_repeat_penalty(
                    &logits,
                    self.repeat_penalty,
                    &tokens[start_at..],
                )?
            };

            let next_token = self.logits_processor.sample(&logits)?;
            tokens.push(next_token);
            generated_tokens += 1;

            if next_token == eos_token || next_token == eos_token2 {
                break;
            }

            if let Some(t) = self.tokenizer.next_token(next_token)? {
                callback(&t)?;
            }
        }

        if let Some(rest) = self.tokenizer.decode_rest().map_err(E::msg)? {
            callback(&rest)?;
        }

        Ok(())
    }
}

#[derive(Clone, Copy, Debug, clap::ValueEnum, PartialEq, Eq)]
pub enum WhichModel {
    #[value(name = "0.5b")]
    W0_5b,
    #[value(name = "1.8b")]
    W1_8b,
    #[value(name = "4b")]
    W4b,
    #[value(name = "7b")]
    W7b,
    #[value(name = "14b")]
    W14b,
    #[value(name = "72b")]
    W72b,
    #[value(name = "moe-a2.7b")]
    MoeA27b,
    #[value(name = "2-0.5b")]
    W2_0_5b,
    #[value(name = "2-1.5b")]
    W2_1_5b,
    #[value(name = "2-7b")]
    W2_7b,
    #[value(name = "2-72b")]
    W2_72b,
    #[value(name = "3-0.6b")]
    W3_0_6b,
    #[value(name = "3-1.7b")]
    W3_1_7b,
    #[value(name = "3-4b")]
    W3_4b,
    #[value(name = "3-8b")]
    W3_8b,
}

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct QwenInferenceParams {
    /// Run on CPU rather than on GPU.
    #[arg(long)]
    pub cpu: bool,

    /// Enable tracing (generates a trace-timestamp.json file).
    #[arg(long)]
    pub tracing: bool,

    #[arg(long)]
    pub use_flash_attn: bool,

    #[arg(long)]
    pub prompt: String,

    /// The temperature used to generate samples.
    #[arg(long)]
    pub temperature: Option<f64>,

    /// Nucleus sampling probability cutoff.
    #[arg(long)]
    pub top_p: Option<f64>,

    /// The seed to use when generating random samples.
    #[arg(long, default_value_t = 299792458)]
    pub seed: u64,

    /// The length of the sample to generate (in tokens).
    #[arg(long, short = 'n', default_value_t = 10000)]
    pub sample_len: usize,

    #[arg(long)]
    pub model_id: Option<String>,

    #[arg(long, default_value = "main")]
    pub revision: String,

    #[arg(long)]
    pub tokenizer_file: Option<String>,

    #[arg(long)]
    pub weight_files: Option<String>,

    /// Penalty to be applied for repeating tokens, 1. means no penalty.
    #[arg(long, default_value_t = 1.1)]
    pub repeat_penalty: f32,

    /// The context size to consider for the repeat penalty.
    #[arg(long, default_value_t = 64)]
    pub repeat_last_n: usize,

    #[arg(long, default_value = "0.5b")]
    pub model: WhichModel,
}

// Implement default for QwenInferenceParams
impl Default for QwenInferenceParams {
    fn default() -> Self {
        Self {
            cpu: true,
            tracing: false,
            prompt: "Hello".to_string(),
            model: WhichModel::W0_5b,
            model_id: None,
            revision: "main".to_string(),
            tokenizer_file: None,
            weight_files: None,
            seed: 299792458,
            temperature: Some(0.7),
            top_p: Some(0.9),
            repeat_penalty: 1.1,
            repeat_last_n: 64,
            use_flash_attn: false,
            sample_len: 2000,
        }
    }
}

pub struct QwenCandleGenerator {
    model: Model,
    tokenizer: Tokenizer,
    params: QwenInferenceParams,
}

impl QwenCandleGenerator {
    pub fn new(params: QwenInferenceParams) -> Result<Self> {
        use tracing_chrome::ChromeLayerBuilder;
        use tracing_subscriber::prelude::*;

        let args = params.clone();
        let _guard = if args.tracing {
            let (chrome_layer, guard) = ChromeLayerBuilder::new().build();
            tracing_subscriber::registry().with(chrome_layer).init();
            Some(guard)
        } else {
            None
        };
        println!(
            "avx: {}, neon: {}, simd128: {}, f16c: {}",
            candle_core::utils::with_avx(),
            candle_core::utils::with_neon(),
            candle_core::utils::with_simd128(),
            candle_core::utils::with_f16c()
        );
        println!(
            "temp: {:.2} repeat-penalty: {:.2} repeat-last-n: {}",
            args.temperature.unwrap_or(0.),
            args.repeat_penalty,
            args.repeat_last_n
        );

        let start = std::time::Instant::now();
        let api = Api::new()?;
        let model_id = match args.model_id {
            Some(model_id) => model_id,
            None => {
                let (version, size) = match args.model {
                    WhichModel::W2_0_5b => ("2", "0.5B"),
                    WhichModel::W2_1_5b => ("2", "1.5B"),
                    WhichModel::W2_7b => ("2", "7B"),
                    WhichModel::W2_72b => ("2", "72B"),
                    WhichModel::W0_5b => ("1.5", "0.5B"),
                    WhichModel::W1_8b => ("1.5", "1.8B"),
                    WhichModel::W4b => ("1.5", "4B"),
                    WhichModel::W7b => ("1.5", "7B"),
                    WhichModel::W14b => ("1.5", "14B"),
                    WhichModel::W72b => ("1.5", "72B"),
                    WhichModel::MoeA27b => ("1.5", "MoE-A2.7B"),
                    WhichModel::W3_0_6b => ("3", "0.6B"),
                    WhichModel::W3_1_7b => ("3", "1.7B"),
                    WhichModel::W3_4b => ("3", "4B"),
                    WhichModel::W3_8b => ("3", "8B"),
                };
                format!("Qwen/Qwen{version}-{size}")
            }
        };
        let repo = api.repo(Repo::with_revision(
            model_id,
            RepoType::Model,
            args.revision,
        ));
        let tokenizer_filename = match args.tokenizer_file {
            Some(file) => std::path::PathBuf::from(file),
            None => repo.get("tokenizer.json")?,
        };
        let filenames = match args.weight_files {
            Some(files) => files
                .split(',')
                .map(std::path::PathBuf::from)
                .collect::<Vec<_>>(),
            None => match args.model {
                WhichModel::W0_5b
                | WhichModel::W2_0_5b
                | WhichModel::W2_1_5b
                | WhichModel::W1_8b
                | WhichModel::W3_0_6b => {
                    vec![repo.get("model.safetensors")?]
                }
                WhichModel::W4b
                | WhichModel::W7b
                | WhichModel::W2_7b
                | WhichModel::W14b
                | WhichModel::W72b
                | WhichModel::W2_72b
                | WhichModel::MoeA27b
                | WhichModel::W3_1_7b
                | WhichModel::W3_4b
                | WhichModel::W3_8b => {
                    candle_examples::hub_load_safetensors(&repo, "model.safetensors.index.json")?
                }
            },
        };
        println!("retrieved the files in {:?}", start.elapsed());
        let tokenizer = Tokenizer::from_file(tokenizer_filename).map_err(E::msg)?;

        let start = std::time::Instant::now();
        let config_file = repo.get("config.json")?;
        let device = candle_examples::device(args.cpu)?;
        let dtype = if device.is_cuda() {
            DType::BF16
        } else {
            DType::F32
        };
        let vb = unsafe { VarBuilder::from_mmaped_safetensors(&filenames, dtype, &device)? };
        let model = match args.model {
            WhichModel::MoeA27b => {
                let config: ConfigMoe = serde_json::from_slice(&std::fs::read(config_file)?)?;
                Model::Moe(ModelMoe::new(&config, vb)?)
            }
            // WhichModel::W3_0_6b | WhichModel::W3_1_7b | WhichModel::W3_4b | WhichModel::W3_8b => {
            //     let config: Config3 = serde_json::from_slice(&std::fs::read(config_file)?)?;
            //     Model::Base3(Model3::new(&config, vb)?)
            // }
            _ => {
                let config: ConfigBase = serde_json::from_slice(&std::fs::read(config_file)?)?;
                Model::Base(ModelBase::new(&config, vb)?)
            }
        };
        println!("loaded the model in {:?}", start.elapsed());

        Ok(Self {
            model,
            tokenizer,
            params,
        })
    }

    pub fn generate_string(&mut self, prompt: &str, sample_len: usize) -> Result<String> {
        let device = candle_examples::device(self.params.cpu)?;
        let mut pipeline = TextGeneration::new(
            self.model.clone(),
            self.tokenizer.clone(),
            self.params.seed,
            self.params.temperature,
            self.params.top_p,
            self.params.repeat_penalty,
            self.params.repeat_last_n,
            &device,
        );

        let mut output = String::new();
        pipeline.run_with_callback(prompt, sample_len, |chunk| {
            output.push_str(chunk);
            Ok(())
        })?;
        
        Ok(output)
    }
    pub fn generate_tokens(&mut self, prompt: &str, sample_len: usize) -> Result<Vec<u32>> {
        let device = candle_examples::device(self.params.cpu)?;
        let mut pipeline = TextGeneration::new(
            self.model.clone(),
            self.tokenizer.clone(),
            self.params.seed,
            self.params.temperature,
            self.params.top_p,
            self.params.repeat_penalty,
            self.params.repeat_last_n,
            &device,
        );

        pipeline.tokenizer.clear();
        let mut tokens = pipeline.tokenizer
            .tokenizer()
            .encode(prompt, true)
            .map_err(E::msg)?
            .get_ids()
            .to_vec();
        
        let initial_len = tokens.len();
        let eos_token = pipeline.tokenizer.get_token("<|endoftext|>")
            .ok_or_else(|| E::msg("cannot find the <|endoftext|> token"))?;
        let eos_token2 = pipeline.tokenizer.get_token("<|im_end|>")
            .ok_or_else(|| E::msg("cannot find the <|im_end|> token"))?;
        
        for index in 0..sample_len {
            let context_size = if index > 0 { 1 } else { tokens.len() };
            let start_pos = tokens.len().saturating_sub(context_size);
            let ctxt = &tokens[start_pos..];
            let input = Tensor::new(ctxt, &pipeline.device)?.unsqueeze(0)?;
            let logits = pipeline.model.forward(&input, start_pos)?;
            let logits = logits.squeeze(0)?.squeeze(0)?.to_dtype(DType::F32)?;
            let logits = if pipeline.repeat_penalty == 1. {
                logits
            } else {
                let start_at = tokens.len().saturating_sub(pipeline.repeat_last_n);
                candle_transformers::utils::apply_repeat_penalty(
                    &logits,
                    pipeline.repeat_penalty,
                    &tokens[start_at..],
                )?
            };

            let next_token = pipeline.logits_processor.sample(&logits)?;
            tokens.push(next_token);
            
            if next_token == eos_token || next_token == eos_token2 {
                break;
            }
        }
        
        // Return only the newly generated tokens
        Ok(tokens[initial_len..].to_vec())
    }
}

// pub struct QwenInferenceParams {
//     pub model_id: Option<String>,
//     pub revision: String,
//     pub tokenizer_file: Option<PathBuf>,
//     pub weight_files: Option<Vec<PathBuf>>,
//     pub config_file: Option<PathBuf>,  // 添加配置文件路径
//     pub which_model: WhichModel,
//     pub device: Device,
//     pub dtype: DType,
//     pub seed: u64,
//     pub temperature: Option<f64>,
//     pub top_p: Option<f64>,
//     pub repeat_penalty: f32,
//     pub repeat_last_n: usize,
//     pub use_flash_attn: bool,  // 添加FlashAttention支持
// }

// pub struct QwenCandleGenerator {
//     pipeline: TextGeneration,
// }

// impl QwenCandleGenerator {
//     pub fn new(params: QwenInferenceParams) -> Result<Self> {
//         let api = Api::new()?;
        
//         // Determine model ID
//         let model_id = params.model_id.clone().unwrap_or_else(|| {
//             let (version, size) = match params.which_model {
//                 WhichModel::W2_0_5b => ("2", "0.5B"),
//                 WhichModel::W2_1_5b => ("2", "1.5B"),
//                 WhichModel::W2_7b => ("2", "7B"),
//                 WhichModel::W2_72b => ("2", "72B"),
//                 WhichModel::W0_5b => ("1.5", "0.5B"),
//                 WhichModel::W1_8b => ("1.5", "1.8B"),
//                 WhichModel::W4b => ("1.5", "4B"),
//                 WhichModel::W7b => ("1.5", "7B"),
//                 WhichModel::W14b => ("1.5", "14B"),
//                 WhichModel::W72b => ("1.5", "72B"),
//                 WhichModel::MoeA27b => ("1.5", "MoE-A2.7B"),
//                 WhichModel::W3_0_6b => ("3", "0.6B"),
//                 WhichModel::W3_1_7b => ("3", "1.7B"),
//                 WhichModel::W3_4b => ("3", "4B"),
//                 WhichModel::W3_8b => ("3", "8B"),
//             };
//             format!("Qwen/Qwen{version}-{size}")
//         });
        
//         // Create repository
//         let repo = api.repo(Repo::with_revision(
//             model_id,
//             RepoType::Model,
//             params.revision.clone(),
//         ));
        
//         // Get tokenizer
//         let tokenizer_filename = match &params.tokenizer_file {
//             Some(file) => file.clone(),
//             None => repo.get("tokenizer.json")?,
//         };
//         let tokenizer = Tokenizer::from_file(tokenizer_filename).map_err(E::msg)?;
        
//         // Get model weights
//         let filenames = match &params.weight_files {
//             Some(files) => files.clone(),
//             None => match params.which_model {
//                 WhichModel::W0_5b
//                 | WhichModel::W2_0_5b
//                 | WhichModel::W2_1_5b
//                 | WhichModel::W1_8b
//                 | WhichModel::W3_0_6b => {
//                     vec![repo.get("model.safetensors")?]
//                 }
//                 _ => {
//                     candle_examples::hub_load_safetensors(&repo, "model.safetensors.index.json")?
//                 }
//             },
//         };
        
//         // Get config file
//         let config_file = match &params.config_file {
//             Some(file) => std::fs::read(file)?,
//             None => std::fs::read(repo.get("config.json")?)?,
//         };
        
//         // Create variable builder
//         let vb = unsafe { 
//             VarBuilder::from_mmaped_safetensors(&filenames, params.dtype, &params.device)? 
//         };
        
//         // Create model
//         let model = match params.which_model {
//             WhichModel::MoeA27b => {
//                 let config: ConfigMoe = serde_json::from_slice(&config_file)?;
//                 Model::Moe(ModelMoe::new(&config, vb)?)
//             }
//             // Uncomment if Qwen3 support is added back
//             // WhichModel::W3_0_6b | WhichModel::W3_1_7b | WhichModel::W3_4b | WhichModel::W3_8b => {
//             //     let config: Config3 = serde_json::from_slice(&config_file)?;
//             //     Model::Base3(Model3::new(&config, vb)?)
//             // }
//             _ => {
//                 let config: ConfigBase = serde_json::from_slice(&config_file)?;
//                 Model::Base(ModelBase::new(&config, vb)?)
//             }
//         };
        
//         // Create text generation pipeline
//         let pipeline = TextGeneration::new(
//             model,
//             tokenizer,
//             params.seed,
//             params.temperature,
//             params.top_p,
//             params.repeat_penalty,
//             params.repeat_last_n,
//             &params.device,
//         );
        
//         Ok(Self { pipeline })
//     }
    
//     // Generate text and collect into a string
//     pub fn generate_string(&mut self, prompt: &str, sample_len: usize) -> Result<String> {
//         let mut output = String::new();
        
//         .pipeline.run_with_callback(prompt, sample_len, |chunk| {
//             output.push_str(chunk);
//             Ok(())
//         })?;
        
//         Ok(output)
//     }
    
//     // Generate text with streaming via callback
//     pub fn generate_streaming<F>(&mut self, prompt: &str, sample_len: usize, callback: F) -> Result<()>
//     where
//         F: FnMut(&str) -> Result<()>,
//     {
//         .pipeline.run_with_callback(prompt, sample_len, callback)
//     }
    
//     // Generate tokens
//     pub fn generate_tokens(&mut self, prompt: &str, sample_len: usize) -> Result<Vec<u32>> {
//         .pipeline.tokenizer.clear();
//         let mut tokens = .pipeline.tokenizer
//             .tokenizer()
//             .encode(prompt, true)
//             .map_err(E::msg)?
//             .get_ids()
//             .to_vec();
        
//         let initial_len = tokens.len();
//         let eos_token = .pipeline.tokenizer.get_token("<|endoftext|>")
//             .ok_or_else(|| E::msg("cannot find the <|endoftext|> token"))?;
//         let eos_token2 = .pipeline.tokenizer.get_token("<|im_end|>")
//             .ok_or_else(|| E::msg("cannot find the <|im_end|> token"))?;
        
//         for index in 0..sample_len {
//             let context_size = if index > 0 { 1 } else { tokens.len() };
//             let start_pos = tokens.len().saturating_sub(context_size);
//             let ctxt = &tokens[start_pos..];
//             let input = Tensor::new(ctxt, &.pipeline.device)?.unsqueeze(0)?;
//             let logits = .pipeline.model.forward(&input, start_pos)?;
//             let logits = logits.squeeze(0)?.squeeze(0)?.to_dtype(DType::F32)?;
//             let logits = if .pipeline.repeat_penalty == 1. {
//                 logits
//             } else {
//                 let start_at = tokens.len().saturating_sub(.pipeline.repeat_last_n);
//                 candle_transformers::utils::apply_repeat_penalty(
//                     &logits,
//                     .pipeline.repeat_penalty,
//                     &tokens[start_at..],
//                 )?
//             };

//             let next_token = .pipeline.logits_processor.sample(&logits)?;
//             tokens.push(next_token);
            
//             if next_token == eos_token || next_token == eos_token2 {
//                 break;
//             }
//         }
        
//         // Return only the newly generated tokens
//         Ok(tokens[initial_len..].to_vec())
//     }
// }