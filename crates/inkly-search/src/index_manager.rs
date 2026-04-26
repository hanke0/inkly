//! SQLite + FTS5 backed document store and full-text search.
//!
//! Layout inside `data_root`:
//! - `db.sqlite3` — primary store (documents + tags + FTS5 index)
//! - `version.data` — JSON `{ data_version, auto_increment }` (see [`crate::storage_meta`])
//!
//! The FTS5 virtual table (`documents_fts`) uses the `simple` tokenizer from
//! [`sqlite-simple-tokenizer`](https://crates.io/crates/sqlite-simple-tokenizer), which indexes
//! Chinese characters individually (with pinyin expansion) and applies English stemming. Queries
//! are built via the `simple_query()` SQL function so that user-entered Chinese / pinyin / English
//! text always produces a valid FTS5 MATCH expression.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};

use rusqlite::{Connection, OptionalExtension, Transaction, params};

use crate::error::{Result, SearchError};
use crate::storage_meta;

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

const MAX_CATALOG_FILES: usize = 5_000;
const SEARCH_SNIPPET_CHARS: usize = 220;

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

/// Escape `%`, `_`, and the escape character itself for use inside a `LIKE` pattern.
/// Queries must include `ESCAPE '\\'` to match the escape rule.
fn escape_like(pattern: &str) -> String {
    let mut out = String::with_capacity(pattern.len());
    for ch in pattern.chars() {
        if matches!(ch, '\\' | '%' | '_') {
            out.push('\\');
        }
        out.push(ch);
    }
    out
}

// ---------------------------------------------------------------------------

struct Inner {
    conn: Connection,
    version_path: PathBuf,
    data_version: u32,
    /// Next `doc_id` to assign.
    next: u64,
}

#[derive(Clone)]
pub struct IndexManager {
    inner: Arc<Mutex<Inner>>,
}

