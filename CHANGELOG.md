# Changelog

What changed in Tinkerbench, for the people who use it. Performance and internal changes get
one line here; the details are in the [devlog](docs/devlog.md). Tinkerbench shows this file
under "What's new".

## Unreleased

### Changed

- Tinkerbench is now licensed under the GNU General Public License, version 2 or later.

### Fixed

- Website screenshots now show “Image subject to change” above their descriptive captions.
- The first-map tutorial and the visual tour now say that Tinkerbench Setup connects the game,
  so the first launch skips the Warcraft III folder step.

## 0.3.0 - 2026-11-04

### Added

- Tinkerbench Setup, for Windows and Linux, installs Tinkerbench for your user, with no
  administrator rights. It checks what Tinkerbench needs from the system and installs it if it
  is missing (WebView2 on Windows, WebKitGTK on Linux), then finds your Warcraft III install,
  your Maps folder and where test maps keep their Lua files. Tinkerbench then opens straight on
  your game. Run the setup again to update or remove Tinkerbench; on Windows it is also listed
  under installed apps.
- When Tinkerbench finds several Warcraft III installations, the setup screen lists them
  with where each came from and its game version, so you can choose yours. It now also looks
  in every Steam library, Heroic, Bottles and Lutris on Linux, and on every drive on Windows.
- The in-game verification page records more checks: custom loading-screen models, a
  script rewrite on a JASS map, a colored ability tooltip, and an imported Interface design
  with keyboard hotkeys. The Map guide says which loading-screen models work.
- Buffs tab: browse, search, edit and create custom buffs and effects, with a chip per race.
- Destructables tab: trees, gates, crates and rocks, with a chip per category and their model
  in the inspector.
- Doodads tab: props, plants, rocks and lights, with a chip per category, their model in the
  inspector, and the tint of every model variation side by side. Every object type of the
  World Editor's Object Editor can now be edited.
- Spell wizard: "New spell" in the Abilities tab asks what the spell should do (damage, heal,
  stun, summon, aura and 9 more), suggests game abilities to start from and why, then sets its
  name, levels, targets, range, cooldown, mana, its own values and tooltips in one table, with
  the tooltip previewed live. Cancelling leaves the map as it was.
- Spell wizard buff step: give the spell its own buff (a copy of the original's, named after
  your spell, with its own tooltip), keep the original's, or pick any other. The spell's buff is
  set at every level, and cancelling removes the new buff too.
- Spell wizard button step: pick the spell's icon from the game's and the map's icons (with
  search), its hotkey and where its button sits on the command card, and for hero spells on
  the learn menu too. Warns when the button would hide Move, Stop, Hold Position, Attack,
  Patrol or Learn, or share their hotkey. Changing the hotkey updates the highlighted key in
  older tooltips.
- Give a spell to units, heroes and buildings: the spell wizard's last step, and "Give to
  units" on any ability's page. Picking a unit shows its command card (and a hero's learn menu)
  with the spell on its spot, and warns when its hotkey or spot is already used or the card is
  full. Hero spells are learned by heroes; other units get them as normal abilities. Taking a
  spell away works from the same place, and cancelling the wizard takes it from every unit it
  was given to.
- Triggered spells in the spell wizard, for effects no game ability has: Attribute Strike
  (damage from strength, agility or intelligence), Knockback Wave (damages and pushes enemies
  away) and Arc Bolt (bounces between enemies). Set their values per level, and Tinkerbench
  writes the Lua code into a folder of your choice, then saves the map and builds a "[DEV]"
  test copy with it, ready to play. The map must use Lua.
- An Interface tab for in-game UI, starting with dialogs: a message in the color you pick and
  buttons with hotkeys (their letter shown in gold, as the game does), each closing the
  dialog, showing another one or running a Lua function you name. The preview shows the
  dialog at the game's proportions. "Save and build test copy" writes the Lua and builds a
  "[DEV]" copy of the map to play. Designs are saved next to the map in "<map> UI.json".
- Multiboards in the Interface tab: a titled board in the top right corner, like a
  scoreboard. Set its columns and widths, fixed rows or a row for each player (with the
  player's name in their color), and each cell's text, color and icon, one cell or a whole row
  or column at a time; the preview shows it as the game does. From Lua, one line updates a
  cell: `UI.Scoreboard.SetValue(player, "Kills", 5)`.
- Leaderboards and timers in the Interface tab. A leaderboard ranks a row for each player
  (their name in their color) and up to four rows of your own, such as teams, highest or
  lowest first; from Lua, `UI.Kills.AddValue(player, 1)`. A timer counts down with a title
  ("Next wave 1:30"), can start with the game, start over each time and run a Lua function
  you name when the time runs out. Both have a live preview.
- Reuse Interface designs in other maps. "Export…" saves chosen designs as a design file
  (.w3ui) with the dialogs their buttons show and the icons imported into the map that they
  use; "Import…" in another map's Interface tab adds them, numbering any name the map already
  has ("Scoreboard 2") and adding the icons it lacks. The windows also list the Lua functions
  the designs call. "Duplicate" copies a design as a starting point for a similar one.
- A Placed units tab for the units placed on the map. Give a placed hero the items it
  starts with, picking them by icon and name from the game's items and the map's own, with
  search and class and level filters. Choose what each unit drops when it dies: nothing, one
  of the map's item tables, or its own item sets (one item per set, by chance, or a random
  item of a class and level). Item tables can be cascades: tiers roll from the rarest
  (Legendary 2%, then Epic 5%, ...) until one hits, then pick one of its items by weight.
  Tinkerbench shows each item's exact chance and the whole percent the World Editor gets,
  warns when an item rounds to 0%, and keeps the tiers in "<map> Loot.json" beside the map.
