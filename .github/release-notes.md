Tinkerbench is an open-source workbench for Warcraft III Reforged maps. It replaces the World
Editor's Object Editor for every object type (units, abilities, items, upgrades, buffs,
destructables and doodads) and also edits placed units and their loot, the map's settings and
gameplay constants, imported models and textures, and in-game interface designs. A spell
wizard, share codes and bundle files, CSV/JSON export and a Lua test harness come with it. It
reads game data from your own Warcraft III install; no Blizzard files are included.

Guides, tutorials and the field reference: https://tinkerbench-docs.vercel.app

## Install

This is a preview release: work on copies of your maps, and close a map in the World Editor
before saving it here.

Download **Tinkerbench Setup** for your system and run it. It installs Tinkerbench for your
user only, with no administrator rights, checks what Tinkerbench needs from the system, finds
your Warcraft III install and Maps folder, and opens Tinkerbench straight on your game. Run it
again to update or remove Tinkerbench.

- **Windows 10 or 11:** `Tinkerbench-Setup-<version>-windows-x64.exe`. Windows may show
  "Windows protected your PC" because the setup is not signed: click **More info**, then
  **Run anyway**.
- **Linux:** `Tinkerbench-Setup-<version>-linux-x64`. Allow it to run first (the file's
  Properties, Permissions, "Allow executing file as program", or `chmod +x`), then open it. If
  WebKitGTK is missing, the setup installs it through your package manager after asking for
  your password.
- **Linux packages:** the `.deb` (`sudo apt install ./<file>.deb`) and the `.AppImage`
  (`chmod +x`, then run it) install only Tinkerbench; point it at your Warcraft III folder on
  first start.

## What's new

Everything that changed is in the app under "What's new" (click the version next to the name)
and in [CHANGELOG.md](https://github.com/gabaudette/wc3-toolkit/blob/main/CHANGELOG.md).

## Tested with

<!-- Fill in from the draft's install tests before publishing: Windows version, Linux
distribution, package type, game build and SD/HD mode actually exercised. -->

## Known limitations

- Placed units and Map tab changes reach the game only when Tinkerbench can rewrite the map
  script's World Editor parts exactly; otherwise the tab says so, and saving the map in the
  World Editor applies them. That fallback was tested on Lua maps, not yet on JASS maps.
- Interface designs, triggered spells and the test harness write Lua, so they need a Lua map.
- Nothing has been tested with several players in one game yet.
- The game's own campaign loading screens cannot be chosen in the Map tab: a typed campaign
  path shows only the race crest. Imported loading-screen models work.

Report problems at https://github.com/gabaudette/wc3-toolkit/issues.
