---

title: "Install Tinkerbench"
description: "Install the editor on Windows or Linux and choose your Warcraft III folder."
---

You need a legal Warcraft III: Reforged installation. Tinkerbench reads the game’s object tables and art from that installation.

## Download

Get a package from the [GitHub releases page](https://github.com/gabaudette/wc3-toolkit/releases).

| System                    | Package      | Install                                                          |
| ------------------------- | ------------ | ---------------------------------------------------------------- |
| Windows                   | `-setup.exe` | Run the installer. It installs for your user.                    |
| Debian, Ubuntu, Mint      | `.deb`       | `sudo apt install ./<file>.deb`                                  |
| Other Linux distributions | `.AppImage`  | Make it executable with `chmod +x <file>.AppImage`, then run it. |

The Windows installer is unsigned. Windows may show **Windows protected your PC**; for a package you downloaded from this project’s releases, choose **More info**, then **Run anyway**.

## Connect the game

On the first launch, connect your installation:
1. Start Tinkerbench.
2. Under **1. Warcraft III folder**, choose **Browse…**.
3. Select the installation directory containing `Data/` and `.build.info`. A Maps folder is not a game installation.
4. Choose whether to enable **Use HD (Reforged) art values and icons**, then select **Load game data**.
5. Wait for the unit count to confirm the game data loaded and the **2. Map** section to appear.
6. Choose **Open a map…** to open a copy of a `.w3m` or `.w3x` map, or **Browse game data only**
   to inspect objects read-only.

<figure class="app-screenshot">
  <a href="../../screenshots/load-game.png">
    <img src="../../screenshots/load-game.png" alt="First-launch Tinkerbench setup with the detected Warcraft III folder, HD art checkbox, Load game data and Prompt for an AI assistant buttons." width="600" height="480" loading="lazy" />
  </a>
  <figcaption>On the first launch, connect the game installation before opening a map.<span class="screenshot-notice">Image subject to change.</span></figcaption>
</figure>

The [visual tour](../visual-tour/#2-open-a-map) shows the next screen and its map-opening options.

Later launches load the last game folder and art choice, then open the last map directly.
While game data is loading, press Escape or choose **Stop and choose** to pick another map.
If there is no remembered map, setup stays open after loading the game data.

To always choose at startup, clear **At start, load this game data and reopen the last map**
under **2. Map** on the setup screen. From the editor, **Ctrl K → Change game folder…**
returns to setup, where you can change that setting, the folder or the art choice.
If automatic loading fails, setup displays the reason; see
[automatic-opening troubleshooting](../troubleshooting/#automatic-opening-returns-to-setup).

On Linux with Proton, choose the Warcraft III directory inside the game’s Proton prefix. The directory name and location depend on how you installed it.

## Run from source

Install Rust using rustup, Bun, a C/C++ compiler and CMake. On Linux, install [Tauri’s system prerequisites](https://v2.tauri.app/start/prerequisites/).

From a checkout:

```sh
cd object-editor/app
bun install
bun run tauri dev
```

The workspace pins its Rust version in `object-editor/rust-toolchain.toml`.

## If the game will not load

Start with [game-folder detection](../troubleshooting/#the-game-folder-is-not-detected) or
[game-data loading checks](../troubleshooting/#the-game-data-will-not-load). The troubleshooting
hub explains the required `Data/` and `.build.info` markers and what to include if loading
still fails.

Continue with [your first map](../first-map/).
