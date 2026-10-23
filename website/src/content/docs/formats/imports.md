---

title: "Imports and protected maps"
description: "Understand war3map.imp, path flags and archive discovery limits."
---

Warcraft III maps are MPQ archives. An imported file can exist in the archive even when the archive does not enumerate its name.

## The import list

`war3map.imp` begins with a version and entry count, followed by entries containing a flag byte and a NUL-terminated path.

| Flag     | Observed path interpretation               |
| -------- | ------------------------------------------ |
| 5 or 8   | Relative to `war3mapImported\`             |
| 10 or 13 | Full path                                  |
| 24       | Reforged relative path (`0x10` added to 8) |

The toolkit currently writes flag **29** for a full-path import. This has not yet been confirmed in the World Editor; test imported assets there and in game.

## Protected maps

Protected maps can omit `(listfile)` and often omit `war3map.imp`. This stops normal enumeration, but reading an existing file by its exact name can still succeed.

The toolkit discovers imported object assets by trying names from art fields and following model references to textures. This is reference discovery, not a complete listing of every file in a protected archive. Missing or dynamically constructed paths can prevent a bundle from including everything.

## Paths and previews

Object fields may name `.blp` icons whose installed texture is actually DDS. SD and HD art can live in different layers, and the editor falls back between them when one is missing. Imported map assets take priority over installation files.

Use [bundle files](../../editor/sharing/) when transferring objects with imported assets, then verify the destination map’s model, icon and texture paths. Never commit extracted game assets to this repository.

See the [import discovery source](https://github.com/gabaudette/wc3-toolkit/blob/main/object-editor/crates/wc3-map/src/imports.rs).
