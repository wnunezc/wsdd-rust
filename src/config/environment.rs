use std::path::{Path, PathBuf};

pub const DEFAULT_WSDD_ENV: &str = r"C:\WSDD-Environment";
pub const DEFAULT_PROJECTS_ROOT: &str = r"C:\WSDD-Projects";
pub const DEFAULT_HOSTS_FILE: &str = r"C:\Windows\System32\drivers\etc\hosts";
pub const DEFAULT_CHOCOLATEY_BIN: &str = r"C:\ProgramData\chocolatey\bin";
pub const DEFAULT_DOCKER_DESKTOP_EXE: &str = r"C:\Program Files\Docker\Docker\Docker Desktop.exe";
pub const DEFAULT_DOCKER_DESKTOP_EXE_X86: &str =
    r"C:\Program Files (x86)\Docker\Docker\Docker Desktop.exe";
pub const DEFAULT_PWSH_EXE: &str = r"C:\Program Files\PowerShell\7\pwsh.exe";
pub const MIN_SUPPORTED_PWSH_VERSION: &str = "7.5.0";
pub const DOCKER_HOST_VALUE: &str = "tcp://localhost:2375";
pub const POWERSHELL_RELEASE_BASE_URL: &str =
    "https://github.com/PowerShell/PowerShell/releases/download";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathConfig {
    environment_root: PathBuf,
    default_projects_root: PathBuf,
    hosts_file: PathBuf,
}

impl Default for PathConfig {
    fn default() -> Self {
        Self {
            environment_root: PathBuf::from(DEFAULT_WSDD_ENV),
            default_projects_root: PathBuf::from(DEFAULT_PROJECTS_ROOT),
            hosts_file: PathBuf::from(DEFAULT_HOSTS_FILE),
        }
    }
}

impl PathConfig {
    pub fn environment_root(&self) -> &Path {
        &self.environment_root
    }

    pub fn default_projects_root(&self) -> &Path {
        &self.default_projects_root
    }

    pub fn config_file(&self) -> PathBuf {
        self.environment_root.join("wsdd-config.json")
    }

    pub fn secrets_file(&self) -> PathBuf {
        self.environment_root.join("wsdd-secrets.json")
    }

    pub fn scripts_dir(&self) -> PathBuf {
        self.environment_root.join("PS-Script")
    }

    pub fn docker_structure_dir(&self) -> PathBuf {
        self.environment_root.join("Docker-Structure")
    }

    pub fn logs_dir(&self) -> PathBuf {
        self.environment_root.join("logs")
    }

    pub fn deploy_log_file(&self, day: u64) -> PathBuf {
        self.logs_dir().join(format!("wsdd-deploy-d{day}.log"))
    }

    pub fn projects_dir(&self) -> PathBuf {
        self.docker_structure_dir().join("projects")
    }

    pub fn project_file(&self, name: &str) -> PathBuf {
        self.projects_dir().join(format!("{name}.json"))
    }

    pub fn ssl_dir(&self) -> PathBuf {
        self.docker_structure_dir().join("ssl")
    }

    pub fn ssl_cert_file(&self, domain: &str) -> PathBuf {
        self.ssl_dir().join(format!("{domain}.crt"))
    }

    pub fn ssl_key_file(&self, domain: &str) -> PathBuf {
        self.ssl_dir().join(format!("{domain}.key"))
    }

    pub fn php_dir(&self, php_dir_name: &str) -> PathBuf {
        self.docker_structure_dir().join("bin").join(php_dir_name)
    }

    pub fn pma_app_dir(&self) -> PathBuf {
        self.docker_structure_dir()
            .join("bin")
            .join("pma")
            .join("app")
    }

    pub fn init_yml(&self) -> PathBuf {
        self.docker_structure_dir().join("init.yml")
    }

    pub fn mysql_dockerfile(&self) -> PathBuf {
        self.docker_structure_dir()
            .join("bin")
            .join("mysql")
            .join("Dockerfile")
    }

    pub fn pma_php_ini(&self) -> PathBuf {
        self.docker_structure_dir()
            .join("bin")
            .join("pma")
            .join("php.ini")
    }

    pub fn options_yml(&self, php_dir_name: &str, compose_tag: &str) -> PathBuf {
        self.php_dir(php_dir_name)
            .join(format!("options.{compose_tag}.yml"))
    }

    pub fn webserver_yml(&self, php_dir_name: &str, compose_tag: &str) -> PathBuf {
        self.php_dir(php_dir_name)
            .join(format!("webserver.{compose_tag}.yml"))
    }

    pub fn active_vhost_conf(&self, php_dir_name: &str) -> PathBuf {
        self.php_dir(php_dir_name).join("vhost").join("vhost.conf")
    }

    pub fn legacy_vhost_conf(&self, php_dir_name: &str) -> PathBuf {
        self.php_dir(php_dir_name).join("vhost.conf")
    }

    pub fn vhost_template(&self, php_dir_name: &str) -> PathBuf {
        self.php_dir(php_dir_name).join("tpl.vhost.conf")
    }

    pub fn hosts_file(&self) -> &Path {
        &self.hosts_file
    }

    pub fn hosts_backup_file(&self) -> PathBuf {
        self.environment_root.join("hosts.backup")
    }

