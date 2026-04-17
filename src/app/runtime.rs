use std::sync::OnceLock;

use anyhow::Context;
use tokio::runtime::{Builder, Runtime};

use super::JobRuntime;

/// Creates or returns the shared Tokio runtime used by background jobs.
pub fn create_job_runtime() -> anyhow::Result<JobRuntime> {
    static RUNTIME: OnceLock<Runtime> = OnceLock::new();
    if let Some(runtime) = RUNTIME.get() {
        return Ok(runtime);
    }

    let runtime = Builder::new_multi_thread()
        .enable_all()
        .thread_name("wsdd-job")
        .build()
        .context("No se pudo crear el runtime de jobs de WSDD")?;

    let _ = RUNTIME.set(runtime);
    RUNTIME
        .get()
        .context("No se pudo inicializar el runtime compartido de WSDD")
}
