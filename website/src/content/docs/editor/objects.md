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

## Checkboxes and option chips

Fields with several allowed options, such as **Targets Allowed**, **Targeted as**,
**Unit Classification** and a building's placement pathing, show their options as chips in
columns under the field; target lists group them by owner, kind and state. Select a chip to
toggle that option. Checked options are gold; a gold dot marks an option that differs from
the game's value, and the game value column counts how many are on. Ordinary on/off fields
use a gold checkbox with a check mark.

Drop-downs use the app's styling for both the field and the open menu; keyboard focus has
a light ring, and a focused text box has a light edge.

<figure class="app-screenshot">
  <a href="../../screenshots/objects-options.png">
    <img src="../../screenshots/objects-options.png" alt="Training construct&#x27;s changed fields: Unit Classification chips with Mechanical checked and marked changed, and Targeted as chips grouped under Owner and Kind." width="1440" height="900" loading="lazy" />
  </a>
  <figcaption><span class="screenshot-notice">Image subject to change.</span>Unit Classification and Targeted as use option chips; the gold dot marks Mechanical as changed.</figcaption>
</figure>

## Damage and bounty dice

Grouped damage and bounty fields show the minimum, maximum and average as you type,
followed by the base plus dice calculation. For example, a base of 10 with two six-sided
dice gives a minimum of 12, maximum of 22 and average of 17: `10 + 2 × d6`.

For hero attacks, the editor also shows the primary attribute's bonus using the map's
damage-per-attribute-point constant. The attack summary is for level 1, before armor,
upgrades, items and auras; it is not a prediction of every hit in a running game.
See [gameplay constants](../map-settings/#change-gameplay-constants) to change the map-wide rule.

A fraction in a whole-number field or an out-of-range value gets an explanation instead
of being saved. Correct the draft, then press Enter or leave the field to commit it.
The [recorded play test](../verification/#map-settings-constants-and-dice) matched the
illustrated level-1 range of 34–44 after resetting the primary-attribute override and starting
a fresh game process. With that constant explicitly set to 2, five attacks stayed within
56–66; ten bounty awards with base 25 and two six-sided dice stayed within 27–37.
These samples check the ranges, not the roll distribution or empirical average.

<figure class="app-screenshot">
  <a href="../../screenshots/objects-dice.png">
    <img src="../../screenshots/objects-dice.png" alt="Sunwell Warden&#x27;s damage fields showing 10 base plus two six-sided dice and 22 Strength, for 34 to 44 damage with an average of 39." width="1440" height="900" loading="lazy" />
  </a>
  <figcaption><span class="screenshot-notice">Image subject to change.</span>The hero&#x27;s level-1 damage preview adds 10 base, two six-sided dice and 22 from Strength: 34–44, average 39.</figcaption>
</figure>

## Ability levels

To make a new spell with guided base selection, values, buff, button and unit assignment,
follow the [spell wizard walkthrough](../../guides/custom-spell/).

Abilities have shared fields and per-level fields. The level table edits values such as mana cost and cooldown independently for each level. Spell-specific data depends on the ability’s base code: the same data column can mean different things for different spells.

Changing a spell’s name or artwork does not change its underlying behavior or order ID. Test multiple abilities together; abilities based on the same spell can conflict.

## Tooltips and art

Tooltips use Warcraft color codes (`|cAARRGGBB` and `|r`) and placeholders such as `<AHhb,DataA1>`. The preview resolves supported placeholders against game and map data. Verify the final text in game, especially when scripts change values at runtime.

The text preview flags incomplete color codes, a `|r` without a color to end, colors left
open, unknown `|` codes, malformed or unclosed placeholders, and placeholders Tinkerbench
cannot fill. For an unfilled placeholder, check the object ID and column.
**Formatting help** shows color syntax (`|cffRRGGBB…|r`), line breaks (`|n`) and value
placeholders, including the `,%` suffix for percentages. Correct warnings in the source
text, then check the preview again; the **In game** label is a preview, not a play-test result.

<figure class="app-screenshot">
  <a href="../../screenshots/objects-formatting.png">
    <img src="../../screenshots/objects-formatting.png" alt="Sunwell Light&#x27;s tooltip source and preview with Formatting help expanded, an incomplete color code, an unmatched reset and an unresolved placeholder." width="1440" height="900" loading="lazy" />
  </a>
  <figcaption><span class="screenshot-notice">Image subject to change.</span>Formatting help stays beside the text; the preview lists the color, reset and placeholder problems in this deliberately incorrect example.</figcaption>
</figure>

The model viewer and icons read imported map assets first, then the installation. A missing preview can mean the referenced file is absent from the installation or imports.

Use [Assets](../assets/) to import files, inspect their uses or make a unit from a model.

For **No model**, missing textures, blank icons or renderer errors, use the
[artwork troubleshooting checks](../troubleshooting/#a-model-or-icon-is-missing).

## Other tabs

Items, upgrades, buffs, destructables and doodads have their own tabs. Items use plain fields; upgrades expose base/step effects and per-level text. Their reference pages explain the differences. Doodad tint fields are edited per model variation; other doodad fields are shared. The current writer assumes variation levels start at 1 with data column 0; that convention still needs confirmation against a World Editor save.
