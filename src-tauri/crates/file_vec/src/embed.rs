use anyhow::Result;
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;

#[derive(Error, Debug)]
pub enum EmbedError {
    #[error("Model initialization error: {0}")]
    ModelInit(String),
    #[error("Embedding generation error: {0}")]
    EmbedGeneration(String),
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Model not initialized")]
    ModelNotInitialized,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbedConfig {
    /// 模型名称，默认使用 BAAI/bge-small-en-v1.5
    pub model_name: String,
    /// 最大序列长度，默认 512
    pub max_length: usize,
    /// 批处理大小，默认 32
    pub batch_size: usize,
    /// 是否显示下载进度，默认 true
    pub show_download_progress: bool,
    /// 缓存目录，默认使用系统缓存目录
    pub cache_dir: Option<String>,
}

impl Default for EmbedConfig {
    fn default() -> Self {
        Self {
            model_name: "BAAI/bge-small-en-v1.5".to_string(),
            max_length: 512,
            batch_size: 32,
            show_download_progress: true,
            cache_dir: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EmbedResult {
    pub text: String,
    pub vector: Vec<f32>,
    pub dimension: usize,
}

/// FastEmbed 文本嵌入包装器
pub struct FastEmbedWrapper {
    model: Arc<RwLock<Option<TextEmbedding>>>,
    config: EmbedConfig,
}

impl FastEmbedWrapper {
    /// 创建新的 FastEmbed 包装器实例
    pub fn new(config: EmbedConfig) -> Self {
        Self {
            model: Arc::new(RwLock::new(None)),
            config,
        }
    }

    /// 使用默认配置创建实例
    pub fn default() -> Self {
        Self::new(EmbedConfig::default())
    }

    /// 初始化嵌入模型
    pub async fn initialize(&self) -> Result<(), EmbedError> {
        info!("初始化 FastEmbed 模型: {}", self.config.model_name);

        let model_name = match self.config.model_name.as_str() {
            "BAAI/bge-small-en-v1.5" => EmbeddingModel::BGESmallENV15,
            "BAAI/bge-base-en-v1.5" => EmbeddingModel::BGEBaseENV15,
            "BAAI/bge-large-en-v1.5" => EmbeddingModel::BGELargeENV15,
            "sentence-transformers/all-MiniLM-L6-v2" => EmbeddingModel::AllMiniLML6V2,
            "sentence-transformers/all-MiniLM-L12-v2" => EmbeddingModel::AllMiniLML12V2,
            _ => {
                warn!("未知模型名称 '{}', 使用默认模型", self.config.model_name);
                EmbeddingModel::BGESmallENV15
            }
        };

        let mut options = InitOptions::new(model_name);

        if let Some(cache_dir) = &self.config.cache_dir {
            options = options.with_cache_dir(cache_dir.into());
        }

        options = options.with_show_download_progress(self.config.show_download_progress);

        let embedding_model =
            TextEmbedding::try_new(options).map_err(|e| EmbedError::ModelInit(e.to_string()))?;

        let mut model_lock = self.model.write().await;
        *model_lock = Some(embedding_model);

        info!("FastEmbed 模型初始化成功");
        Ok(())
    }

    /// 检查模型是否已初始化
    pub async fn is_initialized(&self) -> bool {
        let model_lock = self.model.read().await;
        model_lock.is_some()
    }

    pub async fn embed_text(&self, text: &str) -> Result<EmbedResult, EmbedError> {
        if text.trim().is_empty() {
            return Err(EmbedError::InvalidInput("文本不能为空".to_string()));
        }

        let mut model_lock = self.model.write().await;
        let model = model_lock.as_mut().ok_or(EmbedError::ModelNotInitialized)?;

        debug!("为文本生成嵌入: {}", &text[..text.len().min(50)]);

        let embeddings = model
            .embed(vec![text], Some(self.config.batch_size))
            .map_err(|e| EmbedError::EmbedGeneration(e.to_string()))?;

        let vector = embeddings
            .into_iter()
            .next()
            .ok_or_else(|| EmbedError::EmbedGeneration("未生成嵌入向量".to_string()))?;

        let dimension = vector.len();

        Ok(EmbedResult {
            text: text.to_string(),
            vector,
            dimension,
        })
    }

    /// 为多个文本批量生成嵌入向量
    pub async fn embed_batch(&self, texts: &[String]) -> Result<Vec<EmbedResult>, EmbedError> {
        if texts.is_empty() {
            return Err(EmbedError::InvalidInput("文本列表不能为空".to_string()));
        }

        let mut model_lock = self.model.write().await;
        let model = model_lock.as_mut().ok_or(EmbedError::ModelNotInitialized)?;

        debug!("批量生成嵌入，文本数量: {}", texts.len());

        // 过滤空文本
        let non_empty_texts: Vec<&str> = texts
            .iter()
            .filter(|text| !text.trim().is_empty())
            .map(|s| s.as_str())
            .collect();

        if non_empty_texts.is_empty() {
            return Err(EmbedError::InvalidInput("没有有效的文本".to_string()));
        }

        let embeddings = model
            .embed(non_empty_texts.clone(), Some(self.config.batch_size))
            .map_err(|e| EmbedError::EmbedGeneration(e.to_string()))?;

        let results: Vec<EmbedResult> = non_empty_texts
            .into_iter()
            .zip(embeddings.into_iter())
            .map(|(text, vector)| EmbedResult {
                text: text.to_string(),
                dimension: vector.len(),
                vector,
            })
            .collect();

        debug!("批量嵌入生成完成，生成数量: {}", results.len());
        Ok(results)
    }

    /// 为查询文本生成嵌入（添加 "query:" 前缀）
    pub async fn embed_query(&self, query: &str) -> Result<EmbedResult, EmbedError> {
        let prefixed_query = format!("query: {}", query);
        self.embed_text(&prefixed_query).await
    }

    /// 为文档文本生成嵌入（添加 "passage:" 前缀）
    pub async fn embed_passage(&self, passage: &str) -> Result<EmbedResult, EmbedError> {
        let prefixed_passage = format!("passage: {}", passage);
        self.embed_text(&prefixed_passage).await
    }

    /// 批量为查询生成嵌入
    pub async fn embed_queries(&self, queries: &[String]) -> Result<Vec<EmbedResult>, EmbedError> {
        let prefixed_queries: Vec<String> = queries
            .iter()
            .map(|query| format!("query: {}", query))
            .collect();

        self.embed_batch(&prefixed_queries).await
    }

    /// 批量为文档生成嵌入
    pub async fn embed_passages(
        &self,
        passages: &[String],
    ) -> Result<Vec<EmbedResult>, EmbedError> {
        let prefixed_passages: Vec<String> = passages
            .iter()
            .map(|passage| format!("passage: {}", passage))
            .collect();

        self.embed_batch(&prefixed_passages).await
    }

    /// 获取模型配置信息
    pub fn get_config(&self) -> &EmbedConfig {
        &self.config
    }

    /// 获取模型向量维度（需要先初始化模型）
    pub async fn get_vector_dimension(&self) -> Result<usize, EmbedError> {
        // 使用一个简单的测试文本来获取维度
        let test_result = self.embed_text("test").await?;
        Ok(test_result.dimension)
    }

    /// 计算两个向量的余弦相似度
    pub fn cosine_similarity(vec1: &[f32], vec2: &[f32]) -> Result<f32, EmbedError> {
        if vec1.len() != vec2.len() {
            return Err(EmbedError::InvalidInput("向量维度不匹配".to_string()));
        }

        let dot_product: f32 = vec1.iter().zip(vec2.iter()).map(|(a, b)| a * b).sum();

        let norm1: f32 = vec1.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm2: f32 = vec2.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm1 == 0.0 || norm2 == 0.0 {
            return Ok(0.0);
        }

        Ok(dot_product / (norm1 * norm2))
    }
}

/// 创建带有默认配置的全局嵌入实例
pub async fn create_default_embedder() -> Result<FastEmbedWrapper, EmbedError> {
    let embedder = FastEmbedWrapper::default();
    embedder.initialize().await?;
    Ok(embedder)
}

/// 创建带有自定义配置的嵌入实例
pub async fn create_embedder(config: EmbedConfig) -> Result<FastEmbedWrapper, EmbedError> {
    let embedder = FastEmbedWrapper::new(config);
    embedder.initialize().await?;
    Ok(embedder)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_embed_wrapper_creation() {
        let config = EmbedConfig::default();
        let wrapper = FastEmbedWrapper::new(config);
        assert!(!wrapper.is_initialized().await);
    }

    #[tokio::test]
    async fn test_cosine_similarity() {
        let vec1 = vec![1.0, 0.0, 0.0];
        let vec2 = vec![0.0, 1.0, 0.0];
        let similarity = FastEmbedWrapper::cosine_similarity(&vec1, &vec2).unwrap();
        assert_eq!(similarity, 0.0);

        let vec3 = vec![1.0, 0.0, 0.0];
        let vec4 = vec![1.0, 0.0, 0.0];
        let similarity2 = FastEmbedWrapper::cosine_similarity(&vec3, &vec4).unwrap();
        assert!((similarity2 - 1.0).abs() < 1e-6);
    }

    #[tokio::test]
    async fn test_invalid_input() {
        let wrapper = FastEmbedWrapper::default();
        // 测试未初始化的模型
        let result = wrapper.embed_text("test").await;
        assert!(matches!(result, Err(EmbedError::ModelNotInitialized)));
    }

    // 注意：以下测试需要实际的模型文件，在CI环境中可能需要跳过
    #[tokio::test]
    #[ignore] // 在CI中忽略，因为需要下载模型
    async fn test_embed_functionality() {
        let wrapper = create_default_embedder().await.unwrap();

        let result = wrapper.embed_text("Hello, world!").await.unwrap();
        assert!(!result.vector.is_empty());
        assert_eq!(result.text, "Hello, world!");
        assert_eq!(result.dimension, result.vector.len());
    }

    #[tokio::test]
    #[ignore] // 在CI中忽略，因为需要下载模型
    async fn test_batch_embedding() {
        let wrapper = create_default_embedder().await.unwrap();

        let texts = vec![
            "Hello, world!".to_string(),
            "How are you?".to_string(),
            "This is a test.".to_string(),
        ];

        let results = wrapper.embed_batch(&texts).await.unwrap();
        assert_eq!(results.len(), 3);

        for (i, result) in results.iter().enumerate() {
            assert_eq!(result.text, texts[i]);
            assert!(!result.vector.is_empty());
        }
    }

    #[tokio::test]
    #[ignore] // 在CI中忽略，因为需要下载模型
    async fn test_query_and_passage_embedding() {
        let wrapper = create_default_embedder().await.unwrap();

        let query_result = wrapper
            .embed_query("What is the capital of France?")
            .await
            .unwrap();
        let passage_result = wrapper
            .embed_passage("Paris is the capital of France.")
            .await
            .unwrap();

        assert!(query_result.text.starts_with("query:"));
        assert!(passage_result.text.starts_with("passage:"));

        // 计算相似度
        let similarity =
            FastEmbedWrapper::cosine_similarity(&query_result.vector, &passage_result.vector)
                .unwrap();

        // 相关的查询和文档应该有较高的相似度
        assert!(similarity > 0.0);
    }
}
