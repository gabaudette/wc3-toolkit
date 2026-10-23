---

title: "Object data files"
description: "How Warcraft III stores original-object changes, custom objects and leveled fields."
---

These are implementation notes from the toolkit, not an official Blizzard format specification. They describe the layouts the readers and writers support; unknown flags remain unknown.

## Files and object kinds

| Map member    | Kind                     | Leveled layout |
| ------------- | ------------------------ | -------------- |
| `war3map.w3u` | Units, heroes, buildings | No             |
| `war3map.w3t` | Items                    | No             |
| `war3map.w3b` | Destructables            | No             |
| `war3map.w3d` | Doodads                  | Yes            |
| `war3map.w3a` | Abilities                | Yes            |
| `war3map.w3h` | Buffs and effects        | No             |
| `war3map.w3q` | Upgrades                 | Yes            |

An object file stores [field overrides](../../reference/glossary/#field-override) for original objects and definitions of [custom objects](../../reference/glossary/#custom-object). Each modification has a four-character [field ID](../../reference/glossary/#field) and a typed value. Custom objects identify the base they inherit from.

## Versions and levels

The parser supports versions 1–3. Version 3 groups modifications into sets with a flags word; the meaning of those flags is not established. Observed maps have used zero.

Leveled modifications store **type, level, column** after the field ID, in that order. For abilities, non-level fields use level 0 and column 0. Per-level fields use levels starting at 1; `DataA` uses column 1, `DataB` column 2, and generic fields such as cooldown use column 0. Upgrade data columns use zero.

Bit flags and numbered choices are stored as integers. Do not interpret every metadata choice as a string.

## Skin files

Reforged separates some cosmetic changes into `war3mapSkin` object files, such as `war3mapSkin.w3u`. Name changes are confirmed in this split; other fields follow the metadata’s `netsafe` flag. Read both the gameplay and skin files when reconstructing an object.

Text values can point into the [map string table](../strings/) instead of containing the final text.

## Validate a round trip

From `object-editor/`:

```sh
cargo run --release -p wc3-map --bin wc3map -- path/to/map.w3m check
```

The check rewrites object data and strings in memory and compares the result byte for byte. It does not replace the map. Successful round trips establish preservation for those files, not that the map’s gameplay is correct.

See the [object parser source](https://github.com/gabaudette/wc3-toolkit/blob/main/object-editor/crates/wc3-data/src/objects.rs).
