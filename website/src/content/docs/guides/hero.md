---

title: "Build a hero"
description: "Create a custom hero, choose attributes and assign hero abilities."
---

## Copy a hero base

In **Units**, find Paladin (`Hpal`) and create a custom unit from it. Use a hero base for a hero: changing a regular unit’s name does not turn it into one. Note the new raw code.

## Shape the hero

Change the name, model or icon if needed, and set the primary attribute and starting Strength, Agility and Intelligence. Set the attribute gain per level separately.

Heroes add attribute bonuses on top of base life and mana. If your in-game maximum is higher than the base field, inspect Strength and Intelligence before assuming the save failed. The [unit field reference](../../reference/fields/unit/) describes these relationships.

## Assign abilities

Choose a set of hero abilities, including any [custom spell](../custom-spell/) you created. The hero ability list is separate from the normal ability list. Check the spell’s hero flag and level count, its button positions and hotkeys.

A hero copied from an existing base inherits all unchanged fields. Review requirements, build/train time, food and cost before adding it to a production building.

## Test progression

Save and reopen the map in the World Editor. Place the hero, run the map, learn abilities and test gaining levels. Check starting attributes, attribute growth, life, mana and the learn buttons. Test any imported model in game as well as in the preview.

When sharing the hero, [a bundle](../../editor/sharing/) can carry discovered imported assets with the referenced custom objects.
