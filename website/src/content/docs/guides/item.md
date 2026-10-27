---

title: "Make an item"
description: "Create a three-charge healing potion, retain its game ability, check its tooltip and inventory flags, and test using, dropping and selling it."
---

Make **Wayfarer Draught**, a consumable copied from Potion of Healing. It keeps the game’s
healing ability and has **three charges**. Select a screenshot to open it at full size.

To inspect the finished example, [download the Wayfarer Draught starter](../starters/#wayfarer-draught-and-its-merchant).
It includes a merchant whose stock reference brings the custom potion through sharing.

[Open a disposable test map](../../editor/first-map/) with a hero that has **Inventory (Hero)**,
such as the [hero walkthrough’s Paladin](../hero/). Close the map in the World Editor before
saving from Tinkerbench. Map names and assigned raw codes in screenshots are examples.

## Start from an item

1. Open **Items** and search for **Potion of Healing** or `phea`.
2. Select the original `phea` item and choose **New item based on this**.
3. Note the new raw code in the header. This capture’s copy is `I000`, based on `phea`.

<figure class="app-screenshot">
  <a href="../../screenshots/tutorial-item-base.png">
    <img src="../../screenshots/tutorial-item-base.png" alt="Potion of Healing phea selected in Items, with its icon, one charge, gold cost 150, treasure-chest model preview and New item based on this button." width="1440" height="900" />
  </a>
  <figcaption>The potion supplies the inventory icon, ground model and healing ability. The chest preview is its ground model; the bottle is its icon.<span class="screenshot-notice">Image subject to change.</span></figcaption>
</figure>

Use the center **Filter fields** box to find each field by name or raw code. Enter a value,
then press **Enter** or leave the input to apply it.

| Field                       | Enter                                              |
| --------------------------- | -------------------------------------------------- |
| Name (`unam`)               | Wayfarer Draught                                   |
| Gold Cost (`igol`)          | 125                                                |
| Tooltip - Basic (`utip`)    | Purchase Wayfarer Draught                          |
| Tooltip - Extended (`utub`) | Restores hit points when used. Contains 3 charges. |

<figure class="app-screenshot">
  <a href="../../screenshots/tutorial-item-tooltip.png">
    <img src="../../screenshots/tutorial-item-tooltip.png" alt="Wayfarer Draught I000 with the extended tooltip Restores hit points when used. Contains 3 charges. and the matching in-game text preview." width="1440" height="900" loading="lazy" />
  </a>
  <figcaption>Check the wording in the preview after leaving the text input. This copy has its own name and tooltips while retaining the base item’s artwork.<span class="screenshot-notice">Image subject to change.</span></figcaption>
</figure>

## Choose its inventory behavior

Set **Number of Charges** to **3** and keep these inherited flags:

| Field                                                                         | Value |
| ----------------------------------------------------------------------------- | ----- |
| [Actively Used (`iusa`)](../../reference/fields/item/iusa/)                   | True  |
| [Number of Charges (`iuse`)](../../reference/fields/item/iuse/)               | 3     |
| [Use Automatically When Acquired (`ipow`)](../../reference/fields/item/ipow/) | False |
| [Perishable (`iper`)](../../reference/fields/item/iper/)                      | True  |
| [Can Be Dropped (`idro`)](../../reference/fields/item/idro/)                  | True  |
| [Can Be Sold To Merchants (`ipaw`)](../../reference/fields/item/ipaw/)        | True  |

Clear the filter and scroll within **Stats** to check the flags. Keep **Changed only** off
while reviewing settings inherited from the base.

<figure class="app-screenshot">
  <a href="../../screenshots/tutorial-item-behavior.png">
    <img src="../../screenshots/tutorial-item-behavior.png" alt="Wayfarer Draught with three charges in its summary and Stats showing automatic use False, Perishable True, Can Be Dropped True and Can Be Sold To Merchants True." width="1440" height="900" loading="lazy" />
  </a>
  <figcaption>Picking this item up should keep it in the inventory. Each successful use spends a charge; Perishable removes it after the last one.<span class="screenshot-notice">Image subject to change.</span></figcaption>
</figure>

**Can Be Sold To Merchants** governs selling the carried item back to a shop.
**Can Be Sold By Merchants** governs shops offering it. Place the custom item directly for
this test; configuring a shop’s stock is a separate step.

### Keep the healing ability

Filter for `iabi` and inspect **Abilities**. Keep the inherited **Item Healing** ability
(`AIh1` in this capture). The [Abilities field](../../reference/fields/item/iabi/) determines
what the item does. Changing its tooltip, cost or charge count does not create a different
healing effect.

<figure class="app-screenshot">
  <a href="../../screenshots/tutorial-item-ability.png">
    <img src="../../screenshots/tutorial-item-ability.png" alt="Wayfarer Draught I000 showing the retained Item Healing AIh1 ability card in the Abilities field, with its three-charge summary and saved map in the top bar." width="1440" height="900" loading="lazy" />
  </a>
  <figcaption>The item keeps the game ability that performs the healing. Save the map after reviewing the item’s text, flags and ability.<span class="screenshot-notice">Image subject to change.</span></figcaption>
</figure>

## Test it

1. Choose **Save** in Tinkerbench and wait for the **unsaved** indicator to disappear.
2. Reopen the map in the World Editor. Place the custom `I000` item beside a hero with an
   inventory, using your own assigned code if different. Place a shop that accepts items
   if you also want to test selling.
3. Start a test game and pick up the item. It should enter the inventory with **3 charges**
   and wait for an explicit use.
4. Drop it and pick it up again. It should still have **3 charges**.
5. Make sure the hero is missing hit points, then use the item once. Check that it heals
   and the count becomes **2**.
6. Wait for the item cooldown before each further use and keep the hero injured enough to
   heal. Check **2 → 1 → consumed**: the third successful use removes the item from the inventory.
7. Sell a fresh copy to a merchant that accepts items and confirm the item leaves the
   inventory and the player receives gold.

Pickup, dropping/reacquisition, three healing uses, final-charge consumption and selling
were checked in Warcraft III **3.0.1.24371** on a disposable test map. A full-health hero or
an active item cooldown can prevent a use; allow for both when checking the charge count.

If the custom item is missing or unchanged, check its placed raw code and the
[save/test troubleshooting steps](../../editor/troubleshooting/#my-changes-disappear-or-do-not-show-in-game).

The current sharing UI starts from custom units and includes their discovered item
dependencies. Follow [sharing and bundles](../../editor/sharing/) when transferring items
through that workflow; an item placed beside a hero is not automatically a unit dependency.
