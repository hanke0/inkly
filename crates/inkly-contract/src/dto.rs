use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DocumentIn {
    /// Document id.
    pub doc_id: u64,
    pub title: String,
    pub content: String,
    pub doc_url: String,
    pub tags: Vec<String>,
    pub path: String,
    pub note: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BulkIndexIn {
    pub documents: Vec<DocumentIn>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IndexResponse {
    pub indexed: u64,
    pub deleted: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchQuery {
    pub q: String,
    #[serde(default = "default_limit")]
    pub limit: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub doc_id: u64,
    pub title: String,
    pub doc_url: String,
    pub snippet: String,
    pub score: f32,
    pub created_at: i64,
    pub updated_at: i64,
    pub tags: Vec<String>,
    pub path: String,
    pub note: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchResponse {
    pub total_hits: u64,
    pub results: Vec<SearchResult>,
}

/// Successful Basic auth check (`GET /v1/session`).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SessionResponse {
    pub ok: bool,
}

fn default_limit() -> u32 {
    10
}

