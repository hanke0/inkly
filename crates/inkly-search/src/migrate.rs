//! Offline migration of on-disk search storage (`version.data` + Tantivy `index/`).
//!
//! Currently supported: **data_version 2 → [`STORAGE_DATA_VERSION`](crate::storage_meta::STORAGE_DATA_VERSION)**  
//! (rebuild index with jieba tokenization while preserving `doc_id`, timestamps, and `auto_increment`).
//!
//! The live tree under `documents_root` is not modified until the new index is complete: a sibling
//! staging directory is filled first, then the old directory is renamed to
//! `{name}.old.{rfc3339-utc-with-colons-as-hyphens}.backup` and the staging directory is renamed into place.
//!
//! Copying is **streamed**: each legacy document is read and written in turn with a single Tantivy writer
//! and one `commit`, so peak memory does not include a full in-memory map of all rows (only the
//! [`DocAddress`](tantivy::DocAddress) list is collected for stable ordering).

use std::fs;
use std::path::{Path, PathBuf};

use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

use tantivy::collector::DocSetCollector;
use tantivy::query::AllQuery;
use tantivy::schema::{Field, Value};
use tantivy::{Index, TantivyDocument};

use crate::error::{Result, SearchError};
use crate::index_manager::{DocumentRow, IndexManager};
use crate::storage_meta::{self, VersionData};

/// `data_version` value this binary can migrate **from** (inclusive of export semantics).
pub const MIGRATE_FROM_DATA_VERSION: u32 = 2;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MigrateReport {
    /// `true` when `data_version` already matched the binary; no files were changed.
    pub noop: bool,
    pub documents_migrated: usize,
    pub tenant_count: usize,
    /// Where the pre-migrate `documents_root` directory was moved after a real migration (full tree backup).
    pub previous_data_backup: Option<PathBuf>,
}

fn get_str(doc: &TantivyDocument, field: Field) -> String {
    doc.get_first(field)
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}

fn get_u64(doc: &TantivyDocument, field: Field) -> u64 {
    doc.get_first(field).and_then(|v| v.as_u64()).unwrap_or(0)
}

fn get_i64(doc: &TantivyDocument, field: Field) -> i64 {
    doc.get_first(field).and_then(|v| v.as_i64()).unwrap_or(0)
}

fn require_field(schema: &tantivy::schema::Schema, name: &str) -> Result<Field> {
    schema.get_field(name).map_err(|_| {
        SearchError::InvalidInput(format!(
            "existing index schema is missing required field {name:?}; cannot migrate automatically"
        ))
    })
}

/// Field handles for reading a legacy index row (same names as [`IndexManager`] schema).
struct SourceDocFields {
    tenant_id: Field,
    doc_id: Field,
    doc_url: Field,
    title: Field,
    content: Field,
    summary: Field,
    created: Field,
    updated: Field,
    tags: Field,
    path: Field,
    note: Field,
}

impl SourceDocFields {
    fn resolve(schema: &tantivy::schema::Schema) -> Result<Self> {
        Ok(Self {
            tenant_id: require_field(schema, "tenant_id")?,
            doc_id: require_field(schema, "doc_id")?,
            doc_url: require_field(schema, "doc_url")?,
            title: require_field(schema, "title")?,
            content: require_field(schema, "content")?,
            summary: require_field(schema, "summary")?,
            created: require_field(schema, "created_timestamp")?,
            updated: require_field(schema, "update_timestamp")?,
            tags: require_field(schema, "tags")?,
            path: require_field(schema, "path")?,
            note: require_field(schema, "note")?,
        })
    }

