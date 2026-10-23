# Devlog

The technical story of Tinkerbench, newest first: what changed, why, and what we learned.
Users read the [changelog](../CHANGELOG.md) instead, where performance and internal work is a
single line. Every pull request adds an entry here and to the changelog.

## 2026-10-23 · Documentation website (PR #27, AGE-20)

- Astro Starlight site in `website/`, with the Workshop direction selected from four design
  studies: graphite, mint, DM Sans, and the existing gnome logo/favicon. Fourteen authored
  pages cover installation, the first map, objects, sharing, the Lua harness, common tasks,
  JASS/Lua conventions and the map format notes. The public URL is
  `https://tinkerbench-docs.vercel.app`, also used for canonical links and sitemaps.
- A local CASC export example reads script and field inputs. Only structured declarations
  and field metadata are versioned; game script bodies and comments stay outside the repo.
  The initial extraction records source hashes and its local date without claiming a patch
  version. The generator emits 5,131 API and 1,421 field pages, joins the seven live field
  guides, preserves unverified markers and identifies metadata without an authored guide.
- API URLs include the declaration kind: `rect` and `Rect` otherwise collide after URL
  normalization. A small project-authored grammar highlights JASS. Pagefind indexes the
  static build; CI validates parsing, types, the site build and local links/assets. Vercel
  builds from the repository root so changes to the shared field guides rebuild the site.
  A checked Build Output API package includes the static files and routing explicitly; the
  generic project preset had reported success while producing an empty deployment.
- Vercel's ignored-build script compares against the last successful branch deployment and
  watches the website, shared field guides and hosting configuration. It skips unrelated
  edits, preserves multi-commit changes, and builds if history is unavailable. Integration
  tests exercise real Git commits. A project-level guard also skips branches without docs,
  preventing unrelated main pushes from replacing the live site before this PR merges.

## 2026-10-23 · TGA textures (AGE-18) and the model preview (AGE-17)

- `.tga` imports were found and bundled but not decoded. `tga.rs` reads true-color (24/32-bit)
  and grayscale (8-bit) images, uncompressed and run-length encoded, either row order.
  `decode_texture` now recognizes DDS and BLP by their magic and TGA, which has none, by a
  validated header. None of the 135 maps nor Legion TD imports a `.tga`, so the tests are
  synthetic plus one end to end (a map holding `BTNRed.tga`, found from a `.blp` path).
