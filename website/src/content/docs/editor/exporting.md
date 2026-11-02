---

title: "Export objects as CSV or JSON"
description: "Export object fields for spreadsheets and tools from Tinkerbench or the wc3map command line."
---

Use an object export to inspect balance in a spreadsheet, build a wiki table or show another
tool the map's exact IDs and values. It includes the map's overrides and inherited values
for the chosen objects. To move editable objects between maps, use [sharing](../sharing/)
instead; CSV and JSON are data exports.

## Export from the editor

1. Choose **Export…** above an object list, or **Export objects…** from the **Ctrl K** palette.
2. Under **Objects**, choose the objects the list shows, the selected object, all objects of
   that type, or **Every object type**. Choices depend on where you opened the dialog.
3. For an open map, **From this map** offers **All of them, with this map's changes**,
   **Only what this map adds or changes** or **Only custom objects this map adds**.
4. Set **Format** to **CSV: a table per object type** or **JSON: one file for tools**.
   For CSV, **Cells hold** chooses **Values as the editor shows them** or
   **Values as stored in the data**.
5. Optionally enable **Remove color codes from text**, then choose **Export…** and a
   destination. CSV for several types asks for a folder; one type or JSON asks for a file.

<figure class="app-screenshot">
  <a href="../../screenshots/objects-export.png">
    <img src="../../screenshots/objects-export.png" alt="Export objects dialog selecting Sunwell Warden, custom objects only, CSV, editor-formatted values and the Remove color codes checkbox." width="678" height="463" loading="lazy" />
  </a>
  <figcaption>Choose the object scope, map filter and CSV cell values before selecting a destination.<span class="screenshot-notice">Image subject to change.</span></figcaption>
</figure>

Without an open map, exports use the installation's game objects. With a map, choosing only
changed objects filters the object list; each exported object still contains all applicable
fields, not only its overrides.

## Export with wc3map

With the `wc3map` binary available:

```sh
wc3map --game "/path/to/Warcraft III" MyMap.w3x export json -o MyMap.json
wc3map --game "/path/to/Warcraft III" MyMap.w3x export csv units abilities --changed -o out/
wc3map --game "/path/to/Warcraft III" - export csv -o game/
```

The last command uses `-` in place of a map for game data alone. To run from the checkout,
start in `object-editor/` and replace `wc3map` with
`cargo run --release -p wc3-map --bin wc3map --`.

| Option                                                                         | Selection or output                                             |
| ------------------------------------------------------------------------------ | --------------------------------------------------------------- |
| `units`, `items`, `buffs`, `destructables`, `abilities`, `upgrades`, `doodads` | Types to include; omission selects every type.                  |
| `--changed` / `--custom`                                                       | Only objects the map adds or changes / only its custom objects. |
| `--id ID`                                                                      | A specific raw code; repeat for several objects.                |
| `--plain`                                                                      | Removes color codes from display text and turns `               | n` into a line break. |
| `--raw`                                                                        | Stored values in CSV cells.                                     |
| `--hd` / `--balance SET`                                                       | HD art values / a chosen balance data set.                      |

## Read the output in a tool

JSON is one UTF-8 document with `format: "tinkerbench-objects"` and `version: 1`. Check
those before reading its shape. `map` is the map filename, or `null` for game-only data.
`fields` describes fields once per object type, keyed by their field IDs, with a display
name, category and `perLevel` flag. `objects` holds one list per exported type.

Each object identifies its `id`, `base`, `name`, `custom`, `changed` and `levels`.
`modified` identifies overrides as `fieldId` or `fieldId:level`. Under `values`, each field
has a `{raw, display}` pair, or an array of those pairs for per-level fields. Both values
are strings: `raw` preserves stored codes and lists, while `display` supplies labels,
formatted numbers and resolved tooltip placeholders. Use raw values for analysis that
needs exact IDs; display text is intended for readers. Doodad levels represent variations.

CSV has one file per type, with identity columns followed by a column per field and level.
Headers include field IDs, such as `Field name [fieldId:level]`; absent fields or levels
are empty. It uses UTF-8 with a byte order mark, CRLF line endings and quoted cells where
needed. Objects are sorted by ID and fields by field ID in both formats, giving repeatable
output for unchanged inputs and options.

For the complete schema and a worked JSON example, see the repository's
[export format specification](https://github.com/gabaudette/wc3-toolkit/blob/main/docs/export-format.md).
