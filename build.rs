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

// build.rs — script de compilacion
// Embebe el icono en el exe (PE resources) y declara requireAdministrator via winres.

fn main() {
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-env-changed=CARGO_PKG_VERSION");
    // Recompilar si cambian recursos
    println!("cargo:rerun-if-changed=recursos/recursos.zip");
    println!("cargo:rerun-if-changed=recursos/recursos/PS-Script/");
    println!("cargo:rerun-if-changed=recursos/recursos/Docker-Structure/");
    println!("cargo:rerun-if-changed=assets/WSDD-64.ico");
    println!("cargo:rerun-if-changed=wsdd.manifest");

    // Embeber icono + manifest requireAdministrador en el exe (solo Windows, linker MSVC)
    #[cfg(target_os = "windows")]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/WSDD-64.ico");
        res.set_manifest_file("wsdd.manifest");
        if let Ok(version) = std::env::var("CARGO_PKG_VERSION") {
            res.set("FileVersion", &version);
            res.set("ProductVersion", &version);
        }
        if let Err(e) = res.compile() {
            eprintln!("winres error: {e}");
            std::process::exit(1);
        }
    }
}
