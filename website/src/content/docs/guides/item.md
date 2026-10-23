---

title: "Make an item"
description: "Create an item with an ability and verify charges and inventory behavior."
---

## Start from an item

Open **Items**, find an item close to your goal and create a custom object. A potion is a useful base for a consumable; an equipment item is a better base for a passive bonus.

Give it a distinct name and note its raw code. Set its description, tooltip, icon and cost. Check the item’s ability list: the displayed text does not create the effect.

## Choose its inventory behavior

Review **Number of Charges**, **Perishable**, **Can Be Dropped**, **Can Be Sold To Merchants** and **Use Automatically When Acquired**. Use the [item field reference](../../reference/fields/item/) for exact meanings.

For a new effect, create a custom item ability first and add it to the item. Keep its item ability settings consistent with the base item.

## Test it

Save and reopen the map in the World Editor. Place the item next to a hero with an inventory and test picking it up, using it, dropping it and selling it. For a charged consumable, use the last charge and check that the item disappears only when intended.

The sharing UI starts from custom units. To transfer an item through that flow, it must be a discovered dependency of a selected unit; do not assume the current UI supports selecting any item as a standalone share root.
