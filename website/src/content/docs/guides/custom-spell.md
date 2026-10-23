---

title: "Create a custom spell"
description: "Copy a base ability, set its levels and give it to a hero."
---

## Pick a base ability

Open a test map in Tinkerbench. In **Abilities**, find **Holy Light** (`AHhb`) and create a custom ability from it. Name it **Sunwell Touch** and note its assigned raw code.

Pick a base with behavior close to what you want. Renaming a spell does not replace its targeting, order behavior or built-in effects.

## Set the levels

Keep this first spell a hero ability with three levels. Change the per-level fields:

| Field     | Level 1 | Level 2 | Level 3 |
| --------- | ------- | ------- | ------- |
| Cooldown  | 8       | 7       | 6       |
| Mana Cost | 50      | 65      | 80      |

Leave the spell-specific healing amount at its base values for this first test. Update the tooltip title and description to describe the spell. The [ability field reference](../../reference/fields/ability/) explains generic fields and identifies spell-specific fields by base ability.

## Give it to a hero

Create a custom Paladin in **Units**, then add your ability to its hero ability list. Save the map. Reopen it in the World Editor, place the custom hero and a damaged friendly unit, and test learning and casting the spell. Verify each level’s mana use, cooldown and tooltip.

## Add scripted behavior later

For a spell-effect trigger action, filter on your ability’s raw code:

<section data-example-language="jass">

```jass
if GetSpellAbilityId() == 'A000' then
    // Add behavior here.
endif
```

</section>

<section data-example-language="lua">

```lua
if GetSpellAbilityId() == FourCC("A000") then
    -- Add behavior here.
end
```

</section>

Replace `A000` with your actual code. These fragments belong inside a registered spell-effect action; [the trigger guide](../triggers/) shows the whole registration. A trigger adds behavior to the base spell, so account for effects it already applies.