impl IndexManager {
    fn now_unix_seconds() -> Result<i64> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| SearchError::InvalidInput(format!("time went backwards: {e}")))?
            .as_secs() as i64;
        Ok(now)
    }

    fn lock(&self) -> Result<MutexGuard<'_, Inner>> {
        self.inner.lock().map_err(|_| SearchError::LockPoisoned)
    }

    /// Opens the SQLite store under `data_root/db.sqlite3` and reads `data_root/version.data`.
    /// Creates both on a fresh tree.
    pub fn open_or_create<P: AsRef<Path>>(data_root: P) -> Result<Self> {
        let data_root = data_root.as_ref();
        let version_state = storage_meta::load_or_init_version_state(data_root)?;
        let db_path = storage_meta::sqlite_db_path(data_root);
        let version_path = storage_meta::version_file_path(data_root);

        let conn = open_and_prepare_connection(&db_path)?;

        Ok(Self {
            inner: Arc::new(Mutex::new(Inner {
                conn,
                version_path,
                data_version: version_state.data_version,
                next: version_state.auto_increment,
            })),
        })
    }

    /// Returns the next server-assigned `doc_id` and persists it to `version.data`.
    pub fn allocate_doc_id(&self) -> Result<u64> {
        let mut guard = self.lock()?;
        let id = guard.next;
        let new_next = id
            .checked_add(1)
            .ok_or_else(|| SearchError::InvalidInput("auto_increment overflow".into()))?;
        storage_meta::persist_auto_increment(&guard.version_path, guard.data_version, new_next)?;
        guard.next = new_next;
        Ok(id)
    }

    pub fn index_document(&self, tenant_id: &str, doc: DocumentRow) -> Result<IndexStats> {
        if tenant_id.trim().is_empty() {
            return Err(SearchError::InvalidInput("tenant_id is empty".into()));
        }
        if doc.doc_id == 0 {
            return Err(SearchError::InvalidInput("doc_id is 0".into()));
        }

        let now = Self::now_unix_seconds()?;
        let mut guard = self.lock()?;
        let tx = guard.conn.transaction().map_err(SearchError::Sqlite)?;
        upsert_document(&tx, tenant_id, &doc, None, now)?;
        tx.commit().map_err(SearchError::Sqlite)?;

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
        let mut guard = self.lock()?;
        let tx = guard.conn.transaction().map_err(SearchError::Sqlite)?;

        let mut indexed = 0u64;
        for doc in docs {
            if doc.doc_id == 0 {
                return Err(SearchError::InvalidInput("doc_id is 0".into()));
            }
            upsert_document(&tx, tenant_id, &doc, None, now)?;
            indexed += 1;
        }

        tx.commit().map_err(SearchError::Sqlite)?;
        Ok(IndexStats {
            indexed,
            deleted: 0,
        })
    }

    /// Like [`index_documents`](Self::index_documents) but keeps the given Unix timestamps
    /// (storage migration / reindex).
    pub fn index_documents_with_timestamps(
        &self,
        tenant_id: &str,
        docs: impl IntoIterator<Item = (DocumentRow, i64, i64)>,
    ) -> Result<IndexStats> {
        if tenant_id.trim().is_empty() {
            return Err(SearchError::InvalidInput("tenant_id is empty".into()));
        }

        let mut guard = self.lock()?;
        let tx = guard.conn.transaction().map_err(SearchError::Sqlite)?;

        let mut indexed = 0u64;
        for (doc, created_at, updated_at) in docs {
            if doc.doc_id == 0 {
                return Err(SearchError::InvalidInput("doc_id is 0".into()));
            }
            upsert_document(&tx, tenant_id, &doc, Some(created_at), updated_at)?;
            indexed += 1;
        }

        tx.commit().map_err(SearchError::Sqlite)?;
        Ok(IndexStats {
            indexed,
            deleted: 0,
        })
    }

    /// Streams rows through a **single** transaction and **one** `commit` (migration / bulk reindex).
    ///
    /// Peak memory stays bounded by the iterator (one [`DocumentRow`] at a time). Returns
    /// [`IndexStats::indexed`] and the count of **distinct** `tenant_id` values written.
    pub fn index_rows_with_timestamps_stream(
        &self,
        rows: impl IntoIterator<Item = Result<(String, DocumentRow, i64, i64)>>,
    ) -> Result<(IndexStats, usize)> {
        let mut guard = self.lock()?;
        let tx = guard.conn.transaction().map_err(SearchError::Sqlite)?;

        let mut indexed = 0u64;
        let mut tenants = HashSet::new();

        for item in rows {
            let (tenant_id, doc, created_at, updated_at) = item?;
            if tenant_id.trim().is_empty() {
                return Err(SearchError::InvalidInput("tenant_id is empty".into()));
            }
            // `doc_id == 0` is rejected by the live ingest paths but legacy on-disk data
            // can still contain rows with id 0 (some pre-1.0 builds allocated ids starting
            // from 0). Migration must preserve them verbatim — the SQLite store uses
            // `(tenant_id, doc_id)` as a uniqueness key with no positivity constraint.
            upsert_document(&tx, &tenant_id, &doc, Some(created_at), updated_at)?;
            tenants.insert(tenant_id);
            indexed += 1;
        }

        tx.commit().map_err(SearchError::Sqlite)?;

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
        let guard = self.lock()?;
        search_impl(
            &guard.conn,
            tenant_id,
            query_str,
            limit,
            path_prefix,
            required_tags,
        )
    }

    pub fn catalog_list(&self, tenant_id: &str, dir_path: &str) -> Result<CatalogListing> {
        let guard = self.lock()?;
        catalog_list_impl(&guard.conn, tenant_id, dir_path)
    }

    pub fn get_document(&self, tenant_id: &str, doc_id: u64) -> Result<Option<StoredDocument>> {
        let guard = self.lock()?;
        get_document_impl(&guard.conn, tenant_id, doc_id)
    }

    /// Removes the document for `tenant_id` and `doc_id`. Returns `Ok(false)` when nothing matched.
    pub fn delete_document(&self, tenant_id: &str, doc_id: u64) -> Result<bool> {
        if tenant_id.trim().is_empty() {
            return Err(SearchError::InvalidInput("tenant_id is empty".into()));
        }

        let mut guard = self.lock()?;
        let tx = guard.conn.transaction().map_err(SearchError::Sqlite)?;

        // ON DELETE CASCADE removes tag rows; the FTS5 delete trigger fires on the `documents` row.
        let n = tx
            .execute(
                "DELETE FROM documents WHERE tenant_id = ?1 AND doc_id = ?2",
                params![tenant_id, doc_id as i64],
            )
            .map_err(SearchError::Sqlite)?;

        tx.commit().map_err(SearchError::Sqlite)?;
        Ok(n > 0)
    }
}

// ---------------------------------------------------------------------------
// Connection setup
// ---------------------------------------------------------------------------

