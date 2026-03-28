use std::sync::Arc;

use inkly_search::IndexManager;

#[derive(Clone)]
pub struct AppState {
    pub index: IndexManager,
    expected_username: String,
    expected_password: String,
}

impl AppState {
    pub fn new(index: IndexManager, expected_username: String, expected_password: String) -> Arc<Self> {
        Arc::new(Self {
            index,
            expected_username,
            expected_password,
        })
    }

    pub fn expected_username(&self) -> &str {
        &self.expected_username
    }

    pub fn expected_password(&self) -> &str {
        &self.expected_password
    }
}
