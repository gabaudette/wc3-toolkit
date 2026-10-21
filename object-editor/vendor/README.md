# Vendored libraries

## CascLib

- Source: https://github.com/ladislav-zezula/CascLib
- Version: tag `3.0` (commit `4971d36`, 2026-02-15). The newer `master` did not compile on Linux.
- License: MIT (`CascLib/LICENSE`)
- Removed from the upstream tree: `listfile/` (file-name lists for other Blizzard games),
  `debian/`, Visual Studio `.sln`/`.vcxproj` files and the `.git` directory.
- Built by `crates/casc/build.rs`; the sources are otherwise unmodified.

## StormLib

- Source: https://github.com/ladislav-zezula/StormLib
- Version: tag `v9.40` (commit `6bb1882`, 2026-07-04).
- License: MIT (`StormLib/LICENSE`)
- Removed from the upstream tree: the prebuilt `storm.dll`, `doc/`, `test/`, `.github/`, `.vscode/`, Xcode, Premake and
  Visual Studio project files, and the `.git` directory.
- Built by `crates/mpq/build.rs` through StormLib's own CMake build (with its bundled zlib,
  bzip2 and LibTomCrypt); the sources are otherwise unmodified.