fn open_and_prepare_connection(db_path: &Path) -> Result<Connection> {
    let conn = Connection::open(db_path).map_err(SearchError::Sqlite)?;

    // Register the `simple_tokenizer` (FTS5 `tokenize='simple'`) + helper SQL functions
    // (`simple_query`, `simple_highlight`, …) on *this* connection. The upstream error type is
    // `rusqlite_ext::error::Error`, which we surface as a generic input error.
    sqlite_simple_tokenizer::load(&conn)
        .map_err(|e| SearchError::InvalidInput(format!("failed to load simple tokenizer: {e}")))?;

    conn.execute_batch(
        "PRAGMA journal_mode=WAL;
         PRAGMA busy_timeout=5000;
         PRAGMA foreign_keys=ON;

         CREATE TABLE IF NOT EXISTS documents (
           id         INTEGER PRIMARY KEY AUTOINCREMENT,
           tenant_id  TEXT NOT NULL,
           doc_id     INTEGER NOT NULL,
           title      TEXT NOT NULL DEFAULT '',
           content    TEXT NOT NULL DEFAULT '',
           summary    TEXT NOT NULL DEFAULT '',
           note       TEXT NOT NULL DEFAULT '',
           doc_url    TEXT NOT NULL DEFAULT '',
           path       TEXT NOT NULL DEFAULT '/',
           created_at INTEGER NOT NULL,
           updated_at INTEGER NOT NULL
         );

         CREATE UNIQUE INDEX IF NOT EXISTS idx_documents_tenant_doc
           ON documents(tenant_id, doc_id);

         CREATE INDEX IF NOT EXISTS idx_documents_tenant_path
           ON documents(tenant_id, path);

         CREATE TABLE IF NOT EXISTS document_tags (
           id  INTEGER NOT NULL,
           tag TEXT NOT NULL,
           PRIMARY KEY (id, tag),
           FOREIGN KEY (id) REFERENCES documents(id) ON DELETE CASCADE
         );

         CREATE INDEX IF NOT EXISTS idx_document_tags_tag
           ON document_tags(tag);

         CREATE VIRTUAL TABLE IF NOT EXISTS documents_fts USING fts5(
           title, content, summary, note,
           content='documents',
           content_rowid='id',
           -- `disable_stopword`: the default stopword list drops short pinyin tokens like
           -- `bi`/`lu`/`you` at query time, which breaks Chinese searches such as 笔记/旅游
           -- (their characters convert to those pinyins during indexing). Keeping every token
           -- is also the right default for a document search app.
           tokenize='simple disable_stopword'
         );

         CREATE TRIGGER IF NOT EXISTS documents_ai AFTER INSERT ON documents BEGIN
           INSERT INTO documents_fts(rowid, title, content, summary, note)
           VALUES (new.id, new.title, new.content, new.summary, new.note);
         END;

         CREATE TRIGGER IF NOT EXISTS documents_ad AFTER DELETE ON documents BEGIN
           INSERT INTO documents_fts(documents_fts, rowid, title, content, summary, note)
           VALUES ('delete', old.id, old.title, old.content, old.summary, old.note);
         END;

         CREATE TRIGGER IF NOT EXISTS documents_au AFTER UPDATE ON documents BEGIN
           INSERT INTO documents_fts(documents_fts, rowid, title, content, summary, note)
           VALUES ('delete', old.id, old.title, old.content, old.summary, old.note);
           INSERT INTO documents_fts(rowid, title, content, summary, note)
           VALUES (new.id, new.title, new.content, new.summary, new.note);
         END;",
    )
    .map_err(SearchError::Sqlite)?;

    Ok(conn)
}

// ---------------------------------------------------------------------------
// Write helpers
// ---------------------------------------------------------------------------

