# wc3-toolkit

Open-source tools for Warcraft III Reforged map creation. The first tool is an external
replacement for the World Editor's Object Editor: same map data, better UX. Its requirements
and the user's pet peeves live in `docs/pet-peeves.md`.

- `object-editor/` — Rust workspace (Tauri app later). `crates/casc` reads the game's CASC
  storage via the vendored CascLib; `crates/wc3-data` parses SLK tables, profile text files and
  `*MetaData.slk` field definitions.
- `map/` — Lua scripts (`src/`) injected into a dev copy of a test map by `tools/build.py`
  (temporary, Linux-only; map writing moves into the Rust editor). It needs
  `map/tools/lib/libstorm.so` (StormLib, built with `-DBUILD_SHARED_LIBS=ON
  -DSTORM_USE_BUNDLED_LIBRARIES=ON`) and `map/tools/bin/luac` (Lua 5.3, `make posix`). Both are
  local builds and are not committed.

Coding standards: `docs/standards/` (AGE Studio engineering standards) and their application
to this repo plus validation commands in `CONTRIBUTING.md`; follow them for every change. Machine-specific notes (game paths, test maps) are in `CLAUDE.local.md` when present.

## Game data notes

- CASC layers: `war3.w3mod:` base, `_balance\custom_v1.w3mod:` etc. for balance versions,
  `_locales\enus.w3mod:` for text, `_hd.w3mod:` for HD art. Which balance layer a map uses
  comes from its `war3map.w3i`.
- Reforged keeps SD/HD variants of some profile keys (`walk:sd`, `walk:hd`).
- Icons are `.dds` (DXT1/DXT5) even where data files say `.blp`.
- Never commit Blizzard game files or assets; tools read them from the user's install.
