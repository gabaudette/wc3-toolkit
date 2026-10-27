# Devlog

The technical story of Tinkerbench, newest first: what changed, why, and what we learned.
Users read the [changelog](../CHANGELOG.md) instead, where performance and internal work is a
single line. Every pull request adds an entry here and to the changelog.

## 2026-10-27 · Downloadable hero and item starters (AGE-27)

- Added two project-authored `.w3obj` starters with equivalent text share codes. The hero
  pack contains Sunwell Warden (`H000` based on `Hpal`), matching the illustrated guide's
  attributes and growth. The item pack contains Wayfarer Supplies (`n000`/`ngme`) whose
  Items Sold (`usei`) references Wayfarer Draught (`I000`/`phea`), so sharing discovers the
  potion as a dependency. No custom spells, game map/scripts or standalone assets ship.
- The first shop test exposed the base potion's initial stock delay. Added `isst = 0` to
  the starter's item edits so purchases are available immediately; the healing cooldown
  remains inherited. The starter page documents this extra field alongside the guide's
  name, cost, charges and tooltips.
- A Rust example defines the edits and generates both formats through the existing
  create/set/export functions using local game metadata. Shipped-artifact tests validate
  all three objects, documented fields, empty asset lists and file/code equivalence.
  Refresh instructions explain generation to a review directory and collision/game checks.
- Added a starter page with downloads, complete object/field inventory, actual import UI
  labels, placement and gameplay steps, collision guidance and links from both illustrated
  guides and sharing. Two genuine import previews have alt text, captions, full-size links
  and the shared centered muted change notice.
- Imported the final hero and item bundles through the app's Bundle file flow into a
  separate map with occupied IDs; imported the item code a second time through Share code.
  Saved/read back `H000` → `H002`, `I000` → `I001`, then `n000` → `n001` and `I000` → `I002`.
  Both merchants point to their assigned potion and retain `isst = 0`; eight existing
  gameplay/skin object entries and their resolved strings are unchanged.
- A temporary Lua harness on a separate copy checks the actual imported IDs in Warcraft
  III **3.0.1.24371**, Linux/GE-Proton11-7. All twelve runtime checks pass: level-1/3
  attributes, Holy Light ranks 1/2, both merchants' 125-gold purchases, pickup and
  dropping/reacquisition, three healing uses through final consumption and merchant sale.
  It leaves the real healing cooldown in place and waits 65 game seconds between uses.
  [Gameplay evidence](verification/age27-gameplay.png) records the results and hero HUD.
- Validation: pending final website build and commit hooks.

## 2026-10-27 · Illustrated hero and item tutorials (PR #55, AGE-25)

- Expanded the existing hero/item URLs into short walkthroughs with seven real Tinkerbench
  screenshots, full-size links, alt text, descriptive captions and the shared centered,
  muted change notice. Kept the original section anchors and the separate spell guide.
- Sunwell Warden copies Paladin (`Hpal`, assigned `H001` here), keeps the four game hero
  spells and normal inventory, and sets starting STR/AGI/INT to 24/15/20 with gains 2/1/3.
  Wayfarer Draught copies `phea` as `I000`, keeps Item Healing (`AIh1`), sets three charges
  and a 125-gold cost, and has its own name/shop/extended tooltip. Explained base life versus
  hero bonuses, normal/hero ability lists, tooltip versus effect, and to/by-merchant flags.
- Followed both workflows in the current app on a disposable map. Saved fields/text were
  read back; all seven object/string files round-trip identically. Recorded refresh
  conventions in the website README; no app behavior or starter downloads changed.
- Played a separate copy in Warcraft III **3.0.1.24371** on Linux/GE-Proton11-7. A temporary
  Lua harness spawned the saved objects, disabled the solo fixture's melee victory timer,
  and checked ten runtime conditions: level-1 attributes, Holy Light rank 1, level-3
  attributes (28/17/26), Holy Light rank 2, three-charge pickup, dropping/reacquisition,
  first/second healing uses (two/one charges), final-charge consumption and merchant sale.
  All pass; [gameplay evidence](verification/age25-gameplay.png) shows the result and HUD.
  The initial rapid-use check hit the item's cooldown; the successful rerun keeps its real
  cooldown and waits 65 game seconds between uses. The guide calls out that wait and the
  need for missing health. No game map, game script or standalone art was committed.
- Website validation: 119 tests pass; Astro checks 35 files with no diagnostics; the
  6,610-page build and full-site local link/asset check pass. A focused built-page audit
  checked seven image dimensions, alt texts, full-size links and notices, seven retained
  section anchors and 34 internal fragments. Browser checks pass at 1440 × 1000 and
  320 × 740 in both themes, without horizontal overflow; images load, notices stay centered
  and muted, and keyboard activation opens the full-size PNGs. Guides-filtered search
  ranks Sunwell Warden/hero attribute growth and Wayfarer Draught/item charges first on
  their respective pages. No browser errors.

## 2026-10-27 · Spell wizard walkthrough (PR #54, AGE-56)

- Replaced the manual-copy custom-spell introduction with the finished New spell workflow:
  Storm Bolt → Sunwell Bolt, three levels, its own buff, distinct cast/learn buttons and
  explicit Give the spell followed by Create spell and Save. Nine real, focused dialog
  captures have alt text, full-size links and the shared centered, muted change notice.
- Added a separate Attribute Strike walkthrough for Lua maps: effect values, button/unit
  assignment, per-map module folder, Create spell and Save and build test copy. Kept the
  source-map/module/dev-copy distinction and Lua-only requirement explicit. Linked the
  workflow from the object-editing and harness guides and documented capture maintenance.
- AGE-19 is done, but AGE-25/27 also waited for a current tutorial. Tracked that prerequisite
  as AGE-56 and replaced their completed-tool blocker with this tutorial, preserving their
  hero/item scope. The original spell guide URL and main section anchors remain stable.
