Tinkerbench is an open-source workbench for Warcraft III Reforged maps. It replaces the World
Editor's Object Editor for every object type (units, abilities, items, upgrades, buffs,
destructables and doodads) and also edits placed units and their loot, the map's settings and
gameplay constants, imported models and textures, and in-game interface designs. A spell
wizard, share codes and bundle files, CSV/JSON export and a Lua test harness come with it. It
reads game data from your own Warcraft III install; no Blizzard files are included.

Guides, tutorials and the field reference: https://tinkerbench-docs.vercel.app

## Install

- **Windows:** download the `-setup.exe` file and run it. It installs for your user only, no
  administrator rights needed. Windows may show "Windows protected your PC" because the
  installer is not signed: click **More info**, then **Run anyway**.
- **Linux (Debian, Ubuntu, Mint):** download the `.deb` file and install it with
  `sudo apt install ./<file>.deb`.
- **Linux (other):** download the `.AppImage` file, make it executable
  (`chmod +x <file>.AppImage`) and run it.

On first start, point Tinkerbench at your Warcraft III folder; after that it reopens your last
map. Work on a copy of your map, and close it in the World Editor before saving it here.

## What's new

Everything that changed is in the app under "What's new" (click the version next to the name)
and in [CHANGELOG.md](https://github.com/gabaudette/wc3-toolkit/blob/main/CHANGELOG.md).

## Tested with

<!-- Fill in from the draft's install tests before publishing: Windows version, Linux
distribution, package type, game build and SD/HD mode actually exercised. -->

## Known limitations

- Placed units and Map tab changes reach the game only when Tinkerbench can rewrite the map
  script's World Editor parts exactly; otherwise the tab says so, and saving the map in the
  World Editor applies them. This was tested on Lua maps, not yet on JASS maps.
- Interface designs, triggered spells and the test harness write Lua, so they need a Lua map.
- Nothing has been tested with several players in one game yet.
- A custom loading screen model chosen in the Map tab did not replace the game's loading art in
  testing.

Report problems at https://github.com/gabaudette/wc3-toolkit/issues.
