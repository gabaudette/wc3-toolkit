---
title: "Which field do I change?"
description: "Find object fields by goal: move faster, change health, attack damage, range or cooldown, resize models, edit item charges and shop costs."
---

Start with the result you want, then follow the field links for meanings, allowed values and
raw codes. This is a field lookup guide; the [first-map walkthrough](../../editor/first-map/)
shows how to make, save and test an edit.

## Pick a task

<div class="field-task-index">

| I want to… | Object tab |
| --- | --- |
| [Make a unit move faster](#make-a-unit-move-faster) | Units |
| [Make a unit turn faster](#make-a-unit-turn-faster) | Units |
| [Change maximum life or life regeneration](#change-maximum-life-or-life-regeneration) | Units, Destructables |
| [Change maximum mana or starting mana](#change-maximum-mana-or-starting-mana) | Units |
| [Change day or night vision](#change-day-or-night-vision) | Units |
| [Change attack damage](#change-attack-damage) | Units |
| [Make a unit attack faster](#make-a-unit-attack-faster) | Units |
| [Increase attack range](#increase-attack-range) | Units |
| [Change what an attack can hit](#change-what-an-attack-can-hit) | Units |
| [Change armor or defense type](#change-armor-or-defense-type) | Units |
| [Change training cost or build time](#change-training-cost-or-build-time) | Units |
| [Change food used or provided](#change-food-used-or-provided) | Units |
| [Change an item's shop price](#change-an-items-shop-price) | Items |
| [Change item stock or restock time](#change-item-stock-or-restock-time) | Items, Units |
| [Change item charges](#change-item-charges) | Items |
| [Change a name or tooltip](#change-a-name-or-tooltip) | Units, Items |
| [Change a model or icon](#change-a-model-or-icon) | Units, Items, Doodads, Destructables |
| [Make a model larger or smaller](#make-a-model-larger-or-smaller) | Units, Items, Doodads |
| [Tint a model](#tint-a-model) | Units, Items, Doodads, Destructables |

</div>

## Before editing

1. Open a copy of your map and select the correct object tab. Game-data-only browsing is
   read-only. Editing a game object changes that object type wherever the map uses it;
   create a custom object when you need a separate type.
2. Find the object, then use **Filter fields** to locate the linked field label or raw code.
   Clear **Changed only** if it hides an unchanged field.
3. Commit the field edit with **Enter** or by leaving the field, save, and test the saved map.
   The [editing guide](../../editor/objects/#find-and-change-fields) explains base values and reset.

**Review status:** The field choices and labels below were checked against the project's
existing field explanations and metadata snapshot. These task combinations have not been
tested in game for this guide. Follow any **Unverified** notes on the linked reference pages;
bonuses, gameplay constants and triggers can affect the final result.

## Movement

### Make a unit move faster

**Tab: Units.** Start with [Speed Base (`umvs`)](../fields/unit/umvs/).

Check [Type (`umvt`)](../fields/unit/umvt/) in the movement fields as well: a stationary unit
needs a suitable movement type. [Animation - Walk Speed (`uwal`)](../fields/unit/uwal/) and
[Animation - Run Speed (`urun`)](../fields/unit/urun/) describe the model's animation speeds;
they do not set actual movement speed. Use the field reference's limits rather than assuming
that any larger number will produce that speed in game.

### Make a unit turn faster

**Tab: Units.** Start with [Turn Rate (`umvr`)](../fields/unit/umvr/).

Turn rate controls facing changes. It does not replace [Speed Base (`umvs`)](../fields/unit/umvs/)
for travel speed. Change one at a time and check both turning and moving with the selected model.

## Life, mana and vision

### Change maximum life or life regeneration

**Tab: Units.** Use [Hit Points Maximum (Base) (`uhpm`)](../fields/unit/uhpm/) for base maximum
life, [Hit Points Regeneration Rate (`uhpr`)](../fields/unit/uhpr/) for recovery rate, and
[Hit Points Regeneration Type (`uhrt`)](../fields/unit/uhrt/) for when recovery is allowed.

A larger regeneration rate will not help when the regeneration type disallows it. Heroes
also gain life from [Starting Strength (`ustr`)](../fields/unit/ustr/), so base life alone
is not their final displayed maximum.

**Tab: Destructables.** A tree, gate or crate instead uses [Hit Points (`bhps`)](../fields/destructable/bhps/).
Select the destructable type rather than looking for the unit field on that tab.

### Change maximum mana or starting mana

**Tab: Units.** Use [Mana Maximum (`umpm`)](../fields/unit/umpm/) for capacity and
[Mana Initial Amount (`umpi`)](../fields/unit/umpi/) for the amount a newly created unit starts with.
[Mana Regeneration (`umpr`)](../fields/unit/umpr/) changes recovery over time.

Starting mana and maximum mana are separate settings. Heroes also gain mana from
[Starting Intelligence (`uint`)](../fields/unit/uint/); changing a tooltip does not change any of these values.

### Change day or night vision

**Tab: Units.** Use [Sight Radius (Day) (`usid`)](../fields/unit/usid/) and
[Sight Radius (Night) (`usin`)](../fields/unit/usin/).

Edit both if the intended vision should be the same throughout the day/night cycle. These
fields describe sight, while [Acquisition Range (`uacq`)](../fields/unit/uacq/) controls when
a unit looks for enemies to attack on its own.

## Attacks and defense

For units with two attacks, check [Attacks Enabled (`uaen`)](../fields/unit/uaen/) before
editing a field. Attack 1 and Attack 2 have separate damage, cooldown, range and target settings.

### Change attack damage

**Tab: Units.** Edit the three fields for the attack you use:

- Attack 1: [Attack 1 - Damage Base (`ua1b`)](../fields/unit/ua1b/),
  [Attack 1 - Damage Number of Dice (`ua1d`)](../fields/unit/ua1d/), and
  [Attack 1 - Damage Sides per Die (`ua1s`)](../fields/unit/ua1s/).
- Attack 2: [Attack 2 - Damage Base (`ua2b`)](../fields/unit/ua2b/),
  [Attack 2 - Damage Number of Dice (`ua2d`)](../fields/unit/ua2d/), and
  [Attack 2 - Damage Sides per Die (`ua2s`)](../fields/unit/ua2s/).

For ordinary positive dice, the base range is **base + dice** through **base + dice × sides**.
For example, base `10`, `2` dice and `6` sides gives `12–22` before other contributions.
Heroes add their [Primary Attribute (`upra`)](../fields/unit/upra/). Upgrades, bonuses and
the target's defense can change the damage seen in game; this range is not a full combat prediction.

### Make a unit attack faster

**Tab: Units.** Start with [Attack 1 - Cooldown Time (`ua1c`)](../fields/unit/ua1c/) or
[Attack 2 - Cooldown Time (`ua2c`)](../fields/unit/ua2c/). The value is a time interval;
a smaller positive value means a shorter base interval between attacks.

Do not confuse cooldown with [Attack 1 - Animation Damage Point (`udp1`)](../fields/unit/udp1/)
or [Attack 1 - Animation Backswing Point (`ubs1`)](../fields/unit/ubs1/). Those control parts
of the attack animation. Agility and attack-speed bonuses can also change the observed rate.

### Increase attack range

**Tab: Units.** Start with [Attack 1 - Range (`ua1r`)](../fields/unit/ua1r/) or
[Attack 2 - Range (`ua2r`)](../fields/unit/ua2r/).

Also check [Acquisition Range (`uacq`)](../fields/unit/uacq/) for automatic engagement and
[Minimum Attack Range (`uamn`)](../fields/unit/uamn/) for targets that are too close. Raising
range alone does not define a ranged weapon: [Attack 1 - Weapon Type (`ua1w`)](../fields/unit/ua1w/)
and [Attack 1 - Projectile Art (`ua1m`)](../fields/unit/ua1m/) describe how that attack travels and looks.

### Change what an attack can hit

**Tab: Units.** Start with [Attack 1 - Targets Allowed (`ua1g`)](../fields/unit/ua1g/) or
[Attack 2 - Targets Allowed (`ua2g`)](../fields/unit/ua2g/).

Check [Attacks Enabled (`uaen`)](../fields/unit/uaen/) and the target object's
[Targeted as (`utar`)](../fields/unit/utar/) classification. The attacker's allowed targets
and the target's classification are different sides of the check. Confirm the intended
attack is enabled before changing the target to compensate.

### Change armor or defense type

**Tab: Units.** Use [Defense Base (`udef`)](../fields/unit/udef/) for the base armor value and
[Defense Type (`udty`)](../fields/unit/udty/) for the defense category used by the damage table.

[Armor Type (`uarm`)](../fields/unit/uarm/) describes the hit-sound material, such as flesh
or metal. Changing it does not select the defense category. For the other side of the damage
table, see the attacker's [Attack 1 - Attack Type (`ua1t`)](../fields/unit/ua1t/).

The [armor-field comparison](../common-confusions/#armor-type-versus-defense-type) walks
through choosing a sound material, defense category or base armor amount.

## Costs, shops and charges

### Change training cost or build time

**Tab: Units.** Use [Gold Cost (`ugol`)](../fields/unit/ugol/),
[Lumber Cost (`ulum`)](../fields/unit/ulum/), and [Build Time (`ubld`)](../fields/unit/ubld/).

These describe the unit or building being produced. Editing cost does not add it to a
producer's [Units Trained (`utra`)](../fields/unit/utra/) or
[Structures Built (`ubui`)](../fields/unit/ubui/) list. Item shop prices use separate item fields below.

### Change food used or provided

**Tab: Units.** Use [Food Cost (`ufoo`)](../fields/unit/ufoo/) for supply consumed and
[Food Produced (`ufma`)](../fields/unit/ufma/) for supply provided.

A Farm-like provider and a trained unit's food requirement use different fields. Check the
intended object type before changing both.

### Change an item's shop price

**Tab: Items.** Use [Gold Cost (`igol`)](../fields/item/igol/) and
[Lumber Cost (`ilum`)](../fields/item/ilum/).

These set purchase costs. The shop's [Items Sold (`usei`)](../fields/unit/usei/) list is a
**Units** field; changing an item's price does not add the item to that list. Stock and
restock time are separate settings.

### Change item stock or restock time

**Tab: Items.** Use [Stock Maximum (`isto`)](../fields/item/isto/) for capacity and
[Stock Replenish Interval (`istr`)](../fields/item/istr/) for restocking time.

Also check [Stock Start Delay (`isst`)](../fields/item/isst/) and
[Stock Initial After Start Delay (`isit`)](../fields/item/isit/) when the item does not appear
as expected at the start of the map. The offering shop still needs the item in its
[Items Sold (`usei`)](../fields/unit/usei/) list on the **Units** tab.

### Change item charges

**Tab: Items.** Start with [Number of Charges (`iuse`)](../fields/item/iuse/).

Check [Actively Used (`iusa`)](../fields/item/iusa/) for clickable use and
[Perishable (`iper`)](../fields/item/iper/) for removal after the last charge. The item's
[Abilities (`iabi`)](../fields/item/iabi/) determine its effects; changing the charge count
alone does not give it an effect.

## Names and appearance

### Change a name or tooltip

**Tabs: Units and Items.** Both use `unam`, `utip` and `utub`, but follow the link for the tab
you are editing:

- Units: [Name (`unam`)](../fields/unit/unam/),
  [Tooltip - Basic (`utip`)](../fields/unit/utip/), and
  [Tooltip - Extended (`utub`)](../fields/unit/utub/).
- Items: [Name (`unam`)](../fields/item/unam/),
  [Tooltip - Basic (`utip`)](../fields/item/utip/), and
  [Tooltip - Extended (`utub`)](../fields/item/utub/).

The tooltip fields describe text on training, shop or inventory UI, depending on the object.
Changing the wording does not change damage, cost or other gameplay values. See
[tooltips and art](../../editor/objects/#tooltips-and-art) for formatting and placeholder previews.

### Change a model or icon

Use the link for the object's tab:

- **Units:** [Model File (`umdl`)](../fields/unit/umdl/) and
  [Icon - Game Interface (`uico`)](../fields/unit/uico/).
- **Items:** [Model Used (`ifil`)](../fields/item/ifil/) for the ground model and
  [Interface Icon (`iico`)](../fields/item/iico/) for inventory and shop UI.
- **Doodads:** [Model File (`dfil`)](../fields/doodad/dfil/).
- **Destructables:** [Model File (`bfil`)](../fields/destructable/bfil/).

A model field and an icon field serve different views. A file path also needs an available
asset; use the [artwork troubleshooting checks](../../editor/troubleshooting/#a-model-or-icon-is-missing)
if a preview is missing. Keep terrain and placement work in the World Editor.

### Make a model larger or smaller

Choose the scaling field for the object's tab:

- **Units:** [Scaling Value (`usca`)](../fields/unit/usca/).
- **Items:** [Scaling Value (`isca`)](../fields/item/isca/) for the ground model.
- **Doodads:** [Default Scale (`ddes`)](../fields/doodad/ddes/) for the starting size of a placed copy.

A value of `1` means the model's normal size. For units, review
[Selection Scale (`ussc`)](../fields/unit/ussc/) and [Collision Size (`ucol`)](../fields/unit/ucol/)
separately: model appearance, the selection circle and collision are distinct settings.
For doodads, also check the scale of the placed instance in the World Editor.

### Tint a model

Edit the red, green and blue channels for the selected object type:

- **Units:** [Tinting Color 1 (Red) (`uclr`)](../fields/unit/uclr/),
  [Tinting Color 2 (Green) (`uclg`)](../fields/unit/uclg/), and
  [Tinting Color 3 (Blue) (`uclb`)](../fields/unit/uclb/).
- **Items:** [Tinting Color 1 (Red) (`iclr`)](../fields/item/iclr/),
  [Tinting Color 2 (Green) (`iclg`)](../fields/item/iclg/), and
  [Tinting Color 3 (Blue) (`iclb`)](../fields/item/iclb/) for the ground model.
- **Destructables:** [Tinting Color 1 (Red) (`bvcr`)](../fields/destructable/bvcr/),
  [Tinting Color 2 (Green) (`bvcg`)](../fields/destructable/bvcg/), and
  [Tinting Color 3 (Blue) (`bvcb`)](../fields/destructable/bvcb/).
- **Doodads:** [Tinting Color 1 (Red) (`dvr1`)](../fields/doodad/dvr1/),
  [Tinting Color 2 (Green) (`dvg1`)](../fields/doodad/dvg1/), and
  [Tinting Color 3 (Blue) (`dvb1`)](../fields/doodad/dvb1/), edited per model variation.

Channels use `0–255`; `255` in all three retains the model's original colors. Tinting the
model does not recolor its icon. The [other-tabs notes](../../editor/objects/#other-tabs)
describe the current doodad variation-writing limitation; confirm the saved result in game.

## If the result differs

Reset unrelated changes and test one field at a time on the same object type. Check that
you saved and launched the intended map, and account for bonuses or trigger changes.
Use [the troubleshooting checklist](../../editor/troubleshooting/#my-changes-disappear-or-do-not-show-in-game)
when the stored value and game result disagree.

For a field not listed here, browse the [complete object-field reference](../fields/).
