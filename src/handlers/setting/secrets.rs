use serde::{Deserialize, Serialize};

use crate::config::environment::path_config;
use crate::errors::InfraError;

use super::credentials::{normalize_webmin_credentials, PrereqCredentials, WebminCredentials};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub(super) struct AppSecrets {
    #[serde(default)]
    pub prereq_credentials: PrereqCredentials,
    #[serde(default)]
    pub webmin_credentials: Vec<WebminCredentials>,
}

/// Loads credentials from `wsdd-secrets.json` when the file exists.
pub(super) fn load() -> Result<Option<AppSecrets>, InfraError> {
    let path = path_config().secrets_file();
    if !path.exists() {
        return Ok(None);
    }

    let content = std::fs::read_to_string(path)?;
    let mut secrets: AppSecrets = serde_json::from_str(&content)?;
    secrets.prereq_credentials = secrets.prereq_credentials.normalize_loaded();
    normalize_webmin_credentials(&mut secrets.webmin_credentials);
    Ok(Some(secrets))
}

/// Saves credentials to `wsdd-secrets.json`.
pub(super) fn save(
    prereq_credentials: &PrereqCredentials,
    webmin_credentials: &[WebminCredentials],
) -> Result<(), InfraError> {
    let secrets_path = path_config().secrets_file();
    if let Some(parent) = secrets_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let secrets = AppSecrets {
        prereq_credentials: prereq_credentials.clone(),
        webmin_credentials: webmin_credentials.to_vec(),
    };
    let secrets_content = serde_json::to_string_pretty(&secrets)?;
    std::fs::write(&secrets_path, secrets_content)?;
    Ok(())
}
