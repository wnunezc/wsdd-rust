//! ngrok Agent API integration for temporary WSDD project publishing.

use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::errors::InfraError;
use crate::handlers::ps_script::{PsRunner, ScriptRunner};
use crate::handlers::setting::AppSettings;
use crate::models::project::Project;

const NGROK_CONTAINER: &str = "WSDD-Ngrok-Server";
const WSDD_PROXY_UPSTREAM: &str = "http://WSDD-Proxy-Server:80";

/// Runtime readiness of the optional ngrok service.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NgrokStatus {
    pub enabled: bool,
    pub configured: bool,
    pub container_running: bool,
    pub api_reachable: bool,
}

impl NgrokStatus {
    /// Returns true when a project publish operation can be attempted.
    pub fn can_publish(&self) -> bool {
        self.enabled && self.configured && self.container_running && self.api_reachable
    }
}

/// Public ngrok URL currently associated with a WSDD project.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NgrokPublication {
    pub project_name: String,
    pub project_domain: String,
    pub tunnel_name: String,
    pub public_url: String,
}

#[derive(Debug, Deserialize)]
struct TunnelListResponse {
    tunnels: Vec<TunnelResponse>,
}

#[derive(Debug, Deserialize)]
struct TunnelResponse {
    name: String,
    public_url: Option<String>,
}

#[derive(Debug, Serialize)]
struct CreateTunnelRequest<'a> {
    name: &'a str,
    proto: &'a str,
    addr: &'a str,
    inspect: bool,
    host_header: &'a str,
}

/// Returns the stable ngrok tunnel name for a project.
pub fn tunnel_name_for_project(project: &Project) -> String {
    let mut slug = String::with_capacity(project.name.len());
    for ch in project.name.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
        } else if !slug.ends_with('-') {
            slug.push('-');
        }
    }
    let slug = slug.trim_matches('-');
    if slug.is_empty() {
        format!("wsdd-{}", project.domain.replace('.', "-"))
    } else {
        format!("wsdd-{slug}")
    }
}

/// Checks ngrok settings, Docker container state, and Agent API reachability.
///
/// # Errors
/// Returns [`InfraError`] when Docker or the Agent API check fails unexpectedly.
pub fn agent_status(settings: &AppSettings, runner: &PsRunner) -> Result<NgrokStatus, InfraError> {
    let enabled = settings.optional_services.ngrok.enabled;
    let configured = !settings.ngrok_authtoken.trim().is_empty();
    let container_running = if enabled {
        ngrok_container_running(runner)?
    } else {
        false
    };
    let api_reachable = if enabled && configured && container_running {
        agent_api_client(settings)
            .get(agent_api_url(settings, "tunnels"))
            .send()
            .map(|response| response.status().is_success())
            .unwrap_or(false)
    } else {
        false
    };

    Ok(NgrokStatus {
        enabled,
        configured,
        container_running,
        api_reachable,
    })
}

/// Creates or reuses a temporary public tunnel for one WSDD project.
///
/// # Errors
/// Returns [`InfraError`] when ngrok is not ready or the Agent API rejects the request.
pub fn publish_project(
    project: &Project,
    settings: &AppSettings,
    runner: &PsRunner,
) -> Result<NgrokPublication, InfraError> {
    let status = agent_status(settings, runner)?;
    if !status.can_publish() {
        return Err(InfraError::PrerequisiteNotMet(
            "ngrok must be enabled, configured, running, and API-reachable before publishing"
                .to_string(),
        ));
    }

    let tunnel_name = tunnel_name_for_project(project);
    if let Some(existing) = list_publications(settings)?
        .into_iter()
        .find(|publication| publication.tunnel_name == tunnel_name)
    {
        return Ok(existing);
    }

    let client = agent_api_client(settings);
    let payload = CreateTunnelRequest {
        name: &tunnel_name,
        proto: "http",
        addr: WSDD_PROXY_UPSTREAM,
        inspect: settings.optional_services.ngrok.inspect,
        host_header: &project.domain,
    };
    let response = client
        .post(agent_api_url(settings, "tunnels"))
        .json(&payload)
        .send()
        .map_err(|e| InfraError::DockerUnreachable(format!("ngrok Agent API error: {e}")))?;

    if !response.status().is_success() {
        return Err(InfraError::DockerUnreachable(format!(
            "ngrok Agent API rejected tunnel creation: HTTP {}",
            response.status()
        )));
    }

    let tunnel: TunnelResponse = response
        .json()
        .map_err(|e| InfraError::UnexpectedOutput("ngrok Agent API".to_string(), e.to_string()))?;
    let public_url = tunnel.public_url.ok_or_else(|| {
        InfraError::UnexpectedOutput(
            "ngrok Agent API".to_string(),
            "tunnel response did not include public_url".to_string(),
        )
    })?;

    Ok(NgrokPublication {
        project_name: project.name.clone(),
        project_domain: project.domain.clone(),
        tunnel_name,
        public_url,
    })
}

