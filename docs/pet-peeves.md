# Object Editor pet peeves

What's wrong with the World Editor's Object Editor, and what this tool does instead.
Each item is a requirement. Check it off when the tool handles it.

## Finding things
- [x] Tree navigation (race → melee/campaign → units → heroes) and raw codes.
      → One search box for names, raw codes (`Hpal`, `AHhb`) and field values.

## Editing is slow
- [x] A popup dialog for every field.
      → Edit values in place, keyboard navigation (arrows, Enter, Tab).
- [x] Can't tell what changed from the base object.
      → Modified fields highlighted, reset per field, "modified only" filter.

## Field list (see screenshot of the Paladin's fields)
- [x] One flat alphabetical list of 200+ fields with the category glued to each name ("Art - …").
      → Real sections (Stats, Attack 1, Attack 2, Abilities, Art, Sound, Pathing, Text, Tech tree),
        common fields first, rarely used ones in a collapsed "Advanced" group.
- [x] Field names cut off ("Combat - Attack 1 - Damag…" six times).
      → Full names always visible.
- [x] No explanation of what a field does.
      → A short description for every field (hover or expand), plus allowed range and default.
- [x] Values shown without context.
      → Icon fields show the icon, model fields show a 3D preview, colors show a color picker,
        button position shows the command card grid.
- [x] Related values split across rows (damage base / dice / sides, projectile X/Y/Z, tint R/G/B).
      → Grouped into one control, with computed results (damage range including attribute bonus).
- [x] Key numbers buried in the list.
      → Summary at the top: damage, armor, attributes, speed, HP, mana.

## Abilities and levels
- [x] Adding an ability (e.g. Holy Light) tells you nothing: no icon, no description, no values.
      → Ability card everywhere an ability appears: icon, name, rendered tooltip, values per level,
        targets, "used by", one click to open it.
- [x] Multi-level data in long "Level 1 / Level 2 / …" lists.
      → Table with one column per level, edited from the keyboard like a spreadsheet.

## Moving objects between maps
- [ ] No easy way to reuse an object made in another map.
      → Share codes: select objects, copy a text code (`W3OBJ1:…`), paste it into any map,
        preview what will be added before writing.
      → Import from map: open any .w3m/.w3x, browse its custom objects, copy the chosen ones.
      → Dependencies come along (a hero's custom abilities and their buffs).
      → ID conflicts (`A001` already taken) remapped to a free ID, with every reference updated.
      → Bundle file (`.w3obj`) when objects use custom models or icons imported into the map.

## Tooltips and text
- [x] Color codes edited blind.
      → Live preview rendered like the game, with placeholder values (`<AHhb,DataA1>`) filled in.
