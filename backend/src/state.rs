use std::sync::Arc;

use inkly_search::IndexManager;

#[derive(Clone)]
pub struct AppState {
    pub index: IndexManager,
    pub jwt_secret: Arc<String>,
}

impl AppState {
    pub fn new(index: IndexManager, jwt_secret: String) -> Arc<Self> {
        Arc::new(Self {
            index,
            jwt_secret: Arc::new(jwt_secret),
        })
    }
}

