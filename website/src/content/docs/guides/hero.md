---

title: "Build a hero"
description: "Create a Paladin-based hero, set its starting attributes and growth, retain its game abilities, and test learning and progression."
---

Create **Sunwell Warden**, a Paladin-based hero with custom attributes and predictable level
gains. This walkthrough keeps the Paladin’s existing game abilities and artwork.
Select a screenshot to open it at full size.

[Load game data and open a disposable map copy](../../editor/first-map/) first. Close that
map in the World Editor before saving it from Tinkerbench. The screenshots use
`hero-item-workshop.w3m`; your map name and assigned raw codes can differ.

## Copy a hero base

1. Open **Units** and search for `Hpal` or **Paladin**. Select the original whose raw code is
   `Hpal`, rather than another map’s custom copy.
2. Choose **New unit based on this**. The new object’s page opens with its own raw code.
3. Note that code. In these screenshots it is `H001`, based on `Hpal`.

<figure class="app-screenshot">
  <a href="../../screenshots/tutorial-hero-base.png">
    <img src="../../screenshots/tutorial-hero-base.png" alt="Paladin Hpal selected in Units, with its base attributes, model preview and New unit based on this button." width="1440" height="900" />
  </a>
  <figcaption>Start from the hero you want to inherit. The new custom object keeps its base’s unchanged fields.<span class="screenshot-notice">Image subject to change.</span></figcaption>
</figure>

Use a hero base for a hero. The new page should still say **Hero · Human**. Creating the
object defines a unit type; you place an instance of that type in the map later.

## Shape the hero

Use the center **Filter fields** box to find a field by its name or raw code. Change a
value, then press **Enter** or leave the input to commit it.
1. Find **Name** (`unam`) and enter **Sunwell Warden**.
2. Keep **Primary Attribute** (`upra`) as **Strength**.
3. Set the six attribute fields below. Filtering for **Starting** finds the first three;
   **per Level** finds the growth fields.

| Field                                                                | Value |
| -------------------------------------------------------------------- | ----- |
| [Starting Strength (`ustr`)](../../reference/fields/unit/ustr/)      | 24    |
| [Starting Agility (`uagi`)](../../reference/fields/unit/uagi/)       | 15    |
| [Starting Intelligence (`uint`)](../../reference/fields/unit/uint/)  | 20    |
| [Strength per Level (`ustp`)](../../reference/fields/unit/ustp/)     | 2     |
| [Agility per Level (`uagp`)](../../reference/fields/unit/uagp/)      | 1     |
| [Intelligence per Level (`uinp`)](../../reference/fields/unit/uinp/) | 3     |

Clear the field filter and enable **Changed only** to review your edits. The header’s
**STR / AGI / INT** summary should read **24 / 15 / 20**. Clear the left object search if
its earlier results hide the new object.

<figure class="app-screenshot">
  <a href="../../screenshots/tutorial-hero-attributes.png">
    <img src="../../screenshots/tutorial-hero-attributes.png" alt="Custom Sunwell Warden H001 showing starting Strength 24, Agility 15, Intelligence 20 and per-level gains 2, 1, 3 with Changed only enabled." width="1440" height="900" loading="lazy" />
  </a>
  <figcaption>Review both starting attributes and growth. The rightmost column shows the original Paladin values; gold marks the changes.<span class="screenshot-notice">Image subject to change.</span></figcaption>
</figure>

Keep the copied model and icon for this example and check that the model preview loads.
The header’s **Hit points** is a base value: heroes also receive attribute bonuses in game.
See [base life and hero attributes](../../reference/fields/unit/uhpm/) before changing life
to compensate for a difference you see while playing.

The hero can still have an inherited proper name in game. **Name** changes its unit type’s
name; [Proper Names (`upro`)](../../reference/fields/unit/upro/) controls its given names.

## Assign abilities

Turn **Changed only** off before reviewing inherited fields. Filter for `uhab` to see
**Abilities → Hero** and keep these four game abilities:

| Ability       | Raw code |
| ------------- | -------- |
| Holy Light    | `AHhb`   |
| Divine Shield | `AHds`   |
| Resurrection  | `AHre`   |
| Devotion Aura | `AHad`   |

<figure class="app-screenshot">
  <a href="../../screenshots/tutorial-hero-abilities.png">
    <img src="../../screenshots/tutorial-hero-abilities.png" alt="Sunwell Warden with the Hero ability list containing Holy Light, Divine Shield, Resurrection and Devotion Aura, each shown as an ability card." width="1440" height="900" loading="lazy" />
  </a>
  <figcaption>The inherited hero list supplies the learnable spells. Use Details to inspect a card and review the Checks panel for hotkey clashes.<span class="screenshot-notice">Image subject to change.</span></figcaption>
</figure>

The **Hero** list (`uhab`) is separate from **Normal** (`uabi`). Keep **Inventory (Hero)**
in the normal list so the hero can carry and use items, including the
[item tutorial’s potion](../item/). A hero has at most **five** hero abilities; keeping these
four leaves a free slot. Some spells require a higher hero level before they can be learned.

For a new spell later, follow the separate [spell wizard walkthrough](../custom-spell/).
This hero example uses the existing game spells throughout.

## Test progression

1. Choose **Save** in Tinkerbench. Wait for the **unsaved** indicator to disappear.
2. Reopen the saved map in the World Editor. Find **Sunwell Warden** under custom units
   and place that custom type, using its recorded raw code to distinguish it from Paladin.
3. Run a test game and inspect a level-1 copy without attribute-boosting items.
4. Compare a level-3 copy: set a second placed hero’s level in the World Editor, or gain
   experience in your test game. Two level gains should produce the values below.
5. Learn Holy Light at level 1 and its second rank when the hero reaches level 3. Check
   the learn buttons, cast button, hotkey, life and mana in game too.

| Hero level | STR | AGI | INT |
| ---------- | --- | --- | --- |
| 1          | 24  | 15  | 20  |
| 3          | 28  | 17  | 26  |

These attribute values and Holy Light ranks 1 and 2 were checked in Warcraft III
**3.0.1.24371** on a disposable test map. Test your own map’s other ability combinations
and any replacement artwork as well.

If you see the original hero or old values, check the placed unit’s raw code and the
[save/test troubleshooting steps](../../editor/troubleshooting/#my-changes-disappear-or-do-not-show-in-game).
When transferring the hero, [sharing and bundles](../../editor/sharing/) explains which
referenced objects and imported assets travel with it.
