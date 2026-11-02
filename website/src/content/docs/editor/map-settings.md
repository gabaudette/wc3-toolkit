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

<!-- Screenshot placeholder: Map tab, Fog, weather and water section; Terrain fog enabled,
Style open, weather chosen and Tint the water enabled. Use an app-screenshot figure,
descriptive alt text and an Image subject to change. notice. -->

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

<!-- Screenshot placeholder: Map tab, Forces section; custom forces enabled, two named
forces with player membership and alliance/vision/control checkboxes visible. Use an
app-screenshot figure, descriptive alt text and an Image subject to change. notice. -->

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

<!-- Screenshot placeholder: Map tab, Gameplay constants section; damage table grid with
one edited multiplier, its Reset control and Use custom gameplay constants visible.
Use an app-screenshot figure, descriptive alt text and an Image subject to change. notice. -->

## Save and verify the result

Choose **Save** or **Ctrl S**. Tinkerbench saves properties and changed constants into the
map. It also rewrites the matching player, force and environment script for Lua or JASS
maps, but only if it could reproduce the World Editor's original parts exactly when loading.
If the tab says it cannot rewrite those parts, save in Tinkerbench, then open and save in
the World Editor to apply those settings to the game script.
See [script-rewrite troubleshooting](../troubleshooting/#a-tab-cannot-rewrite-the-map-script).

These editing and save instructions were checked against the source. The settings used
here are **not yet confirmed in game**; individual constant explanations also mark
unconfirmed behavior. Test your loading screen, lobby teams, environment and changed rules
in a separate map copy before relying on them.
