use std::sync::{Arc, Mutex};

use inkly_search::IndexManager;
use inkly_summarize::Summarizer;

use crate::summary_queue::SummaryQueue;

#[derive(Clone)]
pub struct AppState {
    pub index: IndexManager,
    pub summarizer: Option<Arc<Mutex<Summarizer>>>,
    pub summary_queue: Option<Arc<SummaryQueue>>,
    expected_username: String,
    expected_password: String,
}

impl AppState {
    pub fn new(
        index: IndexManager,
        summarizer: Option<Summarizer>,
        summary_queue: Option<Arc<SummaryQueue>>,
        expected_username: String,
        expected_password: String,
    ) -> Arc<Self> {
        Arc::new(Self {
            index,
            summarizer: summarizer.map(|s| Arc::new(Mutex::new(s))),
            summary_queue,
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
