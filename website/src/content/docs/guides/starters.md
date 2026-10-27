---

title: "Hero and item starters"
description: "Download and import Sunwell Warden and Wayfarer Draught, inspect their fields and dependencies, and test them in your own map."
---

Start with the finished objects from [Build a hero](../hero/) and [Make an item](../item/),
then inspect or change them in your own map. These examples use existing game abilities
and artwork from your Warcraft III installation.

## Choose a starter

| Starter          | Download                                                                                | Included custom objects                           |
| ---------------- | --------------------------------------------------------------------------------------- | ------------------------------------------------- |
| Sunwell Warden   | [Hero bundle (.w3obj)](../../downloads/starters/sunwell-warden-v1.w3obj)                | One Paladin-based hero                            |
| Wayfarer Draught | [Item and merchant bundle (.w3obj)](../../downloads/starters/wayfarer-draught-v1.w3obj) | One healing potion and the merchant that sells it |

Download the `.w3obj` file; keep its extension. For the text alternative, open the
[hero share code](../../downloads/starters/sunwell-warden-v1.txt) or
[item share code](../../downloads/starters/wayfarer-draught-v1.txt) and copy its entire
`W3OBJ1:` line. The file and code versions contain the same objects.

These are object bundles. Open a [disposable destination map](../../editor/first-map/)
first; the bundles do not supply terrain, placed objects or triggers. They contain no
imported files. Unchanged fields, including models, icons and game abilities, inherit
from the named bases in your installation.

## Import into your map

1. In Tinkerbench, load your game data and open the destination map. Close that map in
   the World Editor while saving changes from Tinkerbench.
2. Choose **Import… → Bundle file → Choose file…** and select the downloaded `.w3obj`.
   For the text alternative, use **Share code**, paste the whole code and choose **Preview**.
3. Choose **Preview** for the bundle too. Expect **1 object** for the hero, or **2 objects**
   for the item pack: Wayfarer Supplies and Wayfarer Draught.
4. Record the **assigned raw codes** shown in the preview. The source codes below can
   change when the destination already uses them.
5. Choose **Import 1 object** or **Import 2 objects**, then **Save**. Importing again adds
   another copy; it does not update the first import.

See [sharing and bundles](../../editor/sharing/) for the full workflow. Importing adds
object definitions; use the World Editor to place those custom types after saving.

## Sunwell Warden

The hero bundle contains **Sunwell Warden**, source raw code **`H000`**, based on Paladin
**`Hpal`**. The illustrated guide's `H001` is a different map's assignment.

<figure class="app-screenshot">
  <a href="../../screenshots/starter-hero-import.png">
    <img src="../../screenshots/starter-hero-import.png" alt="Bundle file preview for sunwell-warden-v1.w3obj showing one Sunwell Warden unit, reassigned from H000 to H002, and the Import 1 object button." width="680" height="370" />
  </a>
  <figcaption>This destination already uses H000 and H001, so the preview assigns H002. Your assigned code can differ.<span class="screenshot-notice">Image subject to change.</span></figcaption>
</figure>

| Changed field                                                        | Value          |
| -------------------------------------------------------------------- | -------------- |
| Name (`unam`)                                                        | Sunwell Warden |
| Starting Strength / Agility / Intelligence (`ustr`, `uagi`, `uint`)  | 24 / 15 / 20   |
| Strength / Agility / Intelligence per Level (`ustp`, `uagp`, `uinp`) | 2 / 1 / 3      |

The hero retains Strength as its primary attribute, the Paladin's model/icon and proper
names, normal **Inventory (Hero)**, and the four game hero spells: Holy Light (`AHhb`),
Divine Shield (`AHds`), Resurrection (`AHre`) and Devotion Aura (`AHad`). No custom spell
definitions are bundled.

