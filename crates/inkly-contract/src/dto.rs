use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DocumentIn {
    /// Set explicitly to update/replace that id. Omit, `null`, or `0` for a server-assigned id.
    #[serde(default)]
    pub doc_id: Option<u64>,
    pub title: String,
    pub content: String,
    pub doc_url: String,
    pub tags: Vec<String>,
    /// Parent directory path, normalized by the API to `/` or `/segment/.../` (trailing slash except root).
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
    /// Single-document index: assigned id when `doc_id` was omitted / null / 0.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub doc_id: Option<u64>,
    /// Bulk index: final document id for each item, in request order (includes explicit ids).
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

