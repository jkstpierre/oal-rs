extern crate cmake;

use cmake::Config;
use std::{env, fs, path::PathBuf, process::Command};

/**
 * Clone and build the OpenAL software github repository
 */
fn build_openal_soft() {
    const BRANCH: &str = "v1.19";

    let openal_directory = PathBuf::from(env::var("OUT_DIR").unwrap()).join("openal-soft");

    // Perform git clone for OpenAL-Soft
    let status = Command::new("git")
        .arg("clone")
        .args(&["--branch", BRANCH])
        .args(&["--depth", "1"])
        .arg("https://github.com/kcat/openal-soft.git")
        .arg(&openal_directory)
        .status()
        .unwrap();

    if !status.success() {
        let status = Command::new("git")
            .arg("clean")
            .arg("-fdx")
            .current_dir(&openal_directory)
            .status()
            .unwrap();
        assert!(status.success(), "failed to clone openal-soft");
    }

    // Compile OpenAL-Soft into a shared library
    let dst = Config::new(openal_directory)
        .define("LIBTYPE", "SHARED")
        .define("ALSOFT_UTILS", "OFF")
        .define("ALSOFT_EXAMPLES", "OFF")
        .define("ALSOFT_TESTS", "OFF")
        .no_build_target(true)
        .build();

    // Set link search for cargo
    fs::write(
        env::var("OUT_DIR").unwrap() + "/test.txt",
        dst.display().to_string(),
    );
    println!("cargo:rustc-link-search=native={}/build", dst.display());

    // Link dynamic libraries
    println!("cargo:rustc-link-lib={}=common", "dylib");
    println!("cargo:rustc-link-lib={}=openal", "dylib");
}

fn main() {
    build_openal_soft()
}
