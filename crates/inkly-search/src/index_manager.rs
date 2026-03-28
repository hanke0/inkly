use std::collections::HashSet;
use std::path::Path;

use tantivy::collector::TopDocs;
use tantivy::query::{BooleanQuery, Occur, Query, QueryParser, TermQuery};
use tantivy::schema::{
    Field, IndexRecordOption, TextFieldIndexing, TextOptions, FAST, INDEXED, STORED, STRING, TEXT,
    Value,
};
use tantivy::{doc, schema, Index, Term};

use crate::error::{Result, SearchError};

#[derive(Clone, Debug)]
pub struct IndexStats {
    pub indexed: u64,
    pub deleted: u64,
}

#[derive(Clone, Debug)]
pub struct SearchResultItem {
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

const MAX_CATALOG_SCAN: usize = 50_000;
const MAX_CATALOG_FILES: usize = 5_000;

/// One directory level in the catalog API (`name`, normalized `path`).
#[derive(Clone, Debug)]
pub struct CatalogListing {
    pub path: String,
    pub subdirs: Vec<(String, String)>,
    pub files: Vec<(u64, String)>,
}

#[derive(Clone, Debug)]
pub struct StoredDocument {
    pub doc_id: u64,
    pub title: String,
    pub content: String,
    pub doc_url: String,
    pub path: String,
    pub note: String,
    pub tags: Vec<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

fn direct_subdir_under(parent: &str, indexed_path: &str) -> Option<String> {
    if indexed_path == parent {
        return None;
    }
    if !indexed_path.starts_with(parent) {
        return None;
    }
    let suffix = indexed_path[parent.len()..].trim_start_matches('/');
    if suffix.is_empty() {
        return None;
    }
    let first = suffix.split('/').next()?;
    if first.is_empty() {
        return None;
    }
    if parent == "/" {
        Some(format!("/{first}/"))
    } else {
        let base = parent.trim_end_matches('/');
        Some(format!("{base}/{first}/"))
    }
}

#[derive(Clone)]
pub struct IndexManager {
    index: Index,
    tenant_id_field: Field,
    doc_id_field: Field,
    doc_url_field: Field,
    title_field: Field,
    content_field: Field,
    created_timestamp_field: Field,
    update_timestamp_field: Field,
    tags_field: Field,
    path_field: Field,
    note_field: Field,
}

impl IndexManager {
    fn now_unix_seconds() -> Result<i64> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| SearchError::InvalidInput(format!("time went backwards: {e}")))?
            .as_secs() as i64;
        Ok(now)
    }

    fn existing_created_at(
        &self,
        searcher: &tantivy::Searcher,
        tenant_id: &str,
        doc_id: u64,
    ) -> Result<Option<i64>> {
        let tenant_term = Term::from_field_text(self.tenant_id_field, tenant_id);
        let doc_id_term = Term::from_field_u64(self.doc_id_field, doc_id);

        let tenant_query = TermQuery::new(tenant_term, IndexRecordOption::Basic);
        let doc_id_query = TermQuery::new(doc_id_term, IndexRecordOption::Basic);
        let query = BooleanQuery::new(vec![
            (Occur::Must, Box::new(tenant_query)),
            (Occur::Must, Box::new(doc_id_query)),
        ]);

        let hits = searcher.search(&query, &TopDocs::with_limit(1))?;
        let (_, doc_address) = match hits.into_iter().next() {
            Some(h) => h,
            None => return Ok(None),
        };

        let retrieved = searcher.doc::<tantivy::TantivyDocument>(doc_address)?;
        let created_at = retrieved
            .get_first(self.created_timestamp_field)
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        Ok(Some(created_at))
    }

