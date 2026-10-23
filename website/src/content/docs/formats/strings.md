---

title: "Map strings"
description: "Resolve TRIGSTR references through war3map.wts."
---

`war3map.wts` maps numeric IDs to text. An object field may contain `TRIGSTR_007` instead of a literal name or tooltip. Resolve that reference before displaying the value to a user.

A small synthetic example:

```text
STRING 7
{
A custom unit name
}
```

The table can have a UTF-8 byte order mark, CRLF line endings and `//` comments. Text is stored between braces and can span multiple lines. A value that looks numeric may still be a text reference: the World Editor stores some upgrade choices, including class and effect, as `TRIGSTR` values.

The editor routes unit, ability, item and upgrade text through the map’s shared string table. A transfer must resolve source references and allocate destination entries instead of copying a `TRIGSTR` token blindly.

The [string parser](https://github.com/gabaudette/wc3-toolkit/blob/main/object-editor/crates/wc3-data/src/strings.rs) preserves round-trip data for the supported layouts. The Lua harness also uses the string table to add `[DEV]` to a resolvable map name.