/// Insert or update the document row and its tags. Triggers sync `documents_fts` automatically.
///
/// `override_created_at`: when `Some`, sets `created_at` to that value (migration path); when `None`,
/// keeps the existing row's `created_at` on update or uses `updated_at` on insert.
fn upsert_document(
    tx: &Transaction<'_>,
    tenant_id: &str,
    doc: &DocumentRow,
    override_created_at: Option<i64>,
    updated_at: i64,
) -> Result<()> {
    let existing: Option<(i64, i64)> = tx
        .query_row(
            "SELECT id, created_at FROM documents WHERE tenant_id = ?1 AND doc_id = ?2",
            params![tenant_id, doc.doc_id as i64],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)),
        )
        .optional()
        .map_err(SearchError::Sqlite)?;

    let id: i64 = match existing {
        Some((id, existing_created)) => {
            let created_at = override_created_at.unwrap_or(existing_created);
            tx.execute(
                "UPDATE documents
                    SET title = ?1,
                        content = ?2,
                        summary = ?3,
                        note = ?4,
                        doc_url = ?5,
                        path = ?6,
                        created_at = ?7,
                        updated_at = ?8
                  WHERE id = ?9",
                params![
                    doc.title,
                    doc.content,
                    doc.summary,
                    doc.note,
                    doc.doc_url,
                    doc.path,
                    created_at,
                    updated_at,
                    id,
                ],
            )
            .map_err(SearchError::Sqlite)?;
            tx.execute("DELETE FROM document_tags WHERE id = ?1", params![id])
                .map_err(SearchError::Sqlite)?;
            id
        }
        None => {
            let created_at = override_created_at.unwrap_or(updated_at);
            tx.execute(
                "INSERT INTO documents (tenant_id, doc_id, title, content, summary, note, doc_url, path, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    tenant_id,
                    doc.doc_id as i64,
                    doc.title,
                    doc.content,
                    doc.summary,
                    doc.note,
                    doc.doc_url,
                    doc.path,
                    created_at,
                    updated_at,
                ],
            )
            .map_err(SearchError::Sqlite)?;
            tx.last_insert_rowid()
        }
    };

    if !doc.tags.is_empty() {
        let mut stmt = tx
            .prepare_cached("INSERT OR IGNORE INTO document_tags (id, tag) VALUES (?1, ?2)")
            .map_err(SearchError::Sqlite)?;
        for tag in &doc.tags {
            stmt.execute(params![id, tag])
                .map_err(SearchError::Sqlite)?;
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Read helpers
// ---------------------------------------------------------------------------

fn load_tags(conn: &Connection, id: i64) -> Result<Vec<String>> {
    let mut stmt = conn
        .prepare_cached("SELECT tag FROM document_tags WHERE id = ?1 ORDER BY tag ASC")
        .map_err(SearchError::Sqlite)?;
    let rows = stmt
        .query_map(params![id], |r| r.get::<_, String>(0))
        .map_err(SearchError::Sqlite)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r.map_err(SearchError::Sqlite)?);
    }
    Ok(out)
}

fn get_document_impl(
    conn: &Connection,
    tenant_id: &str,
    doc_id: u64,
) -> Result<Option<StoredDocument>> {
    let mut stmt = conn
        .prepare_cached(
            "SELECT id, doc_id, title, content, summary, doc_url, path, note, created_at, updated_at
               FROM documents
              WHERE tenant_id = ?1 AND doc_id = ?2",
        )
        .map_err(SearchError::Sqlite)?;

    let row = stmt
        .query_row(params![tenant_id, doc_id as i64], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, String>(5)?,
                r.get::<_, String>(6)?,
                r.get::<_, String>(7)?,
                r.get::<_, i64>(8)?,
                r.get::<_, i64>(9)?,
            ))
        })
        .optional()
        .map_err(SearchError::Sqlite)?;

    let Some((id, doc_id_i, title, content, summary, doc_url, path, note, created_at, updated_at)) =
        row
    else {
        return Ok(None);
    };

    let tags = load_tags(conn, id)?;

    Ok(Some(StoredDocument {
        doc_id: doc_id_i as u64,
        title,
        content,
        summary,
        doc_url,
        path,
        note,
        tags,
        created_at,
        updated_at,
    }))
}

fn build_snippet(summary: &str, content: &str, note: &str) -> String {
    let source = if !summary.trim().is_empty() {
        summary
    } else if content.trim().is_empty() {
        note
    } else {
        content
    };
    source.chars().take(SEARCH_SNIPPET_CHARS).collect()
}

