//! On-disk layout under `DATA_DIR/documents`:
//! - `db.sqlite3` — SQLite database (documents + FTS5 `simple` tokenizer)
//! - `version.data` — JSON `{ "data_version", "auto_increment" }` (`auto_increment` = next `doc_id` to hand out)
//!
//! Legacy (`data_version` 2/3/4): an `index/` directory with a Tantivy index instead of `db.sqlite3`.
//! The offline `migrate` tool rebuilds those trees into the SQLite layout.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{Result, SearchError};

/// Bump when on-disk semantics change; mismatch causes startup failure.
///
/// **5** — Storage is SQLite (`db.sqlite3`) with an FTS5 `simple` tokenizer for Chinese + pinyin search.
/// **4** — Tantivy: add indexed `path_ancestors` terms for fast directory-prefix filtering without regex.
/// **3** — Tantivy: text fields (`title`, `content`, `summary`, `note`) use the `jieba` tokenizer.
/// **2** — Tantivy: previous schema (default English-oriented tokenizer for those fields).
pub const STORAGE_DATA_VERSION: u32 = 5;

/// First value returned by `allocate_doc_id` on a fresh store (`auto_increment` in file).
pub const DEFAULT_AUTO_INCREMENT_NEXT: u64 = 1000;

const VERSION_FILE_NAME: &str = "version.data";
const SQLITE_FILE_NAME: &str = "db.sqlite3";
const LEGACY_INDEX_DIR_NAME: &str = "index";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VersionData {
    pub data_version: u32,
    /// Next document id to assign (MySQL-style AUTO_INCREMENT).
    pub auto_increment: u64,
}

pub fn sqlite_db_path(data_root: &Path) -> PathBuf {
    data_root.join(SQLITE_FILE_NAME)
}

pub fn legacy_index_dir(data_root: &Path) -> PathBuf {
    data_root.join(LEGACY_INDEX_DIR_NAME)
}

pub fn version_file_path(data_root: &Path) -> PathBuf {
    data_root.join(VERSION_FILE_NAME)
}

pub fn legacy_tantivy_index_present(index_dir: &Path) -> bool {
    index_dir.join("meta.json").is_file()
}

fn sqlite_file_present(data_root: &Path) -> bool {
    sqlite_db_path(data_root).is_file()
}

fn read_version_file(path: &Path) -> Result<VersionData> {
    let raw = fs::read_to_string(path)?;
    let v: VersionData = serde_json::from_str(&raw)?;
    Ok(v)
}

fn write_version_file(path: &Path, data: &VersionData) -> Result<()> {
    let raw = serde_json::to_string_pretty(data)?;
    fs::write(path, raw)?;
    Ok(())
}

/// Ensures `version.data` exists and matches [`STORAGE_DATA_VERSION`]. Creates it on a fresh tree.
///
/// Returns an error when an on-disk store is present (SQLite file or legacy Tantivy `index/`) but
/// `version.data` is missing — the operator must either add a compatible version file or remove the
/// storage artifact. For legacy stores (`data_version` < [`STORAGE_DATA_VERSION`]) this returns
/// [`SearchError::StorageVersionMismatch`]; callers should direct the user to run `inkly migrate`.
pub fn load_or_init_version_state(data_root: &Path) -> Result<VersionData> {
    let ver_path = version_file_path(data_root);
    let sqlite_path = sqlite_db_path(data_root);
    let legacy_index = legacy_index_dir(data_root);

    fs::create_dir_all(data_root)?;

    let storage_present =
        sqlite_file_present(data_root) || legacy_tantivy_index_present(&legacy_index);
    let version_present = ver_path.is_file();

    if storage_present && !version_present {
        return Err(SearchError::InvalidInput(format!(
            "search storage exists at {} (or legacy index at {}) but {} is missing; add a compatible version file or remove the storage artifact",
            sqlite_path.display(),
            legacy_index.display(),
            ver_path.display()
        )));
    }

    if !version_present {
        let initial = VersionData {
            data_version: STORAGE_DATA_VERSION,
            auto_increment: DEFAULT_AUTO_INCREMENT_NEXT,
        };
        write_version_file(&ver_path, &initial)?;
        return Ok(initial);
    }

    let loaded = read_version_file(&ver_path)?;
    if loaded.data_version != STORAGE_DATA_VERSION {
        return Err(SearchError::StorageVersionMismatch {
            expected: STORAGE_DATA_VERSION,
            found: loaded.data_version,
        });
    }

    Ok(loaded)
}

pub fn persist_auto_increment(version_path: &Path, data_version: u32, next: u64) -> Result<()> {
    write_version_file(
        version_path,
        &VersionData {
            data_version,
            auto_increment: next,
        },
    )
}

/// Read `version.data` without enforcing [`STORAGE_DATA_VERSION`] (for offline migration tools).
pub fn read_version_data(data_root: &Path) -> Result<VersionData> {
    read_version_file(&version_file_path(data_root))
}

/// Write `version.data` (used by migration after rebuilding storage).
pub fn write_version_data(data_root: &Path, data: &VersionData) -> Result<()> {
    write_version_file(&version_file_path(data_root), data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_dir_creates_version_file() {
        let dir = tempfile::tempdir().unwrap();
        let v = load_or_init_version_state(dir.path()).unwrap();
        assert_eq!(v.data_version, STORAGE_DATA_VERSION);
        assert_eq!(v.auto_increment, DEFAULT_AUTO_INCREMENT_NEXT);
        assert!(version_file_path(dir.path()).is_file());
    }

    #[test]
    fn existing_legacy_index_without_version_fails() {
        let dir = tempfile::tempdir().unwrap();
        let idx = legacy_index_dir(dir.path());
        fs::create_dir_all(&idx).unwrap();
        fs::write(idx.join("meta.json"), "{}").unwrap();
        let err = load_or_init_version_state(dir.path()).unwrap_err();
        assert!(matches!(err, SearchError::InvalidInput(_)));
    }

    #[test]
    fn existing_sqlite_without_version_fails() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(sqlite_db_path(dir.path()), b"").unwrap();
        let err = load_or_init_version_state(dir.path()).unwrap_err();
        assert!(matches!(err, SearchError::InvalidInput(_)));
    }

    #[test]
    fn version_mismatch_fails() {
        let dir = tempfile::tempdir().unwrap();
        write_version_file(
            &version_file_path(dir.path()),
            &VersionData {
                data_version: STORAGE_DATA_VERSION + 99,
                auto_increment: 1,
            },
        )
        .unwrap();
        let err = load_or_init_version_state(dir.path()).unwrap_err();
        assert!(matches!(err, SearchError::StorageVersionMismatch { .. }));
    }
}
