# Changelog

What changed in Tinkerbench, for the people who use it. Performance and internal changes get
one line here; the details are in the [devlog](docs/devlog.md). Tinkerbench shows this file
under "What's new".

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
