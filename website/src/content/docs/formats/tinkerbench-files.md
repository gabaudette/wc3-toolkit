---

title: "Tinkerbench's own files"
description: "The files Tinkerbench writes besides the map, and the promise that later versions keep reading them."
---

The map itself stays in the World Editor's own formats: Tinkerbench reads and writes
`war3map.w3u`, `war3mapUnits.doo`, `war3map.w3i` and the rest byte for byte, so the World
Editor and the game read them as before. A few things the World Editor has no place for live
in files of Tinkerbench's own.

## The files

| File              | Where                                                                                                                                       | What it holds                                                                                                                    |
| ----------------- | ------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------- |
| `<map> UI.json`   | Beside the map, for example `My Map UI.json` beside `My Map.w3x`                                                                            | The Interface tab's dialogs, multiboards, leaderboards and timer windows.                                                        |
| `<map> Loot.json` | Beside the map                                                                                                                              | The tiers of each cascade item table. The map itself holds the result as World Editor item sets.                                 |
| `.w3ui`           | Wherever you save it                                                                                                                        | Interface designs exported to move to another map, with the imported icons they use.                                             |
| `.w3obj`          | Wherever you save it                                                                                                                        | A bundle file: custom units with the abilities, buffs, items and upgrades they use, and the imported models, textures and icons. |
| `W3OBJ1:…`        | Pasted as text                                                                                                                              | A share code: the same objects as a bundle file, without the imported files.                                                     |
| `setup.json`      | Tinkerbench's settings folder: `%APPDATA%\io.github.gabaudette.wc3toolkit` on Windows, `~/.config/io.github.gabaudette.wc3toolkit` on Linux | The choices Tinkerbench Setup made: the game folder, HD art, the Maps folder and where Lua modules go.                           |
| `install.json`    | Beside `setup.json`                                                                                                                         | What Tinkerbench Setup installed, so it can update or remove it.                                                                 |

Keep `<map> UI.json` and `<map> Loot.json` with the map when you copy or back it up: without
them the designs and the cascade tiers are gone, though everything already saved into the map
still works in game.

## Compatibility promise

From Tinkerbench 0.3 on:
- **Later versions read earlier files.** A `UI.json`, `Loot.json`, `.w3ui` or `.w3obj` file,
  or a share code, made by any version from 0.3 on opens or imports in every later version,
  with the same designs, tiers and objects.
- **Older versions don't damage newer files.** When a format changes in a way an older
  Tinkerbench could not keep, its version number goes up. An older Tinkerbench then refuses
  the file and says it was made by a newer Tinkerbench, rather than opening it and losing
  what it doesn't know on the next save. Tinkerbench 0.3.0 and 0.3.1 lack this check for
  `Loot.json`: don't open a loot file from a newer version with them.
- **Setups update earlier installs.** Every later Tinkerbench Setup reads the `setup.json` and
  `install.json` an earlier one wrote, so it can update or remove that install and keep your
  choices.

Tests check this on every change: each new version must load files written by Tinkerbench 0.3,
kept in the repository, and must refuse files marked with a newer version.

The promise does not cover:
- the Lua modules the Interface tab and the spell wizard write: they are generated code, and
  the Interface tab writes its modules again from the designs whenever you build a test copy;
- `[DEV]` test copies of maps;
- what Tinkerbench remembers between sessions, such as the last map and folded sections, which
  may reset after an update;
- hand-edited files: the formats are JSON so they can be read, but Tinkerbench writes them
  for itself.