fn search_impl(
    conn: &Connection,
    tenant_id: &str,
    query_str: &str,
    limit: usize,
    path_prefix: Option<&str>,
    required_tags: &[String],
) -> Result<(u64, Vec<SearchResultItem>)> {
    let mut where_clauses: Vec<String> = vec!["d.tenant_id = ?".to_string()];
    let mut bind_values: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(tenant_id.to_string())];

    let fts = !query_str.is_empty();
    if fts {
        where_clauses.push("documents_fts MATCH simple_query(?)".to_string());
        bind_values.push(Box::new(query_str.to_string()));
    }

    if let Some(prefix) = path_prefix
        && !prefix.is_empty()
        && prefix != "/"
    {
        // `prefix` is a canonical directory path like `/a/b/`; any document whose path starts
        // with it belongs to that subtree.
        where_clauses.push("d.path LIKE ? ESCAPE '\\'".to_string());
        bind_values.push(Box::new(format!("{}%", escape_like(prefix))));
    }

    for tag in required_tags {
        where_clauses.push("d.id IN (SELECT id FROM document_tags WHERE tag = ?)".to_string());
        bind_values.push(Box::new(tag.clone()));
    }

    let where_sql = where_clauses.join(" AND ");

    // Count query reuses the same filters.
    let count_sql = if fts {
        format!(
            "SELECT COUNT(*) FROM documents_fts JOIN documents d ON d.id = documents_fts.rowid WHERE {where_sql}"
        )
    } else {
        format!("SELECT COUNT(*) FROM documents d WHERE {where_sql}")
    };

    let total_hits: i64 = {
        let mut stmt = conn.prepare(&count_sql).map_err(SearchError::Sqlite)?;
        let params_vec: Vec<&dyn rusqlite::ToSql> =
            bind_values.iter().map(|b| b.as_ref()).collect();
        stmt.query_row(params_vec.as_slice(), |r| r.get::<_, i64>(0))
            .map_err(SearchError::Sqlite)?
    };

    // Results query: bm25 is a non-positive "cost" (lower = better); we invert it so higher = better.
    let select_sql = if fts {
        format!(
            "SELECT d.id, d.doc_id, d.title, d.content, d.summary, d.note, d.doc_url, d.path,
                    d.created_at, d.updated_at, -bm25(documents_fts) AS score
               FROM documents_fts
               JOIN documents d ON d.id = documents_fts.rowid
              WHERE {where_sql}
              ORDER BY score DESC
              LIMIT ?"
        )
    } else {
        format!(
            "SELECT d.id, d.doc_id, d.title, d.content, d.summary, d.note, d.doc_url, d.path,
                    d.created_at, d.updated_at, 0.0 AS score
               FROM documents d
              WHERE {where_sql}
              ORDER BY d.updated_at DESC
              LIMIT ?"
        )
    };

    bind_values.push(Box::new(limit as i64));

    let mut stmt = conn.prepare(&select_sql).map_err(SearchError::Sqlite)?;
    let params_vec: Vec<&dyn rusqlite::ToSql> = bind_values.iter().map(|b| b.as_ref()).collect();
    let rows = stmt
        .query_map(params_vec.as_slice(), |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, String>(5)?,
                r.get::<_, String>(6)?,
                r.get::<_, String>(7)?,
                r.get::<_, i64>(8)?,
                r.get::<_, i64>(9)?,
                r.get::<_, f64>(10)?,
            ))
        })
        .map_err(SearchError::Sqlite)?;

    let mut results = Vec::with_capacity(limit.min(16));
    for r in rows {
        let (
            id,
            doc_id_i,
            title,
            content,
            summary,
            note,
            doc_url,
            path,
            created_at,
            updated_at,
            score,
        ) = r.map_err(SearchError::Sqlite)?;
        let tags = load_tags(conn, id)?;
        let snippet = build_snippet(&summary, &content, &note);
        results.push(SearchResultItem {
            doc_id: doc_id_i as u64,
            title,
            doc_url,
            snippet,
            summary,
            score: score as f32,
            created_at,
            updated_at,
            tags,
            path,
            note,
        });
    }

    Ok((total_hits as u64, results))
}

