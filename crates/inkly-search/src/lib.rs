pub mod error;
pub mod index_manager;
pub mod storage_meta;

pub use error::SearchError;
pub use index_manager::{
    CatalogListing, IndexManager, IndexStats, SearchResultItem, StoredDocument,
};
pub use storage_meta::STORAGE_DATA_VERSION;
