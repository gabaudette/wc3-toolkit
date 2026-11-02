---

title: "Placed units and loot"
description: "Move and edit placed units, give heroes starting items, and build item drop tables and rarity cascades."
---

Use **Placed units** to edit a particular [placed unit](../../reference/glossary/#placed-unit),
such as one Footman standing on your map. **Units** edits the reusable type shared by every
Footman. Open a map, then select a unit on the terrain view or in the list; **Open unit type**
takes you to its object fields.

## Move, turn and add units

Drag empty ground to pan and scroll to zoom, or use **Zoom in**, **Zoom out** and **Whole
map**. Pick a unit and drag it to move it; drag its handle to turn it. The list and map
selection follow each other. Search by name, ID or player, and narrow the view with
**Heroes** or **With drops**.

| Key on the picked unit | Action                                     |
| ---------------------- | ------------------------------------------ |
| Arrow keys             | Move the unit; hold Shift to move farther. |
| `[` / `]`              | Turn the unit.                             |
| Delete                 | Request removal.                           |

Choose **Add unit**, pick its type and set **Owner of new units**, then click the map to
place copies. **Stop placing** or Escape ends placement. To remove a picked unit, choose
**Remove from map** and confirm with **Remove unit**.

Under **Placement**, edit **Owner**, **X**, **Y**, **Facing (degrees)**, **Hit points (%)**
and **Mana**. Heroes also have **Hero level**, **Strength**, **Agility** and **Intelligence**.
Leave hit points, mana or attributes empty to keep the unit's normal values. A random unit
must be changed in the World Editor.

<figure class="app-screenshot">
  <a href="../../screenshots/placed-map.png">
    <img src="../../screenshots/placed-map.png" alt="Paladin selected on the Placed units terrain view, with its rotation handle, matching list selection, owner, position, facing and hero settings." width="1440" height="900" loading="lazy" />
  </a>
  <figcaption>Picking the level-3 Paladin links its terrain marker, list entry and placement fields; the handle turns that placement.<span class="screenshot-notice">Image subject to change.</span></figcaption>
</figure>

## Give a hero starting items

Select a placed hero and use its **Inventory** slots. Choose an item by icon and name;
**Search items**, **This map**, **Game**, class chips and **Item level** help narrow the
choices. The picker includes custom map items. Use the slot's clear control to empty it.
Only heroes receive starting items here; other units can still have drops.

<figure class="app-screenshot">
  <a href="../../screenshots/placed-inventory.png">
    <img src="../../screenshots/placed-inventory.png" alt="Inventory slot 1 item picker with item icons, names, raw codes, This map and Game filters, class chips and Any level." width="1062" height="601" loading="lazy" />
  </a>
  <figcaption>A hero&#x27;s inventory slot opens the item picker, including map/game, class and level filters.<span class="screenshot-notice">Image subject to change.</span></figcaption>
</figure>

## Choose a unit's drops

Under **Drops when it dies**, choose:

| Choice                 | What you edit                                                                    |
| ---------------------- | -------------------------------------------------------------------------------- |
| **Nothing**            | No item drops for this placement.                                                |
| **From an item table** | A shared map table selected under **Item table**; **Edit item tables** opens it. |
| **Its own item sets**  | Drop sets belonging only to this placement.                                      |

An item set rolls at most one item. Use **Add item**, set each chance in percent, and
leave part of the total below 100% for a chance of nothing. **Add item set** adds another
independent set, allowing several items to drop. A warning appears when chances exceed
100%, because later entries can become unreachable.

For drops, the picker also offers **Random item class**, **Random item level** and **Use a
random item**. This stores a class/level choice rather than a particular item ID.

## Share an item table

Switch to **Item tables** on the left. An [item table](../../reference/glossary/#item-table)
lets several placed units share the same drops. **Plain table** creates the World Editor's
item sets; name it, add sets and items, then choose it from each unit's **From an item table**
option. Editing the table changes the shared definition. **Remove table** warns when units
use it; removing it leaves those units dropping nothing.

## Build a cascade of rarity tiers

Choose **Cascade** under **Item tables** for a [cascade](../../reference/glossary/#cascade).
Put the rarest tier first. Each tier rolls only if all earlier tiers missed; on a hit it
selects one item by weight and stops. Use **Add tier**, **Add item** and the tier's move
controls to adjust the order. A hit on an empty tier drops nothing.

For example, a first tier at 2% followed by a tier at 5% gives the second tier a 4.9%
overall chance: `98% × 5%`. Two equally weighted items in that second tier each have a
calculated chance of 2.45%. Weight divides a tier's share; it is not a percent by itself.

<figure class="app-screenshot">
  <a href="../../screenshots/placed-cascade.png">
    <img src="../../screenshots/placed-cascade.png" alt="Creep rewards cascade with Legendary, Epic and Common tiers, weights, calculated-to-whole-percent chances and a warning that one entry rounds to zero." width="1440" height="900" loading="lazy" />
  </a>
  <figcaption>The Legendary entry&#x27;s calculated 0.20% becomes 0%; the warning below the tiers explains that it will never drop from the stored table.<span class="screenshot-notice">Image subject to change.</span></figcaption>
</figure>

Each item shows its exact calculated chance followed by the whole percent the World Editor
gets. Tinkerbench converts the cascade to one item set with whole percentages; do not read
the calculated fractional chance as the probability stored in the map. If an entry rounds
to 0%, raise its tier's chance or weight, or give it its own tier.

The tier definitions are saved beside the map in `<map> Loot.json`; the flattened item set
goes into the map. Keep that JSON file with the map to continue editing the tiers.

## Save and test

Choose **Save** or **Ctrl S**. Tinkerbench writes the placement and table data, and updates
the World Editor's unit-creation and drop script when it can reproduce those parts exactly.
This applies to Lua and JASS maps. If the tab says it cannot rewrite the unit script, save
in Tinkerbench, then open and save the map in the World Editor before playing.
See [script-rewrite troubleshooting](../troubleshooting/#a-tab-cannot-rewrite-the-map-script).

The controls and save path described here were checked against the source. Placement,
starting inventory and drop outcomes in this guide are **not yet confirmed in game**.
Test a separate copy: check unit ownership, position and hero inventory, then kill units
with each drop setup. A few drops cannot establish a probability; compare the stored
whole percentages as well as observed outcomes.
