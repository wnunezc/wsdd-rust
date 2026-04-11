fn main() {
    println!("cargo:rerun-if-changed=../assets/WSDD-64.ico");
    println!("cargo:rerun-if-changed=wsdd-launcher.manifest");

    #[cfg(target_os = "windows")]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("../assets/WSDD-64.ico");
        res.set_manifest_file("wsdd-launcher.manifest");
        if let Err(e) = res.compile() {
            eprintln!("winres error: {e}");
            std::process::exit(1);
        }
    }
}
