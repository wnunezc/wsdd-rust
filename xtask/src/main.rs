use std::env;
use std::path::PathBuf;
use std::process::Command;

use anyhow::{bail, Context, Result};

fn main() -> Result<()> {
    match env::args().nth(1).as_deref() {
        Some("wix-msi") => build_wix_msi(),
        _ => {
            bail!("uso: cargo wix-msi");
        }
    }
}

fn build_wix_msi() -> Result<()> {
    let workspace_root = workspace_root()?;

    run_cargo(
        &workspace_root,
        &[
            "build",
            "-p",
            "wsdd-launcher",
            "--release",
            "--target",
            "x86_64-pc-windows-msvc",
        ],
    )?;

    run_cargo(
        &workspace_root,
        &["wix", "-p", "wsdd", "--target", "x86_64-pc-windows-msvc"],
    )?;

    Ok(())
}

fn workspace_root() -> Result<PathBuf> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(PathBuf::from)
        .context("xtask no pudo resolver el workspace root")
}

fn run_cargo(workspace_root: &PathBuf, args: &[&str]) -> Result<()> {
    let status = Command::new("cargo")
        .args(args)
        .current_dir(workspace_root)
        .status()
        .with_context(|| format!("no se pudo ejecutar cargo {}", args.join(" ")))?;

    if !status.success() {
        bail!("cargo {} fallo con exit code no exitoso", args.join(" "));
    }

    Ok(())
}
