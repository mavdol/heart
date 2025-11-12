use fastembed::{EmbeddingModel, Error, InitOptions, TextEmbedding};
use std::fmt;
use std::sync::Arc;
use tokio::sync::Mutex;

pub const EMBEDDING_VECTOR_DIMENSION: i32 = 384;

#[derive(Debug)]
pub enum EmbeddingServiceError {
    FastEmbedError(String),
}

impl fmt::Display for EmbeddingServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EmbeddingServiceError::FastEmbedError(msg) => write!(f, "FastEmbed error: {}", msg),
        }
    }
}

impl From<Error> for EmbeddingServiceError {
    fn from(error: Error) -> Self {
        EmbeddingServiceError::FastEmbedError(error.to_string())
    }
}

#[derive(Clone)]
pub struct EmbeddingService {
    pub model: Arc<Mutex<TextEmbedding>>,
}

impl EmbeddingService {
    pub fn new() -> Result<Self, EmbeddingServiceError> {
        let model =
            TextEmbedding::try_new(InitOptions::new(EmbeddingModel::AllMiniLML6V2).with_show_download_progress(true))?;

        Ok(Self {
            model: Arc::new(Mutex::new(model)),
        })
    }

    pub async fn embed_text(&self, text: &str) -> Result<Vec<f32>, EmbeddingServiceError> {
        let embeddings = self.model.lock().await.embed(vec![text.to_string()], None)?;
        Ok(embeddings[0].clone())
    }
}
