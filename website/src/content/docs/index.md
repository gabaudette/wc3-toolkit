---

title: "The map maker’s field guide"
description: "Tinkerbench, JASS and Lua, and Warcraft III object data in one place."
---

Make a unit, change a spell, understand a field. This is the working handbook for **Tinkerbench** and Warcraft III map making.

:::caution[Early development]
Tinkerbench is in early development. Use a copy of your map while trying the editor.
:::

## Meet Tinkerbench

Browse your map’s objects, edit their fields, and see what each value means without leaving
the workbench.

<figure class="app-screenshot">
  <a href="./screenshots/unit-editor.png">
    <img src="./screenshots/unit-editor.png" alt="Tinkerbench with the Paladin selected: searchable objects on the left, grouped fields in the center, and a 3D model and field explanation on the right." width="1440" height="900" fetchpriority="high" />
  </a>
  <figcaption>The unit editor, running with a real Warcraft III installation. Select the screenshot to see it at full size.<span class="screenshot-notice">Image subject to change.</span></figcaption>
</figure>

- **Find an object quickly.** Search by name or raw code, or use **Ctrl K** across object types.
- **Understand your changes.** Read field explanations beside the values, compare with the base,
  and preview models and tooltips.
- **Balance spells by level.** Edit mana cost, cooldown and other values in a table with a column
  for each level.

[Take the visual tour](./editor/visual-tour/) · [Install Tinkerbench](./editor/install/) · [Download a release](https://github.com/gabaudette/wc3-toolkit/releases)

## Start at the workbench

- [Install Tinkerbench](./editor/install/) and connect your Warcraft III installation.
- [Take the visual tour](./editor/visual-tour/): find your way around the application with screenshots.
- [Edit your first map](./editor/first-map/): change a unit, save, and test in game.
- [Work with units and abilities](./editor/objects/): base values, levels and tooltips.
- [Share your objects](./editor/sharing/) with share codes and bundle files.
- [Add Lua modules](./editor/lua-harness/) without losing them when the World Editor saves.
- [Troubleshoot a problem](./editor/troubleshooting/): loading, editing, saving, previews and imports.

## Make something

[Create a custom spell](./guides/custom-spell/) · [Build a hero](./guides/hero/) · [Make an item](./guides/item/) · [Write a trigger](./guides/triggers/)

Choose **Lua** or **JASS** in the top bar (in the menu on mobile) to switch examples and
language-specific explanations. Browse the [code examples cookbook](./reference/examples/) for practical recipes.

## Look it up

- **[Map-making glossary](./reference/glossary/)** — definitions and examples for raw codes, custom objects, dependencies, handles and other terms used in the guides.
- **[Commonly confused concepts](./reference/common-confusions/)** — compare armor fields, removing versus killing units, and coordinates versus location handles.
- **[Visual scripting lifecycles](./reference/lifecycles/)** — follow timer, group and location ownership through diagrams and code examples.
- **[Which field do I change?](./reference/field-tasks/)** — start with a goal, from movement speed and attack range to item charges or model size.
- **[API reference](./reference/api/)** — generated declarations, parameter types, call syntax and selected examples in your chosen language. Search an exact name like `CreateUnit`.
- **[Object field reference](./reference/fields/)** — raw codes, meanings, values and gotchas from the same field guides as the editor.
- **[Map formats](./formats/object-data/)** — object files, strings, imports and protected maps, based on the toolkit’s readers and writers.

Use **Code theme** above any snippet to choose its colors. The choice applies to all snippets
and is remembered across pages. **Follow site theme** matches the site’s dark or light mode.

Press **Ctrl K** (or **⌘ K**) to search. The published site searches guides, API symbols and raw field codes together. Use **Search in** to
choose **All content**, **Guides**, **API** or **Object fields**. Practical API explanations
appear in both Guides and API.

## About this guide

Want to help? The [Contributing section](./contributing/) covers bug reports, local
development, documentation changes and pull requests.

The editor reads game data from your own legal Warcraft III: Reforged installation. This site contains project-authored guides, application screenshots and extracted declaration metadata. API and field metadata have a recorded source snapshot; they may differ from a newer patch.

[Source and releases](https://github.com/gabaudette/wc3-toolkit). Warcraft and Blizzard Entertainment are trademarks or registered trademarks of Blizzard Entertainment, Inc. This project is not affiliated with or endorsed by Blizzard Entertainment.
