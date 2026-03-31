use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum SummarizeError {
    #[error("article text is empty")]
    EmptyArticle,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("llama.cpp error: {0}")]
    Llama(String),

    #[error("hf-hub error: {0}")]
    Hub(#[from] hf_hub::api::sync::ApiError),

    #[error("failed to load GGUF model from {path:?}: {message}")]
    GgufLoad { path: PathBuf, message: String },

    #[error("internal summarizer error")]
    Internal,
}
