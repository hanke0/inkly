use std::sync::{Arc, Mutex};

use inkly_search::IndexManager;
use inkly_summarize::Summarizer;

#[derive(Clone)]
pub struct AppState {
    pub index: IndexManager,
    pub summarizer: Arc<Mutex<Summarizer>>,
    expected_username: String,
    expected_password: String,
}

impl AppState {
    pub fn new(
        index: IndexManager,
        summarizer: Summarizer,
        expected_username: String,
        expected_password: String,
    ) -> Arc<Self> {
        Arc::new(Self {
            index,
            summarizer: Arc::new(Mutex::new(summarizer)),
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
