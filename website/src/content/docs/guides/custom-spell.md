---

title: "Create a custom spell"
description: "Use the spell wizard to set three levels, choose a buff and button, give the spell to a hero, and build a generated-Lua test copy."
---

Make **Sunwell Bolt**, a three-level hero spell based on Storm Bolt, using **New spell**.
Then follow the optional [generated-Lua example](#add-scripted-behavior-later) for an effect
provided by a Lua template.

These screenshots follow the current source build. If your app has no **New spell** button,
follow the [development setup](../../contributing/development/). Select a screenshot to open
it at full size. Map names, folders and assigned raw codes in the screenshots are examples.

## Before you start

1. Make a disposable copy of a test map, [load game data and open that copy](../../editor/first-map/).
   You need an open map; browsing game data alone does not create spells.
2. Prepare a [custom Paladin](../hero/) with room for one more hero ability. The screenshots
   use a Paladin-based hero named `test_hero` with four existing hero abilities.
3. Close the map in the World Editor before saving changes from Tinkerbench. Reopen it there
   afterwards to place your hero and prepare the in-game test.

The game-ability path below edits object data. The generated-Lua path later in the guide
also needs a map whose **Script Language** is **Lua**.

## Pick a base ability

1. Open **Abilities → New spell**.
2. Under **Based on a game ability**, choose **Damage one target**.
3. Choose **Storm Bolt** (`AHtb`). It is already a three-level hero spell that damages and
   stuns one target. The explanation beside each suggestion helps you choose a suitable
   base; **Another ability…** lets you choose a different one.

<figure class="app-screenshot">
  <a href="../../screenshots/spell-base.png">
    <img src="../../screenshots/spell-base.png" alt="New spell: start from, with Storm Bolt, Firebolt and Death Coil suggestions and an explanation of each base ability." width="640" height="488" />
  </a>
  <figcaption>Choose a base for its behavior. Storm Bolt brings both damage and stun; a new name or icon does not replace those effects.<span class="screenshot-notice">Image subject to change.</span></figcaption>
</figure>

Choosing the base creates a draft custom ability. The original game ability stays unchanged.
Before finishing the wizard, **Cancel** discards its draft, its newly created buff and its
unit assignments. Use the wizard’s buttons to navigate between steps.

## Set the levels

In **New spell: values**, set **Name** to **Sunwell Bolt** and keep **Levels** at **3**.
Click each value in the table, type its replacement, then press **Enter** or **Tab** to
commit it.

| Field                          | Level 1 | Level 2 | Level 3 |
| ------------------------------ | ------- | ------- | ------- |
| Cooldown (`acdn`)              | 8       | 7       | 6       |
| Mana Cost (`amcs`)             | 50      | 65      | 80      |
| Damage (`Htb1`, for this base) | 100     | 175     | 250     |

For this first example, keep **Targets Allowed**, **Cast Range** and both **Duration** rows
at their copied values. Storm Bolt’s normal-target and hero-target durations are separate.
The [ability field reference](../../reference/fields/ability/) explains these fields; a
spell-specific field belongs to its particular base ability.

<figure class="app-screenshot">
  <a href="../../screenshots/spell-values.png">
    <img src="../../screenshots/spell-values.png" alt="Sunwell Bolt in the values step with three levels: cooldown 8, 7, 6; mana cost 50, 65, 80; damage 100, 175, 250." width="1024" height="765" loading="lazy" />
  </a>
  <figcaption>Gold cells show values changed from the base. Keep all three columns consistent with the spell you intend to test.<span class="screenshot-notice">Image subject to change.</span></figcaption>
</figure>

### Check the live tooltip

Scroll down to **Tooltip - Normal - Extended**. The wizard points copied placeholders at
the new ability’s own values. Check that the preview shows **100**, **175** and **250**
damage at the three levels. In this capture, the new ability is `A001`; yours may receive
a different code.

<figure class="app-screenshot">
  <a href="../../screenshots/spell-tooltip.png">
    <img src="../../screenshots/spell-tooltip.png" alt="Sunwell Bolt extended tooltips with placeholders referring to A001 and previews resolving to 100, 175 and 250 damage." width="1024" height="765" loading="lazy" />
  </a>
  <figcaption>The preview resolves the new spell’s placeholders. Editing tooltip text changes the explanation; the table controls the spell’s values.<span class="screenshot-notice">Image subject to change.</span></figcaption>
</figure>

Choose **Next: buff** when the values and text agree.

## Choose a buff and button

### Give the spell its own buff

Choose **Its own buff (recommended)**. For Storm Bolt, this copies its Stunned buff and
names the copy after your spell. Set **Tooltip text** to `Stunned by Sunwell Bolt.`
The wizard wires the chosen buff into the spell at every level.

<figure class="app-screenshot">
  <a href="../../screenshots/spell-buff.png">
    <img src="../../screenshots/spell-buff.png" alt="New spell: buff showing the custom Sunwell Bolt buff B000, its tooltip title, edited tooltip text and preview." width="640" height="765" loading="lazy" />
  </a>
  <figcaption>The spell’s own buff has a separate name and tooltip. Its icon and art can be edited in the Buffs tab afterwards.<span class="screenshot-notice">Image subject to change.</span></figcaption>
</figure>

You can also keep the base buff or pick **Another buff**. A different buff’s label or art
does not add a new spell mechanic. Some bases have no buff to copy; that step explains when
there is nothing to set.

### Set the cast and learn buttons

Choose **Next: button**. For this example:
- Keep the Storm Bolt icon. The icon picker can search game icons and the map’s imported icons.
- Set the cast **Hotkey** to **B** and check the learn-menu hotkey too.
- Put the **Command card** button in the second slot of the second row.
- Put the **Learn menu** button in the first slot of the second row.

<figure class="app-screenshot">
  <a href="../../screenshots/spell-button.png">
    <img src="../../screenshots/spell-button.png" alt="New spell: button with the Storm Bolt icon, hotkey B for cast and learn, and separate command-card and learn-menu grids." width="640" height="672" loading="lazy" />
  </a>
  <figcaption>A hero spell has a cast button and a learn button. The chosen slots leave the test Paladin’s existing abilities and basic orders visible.<span class="screenshot-notice">Image subject to change.</span></figcaption>
</figure>

These positions fit the hero used here. The next step checks the actual unit’s other
abilities; resolve any hotkey or position warnings for your own hero.

## Give it to a hero

1. Choose **Next: units** and select your custom Paladin. Search by name or raw code if needed.
2. Review its **Command card** and **Learn menu** previews, then choose **Give the spell**.
   Selecting the unit alone does not assign the spell.
3. Choose **Create spell** to finish. The new ability’s normal page opens.
4. Choose **Save** in the top bar to write the map.

<figure class="app-screenshot">
  <a href="../../screenshots/spell-units.png">
    <img src="../../screenshots/spell-units.png" alt="Sunwell Bolt assigned to test_hero H000, with command-card and learn-menu previews and the option to take the spell away." width="1024" height="576" loading="lazy" />
  </a>
  <figcaption>The hero learns this spell level by level. The assignment stays with the finished ability; the top-bar Save writes it to disk.<span class="screenshot-notice">Image subject to change.</span></figcaption>
</figure>

A hero can learn at most **five** hero abilities. If it already has five, make room or use
another hero before assigning this one. The test hero here starts with four, so Sunwell
Bolt becomes its fifth. On an ability’s normal page, **Give to units** opens this assignment
flow again. You can continue editing the finished spell there too.

### Test all three levels

Reopen the saved map in the World Editor, place the custom hero and suitable enemy targets,
and run the map in Warcraft III. Advance the hero far enough to learn each rank, then check:
- The spell appears in the learn menu and on the cast command card, with the intended hotkey.
- Each rank’s tooltip, mana use and cooldown match the table above.
- Targeting, damage and stun behave as intended, including against hero targets.
- The custom buff’s tooltip appears on an affected target when appropriate.

The editor preview helps catch data and text mistakes; use this in-game check to verify the
result. If the saved spell is missing or different, follow the
[save and test troubleshooting checks](../../editor/troubleshooting/#my-changes-disappear-or-do-not-show-in-game).

## Add scripted behavior later

For effects offered by a generated template, use the **Triggered spells** group near the
bottom of **New spell: what does it do?**. These choices create a Channel-based ability
(`ANcl`) and a Lua module that runs the effect.

Use another disposable map copy for this example, with a hero that has a free ability slot.
In the World Editor, set **Scenario → Map Options → Script Language** to **Lua**, save and
close the map there, then open it in Tinkerbench. This path generates Lua code regardless
of the documentation site’s Lua/JASS reading preference.

### Configure Attribute Strike

1. Open **Abilities → New spell**, scroll to **Triggered spells** and choose **Attribute Strike**.
2. Keep **Attribute** at **Strength** and the default values below. The effect step also
   shows cooldown, mana, range, targets and a live tooltip.

| Generated effect value            | Level 1 | Level 2 | Level 3 |
| --------------------------------- | ------- | ------- | ------- |
| Damage                            | 25      | 50      | 75      |
| Damage per point of the attribute | 1       | 2       | 3       |

<figure class="app-screenshot">
  <a href="../../screenshots/spell-triggered-effect.png">
    <img src="../../screenshots/spell-triggered-effect.png" alt="Attribute Strike effect step with Strength selected, damage 25, 50, 75, attribute multipliers 1, 2, 3 and a generated tooltip preview." width="1024" height="765" loading="lazy" />
  </a>
  <figcaption>The template’s per-level values describe the Lua effect. Review its generated tooltip before moving to the button and unit steps.<span class="screenshot-notice">Image subject to change.</span></figcaption>
</figure>

Choose **Next: button** and use a free command-card and learn-menu position. On the test
Paladin, the same slots as Sunwell Bolt work on this separate copy; use **G** as the hotkey
to avoid its existing Resurrection hotkey. Then choose **Next: units** and **Give the spell**
to the hero.

### Write the module and build the test copy

Choose **Next: Lua code**. Review the output folder and module. Keep one Lua folder per
map: the test build includes every module in that folder. The wizard uses the new spell’s
actual raw code, so you do not need to replace IDs in the generated code.

<figure class="app-screenshot">
  <a href="../../screenshots/spell-lua-code.png">
    <img src="../../screenshots/spell-lua-code.png" alt="New spell: Lua code showing the per-map module folder and Attribute Strike module with FourCC A001 and three levels of effect values." width="1024" height="534" loading="lazy" />
  </a>
  <figcaption>Review the module and choose a folder on your own machine. Create spell writes the Lua file; the following step builds it into a playable copy.<span class="screenshot-notice">Image subject to change.</span></figcaption>
</figure>

1. Choose **Create spell** to write the module. In this capture it is `attribute_strike_a001.lua`;
   the filename reflects the spell’s name and assigned code.
2. In **New spell: try it**, choose **Save and build test copy**. This saves the source map
   and creates a separate `_dev` map beside it, with the Lua modules included.
3. Confirm the **Built** message names the copy and the module. Choose **Done**.

<figure class="app-screenshot">
  <a href="../../screenshots/spell-test-copy.png">
    <img src="../../screenshots/spell-test-copy.png" alt="New spell: try it reporting a built triggered-workshop_dev.w3m containing attribute_strike_a001.lua, with Save and build test copy and Done controls." width="640" height="437" loading="lazy" />
  </a>
  <figcaption>The build result identifies the map to play and the module included in it. Keep working on the source map and rebuild the copy after changes.<span class="screenshot-notice">Image subject to change.</span></figcaption>
</figure>

Start the generated copy as a custom game in Warcraft III, learn the spell and cast it on
a valid enemy. Check all three ranks and the effect’s attribute scaling. The source map’s
Channel ability needs its generated module in the playable copy for this effect to run.

Rebuild after changing the map or Lua files. The World Editor rewrites the map’s script
when saving, so keep the module in its external folder. The [Lua harness guide](../../editor/lua-harness/)
explains this workflow; the [trigger guide](../triggers/) covers writing behavior beyond
the offered templates.

:::note[What was checked]
Both wizard paths were followed on disposable maps. Saved fields, buff links and hero
assignments were read back, and the generated Lua module and built test copy were checked.
In-game casting was not verified for this walkthrough; run the checks above on your map.
:::
