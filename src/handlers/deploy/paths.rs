use std::path::PathBuf;

use crate::config::environment::{path_config, path_to_string};
use crate::handlers::project as project_handler;
use crate::handlers::yml;
use crate::models::project::Project;

pub(super) fn php_bin_dir(php_dir_name: &str) -> String {
    path_to_string(path_config().php_dir(php_dir_name))
}

pub(super) fn active_vhost_conf_path(php_dir_name: &str) -> String {
    path_to_string(path_config().active_vhost_conf(php_dir_name))
}

pub(super) fn legacy_vhost_conf_path(php_dir_name: &str) -> String {
    path_to_string(path_config().legacy_vhost_conf(php_dir_name))
}

pub(super) fn vhost_template_path(php_dir_name: &str) -> String {
    path_to_string(path_config().vhost_template(php_dir_name))
}

pub(super) fn webserver_yml_path(php_dir_name: &str, compose_tag: &str) -> String {
    path_to_string(path_config().webserver_yml(php_dir_name, compose_tag))
}

pub(super) fn project_file_path(project_name: &str) -> PathBuf {
    project_handler::file_path(project_name)
}

pub(super) fn options_yml_path(project: &Project) -> String {
    yml::options_path(
        project.php_version.dir_name(),
        project.php_version.compose_tag(),
    )
}

pub(super) fn volume_name(project: &Project) -> String {
    format!("{}-{}", project.php_version.compose_tag(), project.domain)
}

pub(super) fn php_container_name(project: &Project) -> String {
    format!("WSDD-Web-Server-{}", project.php_version.container_tag())
}

pub(super) fn ssl_file_paths(project: &Project) -> [PathBuf; 2] {
    [
        path_config().ssl_cert_file(&project.domain),
        path_config().ssl_key_file(&project.domain),
    ]
}