fn catalog_list_impl(conn: &Connection, tenant_id: &str, dir_path: &str) -> Result<CatalogListing> {
    // Distinct paths inside the subtree to compute immediate subdirectories.
    let (path_sql, path_params): (String, Vec<Box<dyn rusqlite::ToSql>>) = if dir_path == "/" {
        (
            "SELECT DISTINCT path FROM documents WHERE tenant_id = ?1".to_string(),
            vec![Box::new(tenant_id.to_string())],
        )
    } else {
        (
            "SELECT DISTINCT path FROM documents WHERE tenant_id = ?1 AND path LIKE ?2 ESCAPE '\\'"
                .to_string(),
            vec![
                Box::new(tenant_id.to_string()),
                Box::new(format!("{}%", escape_like(dir_path))),
            ],
        )
    };

    let mut stmt = conn.prepare(&path_sql).map_err(SearchError::Sqlite)?;
    let params_vec: Vec<&dyn rusqlite::ToSql> = path_params.iter().map(|b| b.as_ref()).collect();
    let rows = stmt
        .query_map(params_vec.as_slice(), |r| r.get::<_, String>(0))
        .map_err(SearchError::Sqlite)?;

    let mut subdir_paths: HashSet<String> = HashSet::new();
    for r in rows {
        let p = r.map_err(SearchError::Sqlite)?;
        if let Some(child) = direct_subdir_under(dir_path, &p) {
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

    // Files directly under `dir_path`.
    let mut file_stmt = conn
        .prepare(
            "SELECT doc_id, title
               FROM documents
              WHERE tenant_id = ?1 AND path = ?2
              ORDER BY LOWER(title) ASC
              LIMIT ?3",
        )
        .map_err(SearchError::Sqlite)?;
    let file_rows = file_stmt
        .query_map(
            params![tenant_id, dir_path, MAX_CATALOG_FILES as i64],
            |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)),
        )
        .map_err(SearchError::Sqlite)?;

    let mut files: Vec<(u64, String)> = Vec::new();
    for r in file_rows {
        let (doc_id, title) = r.map_err(SearchError::Sqlite)?;
        if doc_id != 0 {
            files.push((doc_id as u64, title));
        }
    }

    Ok(CatalogListing {
        path: dir_path.to_string(),
        subdirs,
        files,
    })
}

#[cfg(test)]
mod tests {
    use super::{DocumentRow, IndexManager, direct_subdir_under};
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
        assert_eq!(n, 1, "simple should tokenize 北京 in title");
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
    fn update_document_preserves_created_at() {
        let dir = tempdir().expect("tempdir");
        let im = IndexManager::open_or_create(dir.path()).expect("open");
        let tenant = "t_updates";

        im.index_document(
            tenant,
            DocumentRow {
                doc_id: 1,
                title: "v1".to_string(),
                content: "c1".to_string(),
                summary: String::new(),
                doc_url: String::new(),
                tags: vec![],
                path: "/".to_string(),
                note: String::new(),
            },
        )
        .expect("insert");
        let first = im.get_document(tenant, 1).expect("get").expect("some");

        std::thread::sleep(std::time::Duration::from_millis(1100));

        im.index_document(
            tenant,
            DocumentRow {
                doc_id: 1,
                title: "v2".to_string(),
                content: "c2".to_string(),
                summary: String::new(),
                doc_url: String::new(),
                tags: vec![],
                path: "/".to_string(),
                note: String::new(),
            },
        )
        .expect("update");

        let second = im.get_document(tenant, 1).expect("get2").expect("some2");
        assert_eq!(second.title, "v2");
        assert_eq!(second.created_at, first.created_at);
        assert!(second.updated_at >= first.updated_at);
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

    #[test]
    fn catalog_lists_subdirs_and_files() {
        let dir = tempdir().expect("tempdir");
        let im = IndexManager::open_or_create(dir.path()).expect("open");
        let tenant = "t_catalog";

        for (doc_id, title, path) in [
            (1u64, "root-a", "/"),
            (2, "root-b", "/"),
            (3, "x-file", "/x/"),
            (4, "y-file", "/x/y/"),
            (5, "z-file", "/z/"),
        ] {
            im.index_document(
                tenant,
                DocumentRow {
                    doc_id,
                    title: title.to_string(),
                    content: String::new(),
                    summary: String::new(),
                    doc_url: String::new(),
                    tags: vec![],
                    path: path.to_string(),
                    note: String::new(),
                },
            )
            .expect("idx");
        }

        let root = im.catalog_list(tenant, "/").expect("list /");
        let root_subdir_names: Vec<&str> = root.subdirs.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(root_subdir_names, vec!["x", "z"]);
        let root_files: Vec<&str> = root.files.iter().map(|(_, t)| t.as_str()).collect();
        assert_eq!(root_files, vec!["root-a", "root-b"]);

        let x = im.catalog_list(tenant, "/x/").expect("list /x/");
        assert_eq!(
            x.subdirs
                .iter()
                .map(|(n, _)| n.as_str())
                .collect::<Vec<_>>(),
            vec!["y"]
        );
        assert_eq!(
            x.files.iter().map(|(_, t)| t.as_str()).collect::<Vec<_>>(),
            vec!["x-file"]
        );
    }
}
