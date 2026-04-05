use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use tantivy::collector::TopDocs;
use tantivy::query::{AllQuery, BooleanQuery, Occur, Query, QueryParser, RegexQuery, TermQuery};
use tantivy::schema::{
    FAST, Field, FieldType, INDEXED, IndexRecordOption, STORED, STRING, TextFieldIndexing,
    TextOptions, Value,
};
use tantivy::{Index, Term, doc, schema};

use crate::error::{Result, SearchError};
use crate::storage_meta;

/// Tantivy writer heap budget (bytes). 50 MiB is generous for batch commits at this scale.
const WRITER_HEAP_BYTES: usize = 50_000_000;

#[derive(Clone, Debug)]
pub struct IndexStats {
    pub indexed: u64,
    pub deleted: u64,
}

/// Owned document fields passed to [`IndexManager::index_document`] and
/// [`IndexManager::index_documents`]. Using a named struct instead of a positional tuple
/// prevents argument-order mistakes at call sites.
#[derive(Clone, Debug)]
pub struct DocumentRow {
    pub doc_id: u64,
    pub title: String,
    pub content: String,
    pub doc_url: String,
    pub summary: String,
    pub tags: Vec<String>,
    pub path: String,
    pub note: String,
}

#[derive(Clone, Debug)]
pub struct SearchResultItem {
    pub doc_id: u64,
    pub title: String,
    pub doc_url: String,
    pub snippet: String,
    pub summary: String,
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
    pub summary: String,
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

// ---------------------------------------------------------------------------
// Schema / field helpers
// ---------------------------------------------------------------------------

/// Registered name for [`tantivy_jieba::JiebaTokenizer`] (Chinese + mixed text).
const JIEBA_TOKENIZER: &str = "jieba";

fn register_jieba_tokenizer(index: &Index) {
    index
        .tokenizers()
        .register(JIEBA_TOKENIZER, tantivy_jieba::JiebaTokenizer::new());
}

fn jieba_text_options_stored() -> TextOptions {
    TextOptions::default()
        .set_indexing_options(
            TextFieldIndexing::default()
                .set_tokenizer(JIEBA_TOKENIZER)
                .set_index_option(IndexRecordOption::WithFreqsAndPositions),
        )
        .set_stored()
}

fn field_uses_jieba_tokenizer(entry: &schema::FieldEntry) -> bool {
    match entry.field_type() {
        FieldType::Str(opt) => opt
            .get_indexing_options()
            .is_some_and(|idx| idx.tokenizer() == JIEBA_TOKENIZER),
        _ => false,
    }
}

fn ensure_jieba_text_schema(schema: &schema::Schema) -> Result<()> {
    for name in ["title", "content", "summary", "note"] {
        let field = schema.get_field(name).map_err(|_| {
            SearchError::InvalidInput(format!("missing {name} field in Tantivy schema"))
        })?;
        let entry = schema.get_field_entry(field);
        if !field_uses_jieba_tokenizer(entry) {
            return Err(SearchError::InvalidInput(format!(
                "Tantivy index schema is outdated: field {name} must use the {JIEBA_TOKENIZER} tokenizer for Chinese search. Delete the index directory (DATA_DIR/index) and restart to reindex."
            )));
        }
    }
    Ok(())
}

macro_rules! require_field {
    ($schema:expr, $name:literal) => {
        $schema
            .get_field($name)
            .map_err(|_| SearchError::InvalidInput(concat!("missing ", $name, " field").into()))?
    };
}

// ---------------------------------------------------------------------------
// Field extraction helpers
// ---------------------------------------------------------------------------

fn get_str_field(doc: &tantivy::TantivyDocument, field: Field) -> String {
    doc.get_first(field)
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}

fn get_u64_field(doc: &tantivy::TantivyDocument, field: Field) -> u64 {
    doc.get_first(field).and_then(|v| v.as_u64()).unwrap_or(0)
}

fn get_i64_field(doc: &tantivy::TantivyDocument, field: Field) -> i64 {
    doc.get_first(field).and_then(|v| v.as_i64()).unwrap_or(0)
}

// ---------------------------------------------------------------------------

struct AutoIncrementState {
    version_path: PathBuf,
    data_version: u32,
    /// Next `doc_id` to assign.
    next: u64,
}

#[derive(Clone)]
pub struct IndexManager {
    index: Index,
    auto_increment: Arc<Mutex<AutoIncrementState>>,
    tenant_id_field: Field,
    doc_id_field: Field,
    doc_url_field: Field,
    title_field: Field,
    content_field: Field,
    summary_field: Field,
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

