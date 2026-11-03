---

title: "In-game verification"
description: "Recorded editor play tests, known Interface failures and the limits of the checked results."
---

These results describe specific disposable maps played in Warcraft III. They help distinguish
an editor preview, a saved value, a call reaching the game and a visible gameplay result.
They do not establish that every map, model or generated script works.

## Test environment and method

The play test on **November 2, 2026** used Warcraft III **3.0.1.24371 x86_64** and
Tinkerbench revision **`7ad04d88ba517beb71f30af6cf30c1c4a513d8e1`**. The game and World
Editor ran through Steam GE-Proton11-7 on Linux Mint, with an NVIDIA RTX 3060. Tests used
a small World Editor Lua map, one human and one computer player, and locally imported
Knight artwork. The saved map-data-set selector was `0`; this was a single-player custom
game, without a second network client.

Edits were saved through Tinkerbench, reopened and read back. The generated Interface
modules were built into a `[DEV]` copy through the [Lua harness](../lua-harness/).
Project-authored test modules checked live unit, player, timer and board state. Wrappers
recorded arguments to environment and multiboard natives while forwarding the original
calls. Melee starting units, AI and automatic victory were disabled for the fixture.
Native arguments alone do not establish that an effect rendered correctly.

The [recorded runtime checks](https://github.com/gabaudette/wc3-toolkit/blob/main/docs/verification/editor-runtime-2026-11-02.txt)
retain the test's own result strings. Game maps, extracted scripts and standalone artwork
are not distributed with this record. Custom IDs below identify this fixture; another map
can assign different IDs.

## Placed units and loot

The normal save path passed the following live checks before combat could change the values:

| Setup saved in Tinkerbench                                      | Observed in game                                    |
| --------------------------------------------------------------- | --------------------------------------------------- |
| Neutral Hostile Footman at `(300, -600)`, facing 45°            | Position, owner and facing matched                  |
| Paladin at `(-400, -600)`, facing 135°, owned by player 0       | Position, owner and facing matched                  |
| Paladin level 3, Strength/Agility/Intelligence 30/20/25         | Level and explicit attributes matched               |
| Paladin life 75%, mana 50                                       | Life fraction and absolute mana matched             |
| Starting slots 1–2: `phea`, `rde1`                              | Both items occupied the saved slots                 |
| Add imported-model unit `h000`; add then remove another Footman | The new unit existed; the removed placement did not |
| Two own item sets, each with one 100% entry                     | Killing the host dropped both `phea` and `rde1`     |
| Two hosts using the same item table, with `rat9` at 100%        | Each host dropped `rat9`                            |
| One host using the flattened cascade below                      | Killing it produced one of the stored entries       |

The **Creep rewards** cascade used a 0.2% Legendary tier (`ratf`), a 5% Epic tier
(`rat9`/`ratc`, equal weights), then a 100% Common tier (`phea`/`rde1`, weights 3:1).
The calculated chances were 0.2%, 2.495%, 2.495%, 71.1075% and 23.7025%.
Readback after saving confirmed whole percentages **3, 2, 71 and 24** for `rat9`, `ratc`,
`phea` and `rde1`; the 0% `ratf` entry was omitted. The tier definitions remained in
`<map> Loot.json`. A few kills confirm the drop path, not the probability distribution.

See [Placed units](../placed-units/) for the editing workflow.

## Map settings, constants and dice

The saved human/computer controllers, Human/Orc races, common force membership, alliance,
shared vision and shared/advanced control matched live player state. The lobby showed
the saved force name and races. This does not verify synchronization between clients.

Linear fog, `RAhr` rain, the `forest` sound environment and water RGB `(105, 161, 193)`
reached their game natives. Rain was visible. The fixture had no water surface for a visual
tint check, and the sound environment was not assessed by listening.

The Normal-against-Small damage multiplier was changed to **1.2**. A scripted 100-damage
hit using the game's Normal attack type removed **120 life** from a Small-defense target
with zero armor. The native attack-type constant for this test was `ATTACK_TYPE_MELEE`.

Sunwell Warden (`H000`, based on `Hpal`) used base damage 10, two six-sided dice and
22 Strength. With primary-attribute damage explicitly set to **2**, its native base was
54 and five actual attacks each fell within **56–66**. Ten bounty awards with base 25
and two six-sided dice were **28, 36, 34, 28, 31, 33, 32, 29, 33, 29**: all within
27–37. These small samples do not establish the roll distribution or empirical average.

After resetting the primary-attribute override, a fresh game process with a new map
filename showed **34–44**, matching the guide's level-1 preview and native base 32.
Reusing the old filename within the earlier game process had shown the previous value.
Restart the game and use a distinct test-copy filename when checking changed constants
or loading screens if results appear stale.

The final loading screen displayed the saved title, subtitle and descriptive text. The selected imported Knight model
did not replace the stock loading art; custom loading-model behavior remains unconfirmed.
Prologue and campaign behavior were not exercised.

## World Editor fallback

A separate copy used valid custom expressions in the generated unit and environment
script, preventing Tinkerbench from reproducing those sections exactly. Both tabs warned
that their script parts could not be rewritten. Saving changed the map data while leaving
the original script byte for byte unchanged.

Opening and saving that copy in the World Editor regenerated its script. Tinkerbench then
reopened it with both rewrite checks passing. The resulting game copy applied the new
Paladin position `(-200, -600)`, fog distances 1000–2000 and water RGB `(70, 110, 200)`,
while retaining its owner, facing, level, attributes, life, mana, items and player setup.
The final dedicated fallback run passed all **11** runtime assertions, including the
reset damage preview check. The fixture's missing second starting location was generated
when the World Editor asked during its save.

<figure class="app-screenshot">
  <a href="../../screenshots/verification-fallback.png">
    <img src="../../screenshots/verification-fallback.png" alt="Warcraft III showing eleven passing fallback checks, rain and fog, with the level-1 Sunwell Warden selected and 34–44 damage." width="1452" height="938" loading="lazy" />
  </a>
  <figcaption>The fresh game run applies the regenerated map script and shows the reset damage range.<span class="screenshot-notice">Image subject to change.</span></figcaption>
</figure>

This establishes the fallback on this Lua map. JASS rewrite and fallback behavior still
need a separate play test. Follow [script-rewrite troubleshooting](../troubleshooting/#a-tab-cannot-rewrite-the-map-script)
when either tab reports the warning.

## Imported artwork and text

The same map retained its imported Knight model, texture and icon after saving and
reopening. The unit made from that model (`h000`, Sunwell Knight) appeared in game with
its textured model, portrait and imported icon. This checks one import set; it does not
validate every format, texture dependency or rename/replace/remove operation in game.

<figure class="app-screenshot">
  <a href="../../screenshots/verification-imported-model.png">
    <img src="../../screenshots/verification-imported-model.png" alt="Sunwell Knight selected in the running Warcraft III test map, with its textured mounted model, portrait and imported Knight icon." width="1452" height="938" loading="lazy" />
  </a>
  <figcaption>The imported model and icon are visible in the saved map's game copy.<span class="screenshot-notice">Image subject to change.</span></figcaption>
</figure>

Sunwell Light (`A000`, based on `AHhb`) was corrected to a valid colored tooltip with
`<A000,DataA1>` and a line break; its healing data was 250. The game accepted the custom
ability, but the tooltip was not inspected on the paused fixture hero. Its formatting and
placeholder rendering remain **not yet confirmed in game**. Desktop warning checks alone
do not verify how malformed text renders in Warcraft III.

## Interface results and known failures

The designs were Difficulty/Details dialogs with **H**, **A** and **B** actions; a
Player/Kills/Deaths Scoreboard with a fixed row and one row per playing player; a Kills
leaderboard with player rows and a Waves row; and the repeating Next wave timer with a
three-second duration. Callback functions were `OnHardMode(player)` and `SpawnWave()`.

An earlier run created the designs during map initialization: messages and the timer title
were lost, and the leaderboard was empty. The updated harness runs startup callbacks in
registration order on the first game timer tick after loading. **Rebuild your development
copy with the updated Tinkerbench or command-line tool** to apply that change.

The startup follow-up on **November 3, 2026**, using the same game build and environment,
confirmed the opening Difficulty message, the initially hidden Details message on first
show, the **Next wave** timer title, and all three leaderboard rows. Waves at 10 sorted
above the human's 6; adding 10 moved the human's 16 to the top. These updates ran directly
in a startup callback registered after the generated designs, with no extra construction
or consumer delay. The [startup runtime checks](https://github.com/gabaudette/wc3-toolkit/blob/main/docs/verification/interface-startup-runtime.txt)
record 17 passing assertions and identify the tested harness source.

<figure class="app-screenshot">
  <a href="../../screenshots/verification-interface-startup.png">
    <img src="../../screenshots/verification-interface-startup.png" alt="Warcraft III showing the Difficulty dialog's Choose a difficulty message, the Next wave timer title and a three-row leaderboard with scores 16, 10 and 0." width="1452" height="938" loading="lazy" />
  </a>
  <figcaption>The rebuilt test copy retains the initial message, timer title and leaderboard rows.<span class="screenshot-notice">Image subject to change.</span></figcaption>
</figure>

The multiboard was inspected with the leaderboard hidden. Its fixed row showed blue
**Slain** text, the human's Kills cell showed green **5**, and both displayed the imported
Knight icon. Player names retained their colors; the computer row and remaining values
were present. Show, hide and minimize operations also passed.

<figure class="app-screenshot">
  <a href="../../screenshots/verification-multiboard.png">
    <img src="../../screenshots/verification-multiboard.png" alt="The running map's Sunwell defense multiboard with fixed and player rows, blue Slain text, a green score of 5 and imported Knight icons." width="1452" height="938" loading="lazy" />
  </a>
  <figcaption>Hiding the leaderboard exposes the multiboard's text, colors and imported icons for inspection.<span class="screenshot-notice">Image subject to change.</span></figcaption>
</figure>

The three-second repeating timer invoked `SpawnWave` at least twice without arguments.
Hide kept the countdown running; show restored the window; stop paused and hid it;
`Start(30)` restarted it with the requested duration and showed it.

Clicking Hard invoked `OnHardMode` with player 0 and the expected triggering player.
Clicking Back navigated from Details to Difficulty. This follow-up used mouse activation;
it did not independently reconfirm keyboard hotkeys. The previous play test exercised
hotkeys and navigation, but imported callbacks still need their own game check.

**A reused dialog can still lose its message.** Returning from Details showed Difficulty's
buttons without its original message. The startup fix does not resolve repeated display.
Design transfer was checked through save/readback and generated names, rather than full
independent execution of imported designs.

## Remaining checks

Remaining checks cover repeated and imported dialogs, keyboard activation after rebuilding,
tooltip rendering, a suitable custom loading model, prologue, visible water tint, audible sound,
JASS maps, additional asset workflows and multiple clients. Keep those limits explicit
when using the [Interface](../interface/), [Map](../map-settings/), [Assets](../assets/)
and [Objects](../objects/) guides.

An [AI assistant prompt](../ai-assistant/) can help prepare the next test. The checks here
cover only the concrete modules and objects exercised; they do not verify arbitrary
assistant responses.
