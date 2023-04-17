use std::process::Command;

use rustc_version::version;

fn main() {
    if std::env::var("SKIP_APP_COMPILATION").is_err() {
        let status = Command::new("npm")
            .current_dir("app")
            .args(["run", "build"])
            .env("BUILD_PATH", std::env::var("OUT_DIR").unwrap())
            .status()
            .unwrap();

        assert!(status.success());

        println!("cargo:rerun-if-changed=app/src");
        println!("cargo:rerun-if-changed=app/package.json");
        println!("cargo:rerun-if-changed=app/package-lock.json");
        println!("cargo:rerun-if-changed=app/public");
        println!("cargo:rerun-if-env-changed=API_URL");
    }

    println!("cargo:rerun-if-env-changed=SKIP_APP_COMPILATION");

    let rustc_semver = version().expect("could not parse rustc version");
    println!("cargo:rustc-env=RUSTC_SEMVER={rustc_semver}");
    println!("cargo:rerun-if-env-changed=RUSTC");
}