    pub fn open_or_create<P: AsRef<Path>>(index_dir: P) -> Result<Self> {
        let index_dir = index_dir.as_ref();
        std::fs::create_dir_all(index_dir)?;

        let index = if let Ok(existing) = Index::open_in_dir(index_dir) {
            existing
        } else {
            let schema = Self::build_schema();
            let index = Index::create_in_dir(index_dir, schema)?;
            index
        };

        let schema = index.schema();
        let tenant_id_field = schema
            .get_field("tenant_id")
            .map_err(|_| SearchError::InvalidInput("missing tenant_id field".into()))?;
        let doc_id_field = schema
            .get_field("doc_id")
            .map_err(|_| SearchError::InvalidInput("missing doc_id field".into()))?;
        if !schema.get_field_entry(doc_id_field).is_indexed() {
            return Err(SearchError::InvalidInput(
                "Tantivy index schema is outdated: doc_id must be indexed. Delete the index directory (your DATA_DIR) and restart."
                    .into(),
            ));
        }
        let doc_url_field = schema
            .get_field("doc_url")
            .map_err(|_| SearchError::InvalidInput("missing doc_url field".into()))?;
        let title_field = schema
            .get_field("title")
            .map_err(|_| SearchError::InvalidInput("missing title field".into()))?;
        let content_field = schema
            .get_field("content")
            .map_err(|_| SearchError::InvalidInput("missing content field".into()))?;
        let created_timestamp_field = schema
            .get_field("created_timestamp")
            .map_err(|_| SearchError::InvalidInput("missing created_timestamp field".into()))?;
        let update_timestamp_field = schema
            .get_field("update_timestamp")
            .map_err(|_| SearchError::InvalidInput("missing update_timestamp field".into()))?;
        let tags_field = schema
            .get_field("tags")
            .map_err(|_| SearchError::InvalidInput("missing tags field".into()))?;
        let path_field = schema
            .get_field("path")
            .map_err(|_| SearchError::InvalidInput("missing path field".into()))?;
        if !schema.get_field_entry(path_field).is_indexed() {
            return Err(SearchError::InvalidInput(
                "Tantivy index schema is outdated: path must be indexed. Delete DATA_DIR and restart."
                    .into(),
            ));
        }
        let note_field = schema
            .get_field("note")
            .map_err(|_| SearchError::InvalidInput("missing note field".into()))?;

        Ok(Self {
            index,
            tenant_id_field,
            doc_id_field,
            doc_url_field,
            title_field,
            content_field,
            created_timestamp_field,
            update_timestamp_field,
            tags_field,
            path_field,
            note_field,
        })
    }

    fn build_schema() -> schema::Schema {
        let mut builder = schema::Schema::builder();

        let _tenant_id = builder.add_text_field("tenant_id", STRING | STORED);
        // INDEXED is required for `TermQuery` / `delete_query` on this field (FAST alone is not enough).
        let _doc_id = builder.add_u64_field("doc_id", INDEXED | FAST | STORED);
        let _doc_url = builder.add_text_field("doc_url", STRING | STORED);
        let _title = builder.add_text_field("title", TEXT | STORED);
        let _content = builder.add_text_field("content", TEXT | STORED);
        let _created_timestamp = builder.add_i64_field("created_timestamp", STORED);
        let _update_timestamp = builder.add_i64_field("update_timestamp", STORED);
        let _tags = builder.add_text_field("tags", STRING | STORED);
        // Whole-value indexing (`raw` tokenizer) for exact `path` `TermQuery` (STRING cannot be INDEXED in 0.25).
        let path_opts = TextOptions::default()
            .set_indexing_options(
                TextFieldIndexing::default()
                    .set_tokenizer("raw")
                    .set_index_option(IndexRecordOption::Basic),
            )
            .set_stored();
        let _path = builder.add_text_field("path", path_opts);
        let _note = builder.add_text_field("note", TEXT | STORED);

        builder.build()
    }

