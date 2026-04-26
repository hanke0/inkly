use thiserror::Error;

#[derive(Debug, Error)]
pub enum SearchError {
    #[error("index IO error: {0}")]
    IndexIO(#[from] std::io::Error),

    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    /// Used only by the legacy-read path in [`crate::migrate`].
    #[error("tantivy error: {0}")]
    Tantivy(#[from] tantivy::TantivyError),

    #[error("storage metadata JSON error: {0}")]
    VersionJson(#[from] serde_json::Error),

    #[error("storage data_version mismatch: expected {expected}, found {found}")]
    StorageVersionMismatch { expected: u32, found: u32 },

    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("internal lock error")]
    LockPoisoned,
}

pub type Result<T, E = SearchError> = std::result::Result<T, E>;
