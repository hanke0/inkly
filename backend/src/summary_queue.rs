//! Persistent FIFO queue for document summarization (SQLite).
//! One background thread processes jobs sequentially; survives process restarts.

use std::path::Path;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

use inkly_search::DocumentRow;
use rusqlite::{Connection, params};
use tracing::{info, warn};

use crate::state::AppState;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EnqueueOutcome {
    Enqueued,
    AlreadyQueued,
}

pub struct SummaryQueue {
    conn: Mutex<Connection>,
    signal: Condvar,
    wait_lock: Mutex<()>,
}

impl SummaryQueue {
    pub fn open(path: &Path) -> Result<Self, rusqlite::Error> {
        let conn = Connection::open(path)?;
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA busy_timeout=5000;
             CREATE TABLE IF NOT EXISTS pending_summary (
               id INTEGER PRIMARY KEY AUTOINCREMENT,
               tenant_id TEXT NOT NULL,
               doc_id INTEGER NOT NULL,
               created_at INTEGER NOT NULL,
               UNIQUE(tenant_id, doc_id)
             );",
        )?;
        Ok(Self {
            conn: Mutex::new(conn),
            signal: Condvar::new(),
            wait_lock: Mutex::new(()),
        })
    }

    /// Returns whether a new row was inserted (`Enqueued`) or the pair was already queued.
    pub fn enqueue(&self, tenant_id: &str, doc_id: u64) -> Result<EnqueueOutcome, rusqlite::Error> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        let g = self.conn_lock();
        let n = g.execute(
            "INSERT OR IGNORE INTO pending_summary (tenant_id, doc_id, created_at) VALUES (?1, ?2, ?3)",
            params![tenant_id, doc_id as i64, now],
        )?;
        Ok(if n > 0 {
            // Wake the worker immediately instead of waiting for polling interval.
            self.signal.notify_one();
            EnqueueOutcome::Enqueued
        } else {
            EnqueueOutcome::AlreadyQueued
        })
    }

    fn peek_next(&self) -> Result<Option<(i64, String, u64)>, rusqlite::Error> {
        let g = self.conn_lock();
        let mut stmt =
            g.prepare("SELECT id, tenant_id, doc_id FROM pending_summary ORDER BY id ASC LIMIT 1")?;
        let mut rows = stmt.query([])?;
        if let Some(row) = rows.next()? {
            let id: i64 = row.get(0)?;
            let tenant_id: String = row.get(1)?;
            let doc_id: i64 = row.get(2)?;
            Ok(Some((id, tenant_id, doc_id as u64)))
        } else {
            Ok(None)
        }
    }

    fn remove_id(&self, id: i64) -> Result<(), rusqlite::Error> {
        let g = self.conn_lock();
        g.execute("DELETE FROM pending_summary WHERE id = ?1", [id])?;
        Ok(())
    }

    fn wait_for_work(&self, timeout: Duration) {
        let guard = self.wait_lock.lock().unwrap_or_else(|e| e.into_inner());
        let _ = self
            .signal
            .wait_timeout(guard, timeout)
            .unwrap_or_else(|e| e.into_inner());
    }

    fn conn_lock(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.conn.lock().unwrap_or_else(|e| e.into_inner())
    }
}

/// Runs until process exit. Processes at most one job at a time.
pub fn run_worker_loop(state: Arc<AppState>, queue: Arc<SummaryQueue>) {
    loop {
        let job = match queue.peek_next() {
            Ok(Some(j)) => j,
            Ok(None) => {
                // Block until enqueue notifies, with periodic timeout as a safety net.
                queue.wait_for_work(Duration::from_secs(30));
                continue;
            }
            Err(e) => {
                tracing::error!(error = %e, "summary queue peek failed");
                queue.wait_for_work(Duration::from_secs(5));
                continue;
            }
        };

        let (job_id, tenant_id, doc_id) = job;
        process_one_job(&state, &queue, job_id, &tenant_id, doc_id);
    }
}

