---

title: "Units and abilities"
description: "Understand base objects, custom IDs, changed fields, levels and tooltips."
---

## Game objects and custom objects

A game object is identified by a four-character [raw code](../../reference/glossary/#raw-code), such as `hfoo` for Footman. A [custom object](../../reference/glossary/#custom-object) has its own code and copies a [base object](../../reference/glossary/#base-object). Changes are stored as [field overrides](../../reference/glossary/#field-override); unchanged fields inherit their base values.

Open a map before editing. Without a map, the editor browses game data read-only. Use the create action in an object list to make a custom object from a selected base. Keep the assigned raw code when you refer to it in scripts.

## Find and change fields

The list search finds names, raw codes and field values. **Ctrl K** opens commands and object search. Field filters narrow the current page.

Fields are grouped by topic. Changed values are highlighted, and the base-value column shows what the game or base object supplies. Reset one field to remove its override. Focus a field or its information button to read its explanation in the inspector.

Know the result you want but not the field name? [Which field do I change?](../../reference/field-tasks/)
maps common goals to the right object tab, field labels and raw codes, with companion fields to check.

**Armor Type** (`uarm`) controls the hit sound. **Defense Type** (`udty`) controls the damage table. Use the [field reference](../../reference/fields/) when a label is ambiguous.

For help choosing, [compare Armor Type, Defense Type and Defense Base](../../reference/common-confusions/#armor-type-versus-defense-type).

## Ability levels

To make a new spell with guided base selection, values, buff, button and unit assignment,
follow the [spell wizard walkthrough](../../guides/custom-spell/).

Abilities have shared fields and per-level fields. The level table edits values such as mana cost and cooldown independently for each level. Spell-specific data depends on the ability’s base code: the same data column can mean different things for different spells.

Changing a spell’s name or artwork does not change its underlying behavior or order ID. Test multiple abilities together; abilities based on the same spell can conflict.

## Tooltips and art

Tooltips use Warcraft color codes (`|cAARRGGBB` and `|r`) and placeholders such as `<AHhb,DataA1>`. The preview resolves supported placeholders against game and map data. Verify the final text in game, especially when scripts change values at runtime.

The model viewer and icons read imported map assets first, then the installation. A missing preview can mean the referenced file is absent from the installation or imports.

For **No model**, missing textures, blank icons or renderer errors, use the
[artwork troubleshooting checks](../troubleshooting/#a-model-or-icon-is-missing).

## Other tabs

Items, upgrades, buffs, destructables and doodads have their own tabs. Items use plain fields; upgrades expose base/step effects and per-level text. Their reference pages explain the differences. Doodad tint fields are edited per model variation; other doodad fields are shared. The current writer assumes variation levels start at 1 with data column 0; that convention still needs confirmation against a World Editor save.