    pub fn index_document(
        &self,
        tenant_id: &str,
        doc_id: u64,
        title: &str,
        content: &str,
        doc_url: &str,
        tags: &[String],
        path: &str,
        note: &str,
    ) -> Result<IndexStats> {
        let now = Self::now_unix_seconds()?;

        if tenant_id.trim().is_empty() {
            return Err(SearchError::InvalidInput("tenant_id is empty".into()));
        }
        // u64 doc_id is always "present".

        // Drop the reader before opening a writer — Tantivy can fail or deadlock if both are held.
        let created_at = {
            let reader = self.index.reader()?;
            let searcher = reader.searcher();
            self.existing_created_at(&searcher, tenant_id, doc_id)?
                .unwrap_or(now)
        };
        let updated_at = now;

        let mut writer = self.index.writer(50_000_000)?;
        let tenant_term = Term::from_field_text(self.tenant_id_field, tenant_id);
        let doc_id_term = Term::from_field_u64(self.doc_id_field, doc_id);
        let delete_tenant_query =
            TermQuery::new(tenant_term, IndexRecordOption::Basic);
        let delete_doc_id_query =
            TermQuery::new(doc_id_term, IndexRecordOption::Basic);
        let delete_query = BooleanQuery::new(vec![
            (Occur::Must, Box::new(delete_tenant_query)),
            (Occur::Must, Box::new(delete_doc_id_query)),
        ]);
        writer.delete_query(Box::new(delete_query))?;

        let mut document = doc!(
            self.tenant_id_field => tenant_id,
            self.doc_id_field => doc_id,
            self.doc_url_field => doc_url,
            self.title_field => title,
            self.content_field => content,
            self.created_timestamp_field => created_at,
            self.update_timestamp_field => updated_at,
            self.path_field => path,
            self.note_field => note
        );
        for tag in tags {
            document.add_text(self.tags_field, tag);
        }
        writer.add_document(document)?;

        writer.commit()?;

        Ok(IndexStats {
            indexed: 1,
            deleted: 0, // Tantivy doesn't tell us how many were deleted.
        })
    }

    pub fn index_documents(
        &self,
        tenant_id: &str,
        docs: impl IntoIterator<Item = (u64, String, String, String, Vec<String>, String, String)>,
    ) -> Result<IndexStats> {
        let now = Self::now_unix_seconds()?;

        let docs: Vec<_> = docs.into_iter().collect();

        let created_at_per_doc: Vec<i64> = {
            let reader = self.index.reader()?;
            let searcher = reader.searcher();
            let mut v = Vec::with_capacity(docs.len());
            for (doc_id, _, _, _, _, _, _) in &docs {
                let created_at = self
                    .existing_created_at(&searcher, tenant_id, *doc_id)?
                    .unwrap_or(now);
                v.push(created_at);
            }
            v
        };

        let mut writer = self.index.writer(50_000_000)?;
        let mut indexed = 0u64;

        for ((doc_id, title, content, doc_url, tags, path, note), created_at) in
            docs.into_iter().zip(created_at_per_doc)
        {
            let updated_at = now;

            let tenant_term = Term::from_field_text(self.tenant_id_field, tenant_id);
            let doc_id_term = Term::from_field_u64(self.doc_id_field, doc_id);
            let delete_tenant_query =
                TermQuery::new(tenant_term, IndexRecordOption::Basic);
            let delete_doc_id_query =
                TermQuery::new(doc_id_term, IndexRecordOption::Basic);
            let delete_query = BooleanQuery::new(vec![
                (Occur::Must, Box::new(delete_tenant_query)),
                (Occur::Must, Box::new(delete_doc_id_query)),
            ]);
            writer.delete_query(Box::new(delete_query))?;

            let mut document = doc!(
                self.tenant_id_field => tenant_id,
                self.doc_id_field => doc_id,
                self.doc_url_field => doc_url,
                self.title_field => title,
                self.content_field => content,
                self.created_timestamp_field => created_at,
                self.update_timestamp_field => updated_at,
                self.path_field => path,
                self.note_field => note
            );
            for tag in &tags {
                document.add_text(self.tags_field, tag);
            }
            writer.add_document(document)?;

            indexed += 1;
        }

        writer.commit()?;

        Ok(IndexStats {
            indexed,
            deleted: 0,
        })
    }

