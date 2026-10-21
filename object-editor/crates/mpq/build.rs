use std::path::PathBuf;

fn main() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let source = manifest.join("../../vendor/StormLib");

    // StormLib's own CMake build lists its 240+ sources, including bundled zlib,
    // bzip2 and LibTomCrypt, so the project does not duplicate that list. CMake
    // only prints the vendored code's compiler warnings when the build fails
    // (AGE-QUAL-003: third-party code this project does not modify).
    // Always a release build: on MSVC, a debug build would link the debug C
    // runtime while Rust links the release one, and the two cannot be mixed.
    let out = cmake::Config::new(&source)
        .profile("Release")
        .define("STORM_USE_BUNDLED_LIBRARIES", "ON")
        .define("STORM_BUILD_TESTS", "OFF")
        .define("BUILD_SHARED_LIBS", "OFF")
        .build();

    for dir in ["lib", "lib64"] {
        println!("cargo:rustc-link-search=native={}", out.join(dir).display());
    }

    // StormLib's CMake build names the library `StormLib` on Windows, `storm` elsewhere.
    let target = std::env::var("TARGET").unwrap_or_default();
    let library = if target.contains("windows") {
        "StormLib"
    } else {
        "storm"
    };
    println!("cargo:rustc-link-lib=static={library}");

    // StormLib's HTTP stream support uses WinINet and `wsprintf` (user32) on
    // Windows. Its CMake file only passes these to CMake projects, so link them here.
    if target.contains("windows") {
        for system_library in ["wininet", "user32"] {
            println!("cargo:rustc-link-lib={system_library}");
        }
    }

    // StormLib is C++; link the platform's C++ runtime (MSVC links it implicitly).
    if target.contains("apple") {
        println!("cargo:rustc-link-lib=c++");
    } else if !target.contains("msvc") {
        println!("cargo:rustc-link-lib=stdc++");
    }

    println!("cargo:rerun-if-changed={}", source.display());
}