/// Finds an active ngrok tunnel for one project without creating a new tunnel.
///
/// # Errors
/// Returns [`InfraError`] when the Agent API cannot be queried.
pub fn publication_for_project(
    project: &Project,
    settings: &AppSettings,
) -> Result<Option<NgrokPublication>, InfraError> {
    let tunnel_name = tunnel_name_for_project(project);
    Ok(list_publications(settings)?
        .into_iter()
        .find(|publication| publication.tunnel_name == tunnel_name)
        .map(|publication| hydrate_project_publication(project, publication)))
}

/// Stops the temporary ngrok tunnel for one WSDD project.
///
/// # Errors
/// Returns [`InfraError`] when the Agent API cannot process the stop request.
pub fn stop_project_publication(
    project: &Project,
    settings: &AppSettings,
) -> Result<(), InfraError> {
    let tunnel_name = tunnel_name_for_project(project);
    let response = agent_api_client(settings)
        .delete(agent_api_url(settings, &format!("tunnels/{tunnel_name}")))
        .send()
        .map_err(|e| InfraError::DockerUnreachable(format!("ngrok Agent API error: {e}")))?;

    if response.status().is_success() || response.status().as_u16() == 404 {
        return Ok(());
    }

    Err(InfraError::DockerUnreachable(format!(
        "ngrok Agent API rejected tunnel stop: HTTP {}",
        response.status()
    )))
}

/// Lists active WSDD-looking ngrok publications from the Agent API.
///
/// # Errors
/// Returns [`InfraError`] when the Agent API cannot be queried.
pub fn list_publications(settings: &AppSettings) -> Result<Vec<NgrokPublication>, InfraError> {
    let response = agent_api_client(settings)
        .get(agent_api_url(settings, "tunnels"))
        .send()
        .map_err(|e| InfraError::DockerUnreachable(format!("ngrok Agent API error: {e}")))?;
    if !response.status().is_success() {
        return Err(InfraError::DockerUnreachable(format!(
            "ngrok Agent API rejected tunnel list: HTTP {}",
            response.status()
        )));
    }

    let list: TunnelListResponse = response
        .json()
        .map_err(|e| InfraError::UnexpectedOutput("ngrok Agent API".to_string(), e.to_string()))?;

    Ok(list
        .tunnels
        .into_iter()
        .filter(|tunnel| tunnel.name.starts_with("wsdd-"))
        .filter_map(|tunnel| {
            let public_url = tunnel.public_url?;
            Some(NgrokPublication {
                project_name: tunnel.name.trim_start_matches("wsdd-").to_string(),
                project_domain: String::new(),
                tunnel_name: tunnel.name,
                public_url,
            })
        })
        .collect())
}

fn ngrok_container_running(runner: &PsRunner) -> Result<bool, InfraError> {
    let out = runner.run_direct_sync(
        "docker",
        &[
            "ps",
            "--format",
            "{{.Names}}",
            "--filter",
            &format!("name={NGROK_CONTAINER}"),
        ],
        None,
        None,
    )?;
    Ok(out.text.lines().any(|line| line.trim() == NGROK_CONTAINER))
}

fn agent_api_url(settings: &AppSettings, path: &str) -> String {
    format!(
        "http://127.0.0.1:{}/api/{}",
        settings.optional_services.ngrok.api_host_port,
        path.trim_start_matches('/')
    )
}

fn agent_api_client(_settings: &AppSettings) -> reqwest::blocking::Client {
    reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .unwrap_or_else(|_| reqwest::blocking::Client::new())
}

fn hydrate_project_publication(
    project: &Project,
    publication: NgrokPublication,
) -> NgrokPublication {
    NgrokPublication {
        project_name: project.name.clone(),
        project_domain: project.domain.clone(),
        tunnel_name: publication.tunnel_name,
        public_url: publication.public_url,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::project::Project;

    #[test]
    fn tunnel_name_is_stable_and_dns_safe() {
        let project = Project {
            name: "RTM Hub!".to_string(),
            domain: "rtm-hub.dock".to_string(),
            ..Project::default()
        };

        assert_eq!(tunnel_name_for_project(&project), "wsdd-rtm-hub");
    }

    #[test]
    fn ngrok_status_requires_every_readiness_flag() {
        let status = NgrokStatus {
            enabled: true,
            configured: true,
            container_running: true,
            api_reachable: false,
        };

        assert!(!status.can_publish());
    }

    #[test]
    fn hydrate_project_publication_restores_project_identity() {
        let project = Project {
            name: "RTM Hub!".to_string(),
            domain: "rtm-hub.dock".to_string(),
            ..Project::default()
        };
        let listed = NgrokPublication {
            project_name: "rtm-hub".to_string(),
            project_domain: String::new(),
            tunnel_name: "wsdd-rtm-hub".to_string(),
            public_url: "https://example.ngrok-free.app".to_string(),
        };

        let restored = hydrate_project_publication(&project, listed);

        assert_eq!(restored.project_name, "RTM Hub!");
        assert_eq!(restored.project_domain, "rtm-hub.dock");
        assert_eq!(restored.tunnel_name, tunnel_name_for_project(&project));
    }
}