After saving, reopen the map in the World Editor and place **the imported custom hero**.
Without attribute-boosting items, expect STR/AGI/INT **24/15/20 at level 1** and
**28/17/26 at level 3**. Learn Holy Light at level 1 and its second rank at level 3.
Follow [Test progression](../hero/#test-progression) for the complete check.

## Wayfarer Draught and its merchant

The item bundle contains exactly these two custom objects:

| Object            | Source code | Base                       | Changed fields                              |
| ----------------- | ----------- | -------------------------- | ------------------------------------------- |
| Wayfarer Supplies | `n000`      | Goblin Merchant (`ngme`)   | Name (`unam`); Items Sold (`usei`) = `I000` |
| Wayfarer Draught  | `I000`      | Potion of Healing (`phea`) | The six fields below                        |

| Item field                  | Value                                              |
| --------------------------- | -------------------------------------------------- |
| Name (`unam`)               | Wayfarer Draught                                   |
| Gold Cost (`igol`)          | 125                                                |
| Number of Charges (`iuse`)  | 3                                                  |
| Stock Start Delay (`isst`)  | 0 seconds                                          |
| Tooltip - Basic (`utip`)    | Purchase Wayfarer Draught                          |
| Tooltip - Extended (`utub`) | Restores hit points when used. Contains 3 charges. |

The starter adds **Stock Start Delay = 0** to the item walkthrough's edits, allowing an
immediate test purchase instead of waiting for the base potion's initial stock timer.
The item keeps the base's **Item Healing** ability (`AIh1` in the tested installation),
icon, ground model, cooldown and inventory flags. It is actively used, is not consumed
on pickup, can be dropped and sold to a merchant, and disappears after its last charge.
The [item walkthrough](../item/) explains these flags and the difference between a tooltip
and an ability's effect.

Sharing starts from custom units. **Wayfarer Supplies is the root**, and its **Items Sold**
field references the custom potion, bringing the item along as a dependency. The shop's
stock list replaces the original Goblin Merchant list with just this potion. The merchant
otherwise inherits its base's behavior and artwork.

After saving, place **Wayfarer Supplies** as **Neutral Passive** and an inventory-equipped
hero nearby. Give the player at least **125 gold**, select the hero at the shop and buy
the potion. Check that the purchase costs 125 gold and gives **Wayfarer Draught with three
charges**. Alternatively, place the custom potion directly; the merchant is needed for
the transfer, but you do not have to place it in your finished map.

Drop and pick up the potion, then test its three healing uses on an injured hero. Wait
for the item cooldown between uses; expect **3 → 2 → 1 → consumed**. Sell a fresh copy to
an accepting merchant and check that the player receives gold. See [Test it](../item/#test-it).

## When raw codes are already taken

The importer assigns free codes and rewrites the merchant's item reference. For example,
if `I000` is occupied and the potion becomes `I001`, the imported shop's **Items Sold** must
point to **`I001`**. The existing `I000` is preserved. Inspect that field after import.

<figure class="app-screenshot">
  <a href="../../screenshots/starter-item-import.png">
    <img src="../../screenshots/starter-item-import.png" alt="Wayfarer Draught bundle preview showing Wayfarer Supplies n000 and Wayfarer Draught brought along as an item dependency, with I000 reassigned to I001 and Import 2 objects." width="680" height="410" loading="lazy" />
  </a>
  <figcaption>The merchant brings its custom potion along. The item receives a free ID, and the shop's stock reference follows it.<span class="screenshot-notice">Image subject to change.</span></figcaption>
</figure>

Use the IDs from **your preview** when placing objects or writing scripts. These examples
contain no scripts; references embedded in your own handwritten code or text still need
manual updates. See [raw code collisions](../../editor/sharing/#raw-code-collisions).

## Checked results

Both bundles and the item share code were imported into a separate map with occupied
source IDs. The hero became `H002`; the merchants' stock references followed their potions
to `I001` and `I002`, and the destination's existing objects were preserved after saving.

In Warcraft III **3.0.1.24371**, the imported hero's level-1/3 attributes and Holy Light
ranks passed. Both imported merchants sold the correct custom potion for **125 gold**.
Three-charge pickup, dropping/reacquisition, all three healing uses, final-charge
consumption and selling also passed, with the inherited healing cooldown left in place.
Recheck these behaviors in your own map, especially if you change the bases or abilities.

If an object is missing or unchanged, check its assigned raw code and the
[save/test troubleshooting steps](../../editor/troubleshooting/#my-changes-disappear-or-do-not-show-in-game).
The [source definitions](https://github.com/gabaudette/wc3-toolkit/blob/main/object-editor/crates/wc3-map/examples/starter_examples.rs)
and [refresh instructions](https://github.com/gabaudette/wc3-toolkit/blob/main/website/README.md)
live in the repository.
