# Changelog

What changed in Tinkerbench, for the people who use it. Performance and internal changes get
one line here; the details are in the [devlog](docs/devlog.md). Tinkerbench shows this file
under "What's new".

## Unreleased

### Added

- Code snippet theme picker in the documentation, with six color themes and a remembered
  choice across pages.
- [Documentation website](https://tinkerbench-docs.vercel.app): editor guides, JASS and Lua API declarations, and a searchable
  reference for object fields.
- Buffs tab: browse, search, edit and create custom buffs and effects, with a chip per race.
- Destructables tab: trees, gates, crates and rocks, with a chip per category and their model
  in the inspector.
- Doodads tab: props, plants, rocks and lights, with a chip per category, their model in the
  inspector, and the tint of every model variation side by side. Every object type of the
  World Editor's Object Editor can now be edited.

### Changed

- Destructables and doodads show the World Editor's icon for their category (Props, Bridges,
  Water, ...) in lists, on their page and in search, since the game gives them no icon.
- Names with Warcraft color codes show in color everywhere: lists, page titles, search,
  ability cards, sharing and import, and the base value column. Colors too dark to read on the
  editor's panels are lightened a little. Text you edit still shows the codes.

### Fixed

- Long game texts, like tooltips, showed quote marks around their value when edited.
- Destructables available in several tilesets only showed the first one.
- Textures and icons imported into a map as `.tga` files now show.
- The model preview now shows the newer campaign units, such as Roderick, the Scarlet
  Crusade and the Forsaken, and a few cinematic doodads. Models the game does not have say
  so instead of showing a file error.
- The model preview no longer fails with "Missing shader uniform location: uRoughness" on
  some graphics drivers.
- Internal improvements.

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
