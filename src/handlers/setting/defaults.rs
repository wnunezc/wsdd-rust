use crate::i18n::Language;

pub(super) const CURRENT_CONFIG_VERSION: u32 = 7;
pub(super) const CURRENT_WEBMIN_VERSION: &str = "2.630";
pub(super) const LEGACY_WEBMIN_USER: &str = "admin";
pub(super) const LEGACY_WEBMIN_PASSWORD: &str = "admin";
pub(super) const LEGACY_WEBMIN_VERSION_2021: &str = "2.021";

pub(super) fn default_log_max_lines() -> usize {
    500
}

pub(super) fn default_php_memory_limit() -> String {
    "512M".to_string()
}

pub(super) fn default_php_upload_max_filesize() -> String {
    "256M".to_string()
}

pub(super) fn default_php_timezone() -> String {
    "UTC".to_string()
}

pub(super) fn default_xdebug_enabled() -> bool {
    true
}

pub(super) fn default_webmin_version() -> String {
    CURRENT_WEBMIN_VERSION.to_string()
}

pub(super) fn default_mysql_database() -> String {
    "wsdd-database".to_string()
}

pub(super) fn default_mysql_user() -> String {
    "tester".to_string()
}

pub(super) fn default_language() -> Language {
    Language::default()
}

pub(super) fn default_config_version() -> u32 {
    CURRENT_CONFIG_VERSION
}
