use std::env;
use std::path::PathBuf;

fn main() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let native = manifest.join("../../native");
    let kern = native.join("kern");
    let cfg = native.join("config");

    // Original tree is built with MinGW gcc (`__builtin_clzll`, -ffp-contract=off).
    let mut build = cc::Build::new();
    if let Ok(gcc) = env::var("DSEE_GCC") {
        build.compiler(gcc);
    } else if which("gcc") {
        build.compiler("gcc");
    }
    build
        .std("c11")
        .flag_if_supported("-ffp-contract=off")
        .flag_if_supported("-fopenmp")
        .flag_if_supported("-mavx")
        .flag_if_supported("-mfma")
        .opt_level(3)
        .define("DSEE_BUILDING", "1")
        .warnings(false)
        // Note: do not pass -flto on C objects — rustc links with
        // -fno-use-linker-plugin, and mingw ar cannot archive gcc LTO bitcode.
        .include(kern.join("include"))
        .include(kern.join("common"))
        .include(kern.join("common/ext80"))
        .include(kern.join("open/codebook"))
        .include(kern.join("dsx"))
        .include(kern.join("dsx/band"))
        .include(kern.join("dsx/upsample"))
        .include(cfg.join("include"))
        .include(cfg.join("probe"));

    for root in [&kern, &cfg] {
        for entry in walkdir::WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
            let p = entry.path();
            let ext = p.extension().and_then(|s| s.to_str());
            match ext {
                Some("c") => {
                    println!("cargo:rerun-if-changed={}", p.display());
                    build.file(p);
                }
                Some("h") => {
                    println!("cargo:rerun-if-changed={}", p.display());
                }
                _ => {}
            }
        }
    }

    build.compile("dsee_c_ref");
    // OpenMP for per-slot F76/F80 channel parallelism.
    // Dynamic libgomp — pack-portable.ps1 must ship its MinGW DLL closure.
    println!("cargo:rustc-link-lib=gomp");
    println!("cargo:rustc-link-arg=-fopenmp");
}

fn which(name: &str) -> bool {
    std::process::Command::new("where")
        .arg(name)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