    /// Returns a `BooleanQuery` that matches a single (tenant, doc_id) pair.
    /// Reused by index, update, and delete operations to avoid repeated boilerplate.
    fn make_tenant_doc_query(&self, tenant_id: &str, doc_id: u64) -> BooleanQuery {
        BooleanQuery::new(vec![
            (
                Occur::Must,
                Box::new(TermQuery::new(
                    Term::from_field_text(self.tenant_id_field, tenant_id),
                    IndexRecordOption::Basic,
                )) as Box<dyn Query>,
            ),
            (
                Occur::Must,
                Box::new(TermQuery::new(
                    Term::from_field_u64(self.doc_id_field, doc_id),
                    IndexRecordOption::Basic,
                )) as Box<dyn Query>,
            ),
        ])
    }

    fn existing_created_at(
        &self,
        searcher: &tantivy::Searcher,
        tenant_id: &str,
        doc_id: u64,
    ) -> Result<Option<i64>> {
        let query = self.make_tenant_doc_query(tenant_id, doc_id);
        let hits = searcher.search(&query, &TopDocs::with_limit(1).order_by_score())?;
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

    /// Opens or creates the Tantivy index under `data_root/index/` and loads `data_root/version.data`.
    pub fn open_or_create<P: AsRef<Path>>(data_root: P) -> Result<Self> {
        let data_root = data_root.as_ref();
        let version_state = storage_meta::load_or_init_version_state(data_root)?;
        let index_dir = storage_meta::index_dir(data_root);
        let version_path = storage_meta::version_file_path(data_root);

        let index = if let Ok(existing) = Index::open_in_dir(&index_dir) {
            existing
        } else {
            let schema = Self::build_schema();
            Index::create_in_dir(&index_dir, schema)?
        };

        register_jieba_tokenizer(&index);

        let schema = index.schema();
        let tenant_id_field = require_field!(schema, "tenant_id");
        let doc_id_field = require_field!(schema, "doc_id");
        if !schema.get_field_entry(doc_id_field).is_indexed() {
            return Err(SearchError::InvalidInput(
                "Tantivy index schema is outdated: doc_id must be indexed. Delete the index directory (DATA_DIR/index) and restart."
                    .into(),
            ));
        }
        let doc_url_field = require_field!(schema, "doc_url");
        let title_field = require_field!(schema, "title");
        let content_field = require_field!(schema, "content");
        let summary_field = require_field!(schema, "summary");
        let created_timestamp_field = require_field!(schema, "created_timestamp");
        let update_timestamp_field = require_field!(schema, "update_timestamp");
        let tags_field = require_field!(schema, "tags");
        let path_field = require_field!(schema, "path");
        if !schema.get_field_entry(path_field).is_indexed() {
            return Err(SearchError::InvalidInput(
                "Tantivy index schema is outdated: path must be indexed. Delete DATA_DIR/index and restart."
                    .into(),
            ));
        }
        let note_field = require_field!(schema, "note");

        ensure_jieba_text_schema(&schema)?;

        Ok(Self {
            index,
            auto_increment: Arc::new(Mutex::new(AutoIncrementState {
                version_path,
                data_version: version_state.data_version,
                next: version_state.auto_increment,
            })),
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
            summary_field,
        })
    }

    /// Returns the next server-assigned `doc_id` and persists it to `version.data`.
    pub fn allocate_doc_id(&self) -> Result<u64> {
        let mut guard = self
            .auto_increment
            .lock()
            .map_err(|_| SearchError::LockPoisoned)?;
        let id = guard.next;
        guard.next = guard
            .next
            .checked_add(1)
            .ok_or_else(|| SearchError::InvalidInput("auto_increment overflow".into()))?;
        storage_meta::persist_auto_increment(&guard.version_path, guard.data_version, guard.next)?;
        Ok(id)
    }

    fn build_schema() -> schema::Schema {
        let mut builder = schema::Schema::builder();

        let _tenant_id = builder.add_text_field("tenant_id", STRING | STORED);
        // INDEXED is required for `TermQuery` / `delete_query` on this field (FAST alone is not enough).
        let _doc_id = builder.add_u64_field("doc_id", INDEXED | FAST | STORED);
        let _doc_url = builder.add_text_field("doc_url", STRING | STORED);
        let _title = builder.add_text_field("title", jieba_text_options_stored());
        let _content = builder.add_text_field("content", jieba_text_options_stored());
        let _summary = builder.add_text_field("summary", jieba_text_options_stored());
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
        let _note = builder.add_text_field("note", jieba_text_options_stored());

        builder.build()
    }

    fn build_tantivy_doc(
        &self,
        tenant_id: &str,
        doc: &DocumentRow,
        created_at: i64,
        updated_at: i64,
    ) -> tantivy::TantivyDocument {
        let mut document = doc!(
            self.tenant_id_field => tenant_id,
            self.doc_id_field => doc.doc_id,
            self.doc_url_field => doc.doc_url.as_str(),
            self.title_field => doc.title.as_str(),
            self.content_field => doc.content.as_str(),
            self.summary_field => doc.summary.as_str(),
            self.created_timestamp_field => created_at,
            self.update_timestamp_field => updated_at,
            self.path_field => doc.path.as_str(),
            self.note_field => doc.note.as_str()
        );
        for tag in &doc.tags {
            document.add_text(self.tags_field, tag);
        }
        document
    }

    pub fn index_document(&self, tenant_id: &str, doc: DocumentRow) -> Result<IndexStats> {
        if tenant_id.trim().is_empty() {
            return Err(SearchError::InvalidInput("tenant_id is empty".into()));
        }

        let now = Self::now_unix_seconds()?;

        // Drop the reader before opening a writer — Tantivy can fail or deadlock if both are held.
        let created_at = {
            let reader = self.index.reader()?;
            let searcher = reader.searcher();
            self.existing_created_at(&searcher, tenant_id, doc.doc_id)?
                .unwrap_or(now)
        };

        let mut writer = self.index.writer(WRITER_HEAP_BYTES)?;
        writer.delete_query(Box::new(self.make_tenant_doc_query(tenant_id, doc.doc_id)))?;
        writer.add_document(self.build_tantivy_doc(tenant_id, &doc, created_at, now))?;
        writer.commit()?;

        Ok(IndexStats {
            indexed: 1,
            deleted: 0,
        })
    }

    pub fn index_documents(
        &self,
        tenant_id: &str,
        docs: impl IntoIterator<Item = DocumentRow>,
    ) -> Result<IndexStats> {
        if tenant_id.trim().is_empty() {
            return Err(SearchError::InvalidInput("tenant_id is empty".into()));
        }

        let now = Self::now_unix_seconds()?;
        let docs: Vec<DocumentRow> = docs.into_iter().collect();

        let created_at_per_doc: Vec<i64> = {
            let reader = self.index.reader()?;
            let searcher = reader.searcher();
            docs.iter()
                .map(|d| {
                    self.existing_created_at(&searcher, tenant_id, d.doc_id)
                        .map(|opt| opt.unwrap_or(now))
                })
                .collect::<Result<Vec<_>>>()?
        };

        let mut writer = self.index.writer(WRITER_HEAP_BYTES)?;
        let mut indexed = 0u64;

        for (doc, created_at) in docs.into_iter().zip(created_at_per_doc) {
            writer.delete_query(Box::new(self.make_tenant_doc_query(tenant_id, doc.doc_id)))?;
            writer.add_document(self.build_tantivy_doc(tenant_id, &doc, created_at, now))?;
            indexed += 1;
        }

        writer.commit()?;

        Ok(IndexStats {
            indexed,
            deleted: 0,
        })
    }

    /// Like [`index_documents`](Self::index_documents) but keeps the given Unix timestamps (storage migration / reindex).
    pub fn index_documents_with_timestamps(
        &self,
        tenant_id: &str,
        docs: impl IntoIterator<Item = (DocumentRow, i64, i64)>,
    ) -> Result<IndexStats> {
        if tenant_id.trim().is_empty() {
            return Err(SearchError::InvalidInput("tenant_id is empty".into()));
        }

        let pairs: Vec<(DocumentRow, i64, i64)> = docs.into_iter().collect();
        let mut writer = self.index.writer(WRITER_HEAP_BYTES)?;
        let mut indexed = 0u64;

        for (doc, created_at, updated_at) in pairs {
            writer.delete_query(Box::new(self.make_tenant_doc_query(tenant_id, doc.doc_id)))?;
            writer.add_document(self.build_tantivy_doc(
                tenant_id,
                &doc,
                created_at,
                updated_at,
            ))?;
            indexed += 1;
        }

        writer.commit()?;

        Ok(IndexStats {
            indexed,
            deleted: 0,
        })
    }

    /// Streams rows through a **single** writer and **one** `commit` (migration / bulk reindex).
    ///
    /// Peak memory stays bounded by the iterator (one [`DocumentRow`] at a time) instead of materializing
    /// all tenants in memory. Returns [`IndexStats::indexed`] and the count of **distinct** `tenant_id` values written.
    pub fn index_rows_with_timestamps_stream(
        &self,
        rows: impl IntoIterator<Item = Result<(String, DocumentRow, i64, i64)>>,
    ) -> Result<(IndexStats, usize)> {
        let mut writer = self.index.writer(WRITER_HEAP_BYTES)?;
        let mut indexed = 0u64;
        let mut tenants = HashSet::new();

        for item in rows {
            let (tenant_id, doc, created_at, updated_at) = item?;
            if tenant_id.trim().is_empty() {
                return Err(SearchError::InvalidInput("tenant_id is empty".into()));
            }
            tenants.insert(tenant_id.clone());
            writer.delete_query(Box::new(self.make_tenant_doc_query(&tenant_id, doc.doc_id)))?;
            writer.add_document(self.build_tantivy_doc(
                &tenant_id,
                &doc,
                created_at,
                updated_at,
            ))?;
            indexed += 1;
        }

        writer.commit()?;

        let tenant_count = tenants.len();
        Ok((
            IndexStats {
                indexed,
                deleted: 0,
            },
            tenant_count,
        ))
    }

    pub fn search(
        &self,
        tenant_id: &str,
        query_str: &str,
        limit: u32,
        path_prefix: Option<&str>,
        required_tags: &[String],
    ) -> Result<(u64, Vec<SearchResultItem>)> {
        let query_str = query_str.trim();
        let has_path = path_prefix.is_some_and(|p| !p.is_empty() && p != "/");
        let has_tags = !required_tags.is_empty();
        if query_str.is_empty() && !has_path && !has_tags {
            return Err(SearchError::InvalidInput(
                "empty query: provide q and/or path and/or tags".into(),
            ));
        }

        let limit = limit.clamp(1, 50) as usize;
        let reader = self.index.reader()?;
        let searcher = reader.searcher();

        let tenant_query = TermQuery::new(
            Term::from_field_text(self.tenant_id_field, tenant_id),
            IndexRecordOption::Basic,
        );

        let full_query: Box<dyn Query> = if query_str.is_empty() {
            Box::new(AllQuery)
        } else {
            let parser = QueryParser::for_index(
                &self.index,
                vec![
                    self.title_field,
                    self.content_field,
                    self.summary_field,
                    self.note_field,
                ],
            );
            parser.parse_query(query_str)?
        };

        let mut clauses: Vec<(Occur, Box<dyn Query>)> = vec![
            (Occur::Must, Box::new(tenant_query)),
            (Occur::Must, full_query),
        ];

        if let Some(prefix) = path_prefix.filter(|p| !p.is_empty() && *p != "/") {
            // Tantivy FST regex does not accept a leading `^`; match full path terms by prefix + remainder.
            let pattern = format!("{}.*", regex::escape(prefix));
            let path_query = RegexQuery::from_pattern(&pattern, self.path_field)?;
            clauses.push((Occur::Must, Box::new(path_query)));
        }

        for tag in required_tags {
            let term = Term::from_field_text(self.tags_field, tag);
            clauses.push((
                Occur::Must,
                Box::new(TermQuery::new(term, IndexRecordOption::Basic)),
            ));
        }

        let query = BooleanQuery::new(clauses);

        let total_hits = query.count(&searcher)? as u64;
        let hits = searcher.search(&query, &TopDocs::with_limit(limit).order_by_score())?;

        let mut results = Vec::with_capacity(hits.len());
        for (score, doc_address) in hits {
            let retrieved = searcher.doc::<tantivy::TantivyDocument>(doc_address)?;

            let title = get_str_field(&retrieved, self.title_field);
            let content = get_str_field(&retrieved, self.content_field);
            let summary = get_str_field(&retrieved, self.summary_field);
            let note = get_str_field(&retrieved, self.note_field);
            let doc_url = get_str_field(&retrieved, self.doc_url_field);
            let path = get_str_field(&retrieved, self.path_field);
            let doc_id = get_u64_field(&retrieved, self.doc_id_field);
            let created_at = get_i64_field(&retrieved, self.created_timestamp_field);
            let updated_at = get_i64_field(&retrieved, self.update_timestamp_field);
            let tags = retrieved
                .get_all(self.tags_field)
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect::<Vec<_>>();

            let snippet_source = if !summary.trim().is_empty() {
                &summary
            } else if content.trim().is_empty() {
                &note
            } else {
                &content
            };
            let snippet = snippet_source.chars().take(220).collect::<String>();
            results.push(SearchResultItem {
                doc_id,
                title,
                doc_url,
                snippet,
                summary,
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

        let hits = searcher.search(
            &tenant_query,
            &TopDocs::with_limit(MAX_CATALOG_SCAN).order_by_score(),
        )?;

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

        let file_hits = searcher.search(
            &dir_query,
            &TopDocs::with_limit(MAX_CATALOG_FILES).order_by_score(),
        )?;
        let mut files: Vec<(u64, String)> = Vec::new();
        for (_, doc_address) in file_hits {
            let retrieved = searcher.doc::<tantivy::TantivyDocument>(doc_address)?;
            let doc_id = get_u64_field(&retrieved, self.doc_id_field);
            let title = get_str_field(&retrieved, self.title_field);
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
        let query = self.make_tenant_doc_query(tenant_id, doc_id);
        let hits = searcher.search(&query, &TopDocs::with_limit(1).order_by_score())?;
        let (_, doc_address) = match hits.into_iter().next() {
            Some(h) => h,
            None => return Ok(None),
        };

        let retrieved = searcher.doc::<tantivy::TantivyDocument>(doc_address)?;
        Ok(Some(StoredDocument {
            doc_id: get_u64_field(&retrieved, self.doc_id_field),
            title: get_str_field(&retrieved, self.title_field),
            content: get_str_field(&retrieved, self.content_field),
            summary: get_str_field(&retrieved, self.summary_field),
            doc_url: get_str_field(&retrieved, self.doc_url_field),
            path: get_str_field(&retrieved, self.path_field),
            note: get_str_field(&retrieved, self.note_field),
            tags: retrieved
                .get_all(self.tags_field)
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect(),
            created_at: get_i64_field(&retrieved, self.created_timestamp_field),
            updated_at: get_i64_field(&retrieved, self.update_timestamp_field),
        }))
    }

    /// Removes the document for `tenant_id` and `doc_id`. Returns `Ok(false)` when nothing matched.
    pub fn delete_document(&self, tenant_id: &str, doc_id: u64) -> Result<bool> {
        if tenant_id.trim().is_empty() {
            return Err(SearchError::InvalidInput("tenant_id is empty".into()));
        }

        let exists = {
            let reader = self.index.reader()?;
            let searcher = reader.searcher();
            self.existing_created_at(&searcher, tenant_id, doc_id)?
                .is_some()
        };

        if !exists {
            return Ok(false);
        }

        let mut writer = self
            .index
            .writer::<tantivy::TantivyDocument>(WRITER_HEAP_BYTES)?;
        writer.delete_query(Box::new(self.make_tenant_doc_query(tenant_id, doc_id)))?;
        writer.commit()?;

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::direct_subdir_under;
    use super::{DocumentRow, IndexManager};
    use tempfile::tempdir;

    #[test]
    fn search_filters_by_path_prefix_and_tags() {
        let dir = tempdir().expect("tempdir");
        let im = IndexManager::open_or_create(dir.path()).expect("open");
        let tenant = "t_search_filters";

        im.index_document(
            tenant,
            DocumentRow {
                doc_id: 1,
                title: "Alpha".to_string(),
                content: "body one".to_string(),
                summary: String::new(),
                doc_url: String::new(),
                tags: vec!["rust".to_string()],
                path: "/proj/".to_string(),
                note: String::new(),
            },
        )
        .expect("idx1");

        im.index_document(
            tenant,
            DocumentRow {
                doc_id: 2,
                title: "Beta".to_string(),
                content: "body two".to_string(),
                summary: String::new(),
                doc_url: String::new(),
                tags: vec!["rust".to_string(), "cli".to_string()],
                path: "/proj/sub/".to_string(),
                note: String::new(),
            },
        )
        .expect("idx2");

        im.index_document(
            tenant,
            DocumentRow {
                doc_id: 3,
                title: "Gamma".to_string(),
                content: "body three".to_string(),
                summary: String::new(),
                doc_url: String::new(),
                tags: vec![],
                path: "/other/".to_string(),
                note: String::new(),
            },
        )
        .expect("idx3");

        let (n, hits) = im
            .search(tenant, "body", 10, Some("/proj/"), &[])
            .expect("search path");
        assert_eq!(n, 2);
        assert_eq!(hits.len(), 2);

        let (_, hits2) = im
            .search(tenant, "", 10, Some("/proj/"), &["rust".to_string()])
            .expect("search tag+path");
        assert_eq!(hits2.len(), 2);

        let (_, hits3) = im
            .search(
                tenant,
                "",
                10,
                None,
                &["rust".to_string(), "cli".to_string()],
            )
            .expect("search tags and");
        assert_eq!(hits3.len(), 1);
        assert_eq!(hits3[0].doc_id, 2);
    }

    #[test]
    fn search_matches_chinese_text() {
        let dir = tempdir().expect("tempdir");
        let im = IndexManager::open_or_create(dir.path()).expect("open");
        let tenant = "t_search_cn";

        im.index_document(
            tenant,
            DocumentRow {
                doc_id: 1,
                title: "北京旅游笔记".to_string(),
                content: "天安门广场与故宫".to_string(),
                summary: String::new(),
                doc_url: String::new(),
                tags: vec![],
                path: "/".to_string(),
                note: String::new(),
            },
        )
        .expect("idx");

        let (n, hits) = im
            .search(tenant, "北京", 10, None, &[])
            .expect("search 北京");
        assert_eq!(n, 1, "jieba should tokenize 北京 in title");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].doc_id, 1);

        let (n2, hits2) = im
            .search(tenant, "故宫", 10, None, &[])
            .expect("search 故宫");
        assert_eq!(n2, 1);
        assert_eq!(hits2[0].doc_id, 1);
    }

    #[test]
    fn search_matches_summary_text() {
        let dir = tempdir().expect("tempdir");
        let im = IndexManager::open_or_create(dir.path()).expect("open");
        let tenant = "t_search_summary";

        im.index_document(
            tenant,
            DocumentRow {
                doc_id: 1,
                title: "Plain title".to_string(),
                content: "generic body text".to_string(),
                summary: "UniqueSummaryToken xyzabc for search".to_string(),
                doc_url: String::new(),
                tags: vec![],
                path: "/".to_string(),
                note: String::new(),
            },
        )
        .expect("idx");

        let (n, hits) = im
            .search(tenant, "UniqueSummaryToken", 10, None, &[])
            .expect("search summary");
        assert_eq!(n, 1);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].doc_id, 1);
    }

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

    #[test]
    fn delete_document_removes_and_reports_missing() {
        let dir = tempdir().expect("tempdir");
        let im = IndexManager::open_or_create(dir.path()).expect("open");
        let tenant = "t_delete";
        im.index_document(
            tenant,
            DocumentRow {
                doc_id: 42,
                title: "Hi".to_string(),
                content: "body".to_string(),
                summary: String::new(),
                doc_url: String::new(),
                tags: vec![],
                path: "/".to_string(),
                note: String::new(),
            },
        )
        .expect("idx");
        assert!(im.delete_document(tenant, 42).expect("del"));
        assert!(!im.delete_document(tenant, 42).expect("del again"));
        assert!(im.get_document(tenant, 42).expect("get").is_none());
    }
}
