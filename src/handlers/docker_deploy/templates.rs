// WebStack Deployer for Docker
// Copyright (c) 2026 Walter Nunez / Icaros Net S.A
// All Rights Reserved.
//
// This software is provided for development use only.
// Unauthorized commercial use is prohibited.
//
// Redistribution and modification allowed only through
// the official GitHub repository.
//
// This software is provided AS IS, without warranty of any kind.
// The author shall not be liable for any damages.
//
// Contact: wnunez@lh-2.net
//! Managed Docker template rendering for WSDD resources.

use crate::handlers::setting::{AppSettings, PrereqCredentials, WebminCredentials};
use crate::models::project::PhpVersion;

const INIT_YML_TEMPLATE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/recursos/recursos/Docker-Structure/init.yml"
));
const PHP56_DOCKERFILE_TEMPLATE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/recursos/recursos/Docker-Structure/bin/php5.6/Dockerfile"
));
const PHP72_DOCKERFILE_TEMPLATE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/recursos/recursos/Docker-Structure/bin/php7.2/Dockerfile"
));
const PHP74_DOCKERFILE_TEMPLATE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/recursos/recursos/Docker-Structure/bin/php7.4/Dockerfile"
));
const PHP81_DOCKERFILE_TEMPLATE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/recursos/recursos/Docker-Structure/bin/php8.1/Dockerfile"
));
const PHP82_DOCKERFILE_TEMPLATE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/recursos/recursos/Docker-Structure/bin/php8.2/Dockerfile"
));
const PHP83_DOCKERFILE_TEMPLATE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/recursos/recursos/Docker-Structure/bin/php8.3/Dockerfile"
));
const PHP84_DOCKERFILE_TEMPLATE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/recursos/recursos/Docker-Structure/bin/php8.4/Dockerfile"
));
const PHP56_WEBSERVER_TEMPLATE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/recursos/recursos/Docker-Structure/bin/php5.6/webserver.php56.yml"
));
const PHP72_WEBSERVER_TEMPLATE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/recursos/recursos/Docker-Structure/bin/php7.2/webserver.php72.yml"
));
const PHP74_WEBSERVER_TEMPLATE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/recursos/recursos/Docker-Structure/bin/php7.4/webserver.php74.yml"
));
const PHP81_WEBSERVER_TEMPLATE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/recursos/recursos/Docker-Structure/bin/php8.1/webserver.php81.yml"
));
const PHP82_WEBSERVER_TEMPLATE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/recursos/recursos/Docker-Structure/bin/php8.2/webserver.php82.yml"
));
const PHP83_WEBSERVER_TEMPLATE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/recursos/recursos/Docker-Structure/bin/php8.3/webserver.php83.yml"
));
const PHP84_WEBSERVER_TEMPLATE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/recursos/recursos/Docker-Structure/bin/php8.4/webserver.php84.yml"
));

/// Renders the base `init.yml` with installation-specific credentials.
pub(super) fn render_init_yml(credentials: &PrereqCredentials) -> String {
    INIT_YML_TEMPLATE
        .replace(
            "__WSDD_MYSQL_ROOT_PASSWORD__",
            &yaml_single_quoted(&credentials.mysql_root_password),
        )
        .replace(
            "__WSDD_MYSQL_DATABASE__",
            &yaml_single_quoted(&credentials.mysql_database),
        )
        .replace(
            "__WSDD_MYSQL_USER__",
            &yaml_single_quoted(&credentials.mysql_user),
        )
        .replace(
            "__WSDD_MYSQL_PASSWORD__",
            &yaml_single_quoted(&credentials.mysql_password),
        )
}

/// Renders a PHP Webmin compose file for one PHP version.
pub(super) fn render_webserver_yml(
    settings: &AppSettings,
    php_version: &PhpVersion,
    credentials: &WebminCredentials,
) -> String {
    webserver_template(php_version)
        .replace(
            "__WSDD_WEBMIN_VERSION__",
            &yaml_single_quoted(&settings.webmin_version),
        )
        .replace(
            "__WSDD_WEBMIN_USER__",
            &yaml_single_quoted(&credentials.username),
        )
        .replace(
            "__WSDD_WEBMIN_PASS__",
            &yaml_single_quoted(&credentials.password),
        )
}

