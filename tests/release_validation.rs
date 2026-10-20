use std::{fs, path::Path};

fn workspace_file(path: impl AsRef<Path>) -> String {
    let full_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    fs::read_to_string(&full_path)
        .unwrap_or_else(|err| panic!("could not read {}: {err}", full_path.display()))
}

fn package_version(manifest: &str) -> &str {
    manifest
        .lines()
        .find_map(|line| {
            let trimmed = line.trim();
            trimmed
                .strip_prefix("version = \"")
                .and_then(|value| value.strip_suffix('"'))
        })
        .unwrap_or_else(|| panic!("manifest does not contain a package version"))
}

#[test]
fn workspace_package_versions_stay_aligned() {
    let app_manifest = workspace_file("Cargo.toml");
    let launcher_manifest = workspace_file("wsdd-launcher/Cargo.toml");

    let app_version = package_version(&app_manifest);
    let launcher_version = package_version(&launcher_manifest);

    assert_eq!(app_version, launcher_version);
    assert!(
        app_version == "2.2.0" || app_version.starts_with("2.2.0-rc."),
        "release versions must be stable 2.2.0 or a 2.2.0-rc.N candidate"
    );
}

#[test]
fn release_validation_strategy_tracks_wp09_layers() {
    let strategy = workspace_file("docs/release-validation.md");

    for required in [
        "Unit tests",
        "Isolated integration",
        "Elevated manual",
        "Release smoke",
        "Evidence to record",
    ] {
        assert!(
            strategy.contains(required),
            "release validation strategy must mention {required}"
        );
    }
}

#[test]
fn release_workflow_keeps_msi_and_checksum_artifacts() {
    let workflow = workspace_file(".github/workflows/release.yml");

    for required in [
        "cargo wix-msi",
        "Get-FileHash",
        "target/wix/*.msi",
        "target/wix/*.sha256.txt",
    ] {
        assert!(
            workflow.contains(required),
            "release workflow must preserve {required}"
        );
    }
}

#[test]
fn php_dockerfiles_provide_composer_in_path() {
    for version in ["5.6", "7.2", "7.4", "8.1", "8.2", "8.3", "8.4"] {
        let dockerfile = workspace_file(format!(
            "recursos/recursos/Docker-Structure/bin/php{version}/Dockerfile"
        ));

        assert!(
            dockerfile.contains("https://getcomposer.org/installer"),
            "php{version} Dockerfile must install Composer from the official installer"
        );
        assert!(
            dockerfile.contains("--install-dir=/usr/local/bin --filename=composer --quiet"),
            "php{version} Dockerfile must install composer into /usr/local/bin"
        );
        assert!(
            dockerfile.contains("composer --version"),
            "php{version} Dockerfile must verify composer is available in PATH"
        );
    }
}

#[test]
fn legacy_php_dockerfiles_use_composer_22_channel_only() {
    let php56 = workspace_file("recursos/recursos/Docker-Structure/bin/php5.6/Dockerfile");
    assert!(
        php56.contains("COMPOSER_CHANNEL=\"--2.2\""),
        "php5.6 must use Composer 2.2 LTS"
    );

    for version in ["7.2", "7.4", "8.1", "8.2", "8.3", "8.4"] {
        let dockerfile = workspace_file(format!(
            "recursos/recursos/Docker-Structure/bin/php{version}/Dockerfile"
        ));

        assert!(
            dockerfile.contains("COMPOSER_CHANNEL=\"--2\""),
            "php{version} must stay on the Composer 2 channel"
        );
    }
}

#[test]
fn php_options_define_client_max_body_size() {
    for (version, compose_tag) in [
        ("5.6", "php56"),
        ("7.2", "php72"),
        ("7.4", "php74"),
        ("8.1", "php81"),
        ("8.2", "php82"),
        ("8.3", "php83"),
        ("8.4", "php84"),
    ] {
        let options = workspace_file(format!(
            "recursos/recursos/Docker-Structure/bin/php{version}/options.{compose_tag}.yml"
        ));

        assert!(
            options.contains("CLIENT_MAX_BODY_SIZE: 16m"),
            "php{version} options must define CLIENT_MAX_BODY_SIZE: 16m"
        );
    }
}

#[test]
fn docker_structure_zip_contains_client_max_body_size_options() {
    let zip_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("recursos/docker-structure.zip");
    let zip_file = fs::File::open(&zip_path)
        .unwrap_or_else(|err| panic!("could not open {}: {err}", zip_path.display()));
    let mut archive = zip::ZipArchive::new(zip_file)
        .unwrap_or_else(|err| panic!("could not read {}: {err}", zip_path.display()));

    for (version, compose_tag) in [
        ("5.6", "php56"),
        ("7.2", "php72"),
        ("7.4", "php74"),
        ("8.1", "php81"),
        ("8.2", "php82"),
        ("8.3", "php83"),
        ("8.4", "php84"),
    ] {
        let entry = format!("bin/php{version}/options.{compose_tag}.yml");
        let mut file = archive
            .by_name(&entry)
            .unwrap_or_else(|err| panic!("zip missing {entry}: {err}"));
        let mut contents = String::new();
        use std::io::Read;
        file.read_to_string(&mut contents)
            .unwrap_or_else(|err| panic!("could not read {entry}: {err}"));

        assert!(
            contents.contains("CLIENT_MAX_BODY_SIZE: 16m"),
            "zip {entry} must define CLIENT_MAX_BODY_SIZE: 16m"
        );
    }
}