- A map view in the Placed units tab, drawn from the map's terrain. Zoom and pan, pick a unit
  on the map or in the list, drag it to move it and turn it by its handle, or use the arrow
  keys and [ ] on the picked unit. Add units of any type for any player by clicking the map,
  remove them, and change their owner, position, facing, hit points, mana, and a hero's level
  and attributes. "Open unit type" goes to the unit's page.
- A Map tab for the map's own settings. Edit its name, author, description and suggested
  players, the loading screen and prologue texts, map options, terrain fog, weather and water
  tint, the players (name, controller, race, fixed start) and the forces (name, members,
  alliance and shared vision or control).
- Gameplay constants with a plain explanation for every one of them: what it does in game, its
  unit and an example. Search them, show only the ones the map changed, see the game's value
  next to each, and reset any. The damage table is a grid of attack types against defense
  types. Like the World Editor, only the constants you change are stored in the map.
- Changes in the Placed units and Map tabs now take effect in game. On save, Tinkerbench
  rewrites the parts of the map script the World Editor writes from them: unit creation,
  drops and item tables, players, forces, fog, weather, water tint and sound. It does this
  only when it can write those parts exactly as the World Editor did (nearly every map);
  otherwise the tab says so, and saving the map in the World Editor applies the changes.
- An Assets tab for the files imported into a map. Import models and textures, single files
  or a whole folder (text `.mdl` models are converted to `.mdx`), choose the path each one
  gets in the map, and see which textures a model needs and whether the map or the game has
  them. Preview models with their textures and images, see which objects use each file,
  rename a file (the fields that use it follow), replace or remove it, and export it, with
  PNG for images. "Create a unit from this model" makes a custom unit from a base unit you
  pick, with the model and an imported icon. "Export a game file…" copies a model, texture or
  icon out of the game for your own use. Imports go into the map when you save it.
- Export objects as CSV or JSON for spreadsheets, wiki pages and other tools. "Export…" above
  any object list exports what the list shows, the open object, all objects of the type, or
  every type; "Export objects…" in the command palette too. Every field comes with its value
  as stored and as the editor shows it (choices by name, tooltip numbers filled in), with the
  map's changes and, if you like, only what the map adds or changes, or only its custom
  objects, without color codes.
  CSV gives a column per field and per level; the JSON format is documented for tool makers.
  `wc3map` exports the same way from the command line.
- Text previews point out mistakes: a color code without its 8 digits, `|r` with no color to
  end, a color left open, an unknown `|` code, an unclosed or malformed placeholder, and
  placeholders Tinkerbench cannot fill. "Formatting help" in the preview lists the codes.
- Damage and bounty dice show the minimum, maximum and average, and how they add up: the base,
  the dice roll and, for heroes, the primary attribute's bonus (using the map's damage per
  attribute point). The result follows the numbers as you type, and a value a field does not
  take (a fraction, or out of its range) is explained instead of saved.
- "Prompt for an AI assistant…" (on the setup screen and in the command palette) gives a
  prompt to paste into Claude, ChatGPT or another assistant. It explains Tinkerbench, what
  stays in the World Editor and the Lua harness, includes the map you describe and a summary
  of the open map, and asks for a build plan, object specs with field IDs, ready Lua modules
  and how to test them.
- Tinkerbench opens straight into the editor: it loads the game data you used last time and
  reopens your last map, with no clicks. Press Escape or "Stop and choose" while it loads to
  pick another map. If the folder or the map cannot be opened, the setup screen says why.
  Turn it off with "At start, load this game data and reopen the last map", and use
  "Change game folder…" in the command palette to switch folders.
