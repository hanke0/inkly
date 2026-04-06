use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DocumentIn {
    /// Explicit id for update flows (`POST /v1/documents/{doc_id}` sets this from the path). Ignored on multipart create (`POST /v1/documents`), which always gets a new id.
    #[serde(default)]
    pub doc_id: Option<u64>,
    pub title: String,
    /// Required for new documents. Ignored on updates (existing content is preserved).
    #[serde(default)]
    pub content: Option<String>,
    pub doc_url: String,
    pub tags: Vec<String>,
    /// Parent directory path, normalized by the API to `/` or `/segment/.../` (trailing slash except root).
    pub path: String,
    pub note: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IndexResponse {
    pub indexed: u64,
    pub deleted: u64,
    /// Single-document index: assigned id when `doc_id` was omitted / null / 0.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub doc_id: Option<u64>,
    /// Reserved; always empty in current API responses.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub doc_ids: Vec<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchQuery {
    /// Full-text query over title, content, and note. May be empty when `path` or `tags` filters are set.
    #[serde(default)]
    pub q: String,
    #[serde(default = "default_limit")]
    pub limit: u32,
    /// Normalized folder path (`/` or `/a/b/`). When not `/`, only documents in this folder or subfolders match.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// Comma-separated tags; the document must contain every tag (AND).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tags: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub doc_id: u64,
    pub title: String,
    pub doc_url: String,
    pub snippet: String,
    /// Short model-generated summary of the document content, in the same language as `content`.
    pub summary: String,
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
    /// Resolved from the request `Accept-Language` header (`en`, `zh-Hans`, …).
    pub locale: String,
}

/// `GET /v1/catalog` — list indexed subdirectories and document titles under a logical path.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CatalogQuery {
    /// Directory path (`/` or `/foo/bar/`). Normalized server-side.
    #[serde(default = "default_catalog_path")]
    pub path: String,
}

fn default_catalog_path() -> String {
    "/".to_string()
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CatalogSubdir {
    pub name: String,
    pub path: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CatalogFile {
    pub doc_id: u64,
    pub title: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CatalogResponse {
    pub path: String,
    pub subdirs: Vec<CatalogSubdir>,
    pub files: Vec<CatalogFile>,
}

/// `POST /v1/documents/{doc_id}/summary` — queue (or acknowledge) async summarization.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SummaryEnqueueResponse {
    /// `true` if a new job was added; `false` if this document was already in the queue.
    pub enqueued: bool,
    /// Localized message for display (from `Accept-Language`).
    pub message: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DocumentDetailResponse {
    pub doc_id: u64,
    pub title: String,
    pub content: String,
    pub summary: String,
    pub doc_url: String,
    pub path: String,
    pub note: String,
    pub tags: Vec<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

fn default_limit() -> u32 {
    10
}