    pub fn search(
        &self,
        tenant_id: &str,
        query_str: &str,
        limit: u32,
    ) -> Result<(u64, Vec<SearchResultItem>)> {
        let query_str = query_str.trim();
        if query_str.is_empty() {
            return Err(SearchError::InvalidInput("q is empty".into()));
        }

        let limit = limit.clamp(1, 50) as usize;
        let reader = self.index.reader()?;
        let searcher = reader.searcher();

        let tenant_term = Term::from_field_text(self.tenant_id_field, tenant_id);
        let tenant_query = TermQuery::new(tenant_term, IndexRecordOption::Basic);

        let parser =
            QueryParser::for_index(&self.index, vec![self.title_field, self.content_field, self.note_field]);
        let full_query = parser.parse_query(query_str)?;

        let query = BooleanQuery::new(vec![(Occur::Must, Box::new(tenant_query)), (Occur::Must, full_query)]);

        let top_docs = TopDocs::with_limit(limit);
        let total_hits = query.count(&searcher)? as u64;
        let hits = searcher.search(&query, &top_docs)?;

        let mut results = Vec::with_capacity(hits.len());
        for (score, doc_address) in hits {
            let retrieved = searcher.doc::<tantivy::TantivyDocument>(doc_address)?;
            let title = retrieved.get_first(self.title_field).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let content = retrieved.get_first(self.content_field).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let note = retrieved.get_first(self.note_field).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let doc_id = retrieved.get_first(self.doc_id_field).and_then(|v| v.as_u64()).unwrap_or(0);
            let doc_url = retrieved.get_first(self.doc_url_field).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let created_at = retrieved.get_first(self.created_timestamp_field).and_then(|v| v.as_i64()).unwrap_or(0);
            let updated_at = retrieved.get_first(self.update_timestamp_field).and_then(|v| v.as_i64()).unwrap_or(0);
            let path = retrieved.get_first(self.path_field).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let tags = retrieved
                .get_all(self.tags_field)
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect::<Vec<_>>();

            let snippet_source = if content.trim().is_empty() { &note } else { &content };
            let snippet = snippet_source.chars().take(220).collect::<String>();
            results.push(SearchResultItem {
                doc_id,
                title,
                doc_url,
                snippet,
                score,
                created_at,
                updated_at,
                tags,
                path,
                note,
            });
        }

        Ok((total_hits, results))
    }

