use rustc_version::version;

fn main() {
    println!("cargo::rustc-check-cfg=cfg(assets)");
    if let Ok(asset_dir) = std::env::var("ASSET_DIR") {
        println!("cargo:rustc-cfg=assets");
        println!("cargo:rerun-if-changed={asset_dir}");
    }
    println!("cargo:rerun-if-env-changed=ASSET_DIR");
    let rustc_semver = version().expect("could not parse rustc version");
    println!("cargo:rustc-env=RUSTC_SEMVER={rustc_semver}");
    println!("cargo:rerun-if-env-changed=RUSTC");
}
