use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DocumentIn {
    /// Client-provided document id (unique within a tenant).
    pub doc_id: String,
    pub title: String,
    pub content: String,
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
    pub doc_id: String,
    pub title: String,
    pub snippet: String,
    pub score: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchResponse {
    pub total_hits: u64,
    pub results: Vec<SearchResult>,
}

fn default_limit() -> u32 {
    10
}

