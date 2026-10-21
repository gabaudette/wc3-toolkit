# Vendored libraries

## CascLib

- Source: https://github.com/ladislav-zezula/CascLib
- Version: tag `3.0` (commit `4971d36`, 2026-02-15). The newer `master` did not compile on Linux.
- License: MIT (`CascLib/LICENSE`)
- Removed from the upstream tree: `listfile/` (file-name lists for other Blizzard games),
  `debian/`, Visual Studio `.sln`/`.vcxproj` files and the `.git` directory.
- Built by `crates/casc/build.rs`; the sources are otherwise unmodified.
