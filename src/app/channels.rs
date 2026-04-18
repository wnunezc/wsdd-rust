use std::time::Instant;

use super::{BackgroundJobEvent, BackgroundJobStatus, WsddApp};

impl WsddApp {
    /// Drains pending log, polling, and background job channels.
    pub fn drain_channels(&mut self) {
        self.drain_container_log_channels();

        while let Ok(event) = self.job_event_rx.try_recv() {
            let BackgroundJobEvent::Finished { key, error } = event;
            if let Some(job) = self.jobs.get_mut(&key) {
                job.status = if error.is_some() {
                    BackgroundJobStatus::Failed
                } else {
                    BackgroundJobStatus::Succeeded
                };
                job.finished_at = Some(Instant::now());
                job.last_error = error;
            }

            if key == "poll:containers" {
                self.container_poll_active = false;
                self.last_container_poll = Instant::now();
            }
        }

        while let Ok(line) = self.main_log_rx.try_recv() {
            if let Some(ref key) = line.key.clone() {
                if let Some(existing) = self
                    .main_log
                    .iter_mut()
                    .find(|line| line.key.as_deref() == Some(key.as_str()))
                {
                    *existing = line;
                } else {
                    self.main_log.push(line);
                }
            } else {
                let is_dup = self
                    .main_log
                    .last()
                    .is_some_and(|last| last.text == line.text);
                if !is_dup {
                    self.main_log.push(line);
                }
            }
        }

        let mut poll_done = false;
        if let Some(rx) = &self.container_poll_rx {
            if let Ok(snapshot) = rx.try_recv() {
                self.containers = snapshot.containers;
                self.docker_status = snapshot.docker_status;
                poll_done = true;
            }
        }
        if poll_done {
            self.container_poll_rx = None;
            self.container_poll_active = false;
            self.last_container_poll = Instant::now();
        }
    }
}
