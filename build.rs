use std::process::Command;

use rustc_version::version;

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    if std::env::var("ASSET_DIR").is_ok() {
        // Don't run npm, and let the ASSET_DIR value pass through to the static asset handler.
    } else if std::env::var("SKIP_APP_COMPILATION") == Ok("true".into()) {
        // Skip asset compilation, but reuse stale assets in the OUT_DIR directory.
        println!("cargo:rustc-env=ASSET_DIR={out_dir}");
    } else {
        // Build the react app.
        let status = Command::new("npm")
            .current_dir("app")
            .args(["run", "build"])
            .env("BUILD_PATH", &out_dir)
            .status()
            .unwrap();

        assert!(status.success());

        println!("cargo:rerun-if-changed=app/src");
        println!("cargo:rerun-if-changed=app/package.json");
        println!("cargo:rerun-if-changed=app/package-lock.json");
        println!("cargo:rerun-if-changed=app/public");
        println!("cargo:rerun-if-changed=app/index.html");
        println!("cargo:rerun-if-env-changed=API_URL");

        // Point the static asset handler at the build script output directory.
        println!("cargo:rustc-env=ASSET_DIR={out_dir}");
    }

    println!("cargo:rerun-if-env-changed=ASSET_DIR");
    println!("cargo:rerun-if-env-changed=SKIP_APP_COMPILATION");

    let rustc_semver = version().expect("could not parse rustc version");
    println!("cargo:rustc-env=RUSTC_SEMVER={rustc_semver}");
    println!("cargo:rerun-if-env-changed=RUSTC");
}
