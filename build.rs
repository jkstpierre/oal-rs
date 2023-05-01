extern crate bindgen;
extern crate cmake;

use bindgen::callbacks::ParseCallbacks;
use cmake::Config;
use std::{env, path::PathBuf, process::Command};

/**
 * Clone and build the OpenAL software github repository
 */
fn build_openal_soft(openal_directory: PathBuf) {
    const BRANCH: &str = "v1.19";

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
    let mut dst = Config::new(openal_directory)
        .define("LIBTYPE", "SHARED")
        .define("ALSOFT_UTILS", "OFF")
        .define("ALSOFT_EXAMPLES", "OFF")
        .define("ALSOFT_TESTS", "OFF")
        .no_build_target(true)
        .build();

    #[cfg(debug_assertions)]
    {
        dst = dst.join("build/Debug");
    }
    #[cfg(not(debug_assertions))]
    {
        dst = dst.join("build/Release");
    }

    // Set link search for cargo
    println!("cargo:rustc-link-search=all={}", dst.display());

    // Link dynamic libraries
    println!("cargo:rustc-link-lib={}=common", "dylib");
    println!("cargo:rustc-link-lib={}=OpenAL32", "dylib");
}

/**
 * Build api bindings to the C functions described in the OpenAL-soft headers
 */
fn build_bindings(mut openal_directory: PathBuf) {
    openal_directory = openal_directory.join("include/AL");

    let bindings = bindgen::Builder::default()
        .header(openal_directory.join("al.h").to_str().unwrap())
        .header(openal_directory.join("alc.h").to_str().unwrap())
        .header(openal_directory.join("alext.h").to_str().unwrap())
        .header(openal_directory.join("efx.h").to_str().unwrap())
        .header(openal_directory.join("efx-presets.h").to_str().unwrap())
        .parse_callbacks(Box::new(CustomParse))
        .generate()
        .expect("Unable to generate openal-soft bindings.");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Failed to write bindings!")
}

#[derive(Debug)]
struct CustomParse;
impl ParseCallbacks for CustomParse {
    fn will_parse_macro(&self, _name: &str) -> bindgen::callbacks::MacroParsingBehavior {
        bindgen::callbacks::MacroParsingBehavior::Default
    }

    fn int_macro(&self, _name: &str, _value: i64) -> Option<bindgen::callbacks::IntKind> {
        Some(bindgen::callbacks::IntKind::I32)
    }

    fn str_macro(&self, _name: &str, _value: &[u8]) {}

    fn func_macro(&self, _name: &str, _value: &[&[u8]]) {}

    fn enum_variant_behavior(
        &self,
        _enum_name: Option<&str>,
        _original_variant_name: &str,
        _variant_value: bindgen::callbacks::EnumVariantValue,
    ) -> Option<bindgen::callbacks::EnumVariantCustomBehavior> {
        None
    }

    fn enum_variant_name(
        &self,
        _enum_name: Option<&str>,
        _original_variant_name: &str,
        _variant_value: bindgen::callbacks::EnumVariantValue,
    ) -> Option<String> {
        None
    }

    fn item_name(&self, _original_item_name: &str) -> Option<String> {
        None
    }

    fn include_file(&self, _filename: &str) {}
}

fn main() {
    // Create path for accessing openal-soft C source from github
    let openal_directory = PathBuf::from(env::var("OUT_DIR").unwrap()).join("openal-soft");

    build_openal_soft(openal_directory.clone());
    build_bindings(openal_directory)
}