- AGE-17 ("Missing shader uniform location: uRoughness" on Legion TD's bench) no longer
  happens: Royal Presence (`u99r`) renders. The throw exists only in war3-model's TypeScript
  source; the shipped 4.0.1 build skips a missing uniform, and the preview never loads
  reflection maps, which is what runs that shader.
- A sweep over every unit model (1852 loads, SD and HD, with their textures) found 99
  failures, all "file not found". The `_de.w3mod:` layer holds about 6000 models, and those of
  the newer campaign units (Roderick, Baron Rivendare, the Forsaken, the Scarlet Crusade)
  exist nowhere else, so `read_art` now looks there last. `model_file` replaces any extension
  with `.mdx`, as the game does, for the cinematic doodads that name `Empty.ma`. What is left
  is not shipped by the game: the `IntentionallyLeftBlank` and `AbominationCIN` models (the
  real file sits under `abomination\`) and the HD `TornadoBound_Diffuse` texture. The sweep
  is now the real-install test `every_unit_model_loads_for_the_preview`, which lists those
  three; the preview says "it is not in the game files or the map" for them.

## 2026-10-23 · Doodads (AGE-15, part 2)

- Doodads are leveled by model variation: the tint (`dvr1`, `dvg1`, `dvb1`) is per variation,
  the other 40 fields are plain. `DoodadDatabase` reads `DoodadMetaData.slk` over
  `Doodads.slk`, with art from `DoodadSkins.txt`, where per-variation values are lists with
  one entry per variation (`vertR=255,255,...`, always 10 entries; Mushrooms have 4 variations).
- Upgrades and doodads now share one map type: `MapUpgrades` became `LeveledObjects`,
  configured by a `LeveledKind` (files, ID letter, label, and the fields holding the name,
  icon, list filter and level count), over a `LeveledData` trait that `UpgradeDatabase` and
  `DoodadDatabase` implement. The app's upgrade commands became catalog-aware
  (`list_leveled`, ...), and `UpgradesPane` became `LeveledPane`. Level tables call their
  columns "Variation" for doodads.
- Variations are stored as levels 1..N, data column 0. That is an assumption: none of the 135
  maps nor Legion TD changes a variation, so it still needs checking against a map saved by
  the World Editor.
- Found on the way: destructable metadata gives list types (tilesets, targets) index 0, which
  the unit database read as "the first entry". It now means the whole list for list types, so
  a destructable offered in two tilesets shows both.
- The units database also loads the editor's categories, since it formats values for every tab.

## 2026-10-23 · Buffs and destructables (AGE-15, part 1)

- Both have the shape items have: plain fields over one table plus profiles. Rather than copy
  the item code twice, `UnitDatabase` gained a catalog (units, items, buffs, destructables)
  and one `load_catalog`, and `MapItems` became `PlainObjects`, configured by a `PlainKind`
  (files, custom ID letter, the label string comments use, the name field).
- Buffs: `AbilityBuffMetaData.slk` over `AbilityBuffData.slk`; their text and art sit in the
  ability profiles next to the abilities (`[BHbd]` in `HumanAbilityStrings.txt`). Names come
  from `EditorName`, else the tooltip title. Destructables: `DestructableMetaData.slk` over
  `DestructableData.slk` and `DestructableSkin.txt`, named by editor string keys
  (`WESTRING_DEST_SUMMER_TREE_WALL`), which the database now resolves.
- Neither metadata file sets the unit/hero/item flags, so `ObjectKind::Buff` and
  `ObjectKind::Destructable` apply every field.
- `MapExtras`, which only kept buffs as stored for sharing, is gone: buffs are a
  `PlainObjects` too, so the buffs abilities bring along can now be edited.
- `OpenMap::load` reads every part of a map itself; `OpenMap::new` would have needed nine
  arguments. The app's item commands became catalog-aware (`list_plain`, `set_plain_field`,
  ...), and `ItemsPane` became `PlainPane`, used by three tabs.
- A first draft of the plain-object commands used raw pointers to borrow the catalog's
  objects and the string table at once; destructuring `OpenMap` gives two separate borrows
  without `unsafe`.
- Destructable categories come from `WorldEditData.txt` (`[DestructibleCategories]`, keyed by
  letter: `D=WESTRING_DTYPE_DESTRUCTABLE,...`), read from the text because profiles lowercase
  their keys. Tilesets stay raw: their letters differ only by case and choices match without
  case. Destructable models carry neither extension nor variation number
  (`AshenTree` → `AshenTree0.mdl` when there are several variations), so the preview adds them.
- Profile text is quoted when long (`Buffubertip="..."`); `raw_value` now returns what is inside
  the quotes, so value boxes no longer show them and typing the same text back is no change.
- Field guides for all 27 buff and 57 destructable fields. Doodads come next: they have
  per-variation values, so they need the level-table page.

## 2026-10-23 · Colors instead of color codes (AGE-22)

- `GameText` renders names everywhere they are displayed: the four object lists, page headers
  and suffixes, ability cards, the ability picker, the Ctrl K palette, the share and import
  lists, the base value column and the inspector. Its `singleLine` mode turns `|n` into a
  space so list rows stay one line.
- `plainGameText` strips the codes where only plain text fits: search matching, `textValue`
  for the palette, screen reader labels, the bundle file name, hotkey clash names.
- `readableColor` lightens colors that would fall under 4.5:1 against `surface-2` (selected
  rows, the lightest panel text sits on), mixing in white 10% at a time so the hue stays.
  Maps do color names nearly black.
- A test caught `BACKGROUND_LUMINANCE` being computed at module load before the constants it
  uses were initialized (a temporal dead zone), which would have crashed the app on start.
- Editing is unchanged: text fields keep the raw codes, with the color menu and preview.

## 2026-10-23 · The pre-push check ran only for app changes

- The changelog and devlog check had `root: object-editor/app/` in `lefthook.yml`. Lefthook
  then runs a job only when a pushed file is under that root, so the check skipped every push
  that changed nothing in the app: Rust-only branches, docs, and the 0.1.0 release commit,
  where lefthook printed "no matching push files". It now runs from the repository root on
  every push.

## 2026-10-23 · Version 0.1.0

- The first tagged version. `Unreleased` in the changelog became `0.1.0 - 2026-10-23`, and the
  `v0.1.0` tag runs the release workflow for the first time: the Windows installer and the tag
  workflow are tested here, on a draft release.

## 2026-10-23 · Fold the game's objects in the lists (AGE-21)

- Each object list splits into the map's own objects (custom or changed: `group === "map"` for
  units, items and upgrades, `inMap` for abilities) and a `ListFold` header over the game's.
  Folding is per list (`wc3-object-editor.fold.<noun>` in local storage, through `useFolded`),
  so Units can stay folded while Abilities stay open.
- Searching ignores the fold: matches from the game still show, in one flat list.
- The filter chips keep working inside each part. A map with no own objects says so instead of
  showing an empty list.

## 2026-10-23 · Release builds, Tinkerbench, changelog and devlog (AGE-16)

- **Release builds.** `.github/workflows/release.yml` runs on `v*` tags: it checks that the tag
  matches `version` in `object-editor/Cargo.toml`, creates a draft release with
  `.github/release-notes.md`, then builds NSIS (Windows) and `.deb` + AppImage (Ubuntu 22.04,
  for an older glibc) with `tauri-action`, uploading to that draft by release ID so the two
  jobs never race to create it. Publishing stays manual, after trying the installers.
- **One version.** `tauri.conf.json` no longer has its own version; Tauri reads the Cargo
  workspace's. The app shows it (`getVersion()`) next to its name.
- **Name and icon.** The app is Tinkerbench (`mainBinaryName: tinkerbench`, so the Linux
  binary, desktop entry and icons are named after it rather than the crate). The icon source
  is `src-tauri/icons/app-icon.png`; `bun run tauri icon` makes every size. Blizzard's icons
  stay out: their license is personal and non-transferable, so installers cannot carry them.
- **Pinned Rust.** `object-editor/rust-toolchain.toml` pins 1.99.1 for the pre-commit hook, CI
  and releases; CI installs it with `rustup toolchain install`. 1.99's Clippy found nothing new.
- **Smaller builds.** The project folder had reached 38 GB: 22 GB of `target/` (debug info for
  about 450 dependencies, separate builds for test and Clippy, and both Rust 1.97 and 1.99's
  files, since Cargo never deletes stale ones) and 17 GB in three old, merged worktrees.
  Development builds now keep only line tables for our code and no debug info for
  dependencies; a fresh build, test and Clippy run takes 1.7 GB.
- **Windows signing.** Not yet: the installer is unsigned and the docs explain the SmartScreen
  warning.
- **Changelog and devlog.** This file and `CHANGELOG.md`, which the app bundles (`?raw` import)
  and shows under "What's new". A lefthook pre-push check refuses a branch that changes
  neither, compared with `main`.
- **Tested.** Built the `.deb` and AppImage locally and ran the AppImage: game data, icons and
  the model viewer work with the release build's `tauri://` origin and CSP. The Windows
  installer and the tag workflow are first exercised by the first real tag (it only makes a
  draft).

## 2026-10-22 · Lean CI: checks before each commit, caches (#19)

- Formatting, linting, the builds and the tests run in a lefthook pre-commit hook
  (`lefthook.yml`), in order: the frontend builds before the Rust tests because the app crate
  embeds it. After a small change the hook takes about 5 seconds.
- CI only builds and tests. Push triggers only on `main` (pull requests ran twice before), a
  newer push to a PR cancels its older run, and `Swatinem/rust-cache` caches `target/`
  (Windows: about 10 minutes to under 3).
- The Linux CI job no longer tests the app crate: apt took 12 to 17 minutes to install WebKit
  and friends, and caching the `.deb` files left out the `-dev` packages they depend on
  (`glib-2.0` missing). Windows tests every crate; the pre-commit Clippy builds the app on
  Linux. Total CI went from up to 22 minutes to about 2.5.

## 2026-10-22 · Items and upgrades (#18, AGE-14)

- Items use the unit metadata's `useItem` fields over `ItemData.slk` and the item profiles
  (`UnitDatabase::load_items`); `MapItems` mirrors `MapUnits` for `war3map.w3t` plus skin.
- Upgrades get their own `UpgradeDatabase`: `UpgradeData.slk` holds one base and one step per
  level (`goldbase`/`goldmod`, `base1`/`mod1`); names, tooltips, hotkeys and icons are
  per-level profile lists, but requirements use a key per level (`Requires`, `Requires1`).
  `MapUpgrades` stores plain fields at level 0 and per-level ones at 1..N, data column 0, as
  Legion TD does. The World Editor fills the `%s` in "Effect 1 - %s" itself.
- The app reuses pages: items render on the unit page (`View<M>` over a `PlainChanges` trait),
  upgrades on the ability page for its level tables.
- Sharing: items and upgrades became `ObjectType`s. Requirements (`techList`) can name units or
  upgrades, so a field now maps to a list of possible targets. `apply` takes a
  `MapObjectsMut` (it had reached 8 arguments).
- Found on the way: `UnitDatabase::contains` always checked the unit table, so item lookups
  failed; and long pages could scroll the window past the layout (fixed with
  `relative overflow-hidden` on the editor root).

## 2026-10-22 · Arcane redesign (#17, AGE-23)

- After two rounds of mockups, the "Arcane" direction: Warcraft III Reforged's menus over a
  workbench layout. Tokens, type (Cormorant SC, Nunito Sans, bundled with `@fontsource`),
  frames, the inspector and the Ctrl K palette are written up in `docs/design.md`.
- A WCAG 2.2 AA review: control edges at 3.36:1 or more, 24 px targets, `aria-current` tabs,
  a main landmark, announced status, field rows as named fieldsets, reduced motion.

## 2026-10-22 · Imported art in the open map (#16)

- `wc3icon:`, `wc3tex:` and `wc3model:` look in the open map first (queued imports, then the
  archive), then the game. Maps name `.blp`/`.mdl` while archives hold `.dds`/`.mdx`, so
  lookups try both.
- A BLP1 decoder for imported icons (JPEG and paletted). Clippy 1.99 asks for `as_chunks`
  instead of `chunks_exact` with a constant size.

## 2026-10-22 · Bundle files (#15)

- `.w3obj` files carry the objects plus the files imported into the source map that they use.
  Protected maps cannot list their files, so `imports` tries names from art fields and follows
  models to their textures. New files get `war3map.imp` entries (flag 29).

## 2026-10-22 · Ability editor (#14)

- Per-level fields (`<field><level>`, `Data<letter><level>`) become tables with keyboard
  editing: arrows move, Enter edits, Ctrl+Enter fills later levels, Delete resets.
- `.w3a` stores fields without levels at level 0, column 0; per-level ones at 1..N with the
  data column (`DataA` = 1).

## 2026-10-22 · Sharing between maps (#13)

- Share codes are `W3OBJ1:` plus base64 of the deflated object files and strings. Imports
  rename taken IDs and every reference to them, found through the metadata's value types.

## 2026-10-22 · Faster unit list (#12)

- Refreshing the unit list was quadratic in the number of map units; it is linear now.

## 2026-10-21 to 2026-10-22 · The desktop editor (#5 to #11)

- Tauri 2 + React 19 app for units (#5), then the field guide (#6), ability cards (#7),
  tooltip preview and color menu (#8), hotkey clash warnings (#9), grouped fields (#10), and
  the 3D model viewer (#11; HD models needed their own texture handling to stop rendering as
  black silhouettes).

## 2026-10-21 · Foundations (#1 to #4)

- Game data from the CASC storage (vendored CascLib) with every unit field and editor-style
  values (#1); map unit data read and written byte for byte (#2, verified on 135 maps and
  Legion TD); the Lua harness in Rust (#3); Windows builds of the C shims (#4: no `strncpy`
  under MSVC, CascLib's header auto-linking a prebuilt library, StormLib needing WinINet and
  user32).
