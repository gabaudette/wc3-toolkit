use std::path::PathBuf;

fn main() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let source = manifest.join("../../vendor/StormLib");

    // StormLib's own CMake build lists its 240+ sources, including bundled zlib,
    // bzip2 and LibTomCrypt, so the project does not duplicate that list. CMake
    // only prints the vendored code's compiler warnings when the build fails
    // (AGE-QUAL-003: third-party code this project does not modify).
    let out = cmake::Config::new(&source)
        .define("STORM_USE_BUNDLED_LIBRARIES", "ON")
        .define("STORM_BUILD_TESTS", "OFF")
        .define("BUILD_SHARED_LIBS", "OFF")
        .build();

    for dir in ["lib", "lib64"] {
        println!("cargo:rustc-link-search=native={}", out.join(dir).display());
    }
    println!("cargo:rustc-link-lib=static=storm");

    // StormLib is C++; link the platform's C++ runtime (MSVC links it implicitly).
    let target = std::env::var("TARGET").unwrap_or_default();
    if target.contains("apple") {
        println!("cargo:rustc-link-lib=c++");
    } else if !target.contains("msvc") {
        println!("cargo:rustc-link-lib=stdc++");
    }

    println!("cargo:rerun-if-changed={}", source.display());
}
