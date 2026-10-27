use std::path::PathBuf;

const RUST_C_RUNTIME_PROFILE: &str = "Release";

fn main() {
    let manifest = crate_dir_at_run_time();
    let source = manifest.join("../../vendor/StormLib");

    // AGE-QUAL-003: CMake prints the vendored StormLib's compiler warnings only when
    // the build fails; this project does not modify that third-party code.
    let out = cmake::Config::new(&source)
        .profile(RUST_C_RUNTIME_PROFILE)
        .define("STORM_USE_BUNDLED_LIBRARIES", "ON")
        .define("STORM_BUILD_TESTS", "OFF")
        .define("BUILD_SHARED_LIBS", "OFF")
        .build();

    for dir in ["lib", "lib64"] {
        println!("cargo:rustc-link-search=native={}", out.join(dir).display());
    }

    let target = std::env::var("TARGET").unwrap_or_default();
    let library = if target.contains("windows") {
        "StormLib"
    } else {
        "storm"
    };

    println!("cargo:rustc-link-lib=static={library}");

    if target.contains("windows") {
        for system_library in ["wininet", "user32"] {
            println!("cargo:rustc-link-lib={system_library}");
        }
    }

    if target.contains("apple") {
        println!("cargo:rustc-link-lib=c++");
    } else if !target.contains("msvc") {
        println!("cargo:rustc-link-lib=stdc++");
    }

    println!("cargo:rerun-if-changed={}", source.display());
}

fn crate_dir_at_run_time() -> PathBuf {
    std::env::var_os("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_default()
}
