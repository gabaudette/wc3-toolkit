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

1. Start Tinkerbench.
2. Under **Warcraft III folder**, choose **Browse…**.
3. Select the installation directory containing `Data/`. A Maps folder is not a game installation.
4. Choose whether to use **HD (Reforged) art values and icons**, then select **Load game data**.
5. Wait for the unit count to confirm the game data loaded and the **2. Map** section to appear.
6. Choose **Open a map…** to open a copy of a `.w3m` or `.w3x` map, or **Browse game data only**
   to inspect objects read-only.

<figure class="app-screenshot">
  <a href="../../screenshots/load-game.png">
    <img src="../../screenshots/load-game.png" alt="Tinkerbench’s startup screen showing the Warcraft III folder, Browse, the HD art option and Load game data." width="600" height="420" loading="lazy" />
  </a>
  <figcaption>The first startup step connects your local game installation. Select the screenshot to see it at full size.</figcaption>
</figure>

The [visual tour](../visual-tour/#2-open-a-map) shows the next screen and its map-opening options.

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

Check that the selected folder contains `Data/`, that the installation is complete, and that your user can read it. Copy the editor’s error message when [reporting a problem](https://github.com/gabaudette/wc3-toolkit/issues).

Continue with [your first map](../first-map/).