- Followed both paths in the current app on separate disposable copies. Read back Sunwell
  Bolt's cooldown/mana/damage, own tooltip placeholders, three buff links, button settings
  and the hero's fifth ability. The basic map's seven object/string files round-trip
  identically; the generated map's five do too. Attribute Strike's module uses its assigned
  code and is present verbatim only in the dev copy's Lua script, which parses as Lua 5.3.
  The source script remains unchanged. **In-game casting was not exercised**; the guide
  says so and gives a rank-by-rank play-test checklist.
- Website validation: all 119 current tests pass; Biome and Astro report no diagnostics.
  Build and local link/asset checks pass across 6,610 pages. A separate audit checks all
  nine image dimensions/alt texts/full-size links/notices, preserves four original section
  anchors and validates 84 internal fragments across the three edited guides. Browser
  checks cover desktop and 320px in both themes, loaded images, readable tables, centered
  muted notices, keyboard full-size access and shared instructions with JASS selected.
  The guide ranks first in Guides searches for spell wizard, Sunwell Bolt and Attribute
  Strike. No browser errors were reported.

## 2026-10-27 · UI tool, part 1: the Interface tab and dialogs (AGE-52)

- First part of AGE-37 (split into AGE-52 to AGE-55: dialogs, multiboards, leaderboards and
  timer windows, reuse across maps). An **Interface** tab next to Doodads: designs on the
  left, the editor and a preview in the middle, the Lua along the bottom.
- Designs live in `<map> UI.json` beside the map (`interface.rs`: a versioned `UiFile` with a
  `kind`-tagged `Design` enum), loaded with the map and written on Save; `OpenMap::unsaved`
  counts them. A file from a newer format version is refused rather than overwritten.
- A dialog is a name, a message (plain text plus a color from the player colors, the game's
  gold by default), buttons with text, hotkey, "gold hotkey letter" and an action (close,
  show another dialog, run a named global Lua function if the map defines it), and "show to
  every player at start". The color codes are added by Tinkerbench (`title_text`,
  `button_text`, mirrored in `interface.ts` for the preview, with the same tests), at the
  user's request: typing `|cff...|r` was not something map makers should have to read.
- Each dialog becomes `ui_<name>.lua` in the map's Lua folder, made from code and checked as
  Lua 5.3: `UI.<Name>.Show(player)`, `.Hide(player)`, `.ShowAll()`, a click trigger that runs
  the button's action, and the show at start after 1 s (dialogs shown during map init never
  appear). Writing removes the modules this tab wrote for deleted designs, recognized by their
  first line, and leaves every other file alone. Lua names come from the design names
  (`ChooseDifficulty`), numbered when two collide; references between dialogs use stable IDs.
- "Save and build test copy" writes the modules, saves the map and its designs, and runs the
  harness's `build_dev_map`, as the spell wizard's last step does.
- The tab saves designs to the map on every change but re-reads the window's map status only
  once typing pauses (300 ms): re-reading it per keystroke re-rendered the whole window and
  dropped keys. Updates run one after another, as in the other panes.
- Every native `<select>` became `Select` (react-aria `Select`, `Popover`, `ListBox`), at the
  user's request: WebKitGTK draws the closed select as a grey theme button and the open list
  as a GTK menu, which no CSS reaches. Option keys get a `v:` prefix, since react-aria treats
  an empty key as none and fields can store an empty value. In a level table a choice cell
  opens a list in the panel the flags already use, under the cell: a pick applies; Escape,
  Tab or a click away leave it. (A pop-up `Select` there closed itself: the Enter that started
  the edit reached it.) Level table cells start editing on a click, which is why a click and
  then Enter seemed to "need two Enters" in AGE-39's notes; that was not a bug.
- Checked on a copy of the test map: "Choose difficulty" with a white message, Normal and
  Hard (gold N and H, Hard running `OnHardMode`), saved to `button-test UI.json`, the module
  written and the `[DEV]` copy built and read back. **Not yet played in game.**

## 2026-10-27 · Checkboxes and multi-option fields (AGE-39)

- `Checkbox` and `CheckChip` (react-aria `Checkbox`, in `Checkbox.tsx`) replace every native
  checkbox: Changed only, Use HD art, Turn, the share dialog's units. The box has a `field` edge
  in a `well`; on, it fills gold with a dark check, so the state reads by shape too, and
  Windows high contrast keeps the mark. Focus is the `glow` ring.
- `FlagsInput` is a `CheckboxGroup` of chips that wrap, sized to their labels. A field with
  more than four options lays them out across the full row under it (the value column shows
  "2 of 31 on"), as ability lists already did: Targeted As went from about 20 rows to 4. The
  level table's option popover is 32 rem wide for the same reason.
- Checked chips are gold at the user's request; the design doc's "gold means changed" now
  excepts checked boxes. An option that differs from the base gets a gold dot (and
  "(changed)" for screen readers): fields carry `baseRaw`, the base object's stored value, to
  compare option by option.
- Read-only groups (browsing game data) use `isReadOnly`, so they stay focusable and are
  announced as read-only instead of being greyed out.
- Focus moved off teal at the user's request: a new `focus` token (`#e8ecf5`, the `ink`
  color) for the `:focus-visible` outline, focus rings and the edge of a focused input (30
  uses). Teal (`glow`) stays for the selected list item and palette result. On checkboxes the
  ring goes around the whole option, not the small box.
- Tailwind 4's `outline-none` sets `outline-style: none`, so `outline-2` alone draws nothing;
  the checkbox focus classes add `outline-solid`.

## 2026-10-27 · Lists hold at most what the game allows

- The metadata's `maxVal` on lists of IDs is how many they hold, as the World Editor enforces:
  5 hero abilities, 4 item abilities, 12 items or units sold, made or upgraded to, 2 per
  Engineering Upgrade pair. The unit page's ability cards already stopped adding at it; now
  `parse_value` refuses longer lists for every object type, so lists typed as codes and
  anything else that sets a field (Give to units, the CLI) keep to it too. On text lists
  `maxVal` is a length, so they are left alone.
- The ability cards said "A hero can have at most" for item abilities too; the message now
  depends on the list.
