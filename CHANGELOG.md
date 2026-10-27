# Changelog

What changed in Tinkerbench, for the people who use it. Performance and internal changes get
one line here; the details are in the [devlog](docs/devlog.md). Tinkerbench shows this file
under "What's new".

## Unreleased

### Added

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
- Contextual documentation problem links that open a GitHub draft with the page, API or
  field identifier and selected example language.
- Visual scripting lifecycles for timers, groups and locations, with six responsive diagrams,
  accessible text explanations and five expandable examples that follow the Lua/JASS toggle.
- Comparisons for Armor Type versus Defense Type, removing versus killing units, and
  coordinates versus location handles, with decision tables, cleanup notes and linked examples.
- A linked map-making glossary with 33 definitions, examples and reference links, including
  raw codes, custom objects, dependencies, handles, natives and Blizzard helpers.
- Website search filters for guides, APIs and object fields, with result-type labels and
  improved discovery of practical guides alongside exact symbol and raw-code lookups.

- A “Which field do I change?” guide mapping 19 common goals to object tabs, field labels,
  raw codes and companion settings, linked from the homepage, field reference and editing guide.
- Practical explanations for 26 common APIs, with parameter and return notes, ownership and
  cleanup guidance, common mistakes, related links and paired JASS/Lua examples. A new
  practical guide index and topic links make the curated pages easier to find.

- Troubleshooting hub for loading game data, editing and saving maps, missing previews and
  import problems, with checks by symptom and a bug-report template.
- Tinkerbench copyright footer throughout the documentation, with contributor attribution
  and a link to the MIT license.
- Contributing section on the documentation website, with bug-report guidance, local editor
  setup, documentation authoring and validation instructions.
- API reference organized into 22 topics, with category pages, declaration counts and links
  from each symbol back to its topic. The complete alphabetical index is still available.
- Screenshot tour of the Tinkerbench application, from loading game data and opening a map
  to editing and sharing objects, with a preview on the documentation homepage and in the README.
  Each screenshot notes that the image is subject to change.
- Documentation cookbook with 43 JASS/Lua code recipes, and a global language toggle that
  remembers your choice across examples and explanations, with language-neutral navigation.
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

- The spell wizard no longer closes, and throws the spell away, when clicking outside it.
- Dialogs open near the top of the window and stay there while their content grows or
  shrinks, so buttons no longer move under the pointer.
- Documentation navbar title is now “Tinkerbench”. Theme, code-theme and search controls use
  a subtle focus underline instead of a blue outer outline. The homepage’s early-development
  notice appears in an inline warning callout.
- Screenshot change notices are centered and muted beneath their images.
- Documentation uses the app icon as its PNG favicon.
- External links in the documentation open in a new tab.
- Destructables and doodads show the World Editor's icon for their category (Props, Bridges,
  Water, ...) in lists, on their page and in search, since the game gives them no icon.
- Names with Warcraft color codes show in color everywhere: lists, page titles, search,
  ability cards, sharing and import, and the base value column. Colors too dark to read on the
  editor's panels are lightened a little. Text you edit still shows the codes.

### Fixed

- Ability, upgrade and doodad pages no longer keep redrawing in the background while open,
  which used processor time for nothing.
- Tooltip previews now update as soon as a value they show changes, not only when the text
  itself is edited.
- Long game texts, like tooltips, showed quote marks around their value when edited.
- Destructables available in several tilesets only showed the first one.
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
- Performance improvements.
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
