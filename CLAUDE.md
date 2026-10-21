# wc3-toolkit

Open-source tools for Warcraft III Reforged map creation. The first tool is an external
replacement for the World Editor's Object Editor: same map data, better UX. Its requirements
and the user's pet peeves live in `docs/pet-peeves.md`.

- `object-editor/` — Rust workspace (Tauri app later). `crates/casc` reads the game's CASC
  storage via the vendored CascLib; `crates/wc3-data` loads game data: `game` (layered file
  access), `units` (every unit with all Object Editor fields and editor-style display values),
  `icon` (DDS → RGBA/PNG), plus the SLK/profile/metadata parsers. `wc3data` is a CLI inspector
  (`cargo run --release -p wc3-data --bin wc3data -- unit Hpal`).
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
- Icons are `.dds` (DXT1/DXT5) even where data files say `.blp`. SD icons are 64 px, HD 256 px.
  Some art exists in only one of SD/HD; `GameFiles::read_art` falls back to the other.
- `_de.w3mod:` is another 256 px art layer of unknown purpose; it is not used yet.
- Editor strings can redirect to other keys (`WESTRING_UE_UNITRACE_HUMAN=WESTRING_RACE_HUMAN`)
  and flag labels read `Ground, &Ground` (display label, menu label). Strings come from both
  `WorldEditStrings.txt` and `WorldEditGameStrings.txt`.
- `versionFlags` fields are bit flags (0 = None), not single choices.
- Reforged moved some SLK columns into skin profile files; lookups fall back to the profile.
- Never commit Blizzard game files or assets; tools read them from the user's install.