- A documentation website, the [Tinkerbench field guide](https://tinkerbench-docs.vercel.app):
  a screenshot tour and a guide for every tab, illustrated hero, item and spell walkthroughs
  with downloadable starters, a "Which field do I change?" guide, a searchable reference for
  object fields, JASS and Lua API declarations organized by topic, 43 code recipes, a
  glossary, comparisons of commonly confused concepts and a troubleshooting hub.

### Changed

- Fields with many options get more room and a tidier layout: options line up in columns in
  a framed area under the field, and target lists (Targets Allowed, Targeted As) are grouped
  by owner, kind and state. The game value column shows how many options are on. The
  per-level target editor of abilities is wider and stays inside the window, scrolling when
  there is not enough room.
- Coming back to a tab is instant: it keeps its list, icons, search, filters and the object
  you had open instead of loading them again. A tab reloads only when something changed in
  another tab since you last looked at it, so it never shows outdated data. Switching between
  the Units and Items tabs shows the new tab two to four times sooner, and 3D previews stop
  drawing while their tab is hidden.
- Tinkerbench now keeps a log of what went wrong, to attach to a bug report:
  `tinkerbench.log` in `~/.local/share/io.github.gabaudette.wc3toolkit/logs` on Linux and
  `%LOCALAPPDATA%\io.github.gabaudette.wc3toolkit\logs` on Windows. Opening a file or
  folder now says so when the dialog itself fails, instead of doing nothing.
- Drop-down menus are drawn in the app's own style, closed and open, instead of the system's
  grey buttons and menus.
- New checkboxes everywhere: a gold box with a check mark when on.
- Keyboard focus is a light ring instead of a teal one, and a focused text box gets a light
  edge, so focus no longer clashes with the gold.
- Dialogs open near the top of the window and stay there while their content grows or
  shrinks, so buttons no longer move under the pointer.
- Destructables and doodads show the World Editor's icon for their category (Props, Bridges,
  Water, ...) in lists, on their page and in search, since the game gives them no icon.
- Names with Warcraft color codes show in color everywhere: lists, page titles, search,
  ability cards, sharing and import, and the base value column. Colors too dark to read on the
  editor's panels are lightened a little. Text you edit still shows the codes.
- Performance improvements.
- Internal improvements.

### Fixed

- When a folder is not a Warcraft III installation, the message says both files it looks for
  and that a Maps folder is not the game.
- Tinkerbench starts on Windows computers that do not have the Visual C++ runtime
  installed.
- The Lua folder you pick in the Interface tab stays after you edit the map in another tab;
  it used to go back to the default, so a build could write its modules elsewhere. The
  spell wizard uses the same folder, and opening another map starts from that map's own.
- Unit Classification takes several choices, as in the game and the World Editor (a building
  can be both a Town Hall and Mechanical): its options are chips instead of a single-choice
  drop-down. So are the Placement Requires and Placement Prevented By pathing fields of
  buildings.
- Lists of units, items and abilities hold no more than the game allows, as in the World
  Editor: 12 items or units sold, made or upgraded to, 4 item abilities, 5 hero abilities.
- Ability, upgrade and doodad pages no longer keep redrawing in the background while open,
  which used processor time for nothing.
- Tooltip previews now update as soon as a value they show changes, not only when the text
  itself is edited.
- Long game texts, like tooltips, showed quote marks around their value when edited.
- Textures and icons imported into a map as `.tga` files now show.
- The model preview now shows the newer campaign units, such as Roderick, the Scarlet
  Crusade and the Forsaken, and a few cinematic doodads. Models the game does not have say
  so instead of showing a file error.
- The model preview no longer fails with "Missing shader uniform location: uRoughness" on
  some graphics drivers.
- On Linux, Tinkerbench could close without a word while browsing objects with the 3D
  preview. It no longer does.
- When the graphics driver resets the model preview, it now says so and comes back on its own,
  instead of staying blank.
- Building from source no longer fails after the project folder moves, or when several copies
  of the project share one build folder.

## 0.1.0 - 2026-10-23

### Added

- Fold the game's own units, abilities, items or upgrades away in each list, so only what
  your map adds or changes shows. Tinkerbench remembers your choice; search still finds
  everything.
- Installers: a Windows installer and Linux packages (.deb and AppImage), published on GitHub.
- A new name, Tinkerbench, and a gnome for an icon. The version shows next to the name;
  click it to see what's new.
- Items tab: browse, search, edit and create custom items, with a chip per item class.
- Upgrades tab: browse, search, edit and create custom upgrades, with names, tooltips, icons
  and requirements for every level side by side.
- Sharing a unit brings along the custom items it sells and the custom upgrades it researches.
- A new look inspired by Warcraft III's own menus, with an inspector that explains the field
  you are editing and a Ctrl K search for everything.
- Abilities tab: edit abilities, with every level's values in one table.
- Share codes and bundle files (`.w3obj`) to copy custom units, with the abilities, buffs,
  models and icons they use, from one map to another. Import straight from another map too.
- Models, textures and icons imported into a map show in the editor.
- A 3D model preview, a live tooltip preview with Warcraft's color codes, and warnings when
  two buttons share a hotkey.
- Related fields edited together: damage dice, bounty, tint and button position.
- Ability cards that show an ability's icon, tooltip and values wherever a unit uses it.
- A field guide: every unit field has a plain description, important fields come first and
  rarely used ones fold away.
- The Units tab: every unit, hero and building with all its Object Editor fields, read from
  your own Warcraft III install, with changes saved into your map.

### Fixed

- Long pages could scroll the whole window past the editor.
- Performance improvements.
- Internal improvements.
