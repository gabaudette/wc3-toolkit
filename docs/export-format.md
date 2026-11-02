# Object export format

Tinkerbench exports Warcraft III objects (units, items, buffs, destructables, abilities,
upgrades and doodads) as CSV or JSON for spreadsheets, wiki pages and other tools. Export
from the app ("Export…" above any object list, or "Export objects…" in the command palette)
or from the command line:

```sh
wc3map --game "/path/to/Warcraft III" MyMap.w3x export json -o MyMap.json
wc3map --game "/path/to/Warcraft III" MyMap.w3x export csv units abilities --changed -o out/
wc3map --game "/path/to/Warcraft III" - export csv -o game/    # the game's own objects
```

Options: object types to include (default: all), `--changed` for only the objects the map
adds or changes, `--custom` for only the custom objects it adds, `--plain` to remove color
codes from shown values, `--raw` for CSV cells with stored values instead of shown ones,
`--id ID` (repeatable) for specific objects, `--hd` for HD art values, `--balance SET` for a
balance data set.

The same objects and options always give the same file: objects are sorted by ID, fields by
field ID, and nothing depends on the time.

## JSON

One UTF-8 file. Each type's fields are described once, under `fields`; each object holds its
values keyed by field ID:

```json
{
  "format": "tinkerbench-objects",
  "version": 1,
  "map": "MyMap.w3x",
  "fields": {
    "abilities": {
      "acdn": { "name": "Cooldown", "category": "Stats", "perLevel": true },
      "anam": { "name": "Name", "category": "Text", "perLevel": false }
    }
  },
  "objects": {
    "abilities": [
      {
        "id": "A000",
        "base": "AHbz",
        "name": "Frost Storm",
        "custom": true,
        "changed": true,
        "levels": 3,
        "modified": ["acdn:3", "anam"],
        "values": {
          "acdn": [
            { "raw": "6", "display": "6.000" },
            { "raw": "6", "display": "6.000" },
            { "raw": "4", "display": "4.000" }
          ],
          "anam": { "raw": "Frost Storm", "display": "Frost Storm" }
        }
      }
    ]
  }
}
```

| Key | Meaning |
|---|---|
| `format`, `version` | Always `tinkerbench-objects`; the version changes only when the shape does. |
| `map` | The map's file name, or `null` for the game's own objects. |
| `fields` | Per exported type, every field its objects have: the World Editor's `name` and `category`, and `perLevel`. Keyed by the field ID stored in map files, e.g. `uhpm`. |
| `objects` | One list per exported type: `units`, `items`, `buffs`, `destructables`, `abilities`, `upgrades`, `doodads`. |
| `id` | The object's four-character ID. |
| `base` | The game object a custom object copies; its own ID for game objects. |
| `name` | The name, with the map's text. |
| `custom` | Added by the map (not a game object). |
| `changed` | The map adds or changes it. |
| `levels` | Levels (abilities, upgrades) or variations (doodads); 1 for other types. |
| `modified` | The values the map changes: a field ID, or `fieldId:level` for a level of a per-level field. |
| `values` | Every field the World Editor shows for the object, changed or not: one value, or a list with one per level for per-level fields. |
| `raw` | The value as stored in the game data or the map: numbers, IDs, `0`/`1` for flags, comma lists. Empty when the object has none. |
| `display` | The value as the editor shows it: choices by their label, numbers formatted, tooltip placeholders such as `<AHbz,DataA1>` filled in. Color codes (`|cffffcc00…|r`) stay unless `--plain` or "Remove color codes" is chosen, which also turns `|n` into a line break. |

Exporting every object of the game gives about 40 MB of JSON; a map's own objects are small.

## CSV

One file per object type (a folder of `<type>.csv` when several types are exported). UTF-8
with a byte order mark and CRLF line ends, so spreadsheets open it without asking; cells with
commas, quotes or line breaks are quoted.

- Columns: `id`, `base`, `name`, `custom`, `changed`, `levels`, then one column per field,
  named `Field name [fieldId]`, or `Field name [fieldId:level]` with a column per level for
  per-level fields. Columns are sorted by field ID.
- Cells hold the shown values by default, or the stored ones (`--raw`, "Values as stored in
  the data"). A field an object does not have, or a level past its last, is empty.