- The previous entry said the unit page's "Abilities - Hero" field accepted more than five; it
  did not.

## 2026-10-27 · Heroes learn at most five spells

- Found playing the Knockback Wave test map: the spell was the test hero's sixth hero ability,
  and the game silently left it out of the learn menu. The World Editor caps "Abilities -
  Hero" at five; "Give to units" (AGE-50) did not.
- `SpellFit` gained `blocked`: set when the spell would go in a hero's learnable abilities
  and the hero already has five (`MAX_HERO_SPELLS`), naming them. `give_spell` refuses then,
  and the panel shows the reason and disables "Give the spell". A hero already holding the
  spell past the fifth gets a warning that the game leaves it out.
- Real-install test: a Paladin with five hero spells cannot get Blizzard, its fifth spell
  gets no warning, and a sixth one already in the list is flagged.
- (Corrected later: the unit page's "Abilities - Hero" field already stopped at five.)

## 2026-10-27 · Spell wizard, part 5: triggered spells with generated Lua (AGE-51)

- Last part of AGE-19. `data/spell-templates.toml` lists triggered spells; each is a copy of
  Channel (`ANcl`) plus a Lua module from `data/spell-templates/` (Attribute Strike, Knockback
  Wave, Arc Bolt). `wc3-data::spells::render_template` fills in `{{SPELL_ID}}`, the name and
  each parameter (a Lua table with a value per level, or a choice's Lua expression), refusing
  unknown placeholders, half bounces and bad IDs; `fill_tooltip` writes the per-level tooltip,
  with the area of effect as `<A001,Area1>` so it follows the field. A `wc3-map` test checks
  every template as Lua 5.3, alone and inside the harness script.
- The modules are `do ... end` blocks with locals only, so several spells never clash, and
  they hook `Dev.OnStart` like `map/src`. Radii come from the spell's own Area of Effect
  (`BlzGetAbilityRealLevelField`), and levels past a list's end use its last value, so adding
  levels on the page doesn't break the code.
- `create_triggered_spell` sets Channel up as a 3-level hero spell: target type from the
  template, Visible (plus the targeting image for point spells), no follow-through or art,
  other abilities usable, and a base order no other Channel copy in the map uses, from a list
  of orders checked against the game's ability data (two spells with one order on a unit
  would cast each other). The button goes to the bottom row (Channel's own spot hides Hold
  Position) and the learn menu's second row (Channel has none).
- The wizard's triggered path: effect (name, values per level, cooldown and the like, the
  tooltip they make) → button → units → Lua code (folder, default `<map> Lua` next to the
  map; the module as it will be) → try it. "Create spell" writes `<name>_<id>.lua`; "Save and
  build test copy" saves the map and runs the harness's `build_dev_map` into `<map>_dev.w3m`.
- `DialogShell` gained `page`, which starts a new wizard step scrolled to the top, and
  `dismissable`; the wizard turns that off, since a click on the backdrop (easy when a step
  gets shorter) cancelled the spell and deleted it.
- Checked in the app on a copy of the test map: Knockback Wave given to `test_hero`, its
  Channel fields as intended, the module written, and the `[DEV]` copy built (5 object and
  string files round-trip; its string table differs only by the prefix). **Not yet played in
  game.**

## 2026-10-27 · Spell wizard, part 4: give the spell to units (AGE-50)

- `SpellUnits` lists units, heroes and buildings (those with the spell first, then the map's,
  search for the rest) and previews the picked one's command card and learn menu with the spell
  placed. It is the wizard's last step and a dialog from "Give to units" on the ability page.
- `hotkeys.rs` now builds the menus once (`menus`, with an optional extra ability) for both
  the existing clash check and the new `spell_fit`: where every button sits, plus warnings for
  a shared hotkey, a shared spot, or a menu with all 12 spots taken. Standard commands got
  their slots from `CommandFunc.txt` (`UnitDatabase::command_button`); the learn menu has its
  Cancel button (no hotkey, so the clash check is unchanged and melee units still have none).
- Hero spells go in a hero's `uhab`; everything else, hero spells on non-heroes included, in
  `uabi` (with a note that it stays at level 1). Commands `spell_units`, `spell_fit`,
  `give_spell`, `take_spell`. `MapUnits::set` drops a change equal to the base, so taking a
  spell from a game unit leaves it unchanged.
- `discard_spell` takes the draft from every unit first, so cancelling the wizard leaves no
  reference to a removed ability.
- The Units tab reloads its list and open page when another tab changes units.
- Checked in the app on a copy of the test map: Fire Bolt given to `test_hero` (Paladin-based)
  from the ability page went into `uhab` and read back from the saved map; in the wizard, a
  Rejuvenation copy given to a Footman and `test_hero`, then cancelled, left both unchanged
  and the 5 object and string files round-trip identically. Seeing the hero learn it in the
  World Editor and in game is still to be checked.

## 2026-10-26 · Spell wizard, part 3: icon, hotkey and button (AGE-49)

- A button step after the buff: icon picker, hotkey and a 4×3 command card for `abpx`/`abpy`;
  heroes get a second card for the learn menu (`arpx`/`arpy`, `arhk`). Picking an icon sets
  `aart`, and `arar` too for heroes.
