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
        app_version == "1.0.0" || app_version.starts_with("1.0.0-rc."),
        "release versions must be stable 1.0.0 or a 1.0.0-rc.N candidate"
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
