# wc3-toolkit

Open-source tools for Warcraft III Reforged map creation. The first tool is an external
replacement for the World Editor's Object Editor: same map data, better UX. Its requirements
and the user's pet peeves live in `docs/pet-peeves.md`.

- `object-editor/` — Rust workspace (Tauri app later). `crates/casc` reads the game's CASC
  storage via the vendored CascLib; `crates/wc3-data` loads game data: `game` (layered file
  access), `units` (every unit with all Object Editor fields and editor-style display values;
  `UnitDatabase::load_items` is the same over the item tables), `upgrades`,
  `icon` (DDS → RGBA/PNG), `blp` (BLP1, JPEG or paletted, as maps import), `spells` (the spell
  wizard's kinds and suggested bases, from `data/spell-recipes.toml`, and its triggered spells:
  Channel plus a Lua module filled in from `data/spell-templates/`, listed in
  `data/spell-templates.toml`), plus the
  SLK/profile/metadata parsers. `wc3data` is a CLI inspector
  (`cargo run --release -p wc3-data --bin wc3data -- unit Hpal`). It also has the map formats:
  `objects` (all Object Editor files, v1–v3, byte-exact round trip) and `strings` (`war3map.wts`).
  `crates/mpq` reads/writes map archives via the vendored StormLib (built with CMake).
  `crates/wc3-map` opens maps, saves through a temp copy (`save_files`), and `units::MapUnits`
  applies a map's unit changes over the game data and edits them; `abilities::MapAbilities`
  does the same for abilities (per level), `items::MapItems` for items and
  `upgrades::MapUpgrades` for upgrades (per level); each owns the tables `transfer` reads and
  imports into. The map's string table lives in `MapUnits`; ability, item and upgrade text goes
  through `MapUnits::strings_mut`. `transfer` moves custom units with the custom units,
  abilities, buffs, items and upgrades they reference between maps: share codes
  (`W3OBJ1:` + base64 of deflated object files) and imports, renaming taken IDs and every
  reference to them (found through field metadata types). Bundle files (`.w3obj`) also carry
  the files imported into the source map that the objects use; `imports` finds them by trying
  names from art fields (protected maps cannot list their files) and following models to their
  textures, and adds them to a map with `war3map.imp` entries. `placements` reads and
  writes `war3mapUnits.doo` (placed units: hero inventories, drops, item table links) and
  `info` the properties, players, forces and item tables of `war3map.w3i`, both byte for
  byte; `constants` reads and writes `war3mapMisc.txt` (gameplay constants). `wc3map` is its CLI
  (`check`, `units`, `unit`, `set`, `new`, `lua`).
- Everything must build and run on Windows and Linux (CI tests both). Windows specifics:
  StormLib's library is named `StormLib` and needs `wininet` + `user32`; CascLib needs
  `ws2_32` and `CASCLIB_NO_AUTO_LINK_LIBRARY` (its header otherwise auto-links a prebuilt
  `CascLibRAD.lib`); MSVC rejects `strncpy` and similar CRT functions as deprecated, and the
  shim builds with warnings as errors.
- `object-editor/app/` — the Tauri 2 editor, named **Tinkerbench** (gnome icon in
  `src-tauri/icons/app-icon.png`; `bun run tauri icon` makes the other sizes): React 19 + TypeScript + Vite + Tailwind 4 +
  react-aria-components, Bun, Biome with the AGE baseline plus the React domain. Rust side in
  `app/src-tauri` (workspace member): `view.rs` shapes game data + map changes for the UI
  (`abilities.rs` the same for abilities: list, cards, pages with per-level cells),
  `commands.rs` (Tauri commands), `mapinfo.rs` (the Map tab: properties, players,
  forces and gameplay constants), `loot.rs` (the Placed units tab: inventories, drops,
  item tables and cascades, whose tiers are kept in `<map> Loot.json`), `interface.rs` (the Interface tab: dialogs, multiboards, leaderboards and timer windows, in `interface/`, designed visually,
  kept in `<map> UI.json` beside the map, and the Lua modules they become; `.w3ui` design files carry them and their imported icons between maps), `icons.rs` (`wc3icon:`/`wc3tex:`/`wc3model:` URL schemes:
  the open map's imported file first, then the game's; they read game files outside the
  session lock, since `casc::Storage` is `Sync`, and textures decode at most 512 px),
  `platform.rs` (install/maps folder detection, and on Linux keeping WebKitGTK's threads off
  real-time priority, whose CPU limit SIGKILLed the app; the only platform-specific module).
  `api.ts`/`types.ts` mirror the Rust types. Run with `bun run tauri dev` from `app/`.
  The look ("Arcane": tokens, type, components, accessibility and copy rules) is in
  `docs/design.md`; follow it for every screen.
