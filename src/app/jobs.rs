use std::future::Future;
use std::time::Instant;

use super::{BackgroundJob, BackgroundJobEvent, BackgroundJobStatus, WsddApp};

impl WsddApp {
    /// Returns true when the job identified by `key` is currently running.
    pub fn is_job_running(&self, key: &str) -> bool {
        self.jobs
            .get(key)
            .is_some_and(|job| job.status == BackgroundJobStatus::Running)
    }

    /// Spawns an async background job if another job with the same key is not running.
    pub fn spawn_async_job<Fut>(
        &mut self,
        ctx: &egui::Context,
        key: impl Into<String>,
        label: impl Into<String>,
        future: Fut,
    ) -> bool
    where
        Fut: Future<Output = Result<(), String>> + Send + 'static,
    {
        let key = key.into();
        if self.is_job_running(&key) {
            return false;
        }

        self.jobs.insert(
            key.clone(),
            BackgroundJob {
                key: key.clone(),
                label: label.into(),
                status: BackgroundJobStatus::Running,
                started_at: Instant::now(),
                finished_at: None,
                last_error: None,
            },
        );

        let event_tx = self.job_event_tx.clone();
        let ctx = ctx.clone();
        self.job_runtime.spawn(async move {
            let error = future.await.err();
            let _ = event_tx.send(BackgroundJobEvent::Finished { key, error });
            ctx.request_repaint();
        });

        true
    }

    /// Runs a blocking task on the shared background runtime.
    pub fn spawn_blocking_job<F>(
        &mut self,
        ctx: &egui::Context,
        key: impl Into<String>,
        label: impl Into<String>,
        task: F,
    ) -> bool
    where
        F: FnOnce() -> Result<(), String> + Send + 'static,
    {
        self.spawn_async_job(ctx, key, label, async move {
            tokio::task::spawn_blocking(task)
                .await
                .map_err(|e| format!("background task join error: {e}"))?
        })
    }
}
