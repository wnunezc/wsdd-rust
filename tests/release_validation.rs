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
        app_version == "1.0.2" || app_version.starts_with("1.0.2-rc."),
        "release versions must be stable 1.0.2 or a 1.0.2-rc.N candidate"
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
