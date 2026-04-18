use serde::{Deserialize, Serialize};

use crate::errors::InfraError;

/// Optional developer services managed outside the base PHP/MySQL stack.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct OptionalServicesSettings {
    pub redis: RedisServiceSettings,
    pub mailpit: MailpitServiceSettings,
    pub memcached: MemcachedServiceSettings,
}

impl OptionalServicesSettings {
    /// Validates optional service ports before saving or deploying them.
    ///
    /// # Errors
    /// Returns [`InfraError`] when a service uses an invalid or conflicting host port.
    pub fn validate(&self) -> Result<(), InfraError> {
        validate_port("Redis host port", self.redis.host_port)?;
        validate_port("Mailpit SMTP host port", self.mailpit.smtp_host_port)?;
        validate_port("Mailpit UI host port", self.mailpit.ui_host_port)?;
        validate_port("Memcached host port", self.memcached.host_port)?;

        if self.mailpit.enabled && self.mailpit.smtp_host_port == self.mailpit.ui_host_port {
            return Err(InfraError::PrerequisiteNotMet(
                "Mailpit SMTP and UI ports must be different".to_string(),
            ));
        }

        if self.mailpit.enabled {
            validate_virtual_host(&self.mailpit.virtual_host)?;
        }

        if self.memcached.enabled && self.memcached.memory_limit_mb == 0 {
            return Err(InfraError::PrerequisiteNotMet(
                "Memcached memory must be greater than 0 MB".to_string(),
            ));
        }

        let mut enabled_ports = Vec::new();
        if self.redis.enabled {
            enabled_ports.push(("Redis", self.redis.host_port));
        }
        if self.mailpit.enabled {
            enabled_ports.push(("Mailpit SMTP", self.mailpit.smtp_host_port));
            enabled_ports.push(("Mailpit UI", self.mailpit.ui_host_port));
        }
        if self.memcached.enabled {
            enabled_ports.push(("Memcached", self.memcached.host_port));
        }

        for (index, (left_label, left_port)) in enabled_ports.iter().enumerate() {
            for (right_label, right_port) in enabled_ports.iter().skip(index + 1) {
                if left_port == right_port {
                    return Err(InfraError::PrerequisiteNotMet(format!(
                        "{left_label} and {right_label} host ports must be different"
                    )));
                }
            }
        }

        Ok(())
    }
}

/// Redis service settings. Disabled by default and deployed only on activation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct RedisServiceSettings {
    pub enabled: bool,
    pub auto_start: bool,
    pub host_port: u16,
}

impl Default for RedisServiceSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            auto_start: false,
            host_port: 6379,
        }
    }
}

/// Memcached service settings. Disabled by default and deployed only on activation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct MemcachedServiceSettings {
    pub enabled: bool,
    pub auto_start: bool,
    pub host_port: u16,
    pub memory_limit_mb: u16,
}

impl Default for MemcachedServiceSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            auto_start: false,
            host_port: 11211,
            memory_limit_mb: 64,
        }
    }
}

/// Mailpit service settings. Disabled by default and deployed only on activation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct MailpitServiceSettings {
    pub enabled: bool,
    pub auto_start: bool,
    pub smtp_host_port: u16,
    pub ui_host_port: u16,
    pub virtual_host: String,
}

impl Default for MailpitServiceSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            auto_start: false,
            smtp_host_port: 1025,
            ui_host_port: 8025,
            virtual_host: "mailpit.wsdd.dock".to_string(),
        }
    }
}

fn validate_port(label: &str, port: u16) -> Result<(), InfraError> {
    if port == 0 {
        return Err(InfraError::PrerequisiteNotMet(format!(
            "{label} must be between 1 and 65535"
        )));
    }
    Ok(())
}

fn validate_virtual_host(host: &str) -> Result<(), InfraError> {
    let trimmed = host.trim();
    if trimmed.is_empty() || trimmed.chars().any(char::is_whitespace) {
        return Err(InfraError::PrerequisiteNotMet(
            "Mailpit virtual host must be a non-empty domain without whitespace".to_string(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn optional_services_are_disabled_by_default() {
        let settings = OptionalServicesSettings::default();

        assert!(!settings.redis.enabled);
        assert!(!settings.redis.auto_start);
        assert!(!settings.mailpit.enabled);
        assert!(!settings.mailpit.auto_start);
        assert!(!settings.memcached.enabled);
        assert!(!settings.memcached.auto_start);
    }

    #[test]
    fn mailpit_ports_must_be_different() {
        let settings = OptionalServicesSettings {
            mailpit: MailpitServiceSettings {
                enabled: true,
                smtp_host_port: 1025,
                ui_host_port: 1025,
                ..MailpitServiceSettings::default()
            },
            ..OptionalServicesSettings::default()
        };

        assert!(settings.validate().is_err());
    }

    #[test]
    fn disabled_mailpit_allows_incomplete_draft_config() {
        let settings = OptionalServicesSettings {
            mailpit: MailpitServiceSettings {
                smtp_host_port: 1025,
                ui_host_port: 1025,
                ..MailpitServiceSettings::default()
            },
            ..OptionalServicesSettings::default()
        };

        assert!(settings.validate().is_ok());
    }

    #[test]
    fn enabled_optional_service_ports_must_be_unique() {
        let settings = OptionalServicesSettings {
            redis: RedisServiceSettings {
                enabled: true,
                host_port: 11211,
                ..RedisServiceSettings::default()
            },
            memcached: MemcachedServiceSettings {
                enabled: true,
                host_port: 11211,
                ..MemcachedServiceSettings::default()
            },
            ..OptionalServicesSettings::default()
        };

        assert!(settings.validate().is_err());
    }
}