- Icons come from the `spell_icons` command: the map's imported images first
  (`ImportList::paths`, new, with a test: listed entries with `war3mapImported\` added when
  the flag's low bits are 5 or 8, plus files queued for import), then every icon the game's
  abilities, units, items and buffs name, deduplicated and sorted by name. Some of those names
  point at files that don't exist; `UnitIcon` gained `onMissing` and the picker drops icons
  whose image fails to load.
- Warnings cover the commands every unit has (Move, Stop, Hold Position, Attack, Patrol, plus
  Learn for heroes, Cancel on the learn menu): covering their spot or sharing their hotkey.
  Clashes with the unit's other abilities wait for AGE-50, when the spell is given to units.
- Changing the hotkey replaces the old key's highlight (`(|cffffcc00T|r)`) in `atp1`, `aret`
  and `aut1` at every level; Reforged tooltips mostly have none. For heroes the learn hotkey
  follows the cast hotkey while the two were the same.
- `DialogShell` now anchors dialogs near the top (`items-start`, `pt-[8vh]`) instead of
  centering them: the wizard's steps differ in height, and a centered dialog moved its footer
  buttons on every step and on every search keystroke.
- Checked in the app on a copy of the test map: Fire Bolt with the fire icon, hotkey F and its
  button at column 2, row 3; warnings for Patrol's spot and hotkey M; a Chain Lightning draft
  whose learn hotkey followed the cast hotkey to G, then cancelled. The saved map's 5 object
  and string files round-trip identically.

## 2026-10-26 · Spell wizard, part 2: the buff (AGE-48)

- A buff step after the values: keep the base ability's buff, give the spell its own, or use
  another. "Its own" creates a custom buff copying the base's first buff (`B000`, ...), names
  it after the spell (`fnam`, `ftip`) and puts it in the ability's `abuf` at every level,
  keeping the base's other buffs (a few abilities leave two). "Another" replaces them all. The
  tooltip title and text of the spell's own buff are edited in the step, with the preview.
- Switching away from "its own", cancelling, or going back past the values removes that buff
  again: `PlainObjects::remove` (with a test) and the `discard_plain` command. On "Create
  spell" the buff is written again for every level, in case levels were added after it was
  picked.
- The values step and the buff step name the base ability from its name at creation: the
  spell's own name has usually changed by then ("The same buff as Storm Bolt", not "as Fire
  Bolt").
- Checked in the app on a copy of the test map: Fire Bolt with its own buff `B000` ("Burning
  and stunned"), `abuf` = `B000` at levels 1-3, a cancelled second spell left no `B001`, and
  the saved map's 7 object and string files round-trip identically.
- Effects (`aeff`) stay the base ability's; picking or creating an effect is not part of this.

## 2026-10-26 · Spell wizard, part 1 (AGE-47)

- First part of the spell creation tool (AGE-19): the wizard, spell kinds, base ability and
  values. Buffs (AGE-48), button and hotkeys (AGE-49), giving the spell to units (AGE-50) and
  triggered spells (AGE-51) come next.
- `data/spell-recipes.toml` (read by `wc3-data::spells`) lists 14 kinds of spell, each with
  suggested base abilities, best first, and a sentence on what each brings. A real-install
  test checks that every suggestion is a game ability.
- The spell is an ordinary custom ability created when the base is picked, so the values step
  edits the real ability with the existing level table and tooltip preview (placeholders
  filled from its own values). Cancelling, or going back to pick another base, removes it:
  `MapAbilities::remove` drops a custom ability and the strings the editor added for it
  (`discard_spell` command). The values step shows the common values (targets, range,
  cooldown, mana, area, durations), then every per-level value in the base's spell-data
  category, then the tooltip title and text; the rest stays on the ability page.
- Copied tooltips name the base in their placeholders (`<AHtb,DataA1>`), so a copy shows the
  original's numbers, in the editor and in game. On creation the wizard points `aub1`, `auu1`
  and `arut` at the new ID, and renaming the spell also renames it in the tooltip titles
  (`atp1`, `aret`).
- `TextPreview` only re-filled placeholders when its text changed, so a tooltip kept showing
  an old damage after the damage was edited. A `PlaceholderValues` context, set to the page's
  object by the unit and ability pages and the wizard, makes previews fill again after edits.
- Found while testing: every ability, upgrade and doodad page re-rendered forever. The level
  table clamped its active cell with `min(active, rows - 1)`, then fixed a -1 to 0 and set a
  new object; with no rows (the text section, whose per-level fields are all texts) that never
  settled. `grid::inside` clamps before comparing, with tests. It was on main too.

## 2026-10-26 · Build scripts find their sources wherever they run

- The `casc` and `mpq` build scripts located the vendored CascLib and StormLib with
  `env!("CARGO_MANIFEST_DIR")`, which fixes the path when the script is compiled. Cargo reuses a
  compiled build script for another checkout of the same crate that shares the target folder
  (a git worktree with `CARGO_TARGET_DIR` pointing at the main one), so after the worktree
  was removed, the main checkout's build looked for `shim.cpp` and StormLib in the deleted
  folder and failed. Both now read `CARGO_MANIFEST_DIR` when they run, as Cargo sets it for
  every build script run; the compiled scripts no longer contain any checkout path.

## 2026-10-26 · Documentation deploys from GitHub Actions

- Every pull request got a Vercel build, comment and check, even editor-only ones. The
  "ignored build step" script compared the commit with `VERCEL_GIT_PREVIOUS_SHA`, the last
  deployment of the same branch; a new branch has none, so it always built.
- Vercel's Git deployments are now off (`git.deploymentEnabled: false` in `vercel.json`), and
  `docs.yml`, which already runs only for documentation paths, deploys what it builds: it
  uploads `.vercel/output` (the Build Output API package) as an artifact, and a `deploy` job
  runs `vercel deploy --prebuilt` (CLI 63.1.2), as a preview for pull requests and to
  production for `main`, writing the URL to the job summary. The skip script and its test
  are gone. The job needs a `VERCEL_TOKEN` secret; without one, or for pull requests from
  forks, the site is still built and checked but not deployed.

## 2026-10-26 · The app closing on its own on Linux (AGE-46)

- **Cause: real-time threads over their CPU limit.** WebKitGTK asks RealtimeKit (system D-Bus)
  to run a few IPC threads with `SCHED_RR`: two `ReceiveQueue` threads in the app process,
  `EventDispatcher` in the web process, `Storage` in the network process. RealtimeKit grants
  that only with `RLIMIT_RTTIME` set (here 160 ms soft, 200 ms hard): a real-time thread that
  runs that long without sleeping gets the process killed with SIGKILL by the kernel. Browsing
  models pushes texture after texture through the IPC threads, and a burst sometimes lasted
  long enough. Hence the silent exits (exit status 137, no panic, no Tauri exit event, no
  out-of-memory kill), and why they never happened under strace or gdb, whose stops reset the
  count. AeroFTP and Signal Desktop hit the same thing on WebKitGTK 2.52.6.
- **Fix:** `platform::keep_threads_at_normal_priority` points `DBUS_SYSTEM_BUS_ADDRESS` at a
  socket that cannot exist, first thing in `main`; WebKit's processes inherit it, the
  RealtimeKit request fails, and every thread stays `SCHED_OTHER`. Nothing else in the app uses
  the system bus (file dialogs and accessibility use the session bus; the open-map dialog was
  checked). Lowering the soft limit, as AeroFTP did, does not help here: it is already below the
  hard limit, and after the first SIGXCPU the kernel raises it by a second, so later bursts go
  straight to SIGKILL. A Flatpak or Snap build would ask through the portal instead and would
  need another way.
- **Measured** with a script clicking through the unit list every 0.1 to 0.15 s: before, the
  app was killed after 100 to 950 selections (6 of 7 runs); after, 0 real-time threads and no
  exit in 2700 selections.
- **The crash dumps were a separate, harmless thing.** `coredumpctl` had 19 `WebKitWebProcess`
  segfaults in `libnvidia-eglcore` (driver 595.99.02). Symbolized with Ubuntu's debuginfod,
  the stack is `dispatchDidCloseAndInvalidate` → `WebProcess::stopRunLoop` →
  `PlatformDisplay::clearGLContexts` → `GLContext::~GLContext` → the driver: the web process
  crashes while freeing GL contexts after the app is gone, on every close (even Alt+F4), with
  or without `WEBKIT_DISABLE_DMABUF_RENDERER` or `WEBKIT_DISABLE_COMPOSITING_MODE`. Users
  see nothing; it is a WebKitGTK/NVIDIA bug.
- **One WebGL context for the whole session.** Pages are keyed by object, so every selection
  mounted a new `ModelViewer` with a new canvas and WebGL context and dropped the last one.
  The preview now draws on one canvas created once (`previewCanvas`), moved into whichever
  preview shows; pointer and wheel handlers sit on its box. On `webglcontextlost` the preview
  asks for the context back and says so; on `webglcontextrestored` it loads the model again.
- Fixed on the way: animation picker keys were `start frame + name`, which repeats (two
  "Stand 2" at frame 0); they are the sequence index now.

## 2026-10-26 · Faster object selection (AGE-40)

Measured first, with a temporary timing log (release Rust, HD art, game data and Legion TD).
The page itself was never the problem: `unit_detail` holds the session lock for about 1 ms,
and the page paints 15 to 25 ms after the click. The 3D preview was.

- **Textures queued behind one lock.** An HD model names about 20 textures; each request read
  its file from CASC while holding the session lock, so they ran one after another and the
  last waited 60 to 220 ms. CascLib supports reads from several threads (it locks its data
  archives and file streams, and its own test extracts with worker threads), so `Storage` is
  now `Sync`, the session holds the game files in an `Arc`, and icon, texture and model
  requests take the lock only to look up the map's file or the cache, then read and decode
  without it. A new real-install test reads 400 files on 8 threads and compares the bytes.
  Page commands, which Tauri runs on the window's main thread, had also waited up to 23 ms
  behind a model read; they now never wait.
- **Full-size textures for a small preview.** HD textures are 2048 px; the preview is about
  300 px wide. `decode_texture_fitting` decodes the largest DDS mip level that fits in 512 px,
  a sixteenth of the pixels to decode, send to the window and upload.
- **Models loaded twice in development.** React's StrictMode mounts the viewer twice in a
  development build; the first load ran to the end before being thrown away. Loads now stop at
  their next step once cancelled and abort their requests. The same applies when selecting
  another object while a model loads.
- Results, model ready after selection (fetch, parse, GL setup and textures): Archmage
  372-538 ms → 146-204 ms, Paladin 245-356 → 107-129 ms, Footman 205-330 → 128-161 ms. With
  Legion TD open, the map's own art adds about 50 µs of lock time per request.
- Left as is: the model response takes about 60 ms to reach the window even when the backend
  serves it in under 1 ms; the time goes to the window, not the backend. Not needed: Rayon.
  Requests already run on their own threads, so removing the lock gave the parallelism.
- Found on the way: the window's web process sometimes crashes inside the NVIDIA driver
  (`libnvidia-eglcore`) while models load, closing the app. It happened 16 times since
  2026-10-21, before this change, and is tracked separately.

## 2026-10-23 · Contextual documentation reports (PR #40, AGE-35)

- Added a shared footer link that opens a GitHub issue draft with page title, canonical URL,
  and prompts for the problem, expected result and relevant details. Static links work
  without JavaScript; GitHub owns sign-in and submission. The site sends no report itself.
- Generated API and field frontmatter preserves exact symbol/source/kind and case-sensitive
  field code/object type. A client helper includes the selected language on paired-example
  pages and refreshes links when that preference changes, including across tabs. Canonical
  context uses the configured site and deployment base, excluding browser query strings.
- Excluded the controls from search and added documentation reporting guidance plus source
  maintenance notes. Added URL tests for encoding, prompts, language and reference context.
- Validation: all 127 tests pass; Biome and Astro report no diagnostics. Production build
  and local link/asset checks pass across 6,610 pages. A rendered-site audit checks one
  encoded report link on all 6,609 documentation pages and verifies 5,131 API plus 1,421
  case-sensitive field references against their sources. Static links retain context
  without inventing a selected language; the 404 page omits the report control.
  Browser checks cover authored/API/field pages, language changes and storage events,
  desktop/320px layouts in both themes, keyboard activation and guidance navigation.
  The signed-out GitHub flow preserves the draft title/body in its sign-in return URL.
  A temporary subpath preview verifies the alternate canonical domain and guidance anchor;
  normalized the base-path slash after that check caught a malformed guidance link.
  Report controls stay out of Pagefind content; no documentation browser errors occurred.
  No GitHub report was submitted, and no game execution was needed.

## 2026-10-23 · Visual scripting lifecycles (PR #39, AGE-36)

- Added six responsive HTML/CSS flow diagrams covering one-shot/periodic timers,
  scratch/reusable groups, temporary locations and the numeric-coordinate alternative.
  Owned handles, borrowed references, repeat paths and final release have explicit labels.
  Visible text-equivalent captions provide the same flows without decorative diagram nodes.
- Added a lifecycle template and recipe expansion helper. Five native Code disclosures
  reuse both languages verbatim from the existing recipe catalog; the global toggle and
  snippet themes apply. Folded duplicate code is excluded from search. Template validation
  runs before generation removes existing output, with tests for source reuse and errors.
- Linked the guide from navigation, the homepage, scripting/ownership/comparison guides,
  API and example indexes, plus 22 targeted API backlinks. Documented generated-directory
  ownership, recipe markers, diagram styling and anchor maintenance.
- Validation: all 124 tests pass, including paired-source reuse and invalid marker cases;
  Biome and Astro report no diagnostics. Build and local link/asset checks pass across
  6,610 pages. A separate audit verifies six figures/text equivalents, five disclosures,
  22 API backlinks and 1,090 internal fragments across 30 pages. PJass accepts each of the
  five JASS recipes against the local scripts; the existing tests parse paired Lua syntax.
  Browser checks cover both themes at desktop and 320px, arrow direction, unclipped cards,
  keyboard disclosure activation, language switching and recipe/backlink navigation.
  Three lifecycle searches find the guide in the first five Guides results; exact API/field
  lookups retain their reference first and folded duplicate code stays out of the index.
  No browser errors were reported. Game behavior was not exercised; caveats remain explicit.

## 2026-10-23 · Commonly confused concepts (PR #38, AGE-34)

- Added three authored comparisons: armor material versus defense category/base armor,
  unit removal versus death/group membership, and numeric coordinates versus allocated
  location handles. Each includes decision guidance, common mistakes, reference links and
  explicit limits on untested game behavior. Existing paired recipes supply complete examples.
- Added JASS/Lua call and cleanup notes that follow the language toggle. Linked the guide
  from the homepage, Object data sidebar, editing/scripting guides, field-task index and
  glossary. A small generator mapping adds targeted backlinks from eight API and three
  unit-field pages; the authored page stays outside generated directories.
- Reviewed names, choices and signatures against the snapshot and existing authored guides.
  Documented the backlink mapping and anchor maintenance; no app behavior changed.
- Validation: all 122 existing tests pass; Biome and Astro report no diagnostics. Build
  and local link/asset checks pass across 6,609 pages. An audit verifies all three comparison
  anchors, 11 generated backlinks, exact field labels and 607 internal fragments across
  18 pages. Browser checks cover desktop/mobile layouts in both themes, all three tables
  at 320px without overflow, paired language notes, API/field backlink round trips and
  existing recipe links. Each comparison is discoverable in Guides search; KillUnit,
  RemoveUnit and uarm retain their exact reference first. No browser errors were reported.
  Game behavior was not exercised; the page labels its source review and unverified cases.

## 2026-10-23 · Linked map-making glossary (PR #37, AGE-33)

- Added 33 alphabetically grouped definitions spanning object editing, sharing, scripting
  and map-file terminology. Entries distinguish object types from placed instances, raw
  codes from field IDs and text tokens, code ownership from player ownership, and natives
  from Blizzard helpers. Examples and paired JASS/Lua notes reuse established guidance.
- Linked term anchors from the editing, scripting, sharing and object-format introductions,
  and exposed the glossary in Start here, the homepage and generated API overview. It is
  authored outside generated subdirectories and uses the existing Guides search category.
- Kept format limits and API ownership/context caveats in their linked references. Added
  maintenance notes for term anchors and source definitions; no app behavior changed.
- Validation: all 122 existing tests pass; Biome and Astro report no diagnostics. Build
  and local link/asset checks pass across 6,608 pages. A separate audit verifies 33 distinct
  term anchors and 205 internal fragments across seven glossary/entry pages. Production
  search finds the glossary for all 33 terms in Guides, with direct definition links.
  Browser checks cover desktop/mobile layouts in both themes, alphabetical jumps, incoming
  term links, homepage navigation, and paired Lua/JASS notes without hiding term headings.
  No browser runtime errors were reported. Definitions were source-reviewed; no game
  behavior was exercised.

## 2026-10-23 · Search by content type (PR #36, AGE-32)

- Added Pagefind Content filters and result-type metadata through the MarkdownContent
  override. Classification follows collection IDs; curated API pages and their practical
  index are marked by the generator and belong to both Guides and API. No separate search
  catalog is needed.
- Weighted guide body content and reduced Pagefind’s extra title-metadata multiplier
  while retaining normal heading weights for exact symbols and raw codes. Wrapped the
  existing search modal with an explicit
  All content/Guides/API/Object fields chooser. It updates Pagefind’s native checkbox
  bindings one kind at a time, avoiding its default AND semantics across disjoint kinds.
  The existing modal retains filter state while closed and preserves its keyboard and
  language-preference behavior.
- Added route/classification tests and documented filter semantics and authoring inputs.
- Validation: 122 tests pass; Biome and Astro report no diagnostics. Root and `/docs-check`
  production builds pass, with local link/asset checks across 6,607 pages. An audit confirms
  all 6,606 indexed pages have content/type metadata and all 26 curated APIs plus their
  index belong to both Guides and API. Browser checks cover conceptual and exact queries,
  every chooser option, recovery through All content, empty results, keyboard selection,
  retained filter state, desktop/mobile layouts in both themes, subpath result URLs and
  preserved JASS preference after navigation. No browser runtime errors were reported.

## 2026-10-23 · Find object fields by goal (PR #35, AGE-31)

- Added an authored “Which field do I change?” guide with 19 tasks across movement, life,
  mana, vision, combat, costs, shops and appearance. Tasks name the correct object tab and
  link exact snapshot field labels and raw codes, with companion settings and distinctions.
- Covered Attack 1/2, acquisition versus attack range, cooldown versus animation timing,
  defense category versus hit-sound material, item versus unit fields and model scale versus
  selection/collision settings. Doodad tint guidance links the existing variation limitation.
- Linked the guide from the homepage, Object data sidebar, editing guide and generated field
  overview. The authored page remains outside the directories owned by the generator.
- Reviewed mappings against checked-in metadata and shared field explanations. The page
  explicitly distinguishes that review from in-game testing; no app behavior changed.
- Validation: all 101 existing tests pass; Biome and Astro report no diagnostics. The
  production build and link/asset checks pass across 6,607 pages. An audit checks all 19
  task anchors, 92 field links covering 88 distinct fields against snapshot labels/shared
  guides, and 148 internal fragments across the guide and its entry pages. Browser checks
  cover desktop/mobile layouts, light/dark themes, homepage-to-task-to-field navigation,
  production search, and keyboard activation. Wrapped index links use block click targets
  so taps between lines still activate the link. No game behavior was exercised.

## 2026-10-23 · Practical API explanations (PR #34, AGE-26)

- Added a separate authored API guide catalog for 26 unit, timer, group, trigger and location
  functions. Generation keeps existing symbol URLs and adds parameter meanings, return notes,
  usage, ownership, common mistakes and related links. The practical index, sidebar and topic
  pages expose the initial set. Language-specific prose follows the existing Lua/JASS toggle.
- Reused existing recipes and added five paired lifecycle/event-context examples, bringing
  the cookbook to 48 recipes. The guides distinguish source review and syntax validation
  from game execution, with specific unverified behavior called out where relevant.
- Added pre-generation validation for symbol/source resolution, exact parameter coverage,
  related targets and recipe consistency. Authored guidance survives API snapshot refreshes.
- Validation: 101 tests pass; Biome and Astro report no diagnostics; all 48 recipes pass
  Lua syntax checks and PJass against local installation scripts. The production build and
  local link/asset checks pass across 6,606 pages. A separate audit checks all 5,131 existing
  symbol URLs, all 26 guide sections and 1,023 internal fragment links. Browser checks cover
  desktop/mobile layouts, language-specific notes, examples and table-of-contents items,
  persisted language choice and guide-to-topic navigation. Game behavior was not exercised.

## 2026-10-23 · Troubleshooting hub (PR #33, AGE-24)

- Added a symptom index and recovery steps for game detection/loading, read-only browsing,
  missing objects or fields, map opening/saving, changes not appearing in game, artwork and
  import previews. Linked the hub from Start here, the homepage and the relevant guides.
- Checked advice against setup, platform detection, field commit behavior, save-file handling,
  model/icon loaders and transfer previews. The install validator requires both Data and
  .build.info even though its error mentions only Data; installation and tour text now name
  both. Explained directory write access, unsaved edit preservation, renderer versus missing
  artwork states, object-only share codes and destination-file conflicts without promising
  that a successful preview establishes game behavior.
- Added a report template and links to the contribution guide, plus maintenance notes for
  future UI/error changes. No editor behavior or spell-creation workflow changed.
- Validation: 89 tests pass; Biome and Astro report no diagnostics; build and local link/asset
  checks pass across 6,605 pages. A separate audit checks 200 internal fragment links across
  the hub and related guides. Browser checks cover sidebar navigation, symptom jumps,
  a Pagefind search for WebGL, and desktop/mobile layouts without page overflow or errors.
  Recovery advice was reviewed against the implementation; game failure scenarios were not
  exercised as part of this documentation change.

## 2026-10-23 · Documentation copyright footer (PR #32)

- Added a Starlight Footer override with a centered, muted Tinkerbench copyright notice,
  the existing license’s Gabriel Audette and wc3-toolkit contributor attribution, and a
  link to the MIT license. The year comes from UTC build time.
- Retained edit/last-updated metadata, previous/next navigation and optional Starlight
  credits by wrapping the existing footer component rather than duplicating its behavior.
- Validation: all 89 tests pass and Biome and Astro report no diagnostics; the production
  build and local link checks pass across 6,604 pages. A built-site audit confirms exactly one copyright notice
  per documentation page. Browser checks verify centered text, license links, preserved
  pagination and mobile wrapping without page overflow or browser errors.

## 2026-10-23 · Website contribution guides (PR #31)

- Added a dedicated Contributing sidebar section with an overview, local-development guide
  and documentation-contribution guide. The overview covers reproducible bug reports and
  focused pull requests; the development page explains frontend-before-Rust validation and
  real-install tests. The documentation page distinguishes authored input from generated
  output and documents the sequential site checks and production-search preview.
- Linked the section from the homepage and root CONTRIBUTING.md. The repository’s existing
  contribution rules and engineering standards remain the detailed source of truth.
- Validation: all 89 documentation tests pass, Biome and Astro report no diagnostics, and
  the build and local link/asset checks pass across 6,604 pages. Browser checks cover the
  sidebar, all three pages, the pull-request anchor and 390 px mobile layouts without page
  overflow or browser errors.

## 2026-10-23 · Categorized API reference (PR #30)

- Replaced the 5,131-declaration API landing list with 22 topic cards linking to generated
  topic pages. Each page groups native functions, Blizzard helpers, handle types and constants;
  constants are collapsible. The former source/kind listing remains as a separate alphabetical
  index, and every original symbol URL is preserved with a link back to its topic.
- Classification uses authored ordered name rules and engine-type ownership. Operation
  precedence keeps handle persistence under storage, event registration under triggers and
  ability changes under abilities even when the API also mentions a unit. Unrecognized future
  names use engine signatures before falling back to general utilities. Tests cover these
  boundaries, constant families, unique category IDs and unique engine-type ownership.
- Shortened the site title to Tinkerbench. Removed the outer blue focus outline from the
  site-theme selector, snippet-theme picker and search input; a mint underline retains a
  visible focus cue without the surrounding ring.
- Turned the homepage’s early-development message into a static Starlight caution callout.
- Screenshot change notices now use a centered, muted span beneath each descriptive caption;
  the root README also centers its notice with smaller text.
- Validation: 89 tests pass; lint and Astro checks report no diagnostics. The production build
  and link check pass across 6,601 pages. A coverage audit confirms all 5,131 original symbol
  pages and alphabetical entries remain and each symbol belongs to exactly one topic.
  Browser checks cover topic-to-symbol navigation, constant expansion, desktop dark/light
  and 390 px mobile layouts, all three focus controls, the warning callout and notice styling.

## 2026-10-23 · Application screenshots and visual tour (PR #29)

- Captured the running Tauri application using xdotool and ImageMagick’s
  window capture: both startup stages (load game data and open a map), the Paladin editor,
  command search, filtered field inspector and Holy Light level table. The captures browse
  the existing test map without editing or saving it; startup uses a separate app instance.
  Editor captures use the full 1440 × 900 window; startup captures focus on the setup panel
  at 600 × 420 and 600 × 600 so the controls stay readable inside the documentation.
- Sharing guide captures show the generated code, selected unit, object preview and bundle
  export button, followed by an import preview demonstrating H000 → H001 collision handling.
  The dialog captures focus on their controls at 680 × 540 and 680 × 420. The import was
  previewed and canceled without applying it. Steps now explicitly generate the code before
  reviewing its object list and use Preview before importing a code or bundle.
- Added a visual tour to the Start here navigation, with descriptions of the editor’s three
  columns, game loading, map opening, search, field overrides, level-table keys, saving and
  sharing. Installation and first-map guides reuse the two startup captures and explicitly
  explain the Load game data action before choosing a map. Linked full-size
  screenshots have descriptive alt text, fixed dimensions and lazy loading below the first
  image. Every screenshot in the site and README includes “Image subject to change.” beneath
  it. Relative asset links work with the site’s optional base path.
- Reused the unit-editor capture in a homepage showcase and the root README. Images live
  under `website/public/screenshots/` so both entry points use the same source, with capture
  and refresh notes in the website README.
- Validation: 54 documentation tests pass; Biome and Astro report no diagnostics; the
  production build and local link/asset checks pass for 6,578 pages. Browser checks cover
  the homepage-to-tour link, all eight images, full-size image navigation, desktop dark/light
  themes and a 390 px mobile viewport without page overflow or browser errors.

## 2026-10-23 · Documentation website (PR #27, AGE-20)

- Astro Starlight site in `website/`, with the Workshop direction selected from four design
  studies: graphite, mint, DM Sans, and the existing gnome logo/favicon. Fourteen authored
  pages cover installation, the first map, objects, sharing, the Lua harness, common tasks,
  JASS/Lua conventions and the map format notes. The public URL is
  `https://tinkerbench-docs.vercel.app`, also used for canonical links and sitemaps.
  The favicon points directly to the app icon’s 128 px PNG, using a new URL so browsers can
  replace a cached favicon. The ICO remains available for browsers that request it directly.
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
- Expressive Code emits GitHub Dark/Light, Dracula, One Dark Pro, Nord and Catppuccin Mocha
  variants, selected with a separate `data-code-theme` attribute. “Follow site theme” maps
  to the GitHub variant matching Starlight's dark/light choice.
- A native, labeled picker above each snippet synchronizes the whole page. Local storage
  restores the choice before paint and across pages; blocked storage still allows changing
  the current page. Copy buttons and code content are preserved.
- The content cache is keyed by themes, selectors and the JASS grammar: cached Markdown
  otherwise kept references to the previous highlighting stylesheet after a palette change.

- Expanded the authored example catalog from 8 to 43 complete paired recipes and generated
  a categorized cookbook from the same source. Each recipe documents setup, an entry point
  and owned-handle cleanup. All JASS recipes pass PJass with the installed common.j and
  blizzard.j; all Lua versions pass Lua 5.3 syntax checks. CI validates the catalog and Lua
  syntax with luaparse, and checks that JASS entry points are distinct and present.
- A remembered global Lua/JASS toggle in the header/mobile menu hides the other language’s
  sections, including setup prose and table-of-contents items. Both variants remain in static
  output for search and no-JavaScript reading. Language choice and code palette are separate.
  The sidebar uses “Scripting”, “Scripting basics” and “API reference” so navigation applies
  to either selected language.
  Scripting explanations also follow the selection: raw codes, callback registration, empty
  references, cleanup and language resources. Shared API tables use engine type labels;
  language-specific setup is filtered on the API index and cookbook as well.
- External web links open in a new tab across content, header and mobile navigation. A
  shared handler preserves existing relationship attributes and adds `noopener noreferrer`,
  including for links inserted after page load.

## 2026-10-23 · Category icons, and the real uRoughness fix (AGE-17)

- Destructables and doodads have no icon field. `WorldEditData.txt` gives each category an icon
  after its label (`O=WESTRING_DTYPE_PROPS,ReplaceableTextures\WorldEditUI\Doodad-Prop`):
  16 px pixel-art tiles the World Editor uses in its trees. `EnumValue` now keeps that icon,
  `UnitDatabase::choice_icon` looks it up, and list rows fall back to it (`bcat` for
  destructables, `dcat` through the units database for doodads). Pages and the command palette
  read the rows, so they get it too. `UnitIcon` scales an image smaller than its box with
  `image-rendering: pixelated` instead of blurring it.
- The last entry's AGE-17 analysis was wrong. war3-model 4.0.1's shipped `initShaderProgram`
  does throw on a missing uniform; the bench rendered then by luck. For HD models on WebGL2 it
  also builds the reflection shaders, which the preview never runs, and this machine's driver
  drops `uRoughness` from the prefilter shader, so every HD model failed in a fresh window.
  `ModelViewer` now wraps the renderer's `initShaderProgram` (private in its types): it builds
  with no required uniforms, then looks each one up and keeps a missing one as `null`, which
  WebGL ignores when a value is set. Checked in the app: Archmage, a tree wall and doodads
  render in a fresh window.

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