    /// `Ok(None)` skips rows with an empty `tenant_id`.
    fn to_migration_tuple(
        &self,
        doc: &TantivyDocument,
    ) -> Result<Option<(String, DocumentRow, i64, i64)>> {
        let tenant_id = get_str(doc, self.tenant_id);
        if tenant_id.trim().is_empty() {
            return Ok(None);
        }

        let tags = doc
            .get_all(self.tags)
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect::<Vec<_>>();

        let row = DocumentRow {
            doc_id: get_u64(doc, self.doc_id),
            title: get_str(doc, self.title),
            content: get_str(doc, self.content),
            doc_url: get_str(doc, self.doc_url),
            summary: get_str(doc, self.summary),
            tags,
            path: get_str(doc, self.path),
            note: get_str(doc, self.note),
        };

        let created_at = get_i64(doc, self.created);
        let updated_at = get_i64(doc, self.updated);

        Ok(Some((tenant_id, row, created_at, updated_at)))
    }
}

fn index_dir_has_meta(index_dir: &Path) -> bool {
    index_dir.join("meta.json").is_file()
}

/// UTC timestamp in [RFC 3339](https://datatracker.ietf.org/doc/html/rfc3339) layout for directory names.
/// Colons in the time portion are replaced with `-` so the token is valid on Windows paths.
fn migrate_path_timestamp_token() -> Result<String> {
    let rfc = OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|e| SearchError::InvalidInput(format!("timestamp format error: {e}")))?;
    Ok(rfc.replace(':', "-"))
}

/// True when one path is a strict subdirectory of the other (same path returns false).
fn paths_strictly_nested(a: &Path, b: &Path) -> bool {
    fn strict_inside(needle: &Path, haystack: &Path) -> bool {
        needle
            .strip_prefix(haystack)
            .ok()
            .is_some_and(|rest| !rest.as_os_str().is_empty())
    }
    strict_inside(a, b) || strict_inside(b, a)
}

/// Ensures `staging` exists as a directory and is empty (or creates it if missing).
fn prepare_staging_dir(staging: &Path) -> Result<()> {
    if staging.exists() {
        if !staging.is_dir() {
            return Err(SearchError::InvalidInput(format!(
                "staging path exists and is not a directory: {}",
                staging.display()
            )));
        }
        let mut rd = fs::read_dir(staging).map_err(SearchError::IndexIO)?;
        if rd.next().is_some() {
            return Err(SearchError::InvalidInput(format!(
                "staging directory must be empty: {}",
                staging.display()
            )));
        }
    }
    fs::create_dir_all(staging).map_err(SearchError::IndexIO)?;
    Ok(())
}