- `map/src/` — example Lua modules for the harness (`wc3map <map> lua <dir>`, implemented in
  `crates/wc3-map/src/lua.rs`): appends the modules plus a `Dev.OnStart` loader and a `main()`
  hook to the editor's `war3map.lua` in a `<map>_dev.w3m` copy named `[DEV] ...`, after
  checking every file as Lua 5.3 with `full_moon`.

Every pull request adds an entry to `CHANGELOG.md` (user-facing, shown in the app under
"What's new"; performance-only or technical-only work is just "Performance improvements." or
"Internal improvements.") and to `docs/devlog.md` (technical details); see `CONTRIBUTING.md`.

Coding standards: `docs/standards/` (AGE Studio engineering standards) and their application
to this repo plus validation commands in `CONTRIBUTING.md`; follow them for every change. Machine-specific notes (game paths, test maps) are in `CLAUDE.local.md` when present.

## Game data notes

- CASC layers: `war3.w3mod:` base, `_balance\custom_v1.w3mod:` etc. for balance versions,
  `_locales\enus.w3mod:` for text, `_hd.w3mod:` for HD art. Which balance layer a map uses
  comes from its `war3map.w3i`.
- Reforged keeps SD/HD variants of some profile keys (`walk:sd`, `walk:hd`).
- Icons are `.dds` (DXT1/DXT5) even where data files say `.blp`. SD icons are 64 px, HD 256 px.
  Some art exists in only one of SD/HD; `GameFiles::read_art` falls back to the other.
- `_de.w3mod:` is another HD art layer (about 6000 models); the newer campaign units' models
  (Roderick, Scarlet Crusade, Forsaken) exist only there. `read_art` uses it last.
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
- Items use the unit metadata's fields marked `useItem` over `ItemData.slk` and the item
  profiles (`ItemFunc.txt`, `ItemSkin.txt`, `ItemStrings.txt`); guide in `data/item-fields.toml`.
- Upgrades (`wc3-data::upgrades`): `UpgradeData.slk` has one value for all levels, as a base and
  a step per level (`goldbase`/`goldmod`, `base1`/`mod1`). Names, tooltips, hotkeys and icons
  are per-level profile lists; requirements use a key per level instead (`Requires` for level 1,
  `Requires1` for level 2). The editor fills the `%s` in `Effect 1 - %s` itself. Guide in
  `data/upgrade-fields.toml`.
- Reforged moved some SLK columns into skin profile files; lookups fall back to the profile.

## Map format notes

- Object files (`war3map.w3u/.w3t/.w3b/.w3d/.w3a/.w3h/.w3q`) share one layout. Version 3 groups
  modifications into sets with a flags word (meaning of the flags not yet known; maps seen so
  far use 0). Leveled files (`.w3d/.w3a/.w3q`) store `type, level, column` after the field ID —
  type first. In `.w3a`, fields without levels use level 0 column 0; per-level fields use
  level 1.. and the data column (`DataA` = 1, generic ones like `Cool` = 0). `.w3q` is the same
  with the data column always 0. The World Editor stores some upgrade choices (class, effect)
  as `TRIGSTR` references.
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
- `war3mapUnits.doo`: versions 12/13 add a group ID after the skin and roll, pitch and
  lights at the end of each unit; subversion 11 adds the item table link and hero
  attributes (the 2.0 World Editor writes 13 with subversion 9). Random drops are
  `Y<class>I<level>` (`YYI` any class, `i`..`o` in JASS `ITEM_TYPE` order).
- `war3mapMisc.txt` (gameplay constants) holds only what a map changes: sorted sections and
  keys, CRLF. Constants are described by `MiscMetaData.slk`; their guide is
  `crates/wc3-data/data/constant-fields.toml`.
- Never commit Blizzard game files or assets; tools read them from the user's install.
- `crates/wc3-data/data/unit-fields.toml` is the field guide (written for this project):
  importance order per category, `advanced` (folded in the UI), a description for every unit
  field, `unverified` where not confirmed in game. A real-install test checks it covers every
  field. "Armor Type" is only the hit sound; "Defense Type" drives the damage table.
