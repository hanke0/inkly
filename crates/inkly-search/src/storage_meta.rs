//! On-disk layout under `DATA_DIR`:
//! - `index/` — Tantivy files
//! - `version.data` — JSON `{ "data_version", "auto_increment" }` (`auto_increment` = next `doc_id` to hand out)

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{Result, SearchError};

/// Bump when on-disk semantics change; mismatch causes startup failure.
pub const STORAGE_DATA_VERSION: u32 = 2;

/// First value returned by `allocate_doc_id` on a fresh store (`auto_increment` in file).
pub const DEFAULT_AUTO_INCREMENT_NEXT: u64 = 1000;

const VERSION_FILE_NAME: &str = "version.data";
const INDEX_DIR_NAME: &str = "index";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VersionData {
    pub data_version: u32,
    /// Next document id to assign (MySQL-style AUTO_INCREMENT).
    pub auto_increment: u64,
}

pub fn index_dir(data_root: &Path) -> PathBuf {
    data_root.join(INDEX_DIR_NAME)
}

pub fn version_file_path(data_root: &Path) -> PathBuf {
    data_root.join(VERSION_FILE_NAME)
}

fn tantivy_index_present(index_dir: &Path) -> bool {
    index_dir.join("meta.json").is_file()
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
/// If a Tantivy tree exists under `index/` but `version.data` is missing, returns an error.
pub fn load_or_init_version_state(data_root: &Path) -> Result<VersionData> {
    let idx_dir = index_dir(data_root);
    let ver_path = version_file_path(data_root);

    fs::create_dir_all(&idx_dir)?;

    let index_present = tantivy_index_present(&idx_dir);
    let version_present = ver_path.is_file();

    if index_present && !version_present {
        return Err(SearchError::InvalidInput(format!(
            "search index exists under {} but {} is missing; add a compatible version file or remove the index directory",
            idx_dir.display(),
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
    fn existing_index_without_version_fails() {
        let dir = tempfile::tempdir().unwrap();
        let idx = index_dir(dir.path());
        fs::create_dir_all(&idx).unwrap();
        fs::write(idx.join("meta.json"), "{}").unwrap();
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
        assert!(matches!(
            err,
            SearchError::StorageVersionMismatch { .. }
        ));
    }
}
