pub mod error;
pub mod index_manager;
pub mod migrate;
pub mod storage_meta;

pub use error::SearchError;
pub use index_manager::{
    CatalogListing, DocumentRow, IndexManager, IndexStats, SearchResultItem, StoredDocument,
};
pub use migrate::{migrate_storage_to_current, MigrateReport, MIGRATE_FROM_DATA_VERSION};
pub use storage_meta::STORAGE_DATA_VERSION;