#[test]
fn docker_structure_zip_contains_ngrok_service_template() {
    let zip_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("recursos/docker-structure.zip");
    let zip_file = fs::File::open(&zip_path)
        .unwrap_or_else(|err| panic!("could not open {}: {err}", zip_path.display()));
    let mut archive = zip::ZipArchive::new(zip_file)
        .unwrap_or_else(|err| panic!("could not read {}: {err}", zip_path.display()));

    let mut file = archive
        .by_name("services/ngrok.yml")
        .unwrap_or_else(|err| panic!("zip missing services/ngrok.yml: {err}"));
    let mut contents = String::new();
    use std::io::Read;
    file.read_to_string(&mut contents)
        .unwrap_or_else(|err| panic!("could not read services/ngrok.yml: {err}"));

    assert!(contents.contains("WSDD-Ngrok-Server"));
    assert!(contents.contains("127.0.0.1:__WSDD_NGROK_API_PORT__:4040"));
}

#[test]
fn embedded_compose_groups_match_managed_sources() {
    use std::io::Read;

    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let archive_file = fs::File::open(root.join("recursos/docker-structure.zip")).unwrap();
    let mut archive = zip::ZipArchive::new(archive_file).unwrap();
    let mut grouped = vec![("init.yml".to_string(), "wsdd-core")];
    grouped.extend(
        [
            "services/redis.yml",
            "services/memcached.yml",
            "services/mailpit.yml",
            "services/ngrok.yml",
        ]
        .into_iter()
        .map(|path| (path.to_string(), "wsdd-tools")),
    );
    grouped.extend(
        [
            ("5.6", "56"),
            ("7.2", "72"),
            ("7.4", "74"),
            ("8.1", "81"),
            ("8.2", "82"),
            ("8.3", "83"),
            ("8.4", "84"),
        ]
        .into_iter()
        .map(|(version, tag)| {
            (
                format!("bin/php{version}/webserver.php{tag}.yml"),
                "wsdd-web",
            )
        }),
    );

    for (path, project) in grouped {
        let source = workspace_file(format!("recursos/recursos/Docker-Structure/{path}"));
        let mut packaged = String::new();
        archive
            .by_name(&path)
            .unwrap()
            .read_to_string(&mut packaged)
            .unwrap();
        assert_eq!(
            packaged, source,
            "embedded {path} differs from managed source"
        );
        assert!(
            packaged.starts_with(&format!("name: {project}\n")),
            "wrong Compose group in {path}"
        );
    }

    let source = workspace_file("recursos/recursos/PS-Script/wsdd-runtime-recovery.ps1");
    let archive_file = fs::File::open(root.join("recursos/ps-script.zip")).unwrap();
    let mut archive = zip::ZipArchive::new(archive_file).unwrap();
    let mut packaged = String::new();
    archive
        .by_name("PS-Script/wsdd-runtime-recovery.ps1")
        .unwrap()
        .read_to_string(&mut packaged)
        .unwrap();
    assert_eq!(
        packaged, source,
        "embedded WSL recovery worker differs from source"
    );
    assert!(packaged.contains("'-p', 'wsdd-web'"));
}

#[test]
fn wsl_runtime_recovery_worker_is_packaged() {
    let source = workspace_file("recursos/recursos/PS-Script/wsdd-runtime-recovery.ps1");
    for token in [
        "Global\\WSDD-WslRuntimeRecovery",
        "Diagnose",
        "Recover",
        "docker info",
        "--no-build",
        "--force-recreate",
        "No WSL runtime targets",
        "RecoveryBlocked",
        "ConsecutiveFailures",
        "MaxRecoveryFailures",
        "ResetCircuit",
        "UnhealthyNonRelay",
    ] {
        assert!(source.contains(token), "worker must contain {token}");
    }

    let zip_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("recursos/ps-script.zip");
    let zip_file = fs::File::open(&zip_path)
        .unwrap_or_else(|err| panic!("could not open {}: {err}", zip_path.display()));
    let mut archive = zip::ZipArchive::new(zip_file)
        .unwrap_or_else(|err| panic!("could not read {}: {err}", zip_path.display()));
    assert!(
        archive
            .by_name("PS-Script/wsdd-runtime-recovery.ps1")
            .is_ok(),
        "ps-script.zip must contain the recovery worker"
    );
}

#[test]
fn launcher_hides_pwsh_discovery_process() {
    let launcher = std::fs::read_to_string("wsdd-launcher/src/main.rs")
        .expect("launcher source should be readable");
    assert!(
        launcher.contains("hidden_command(\"where.exe\")"),
        "where.exe must use the same hidden process configuration as PowerShell"
    );
    assert!(
        launcher.contains("DOCKER_READY_TIMEOUT_SECONDS"),
        "launcher must use the documented Docker readiness timeout"
    );
    assert!(
        launcher.contains("const DOCKER_READY_TIMEOUT_SECONDS: &str = \"180\""),
        "scheduled recovery must wait up to 180 seconds for Docker Desktop"
    );
}