fn process_one_job(
    state: &AppState,
    queue: &SummaryQueue,
    job_id: i64,
    tenant_id: &str,
    doc_id: u64,
) {
    let index = state.index.clone();
    let tid = tenant_id.to_string();
    let stored = match index.get_document(&tid, doc_id) {
        Ok(s) => s,
        Err(e) => {
            warn!(
                error = %e,
                tenant_id = %tid,
                doc_id,
                "summary job: get_document failed; dropping queue row"
            );
            let _ = queue.remove_id(job_id);
            return;
        }
    };

    let Some(doc) = stored else {
        warn!(
            tenant_id = %tid,
            doc_id,
            "summary job: document missing; dropping queue row"
        );
        let _ = queue.remove_id(job_id);
        return;
    };

    let Some(ref summarizer) = state.summarizer else {
        warn!("summary job: summarizer disabled; dropping queue row");
        let _ = queue.remove_id(job_id);
        return;
    };

    let summary = summarize_sync(summarizer, &doc.content, "summary_queue");

    // Re-fetch after summarization so concurrent edits (title, body, tags, …) are not
    // overwritten by the snapshot taken before `summarize_sync`.
    let latest = match index.get_document(&tid, doc_id) {
        Ok(s) => s,
        Err(e) => {
            warn!(
                error = %e,
                tenant_id = %tid,
                doc_id,
                "summary job: get_document after summarize failed; dropping queue row"
            );
            let _ = queue.remove_id(job_id);
            return;
        }
    };

    let Some(latest) = latest else {
        warn!(
            tenant_id = %tid,
            doc_id,
            "summary job: document removed during summarize; dropping queue row"
        );
        let _ = queue.remove_id(job_id);
        return;
    };

    let row = DocumentRow {
        doc_id: latest.doc_id,
        title: latest.title,
        content: latest.content,
        doc_url: latest.doc_url,
        summary,
        tags: latest.tags,
        path: latest.path,
        note: latest.note,
    };

    if let Err(e) = index.index_document(tenant_id, row) {
        warn!(
            error = %e,
            tenant_id = %tid,
            doc_id,
            "summary job: index_document failed; dropping queue row so the queue can advance"
        );
        let _ = queue.remove_id(job_id);
        return;
    }

    if let Err(e) = queue.remove_id(job_id) {
        tracing::error!(
            error = %e,
            job_id,
            "summary job: completed but failed to remove queue row"
        );
        return;
    }

    info!(
        tenant_id = %tid,
        doc_id,
        "summary job completed"
    );
}

fn summarize_sync(
    summarizer: &Arc<std::sync::Mutex<inkly_summarize::Summarizer>>,
    content: &str,
    op: &'static str,
) -> String {
    let mut summary = String::new();
    match summarizer.lock() {
        Ok(mut guard) => {
            let t = std::time::Instant::now();
            match guard.summarize(content) {
                Ok(s) => {
                    let elapsed = t.elapsed();
                    tracing::info!(
                        op,
                        elapsed_ms = elapsed.as_millis(),
                        summary_chars = s.len(),
                        "summarize completed"
                    );
                    summary = s;
                }
                Err(e) => warn!(error = %e, op, "summarizer failed"),
            }
        }
        Err(_) => warn!(op, "summarizer lock poisoned"),
    }
    summary
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn enqueue_inserts_once_per_tenant_doc() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("q.db");
        let q = SummaryQueue::open(&path).unwrap();
        assert_eq!(q.enqueue("tenant_a", 42).unwrap(), EnqueueOutcome::Enqueued);
        assert_eq!(
            q.enqueue("tenant_a", 42).unwrap(),
            EnqueueOutcome::AlreadyQueued
        );
        assert_eq!(q.enqueue("tenant_a", 43).unwrap(), EnqueueOutcome::Enqueued);
        assert_eq!(q.enqueue("tenant_b", 42).unwrap(), EnqueueOutcome::Enqueued);
    }
}