    pub fn catalog_list(&self, tenant_id: &str, dir_path: &str) -> Result<CatalogListing> {
        let reader = self.index.reader()?;
        let searcher = reader.searcher();

        let tenant_term = Term::from_field_text(self.tenant_id_field, tenant_id);
        let tenant_query = TermQuery::new(tenant_term, IndexRecordOption::Basic);

        let hits = searcher.search(&tenant_query, &TopDocs::with_limit(MAX_CATALOG_SCAN))?;

        let mut unique_paths: HashSet<String> = HashSet::new();
        for (_, doc_address) in hits {
            let retrieved = searcher.doc::<tantivy::TantivyDocument>(doc_address)?;
            let p = retrieved
                .get_first(self.path_field)
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if !p.is_empty() {
                unique_paths.insert(p.to_string());
            }
        }

        let mut subdir_paths: HashSet<String> = HashSet::new();
        for p in &unique_paths {
            if let Some(child) = direct_subdir_under(dir_path, p) {
                subdir_paths.insert(child);
            }
        }

        let mut subdirs: Vec<(String, String)> = subdir_paths
            .into_iter()
            .map(|path| {
                let name = path
                    .trim_end_matches('/')
                    .rsplit('/')
                    .find(|s| !s.is_empty())
                    .unwrap_or("")
                    .to_string();
                (name, path)
            })
            .collect();
        subdirs.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));

        let path_term = Term::from_field_text(self.path_field, dir_path);
        let path_query = TermQuery::new(path_term, IndexRecordOption::Basic);
        let dir_query = BooleanQuery::new(vec![
            (
                Occur::Must,
                Box::new(TermQuery::new(
                    Term::from_field_text(self.tenant_id_field, tenant_id),
                    IndexRecordOption::Basic,
                )),
            ),
            (Occur::Must, Box::new(path_query)),
        ]);

        let file_hits = searcher.search(&dir_query, &TopDocs::with_limit(MAX_CATALOG_FILES))?;
        let mut files: Vec<(u64, String)> = Vec::new();
        for (_, doc_address) in file_hits {
            let retrieved = searcher.doc::<tantivy::TantivyDocument>(doc_address)?;
            let doc_id = retrieved
                .get_first(self.doc_id_field)
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let title = retrieved
                .get_first(self.title_field)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            if doc_id != 0 {
                files.push((doc_id, title));
            }
        }
        files.sort_by(|a, b| a.1.to_lowercase().cmp(&b.1.to_lowercase()));

        Ok(CatalogListing {
            path: dir_path.to_string(),
            subdirs,
            files,
        })
    }

    pub fn get_document(&self, tenant_id: &str, doc_id: u64) -> Result<Option<StoredDocument>> {
        let reader = self.index.reader()?;
        let searcher = reader.searcher();
        let tenant_term = Term::from_field_text(self.tenant_id_field, tenant_id);
        let doc_id_term = Term::from_field_u64(self.doc_id_field, doc_id);
        let query = BooleanQuery::new(vec![
            (
                Occur::Must,
                Box::new(TermQuery::new(tenant_term, IndexRecordOption::Basic)),
            ),
            (
                Occur::Must,
                Box::new(TermQuery::new(doc_id_term, IndexRecordOption::Basic)),
            ),
        ]);

        let hits = searcher.search(&query, &TopDocs::with_limit(1))?;
        let (_, doc_address) = match hits.into_iter().next() {
            Some(h) => h,
            None => return Ok(None),
        };

        let retrieved = searcher.doc::<tantivy::TantivyDocument>(doc_address)?;
        Ok(Some(StoredDocument {
            doc_id: retrieved
                .get_first(self.doc_id_field)
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            title: retrieved
                .get_first(self.title_field)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            content: retrieved
                .get_first(self.content_field)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            doc_url: retrieved
                .get_first(self.doc_url_field)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            path: retrieved
                .get_first(self.path_field)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            note: retrieved
                .get_first(self.note_field)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            tags: retrieved
                .get_all(self.tags_field)
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect(),
            created_at: retrieved
                .get_first(self.created_timestamp_field)
                .and_then(|v| v.as_i64())
                .unwrap_or(0),
            updated_at: retrieved
                .get_first(self.update_timestamp_field)
                .and_then(|v| v.as_i64())
                .unwrap_or(0),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::direct_subdir_under;

    #[test]
    fn direct_subdir_from_root() {
        assert_eq!(direct_subdir_under("/", "/a/"), Some("/a/".to_string()));
        assert_eq!(direct_subdir_under("/", "/a/b/"), Some("/a/".to_string()));
    }

    #[test]
    fn direct_subdir_nested() {
        assert_eq!(
            direct_subdir_under("/a/", "/a/b/"),
            Some("/a/b/".to_string())
        );
        assert_eq!(
            direct_subdir_under("/a/", "/a/b/c/"),
            Some("/a/b/".to_string())
        );
    }

    #[test]
    fn direct_subdir_same_or_unrelated() {
        assert_eq!(direct_subdir_under("/a/", "/a/"), None);
        assert_eq!(direct_subdir_under("/a/", "/b/"), None);
        assert_eq!(direct_subdir_under("/a/", "/ab/"), None);
    }
}
