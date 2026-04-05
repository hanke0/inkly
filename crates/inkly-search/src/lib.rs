pub mod error;
pub mod index_manager;
pub mod migrate;
pub mod storage_meta;

pub use error::SearchError;
pub use index_manager::{
    CatalogListing, DocumentRow, IndexManager, IndexStats, SearchResultItem, StoredDocument,
};
pub use migrate::{MIGRATE_FROM_DATA_VERSION, MigrateReport, migrate_storage_to_current};
pub use storage_meta::STORAGE_DATA_VERSION;
