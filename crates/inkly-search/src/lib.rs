pub mod error;
pub mod index_manager;

pub use error::SearchError;
pub use index_manager::{
    CatalogListing, IndexManager, IndexStats, SearchResultItem, StoredDocument,
};
