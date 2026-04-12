fn main() {
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-env-changed=CARGO_PKG_VERSION");
    println!("cargo:rerun-if-env-changed=WSDD_SKIP_WINDOWS_MANIFEST");
    println!("cargo:rerun-if-changed=../assets/WSDD-64.ico");
    println!("cargo:rerun-if-changed=wsdd-launcher.manifest");

    #[cfg(target_os = "windows")]
    {
        let skip_manifest = std::env::var_os("WSDD_SKIP_WINDOWS_MANIFEST").is_some();
        let mut res = winres::WindowsResource::new();
        res.set_icon("../assets/WSDD-64.ico");
        if !skip_manifest {
            res.set_manifest_file("wsdd-launcher.manifest");
        }
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
