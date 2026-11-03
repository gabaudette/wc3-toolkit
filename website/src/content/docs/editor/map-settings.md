---

title: "Map settings and constants"
description: "Edit map properties, loading screens, environment, players, forces and gameplay constants."
---

Open a map and choose **Map** to edit the map's settings. The left column separates its
description, screens, options, environment, players, forces and gameplay constants.
Close the same map in the World Editor before saving from Tinkerbench.

## Describe the map and its screens

Under **Description**, edit **Name**, **Author**, **Suggested players** and **Description**.
The name has a colored text preview. Suggested players is descriptive text; it does not
add player slots.

**Loading screen and prologue** has separate **Loading screen** and **Prologue** sections,
each with **Title**, **Subtitle**, **Text** and **Model**. An imported loading-screen model
can use a path such as `war3mapImported\LoadingScreen.mdx`; an empty loading-screen model
keeps the default. Use [Assets](../assets/) to import the file. Most standalone maps leave
the prologue empty.

**Options** lists the map's option checkboxes. Review them individually before changing a
map you did not create.

## Set fog, weather and water

Open **Fog, weather and water**:
- **Terrain fog** enables **Style**, **Color**, **Starts at height**, **Ends at height** and
  **Density** settings. The styles are **Linear**, **Exponential** and **Exponential, steeper**.
- **Weather** selects a weather effect, or **None**.
- **Sound environment** sets a reverb name, such as `cave` or `forest`; empty keeps the default.
- **Tint the water** enables the tint chosen with **Water color**.

<figure class="app-screenshot">
  <a href="../../screenshots/map-environment.png">
    <img src="../../screenshots/map-environment.png" alt="Map tab&#x27;s Fog, weather and water section with terrain fog enabled, its Style menu open, Ashenvale rain selected and water tint enabled." width="1440" height="900" loading="lazy" />
  </a>
  <figcaption>Fog style, weather and water tint are map settings; this capture shows the editor controls, not the in-game environment.<span class="screenshot-notice">Image subject to change.</span></figcaption>
</figure>

## Configure existing players and forces

**Players** edits the slots already in the map: **Name**, **Controller**, **Race** and
**Fixed start**. **Controller** offers **User**, **Computer**, **Neutral** and **Rescuable**;
**Race** offers **Selectable** and the four races. Use **Fixed player settings (players
cannot change race or team)** when those choices should be fixed.

Adding or removing players and placing their start locations remain World Editor tasks.
Changing a unit's owner is a [Placed units](../placed-units/) edit.

Under **Forces**, enable **Use custom forces (teams set here instead of in the lobby)**
to define teams. **Add force** creates a force; **Remove force** removes one when at least
two exist. Set its name and tick players to move them into that force: each player belongs
to one force. Choose **Allied**, **Allied victory**, **Share vision**, **Share unit control**
and **Share advanced unit control** as needed.

<figure class="app-screenshot">
  <a href="../../screenshots/map-forces.png">
    <img src="../../screenshots/map-forces.png" alt="Forces section with custom forces enabled, Alliance and Reserve forces, player membership and alliance, vision and unit-control checkboxes." width="1440" height="900" loading="lazy" />
  </a>
  <figcaption>Player 1 belongs to Alliance; ticking that player under Reserve would move the membership to the other force.<span class="screenshot-notice">Image subject to change.</span></figcaption>
</figure>

## Change gameplay constants

A [gameplay constant](../../reference/glossary/#gameplay-constant) changes a map-wide rule,
such as attribute bonuses, experience or upkeep. Choose **Gameplay constants**, then use
**Search gameplay constants** to find a name, explanation or ID. Each entry explains its
purpose, unit and an example, and shows **Game:** with the installation's value and the ID.
Less common entries are folded under **Advanced** with a count.

Changing any constant enables **Use custom gameplay constants**. Only changed constants
are stored; **Changed only** shows those entries, and **Reset** restores an entry to the
game's value. Turning off **Use custom gameplay constants** drops the changes when you save.

The damage table is a grid: attack types are rows and defense types are columns. Edit the
multiplier at their intersection; **Reset** on a row restores that row. This is separate
from an individual unit's damage dice and armor amount. See
[Armor Type versus Defense Type](../../reference/common-confusions/#armor-type-versus-defense-type)
and [damage dice](../objects/#damage-and-bounty-dice).

<figure class="app-screenshot">
  <a href="../../screenshots/map-constants.png">
    <img src="../../screenshots/map-constants.png" alt="Gameplay constants filtered to the damage bonus table, with attack rows, defense columns and an edited Normal-against-Small multiplier with Reset." width="1440" height="900" loading="lazy" />
  </a>
  <figcaption>The damage table places attack types on rows and defense types on columns; the changed Normal-against-Small cell is 1.2.<span class="screenshot-notice">Image subject to change.</span></figcaption>
</figure>

## Save and verify the result

Choose **Save** or **Ctrl S**. Tinkerbench saves properties and changed constants into the
map. It also rewrites the matching player, force and environment script for Lua or JASS
maps, but only if it could reproduce the World Editor's original parts exactly when loading.
If the tab says it cannot rewrite those parts, save in Tinkerbench, then open and save in
the World Editor to apply those settings to the game script.
See [script-rewrite troubleshooting](../troubleshooting/#a-tab-cannot-rewrite-the-map-script).

The [recorded play test](../verification/#map-settings-constants-and-dice) confirmed saved
loading text, player/force state, visible rain, environment native arguments and representative
damage constants on one Lua map. A [World Editor fallback](../verification/#world-editor-fallback)
also applied edited position, fog and water arguments. Custom loading art, prologue, visible
water tint, audible sound effects, JASS and multiple clients remain **not yet confirmed in
game**. Individual constant explanations retain their own limits. Test your map separately;
restart the game and use a new test-copy filename if constants or loading text appear stale.
