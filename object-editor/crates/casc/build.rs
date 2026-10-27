use std::path::PathBuf;

const CPP_SOURCES: &[&str] = &[
    "common/Common.cpp",
    "common/Directory.cpp",
    "common/Csv.cpp",
    "common/FileStream.cpp",
    "common/FileTree.cpp",
    "common/ListFile.cpp",
    "common/Mime.cpp",
    "common/RootHandler.cpp",
    "common/Sockets.cpp",
    "hashes/md5.cpp",
    "hashes/sha1.cpp",
    "overwatch/apm.cpp",
    "overwatch/cmf.cpp",
    "overwatch/aes.cpp",
    "CascDecompress.cpp",
    "CascDecrypt.cpp",
    "CascDumpData.cpp",
    "CascFiles.cpp",
    "CascFindFile.cpp",
    "CascIndexFiles.cpp",
    "CascOpenFile.cpp",
    "CascOpenStorage.cpp",
    "CascReadFile.cpp",
    "CascRootFile_Diablo3.cpp",
    "CascRootFile_Install.cpp",
    "CascRootFile_MNDX.cpp",
    "CascRootFile_Text.cpp",
    "CascRootFile_TVFS.cpp",
    "CascRootFile_OW.cpp",
    "CascRootFile_WoW.cpp",
];

const C_SOURCES: &[&str] = &[
    "jenkins/lookup3.c",
    "zlib/adler32.c",
    "zlib/crc32.c",
    "zlib/inffast.c",
    "zlib/inflate.c",
    "zlib/inftrees.c",
    "zlib/zutil.c",
];

fn main() {
    // Read when the script runs, not when it compiles (`env!`): Cargo reuses a
    // compiled build script for another checkout of the same crate that shares
    // the target folder (a worktree), and a baked-in path points at the old one.
    // Cargo runs build scripts from the crate folder, so the fallback is "here".
    let manifest = std::env::var_os("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_default();
    let src = manifest.join("../../vendor/CascLib/src");

    // On MSVC, CascLib.h otherwise asks the linker for a prebuilt
    // `CascLibRAD.lib` (`#pragma comment(lib, ...)`); this crate builds CascLib
    // from source instead.
    let no_auto_link = "CASCLIB_NO_AUTO_LINK_LIBRARY";

    // Archives link in the order they are built, so the shim comes first: its
    // references into CascLib then resolve against the archives that follow.
    cc::Build::new()
        .cpp(true)
        .include(&src)
        .define(no_auto_link, None)
        .file(manifest.join("shim.cpp"))
        .warnings(true)
        .warnings_into_errors(true)
        .compile("casc_shim");

    // AGE-QUAL-003: compiler warnings are silenced only for the vendored,
    // third-party CascLib sources, which this project does not modify.
    cc::Build::new()
        .cpp(true)
        .include(&src)
        .define(no_auto_link, None)
        .files(CPP_SOURCES.iter().map(|f| src.join(f)))
        .warnings(false)
        .compile("casc_cpp");
    cc::Build::new()
        .include(&src)
        .define(no_auto_link, None)
        .files(C_SOURCES.iter().map(|f| src.join(f)))
        .warnings(false)
        .compile("casc_c");

    // CascLib's network code uses Winsock on Windows.
    let target = std::env::var("TARGET").unwrap_or_default();

    if target.contains("windows") {
        println!("cargo:rustc-link-lib=ws2_32");
    }

    println!("cargo:rerun-if-changed=shim.cpp");
    println!("cargo:rerun-if-changed={}", src.display());
}