/// Export from the existing v2 tree (read-only), build a v3 tree in a staging directory, then
/// atomically swap: rename live `documents_root` → `{name}.old.{rfc3339}.backup`, rename staging → `documents_root`.
///
/// `documents_root` is the same path passed to [`IndexManager::open_or_create`] (e.g. `$DATA_DIR/documents`).
/// Its **parent directory** must exist and must be on the same filesystem as `documents_root` (same-volume `rename`).
///
/// `staging_dir`: if `None`, a sibling directory `{parent}/{basename}.migrate.{rfc3339}` is used. If `Some`, that
/// exact path is used (created if missing; must be an empty directory if it already exists). It must not equal
/// `documents_root` or nest inside it (and vice versa). For the final `rename` into `documents_root`, prefer the
/// same filesystem as the live data (otherwise the OS may return `EXDEV`).
///
/// If `version.data` already matches [`STORAGE_DATA_VERSION`](crate::storage_meta::STORAGE_DATA_VERSION),
/// returns [`MigrateReport`] with `noop: true` and does not touch the filesystem.
///
/// Stop the API server before running (writers must not compete with this tool).
pub fn migrate_storage_to_current(
    documents_root: &Path,
    staging_dir: Option<&Path>,
) -> Result<MigrateReport> {
    let ver = storage_meta::read_version_data(documents_root)?;

    if ver.data_version == storage_meta::STORAGE_DATA_VERSION {
        return Ok(MigrateReport {
            noop: true,
            documents_migrated: 0,
            tenant_count: 0,
            previous_data_backup: None,
        });
    }

    if ver.data_version != MIGRATE_FROM_DATA_VERSION {
        return Err(SearchError::InvalidInput(format!(
            "this tool only migrates data_version {MIGRATE_FROM_DATA_VERSION} -> {}; found {}",
            storage_meta::STORAGE_DATA_VERSION,
            ver.data_version
        )));
    }

    let parent = documents_root.parent().ok_or_else(|| {
        SearchError::InvalidInput(
            "documents_root must have a parent directory (cannot migrate a filesystem root path)"
                .into(),
        )
    })?;

    let base = documents_root
        .file_name()
        .and_then(|n| n.to_str())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| {
            SearchError::InvalidInput(
                "documents_root must end with a non-empty directory name".into(),
            )
        })?;

    let ts = migrate_path_timestamp_token()?;

    let backup = parent.join(format!("{base}.old.{ts}.backup"));

    let staging: PathBuf = match staging_dir {
        Some(s) => {
            let s = s.to_path_buf();
            if s == documents_root {
                return Err(SearchError::InvalidInput(
                    "staging_dir must not be the same path as documents_root".into(),
                ));
            }
            if paths_strictly_nested(&s, documents_root) {
                return Err(SearchError::InvalidInput(format!(
                    "staging_dir must not be inside documents_root (or the reverse): {} <-> {}",
                    s.display(),
                    documents_root.display()
                )));
            }
            s
        }
        None => parent.join(format!("{base}.migrate.{ts}")),
    };

    if staging.exists() && staging_dir.is_none() {
        return Err(SearchError::InvalidInput(format!(
            "staging path already exists: {}; retry the migration",
            staging.display()
        )));
    }
    if backup.exists() {
        return Err(SearchError::InvalidInput(format!(
            "backup path already exists: {}; remove or rename it and retry",
            backup.display()
        )));
    }

    prepare_staging_dir(&staging)?;

    let build_result = (|| -> Result<(usize, usize)> {
        let index_dir = storage_meta::index_dir(documents_root);

        storage_meta::write_version_data(
            &staging,
            &VersionData {
                data_version: storage_meta::STORAGE_DATA_VERSION,
                auto_increment: ver.auto_increment,
            },
        )?;

        let mgr = IndexManager::open_or_create(&staging)?;

        let (stats, tenant_count) = if index_dir_has_meta(&index_dir) {
            let source_index = Index::open_in_dir(&index_dir)?;
            let fields = SourceDocFields::resolve(&source_index.schema())?;
            let reader = source_index.reader()?;
            let searcher = reader.searcher();
            let mut addresses: Vec<tantivy::DocAddress> = searcher
                .search(&AllQuery, &DocSetCollector)?
                .into_iter()
                .collect();
            addresses.sort();

            let row_stream = addresses.into_iter().filter_map(|addr| {
                let parsed: Result<Option<(String, DocumentRow, i64, i64)>> = (|| {
                    let doc = searcher.doc::<TantivyDocument>(addr)?;
                    fields.to_migration_tuple(&doc)
                })();
                match parsed {
                    Ok(None) => None,
                    Ok(Some(row)) => Some(Ok(row)),
                    Err(e) => Some(Err(e)),
                }
            });

            mgr.index_rows_with_timestamps_stream(row_stream)?
        } else {
            mgr.index_rows_with_timestamps_stream(std::iter::empty())?
        };

        Ok((tenant_count, stats.indexed as usize))
    })();

    let (tenant_count, documents_migrated) = match build_result {
        Ok(v) => v,
        Err(e) => {
            let _ = fs::remove_dir_all(&staging);
            return Err(e);
        }
    };

    fs::rename(documents_root, &backup).map_err(SearchError::IndexIO)?;

    if let Err(e) = fs::rename(&staging, documents_root) {
        if fs::rename(&backup, documents_root).is_err() {
            return Err(SearchError::InvalidInput(format!(
                "migrate failed after moving live data to {}; automatic rollback failed — restore manually by renaming that directory back to {}",
                backup.display(),
                documents_root.display()
            )));
        }
        let _ = fs::remove_dir_all(&staging);
        return Err(SearchError::IndexIO(e));
    }

    Ok(MigrateReport {
        noop: false,
        documents_migrated,
        tenant_count,
        previous_data_backup: Some(backup),
    })
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tantivy::doc;
    use tantivy::schema::{
        FAST, INDEXED, IndexRecordOption, STORED, STRING, TEXT, TextFieldIndexing, TextOptions,
    };
    use tempfile::tempdir;

    use super::*;
    use crate::index_manager::IndexManager;

    /// Schema matching inkly storage **data_version 2** (pre-jieba text fields).
    fn build_schema_v2() -> tantivy::schema::Schema {
        let mut builder = tantivy::schema::Schema::builder();
        let _ = builder.add_text_field("tenant_id", STRING | STORED);
        let _ = builder.add_u64_field("doc_id", INDEXED | FAST | STORED);
        let _ = builder.add_text_field("doc_url", STRING | STORED);
        let _ = builder.add_text_field("title", TEXT | STORED);
        let _ = builder.add_text_field("content", TEXT | STORED);
        let _ = builder.add_text_field("summary", TEXT | STORED);
        let _ = builder.add_i64_field("created_timestamp", STORED);
        let _ = builder.add_i64_field("update_timestamp", STORED);
        let _ = builder.add_text_field("tags", STRING | STORED);
        let path_opts = TextOptions::default()
            .set_indexing_options(
                TextFieldIndexing::default()
                    .set_tokenizer("raw")
                    .set_index_option(IndexRecordOption::Basic),
            )
            .set_stored();
        let _ = builder.add_text_field("path", path_opts);
        let _ = builder.add_text_field("note", TEXT | STORED);
        builder.build()
    }

    #[test]
    fn migrate_v2_to_current_preserves_docs_timestamps_and_search_cn() {
        let dir = tempdir().expect("tempdir");
        let root = dir.path();
        let idx_dir = storage_meta::index_dir(root);
        fs::create_dir_all(&idx_dir).expect("mkdir index");

        storage_meta::write_version_data(
            root,
            &VersionData {
                data_version: 2,
                auto_increment: 2000,
            },
        )
        .expect("write v2 version");

        let schema = build_schema_v2();
        let index = Index::create_in_dir(&idx_dir, schema).expect("create v2 index");
        let t = index.schema().get_field("tenant_id").unwrap();
        let doc_id = index.schema().get_field("doc_id").unwrap();
        let doc_url = index.schema().get_field("doc_url").unwrap();
        let title = index.schema().get_field("title").unwrap();
        let content = index.schema().get_field("content").unwrap();
        let summary = index.schema().get_field("summary").unwrap();
        let created = index.schema().get_field("created_timestamp").unwrap();
        let updated = index.schema().get_field("update_timestamp").unwrap();
        let path = index.schema().get_field("path").unwrap();
        let note = index.schema().get_field("note").unwrap();
        let tags = index.schema().get_field("tags").unwrap();

        let mut w = index.writer::<TantivyDocument>(50_000_000).expect("writer");
        let mut d = doc!(
            t => "tenant_a",
            doc_id => 7u64,
            doc_url => "https://ex/doc",
            title => "北京笔记",
            content => "故宫与天安门",
            summary => "",
            created => 100i64,
            updated => 200i64,
            path => "/p/",
            note => ""
        );
        d.add_text(tags, "x");
        w.add_document(d).expect("add");
        w.commit().expect("commit");

        let report = migrate_storage_to_current(root, None).expect("migrate");
        assert_eq!(report.documents_migrated, 1);
        assert_eq!(report.tenant_count, 1);
        assert!(!report.noop);
        let backup_path = report
            .previous_data_backup
            .as_ref()
            .expect("backup path after migrate");
        assert!(
            backup_path.is_dir(),
            "backup directory missing: {:?}",
            backup_path
        );
        let backup_ver = storage_meta::read_version_data(backup_path).expect("backup version");
        assert_eq!(backup_ver.data_version, 2);
        assert_eq!(backup_ver.auto_increment, 2000);

        let after = storage_meta::read_version_data(root).expect("read ver");
        assert_eq!(after.data_version, storage_meta::STORAGE_DATA_VERSION);
        assert_eq!(after.auto_increment, 2000);

        let im = IndexManager::open_or_create(root).expect("open new");
        let stored = im.get_document("tenant_a", 7).expect("get").expect("some");
        assert_eq!(stored.title, "北京笔记");
        assert_eq!(stored.content, "故宫与天安门");
        assert_eq!(stored.created_at, 100);
        assert_eq!(stored.updated_at, 200);
        assert_eq!(stored.tags, vec!["x".to_string()]);

        let (n, hits) = im
            .search("tenant_a", "北京", 10, None, &[])
            .expect("search cn");
        assert_eq!(n, 1);
        assert_eq!(hits[0].doc_id, 7);
    }

    #[test]
    fn migrate_with_explicit_staging_dir() {
        let dir = tempdir().expect("tempdir");
        let documents = dir.path().join("documents");
        let staging = dir.path().join("custom_staging");
        let idx_dir = storage_meta::index_dir(&documents);
        fs::create_dir_all(&idx_dir).expect("mkdir index");

        storage_meta::write_version_data(
            &documents,
            &VersionData {
                data_version: 2,
                auto_increment: 500,
            },
        )
        .expect("v2 version");

        let schema = build_schema_v2();
        let index = Index::create_in_dir(&idx_dir, schema).expect("idx");
        let t = index.schema().get_field("tenant_id").unwrap();
        let doc_id = index.schema().get_field("doc_id").unwrap();
        let doc_url = index.schema().get_field("doc_url").unwrap();
        let title = index.schema().get_field("title").unwrap();
        let content = index.schema().get_field("content").unwrap();
        let summary = index.schema().get_field("summary").unwrap();
        let created = index.schema().get_field("created_timestamp").unwrap();
        let updated = index.schema().get_field("update_timestamp").unwrap();
        let path = index.schema().get_field("path").unwrap();
        let note = index.schema().get_field("note").unwrap();
        let tags = index.schema().get_field("tags").unwrap();

        let mut w = index.writer::<TantivyDocument>(50_000_000).expect("writer");
        let mut d = doc!(
            t => "t1",
            doc_id => 1u64,
            doc_url => "",
            title => "T",
            content => "C",
            summary => "",
            created => 1i64,
            updated => 2i64,
            path => "/",
            note => ""
        );
        d.add_text(tags, "tag");
        w.add_document(d).expect("add");
        w.commit().expect("commit");

        assert!(!staging.exists());
        migrate_storage_to_current(&documents, Some(&staging)).expect("migrate");
        assert!(
            !staging.exists(),
            "staging path should be renamed into documents_root"
        );

        let im = IndexManager::open_or_create(&documents).expect("open");
        assert!(im.get_document("t1", 1).expect("get").is_some());
    }

    #[test]
    fn migrate_noop_when_already_current() {
        let dir = tempdir().expect("tempdir");
        let root = dir.path();
        storage_meta::write_version_data(
            root,
            &VersionData {
                data_version: storage_meta::STORAGE_DATA_VERSION,
                auto_increment: 1,
            },
        )
        .expect("write");
        let report = migrate_storage_to_current(root, None).expect("ok");
        assert!(report.noop);
        assert_eq!(report.documents_migrated, 0);
        assert_eq!(report.tenant_count, 0);
        assert!(report.previous_data_backup.is_none());
    }
}
