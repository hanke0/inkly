use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum SummarizeError {
    #[error("article text is empty")]
    EmptyArticle,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("candle error: {0}")]
    Candle(#[from] candle_core::Error),

    #[error("hf-hub error: {0}")]
    Hub(#[from] hf_hub::api::sync::ApiError),

    #[error("tokenizer error: {0}")]
    Tokenizer(String),

    #[error("failed to load GGUF model from {path:?}: {message}")]
    GgufLoad { path: PathBuf, message: String },

    #[error("missing special token {token:?} in tokenizer vocabulary")]
    MissingSpecialToken { token: String },

    #[error("internal summarizer error")]
    Internal,
}
