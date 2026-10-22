# wc3-toolkit

Open-source tools for Warcraft III Reforged map creation. The first tool is an external
replacement for the World Editor's Object Editor: same map data, better UX. Its requirements
and the user's pet peeves live in `docs/pet-peeves.md`.

- `object-editor/` — Rust workspace (Tauri app later). `crates/casc` reads the game's CASC
  storage via the vendored CascLib; `crates/wc3-data` loads game data: `game` (layered file
  access), `units` (every unit with all Object Editor fields and editor-style display values),
  `icon` (DDS → RGBA/PNG), plus the SLK/profile/metadata parsers. `wc3data` is a CLI inspector
  (`cargo run --release -p wc3-data --bin wc3data -- unit Hpal`). It also has the map formats:
  `objects` (all Object Editor files, v1–v3, byte-exact round trip) and `strings` (`war3map.wts`).
  `crates/mpq` reads/writes map archives via the vendored StormLib (built with CMake).
  `crates/wc3-map` opens maps, saves through a temp copy (`save_files`), and `units::MapUnits`
  applies a map's unit changes over the game data and edits them; `abilities::MapAbilities`
  does the same for abilities (per level), and owns the ability tables `transfer` reads and
  imports into. The map's string table lives in `MapUnits`; ability text goes through
  `MapUnits::strings_mut`. `transfer` moves custom
  units with the custom units, abilities and buffs they reference between maps: share codes
  (`W3OBJ1:` + base64 of deflated object files) and imports, renaming taken IDs and every
  reference to them (found through field metadata types). Bundle files (`.w3obj`) also carry
  the files imported into the source map that the objects use; `imports` finds them by trying
  names from art fields (protected maps cannot list their files) and following models to their
  textures, and adds them to a map with `war3map.imp` entries. `wc3map` is its CLI
  (`check`, `units`, `unit`, `set`, `new`, `lua`).
- Everything must build and run on Windows and Linux (CI tests both). Windows specifics:
  StormLib's library is named `StormLib` and needs `wininet` + `user32`; CascLib needs
  `ws2_32` and `CASCLIB_NO_AUTO_LINK_LIBRARY` (its header otherwise auto-links a prebuilt
  `CascLibRAD.lib`); MSVC rejects `strncpy` and similar CRT functions as deprecated, and the
  shim builds with warnings as errors.
- `object-editor/app/` — the Tauri 2 editor: React 19 + TypeScript + Vite + Tailwind 4 +
  react-aria-components, Bun, Biome with the AGE baseline plus the React domain. Rust side in
  `app/src-tauri` (workspace member): `view.rs` shapes game data + map changes for the UI
  (`abilities.rs` the same for abilities: list, cards, pages with per-level cells),
  `commands.rs` (Tauri commands), `icons.rs` (`wc3icon:` URL scheme serving PNG icons),
  `platform.rs` (install/maps folder detection, the only platform-specific module).
  `api.ts`/`types.ts` mirror the Rust types. Run with `bun run tauri dev` from `app/`.
- `map/src/` — example Lua modules for the harness (`wc3map <map> lua <dir>`, implemented in
  `crates/wc3-map/src/lua.rs`): appends the modules plus a `Dev.OnStart` loader and a `main()`
  hook to the editor's `war3map.lua` in a `<map>_dev.w3m` copy named `[DEV] ...`, after
  checking every file as Lua 5.3 with `full_moon`.

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
- Abilities (`wc3-data::abilities`): per-level columns are `<field><level>` (`Cool2`), spell data
  `Data<letter><level>` (`DataA1`); metadata `useSpecific`/`notSpecific` say which apply to which
  ability code and give the labels. `AbilityData.slk` has columns for 6 levels; empty (`-`)
  means 0 in game. Per-level profile text (`Tip`, `Ubertip`) is a comma list, one per level.
  `data/ability-fields.toml` is the guide for the 70 generic ability fields (not the
  spell-specific data fields, whose editor names already explain them). Tooltip placeholders `<ID,Column[,%]>` can point at ability,
  upgrade (`base1`, `mod1`) or unit tables (`realHP`, `mindmg1`); all game tooltips resolve.
- Reforged moved some SLK columns into skin profile files; lookups fall back to the profile.

## Map format notes

- Object files (`war3map.w3u/.w3t/.w3b/.w3d/.w3a/.w3h/.w3q`) share one layout. Version 3 groups
  modifications into sets with a flags word (meaning of the flags not yet known; maps seen so
  far use 0). Leveled files (`.w3d/.w3a/.w3q`) store `type, level, column` after the field ID —
  type first. In `.w3a`, fields without levels use level 0 column 0; per-level fields use
  level 1.. and the data column (`DataA` = 1, generic ones like `Cool` = 0).
- Bit flags (`stackFlags`, `versionFlags`, ...) and numbered choices (`attackBits`,
  `defenseTypeInt`, ...) are stored as integers; `FieldMeta::is_integer` lists them.
  `UnitEditorData.txt` gives other accepted spellings as `NN_Alt` (`invu` = invulnerable).
- Reforged splits unit changes: gameplay fields in `war3map.w3u`, "netsafe" (cosmetic) fields
  in `war3mapSkin.w3u`. Confirmed for names; other fields follow the metadata's `netsafe` flag.
- Text fields hold `TRIGSTR_007` references into `war3map.wts` (UTF-8 BOM, CRLF, `//` comments).
- Round trip verified on every map in the user's Maps folder (135 maps) and Legion TD
  (all 15 object/string files).
- `war3map.imp` (the Import Manager's list): version, count, then a flag byte and a
  NUL-terminated path per entry. Classic flags: 5/8 = under `war3mapImported\`, 10/13 = full
  path; Reforged adds `0x10` (24 = `KoboldDigger.mdx` under `war3mapImported\`). Entries added
  here use 29 (full path); not yet confirmed in the World Editor. Protected maps have no
  `(listfile)` and often no `war3map.imp`, but files still read by name.
- Never commit Blizzard game files or assets; tools read them from the user's install.
- `crates/wc3-data/data/unit-fields.toml` is the field guide (written for this project):
  importance order per category, `advanced` (folded in the UI), a description for every unit
  field, `unverified` where not confirmed in game. A real-install test checks it covers every
  field. "Armor Type" is only the hit sound; "Defense Type" drives the damage table.