/// Returns the managed Dockerfile template for a PHP version.
pub(super) fn dockerfile_template(php_version: &PhpVersion) -> &'static str {
    match php_version {
        PhpVersion::Php56 => PHP56_DOCKERFILE_TEMPLATE,
        PhpVersion::Php72 => PHP72_DOCKERFILE_TEMPLATE,
        PhpVersion::Php74 => PHP74_DOCKERFILE_TEMPLATE,
        PhpVersion::Php81 => PHP81_DOCKERFILE_TEMPLATE,
        PhpVersion::Php82 => PHP82_DOCKERFILE_TEMPLATE,
        PhpVersion::Php83 => PHP83_DOCKERFILE_TEMPLATE,
        PhpVersion::Php84 => PHP84_DOCKERFILE_TEMPLATE,
    }
}

/// Returns the generated webserver compose filename for a PHP version.
pub(super) fn webserver_file_name(php_version: &PhpVersion) -> String {
    format!("webserver.{}.yml", php_version.compose_tag())
}

fn webserver_template(php_version: &PhpVersion) -> &'static str {
    match php_version {
        PhpVersion::Php56 => PHP56_WEBSERVER_TEMPLATE,
        PhpVersion::Php72 => PHP72_WEBSERVER_TEMPLATE,
        PhpVersion::Php74 => PHP74_WEBSERVER_TEMPLATE,
        PhpVersion::Php81 => PHP81_WEBSERVER_TEMPLATE,
        PhpVersion::Php82 => PHP82_WEBSERVER_TEMPLATE,
        PhpVersion::Php83 => PHP83_WEBSERVER_TEMPLATE,
        PhpVersion::Php84 => PHP84_WEBSERVER_TEMPLATE,
    }
}

fn yaml_single_quoted(value: &str) -> String {
    let sanitized = value.replace(['\r', '\n'], "");
    format!("'{}'", sanitized.replace('\'', "''"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rendered_init_yml_replaces_all_placeholders() {
        let credentials = PrereqCredentials {
            mysql_database: "custom-db".to_string(),
            mysql_user: "custom-user".to_string(),
            mysql_password: "custom-pass".to_string(),
            mysql_root_password: "root-pass".to_string(),
        };

        let rendered = render_init_yml(&credentials);

        assert!(rendered.contains("MYSQL_DATABASE: 'custom-db'"));
        assert!(rendered.contains("MYSQL_USER: 'custom-user'"));
        assert!(rendered.contains("MYSQL_PASSWORD: 'custom-pass'"));
        assert!(rendered.contains("MYSQL_ROOT_PASSWORD: 'root-pass'"));
        assert!(!rendered.contains("__WSDD_"));
    }

    #[test]
    fn yaml_single_quoted_escapes_single_quotes() {
        assert_eq!(yaml_single_quoted("o'hara"), "'o''hara'");
    }

    #[test]
    fn rendered_webserver_yml_replaces_all_placeholders() {
        let settings = AppSettings {
            webmin_version: "2.630".to_string(),
            webmin_credentials: vec![WebminCredentials {
                php_version: PhpVersion::Php83,
                username: "walter".to_string(),
                password: "secret".to_string(),
            }],
            ..AppSettings::default()
        };

        let rendered = render_webserver_yml(
            &settings,
            &PhpVersion::Php83,
            settings
                .webmin_credentials_for(&PhpVersion::Php83)
                .expect("missing credentials"),
        );

        assert!(rendered.contains("WEBMIN_VERSION: '2.630'"));
        assert!(rendered.contains("WEBMIN_USER: 'walter'"));
        assert!(rendered.contains("WEBMIN_PASS: 'secret'"));
        assert!(!rendered.contains("__WSDD_"));
    }
}
