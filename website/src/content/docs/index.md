---

title: "The map maker’s field guide"
description: "Tinkerbench, JASS and Lua, and Warcraft III object data in one place."
---

Make a unit, change a spell, understand a field. This is the working handbook for **Tinkerbench** and Warcraft III map making.

Tinkerbench is in early development. Use a copy of your map while trying the editor.

## Start at the workbench

- [Install Tinkerbench](./editor/install/) and connect your Warcraft III installation.
- [Edit your first map](./editor/first-map/): change a unit, save, and test in game.
- [Work with units and abilities](./editor/objects/): base values, levels and tooltips.
- [Share your objects](./editor/sharing/) with share codes and bundle files.
- [Add Lua modules](./editor/lua-harness/) without losing them when the World Editor saves.

## Make something

[Create a custom spell](./guides/custom-spell/) · [Build a hero](./guides/hero/) · [Make an item](./guides/item/) · [Write a trigger](./guides/triggers/)

Choose **Lua** or **JASS** in the top bar (in the menu on mobile) to switch all paired
examples. Browse the [code examples cookbook](./reference/examples/) for practical recipes.

## Look it up

- **[API reference](./reference/api/)** — generated declarations, parameter types, call syntax and selected examples in your chosen language. Search an exact name like `CreateUnit`.
- **[Object field reference](./reference/fields/)** — raw codes, meanings, values and gotchas from the same field guides as the editor.
- **[Map formats](./formats/object-data/)** — object files, strings, imports and protected maps, based on the toolkit’s readers and writers.

Use **Code theme** above any snippet to choose its colors. The choice applies to all snippets
and is remembered across pages. **Follow site theme** matches the site’s dark or light mode.

Press **Ctrl K** (or **⌘ K**) to search. The published site searches guides, API symbols and raw field codes together.

## About this guide

The editor reads game data from your own legal Warcraft III: Reforged installation. This site contains project-authored guides and extracted declaration metadata, not game scripts or assets. API and field metadata have a recorded source snapshot; they may differ from a newer patch.

[Source and releases](https://github.com/gabaudette/wc3-toolkit). Warcraft and Blizzard Entertainment are trademarks or registered trademarks of Blizzard Entertainment, Inc. This project is not affiliated with or endorsed by Blizzard Entertainment.