    pub fn hosts_temp_file(&self) -> PathBuf {
        self.environment_root.join("hosts.tmp")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvConfig {
    min_supported_pwsh_version: &'static str,
}

impl Default for EnvConfig {
    fn default() -> Self {
        Self {
            min_supported_pwsh_version: MIN_SUPPORTED_PWSH_VERSION,
        }
    }
}

impl EnvConfig {
    pub fn min_supported_pwsh_version(&self) -> &'static str {
        self.min_supported_pwsh_version
    }

    pub fn pwsh_candidates(&self) -> Vec<String> {
        vec![DEFAULT_PWSH_EXE.to_string(), "pwsh.exe".to_string()]
    }

    pub fn windows_powershell_exe(&self) -> &'static str {
        "powershell.exe"
    }

    pub fn pwsh_exe(&self) -> &'static str {
        "pwsh.exe"
    }

    pub fn where_exe(&self) -> &'static str {
        "where.exe"
    }

    pub fn curl_exe(&self) -> &'static str {
        "curl.exe"
    }

    pub fn msiexec_exe(&self) -> &'static str {
        "msiexec.exe"
    }

    pub fn powershell_release_base_url(&self) -> &'static str {
        POWERSHELL_RELEASE_BASE_URL
    }

    pub fn docker_exe(&self) -> &'static str {
        "docker"
    }

    pub fn docker_compose_exe(&self) -> &'static str {
        "docker-compose"
    }

    pub fn chocolatey_install_env(&self) -> &'static str {
        "ChocolateyInstall"
    }

    pub fn default_choco_exe(&self) -> PathBuf {
        PathBuf::from(DEFAULT_CHOCOLATEY_BIN).join("choco.exe")
    }

    pub fn default_mkcert_exe(&self) -> PathBuf {
        PathBuf::from(DEFAULT_CHOCOLATEY_BIN).join("mkcert.exe")
    }

    pub fn docker_desktop_candidates(&self) -> Vec<PathBuf> {
        vec![
            PathBuf::from(DEFAULT_DOCKER_DESKTOP_EXE),
            PathBuf::from(DEFAULT_DOCKER_DESKTOP_EXE_X86),
        ]
    }

    pub fn docker_settings_candidates(&self) -> Vec<PathBuf> {
        let Some(appdata) = std::env::var_os("APPDATA") else {
            return Vec::new();
        };
        let docker_dir = PathBuf::from(appdata).join("Docker");
        vec![
            docker_dir.join("settings-store.json"),
            docker_dir.join("settings.json"),
        ]
    }

    pub fn wsl_config_file(&self) -> PathBuf {
        let profile =
            std::env::var("USERPROFILE").unwrap_or_else(|_| r"C:\Users\Default".to_string());
        PathBuf::from(profile).join(".wslconfig")
    }

    pub fn wsl_service_candidates(&self) -> &'static [&'static str] {
        &["WslService", "LxssManager"]
    }
}

pub fn path_config() -> PathConfig {
    PathConfig::default()
}

pub fn env_config() -> EnvConfig {
    EnvConfig::default()
}

pub fn path_to_string(path: impl AsRef<Path>) -> String {
    path.as_ref().display().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_config_defaults_match_existing_contract() {
        let paths = PathConfig::default();
        assert_eq!(paths.environment_root(), Path::new(DEFAULT_WSDD_ENV));
        assert_eq!(
            paths.default_projects_root(),
            Path::new(DEFAULT_PROJECTS_ROOT)
        );
        assert_eq!(paths.hosts_file(), Path::new(DEFAULT_HOSTS_FILE));
    }

    #[test]
    fn path_config_builds_core_files() {
        let paths = PathConfig::default();
        assert_eq!(
            paths.config_file(),
            PathBuf::from(r"C:\WSDD-Environment\wsdd-config.json")
        );
        assert_eq!(
            paths.secrets_file(),
            PathBuf::from(r"C:\WSDD-Environment\wsdd-secrets.json")
        );
        assert_eq!(
            paths.hosts_backup_file(),
            PathBuf::from(r"C:\WSDD-Environment\hosts.backup")
        );
        assert_eq!(
            paths.hosts_temp_file(),
            PathBuf::from(r"C:\WSDD-Environment\hosts.tmp")
        );
    }

    #[test]
    fn path_config_builds_php_derived_paths() {
        let paths = PathConfig::default();
        assert_eq!(
            paths.options_yml("php8.3", "php83"),
            PathBuf::from(r"C:\WSDD-Environment\Docker-Structure\bin\php8.3\options.php83.yml")
        );
        assert_eq!(
            paths.active_vhost_conf("php8.3"),
            PathBuf::from(r"C:\WSDD-Environment\Docker-Structure\bin\php8.3\vhost\vhost.conf")
        );
        assert_eq!(
            paths.webserver_yml("php8.3", "php83"),
            PathBuf::from(r"C:\WSDD-Environment\Docker-Structure\bin\php8.3\webserver.php83.yml")
        );
    }

    #[test]
    fn env_config_keeps_tool_candidates() {
        let env = EnvConfig::default();
        assert_eq!(env.min_supported_pwsh_version(), "7.5.0");
        assert!(env.pwsh_candidates().contains(&"pwsh.exe".to_string()));
        assert_eq!(
            env.default_choco_exe(),
            PathBuf::from(r"C:\ProgramData\chocolatey\bin\choco.exe")
        );
        assert_eq!(
            env.default_mkcert_exe(),
            PathBuf::from(r"C:\ProgramData\chocolatey\bin\mkcert.exe")
        );
        assert!(env
            .docker_desktop_candidates()
            .contains(&PathBuf::from(DEFAULT_DOCKER_DESKTOP_EXE)));
    }
}
