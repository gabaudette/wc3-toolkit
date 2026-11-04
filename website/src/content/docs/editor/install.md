---

title: "Install Tinkerbench"
description: "Install Tinkerbench on Windows or Linux with Tinkerbench Setup, which also finds your Warcraft III install."
---

You need a legal Warcraft III: Reforged installation. Tinkerbench reads the game’s object tables and art from that installation.

## Download

Get **Tinkerbench Setup** from the [GitHub releases page](https://github.com/gabaudette/wc3-toolkit/releases):

| System              | File                                          |
| ------------------- | --------------------------------------------- |
| Windows 10 or 11    | `Tinkerbench-Setup-<version>-windows-x64.exe` |
| Linux (64-bit, x86) | `Tinkerbench-Setup-<version>-linux-x64`       |

**Windows:** the setup is not signed, so Windows may show **Windows protected your PC**. For a
file you downloaded from this project's releases, choose **More info**, then **Run anyway**.

**Linux:** a downloaded file cannot run until you allow it. In your file manager, open the
file's **Properties**, then **Permissions**, and turn on **Allow executing file as program**;
or run `chmod +x Tinkerbench-Setup-*-linux-x64` in a terminal. Then open it.

## Run the setup

The setup installs Tinkerbench for your user only, so it needs no administrator rights. Its
pages, in order:
1. **Welcome** and **License**: Tinkerbench is free and open source, under the MIT License;
   the setup also lists the open-source software it includes.
2. **Install folder**: where Tinkerbench goes, by default `%LOCALAPPDATA%\Programs\Tinkerbench`
   on Windows and `~/.local/share/tinkerbench` on Linux. Tick **Add a shortcut on the desktop**
   if you want one; Tinkerbench is added to the Start menu or your applications menu either way.
3. **Requirements**: Tinkerbench draws its window with Microsoft Edge WebView2 on Windows and
   WebKitGTK on Linux. The setup checks they are installed.
   - On Linux, **Install it** installs WebKitGTK with your distribution's package manager
     (apt, dnf, pacman or zypper) after asking for your password; the setup also shows the
     command, to run in a terminal instead.
   - On Windows, **Install it** downloads the WebView2 Runtime from Microsoft and installs it;
     the page also links to Microsoft's download. Windows 10 and 11 usually have it already.
4. **Warcraft III**: the installs the setup found, with where each came from and its game
   version. Pick yours, or choose **Browse…** and select the folder that contains `Data/` and
   `.build.info`. The setup reads the game data to check it, and offers **Use HD (Reforged) art
   values and icons** when the HD art is installed.
5. **Maps folder**: where **Open a map…** starts, usually `Documents/Warcraft III/Maps` (inside
   the game's prefix on Linux with Proton or Wine). Leave it empty to choose each time.
6. **Test maps**: where **Save and build test copy** writes your Lua modules, either a folder
   next to each map (`My Map Lua` beside `My Map.w3x`, recommended) or one folder for all maps.
7. **Install**, then **Finish**. Leave **Start Tinkerbench now** ticked to open it.

<figure class="app-screenshot">
  <a href="../../screenshots/setup-game.png">
    <img src="../../screenshots/setup-game.png" alt="Tinkerbench Setup on its Warcraft III page: a Steam (Proton) install found with its game version, its folder, a confirmation that the game data reads, and the HD art checkbox." width="820" height="600" loading="lazy" />
  </a>
  <figcaption>The setup lists the installs it finds and checks the game data before you continue.<span class="screenshot-notice">Image subject to change.</span></figcaption>
</figure>

Tinkerbench then opens straight on your game data, with your art choice and maps folder; the
first launch skips its own setup screen. Find it later in the Start menu, your applications
menu or, on Linux, as the `tinkerbench` command.

## Update or remove

Run Tinkerbench Setup again. When Tinkerbench is installed, its first page offers **Update or
repair it, keeping your choices** or **Remove it**. Removal deletes what the setup installed;
tick **Also remove Tinkerbench's settings and saved data** to remove those too. Your maps are
never touched. On Windows, Tinkerbench is also listed under **Settings → Apps → Installed apps**,
where **Uninstall** opens the same removal.

<figure class="app-screenshot">
  <a href="../../screenshots/setup-welcome.png">
    <img src="../../screenshots/setup-welcome.png" alt="Tinkerbench Setup's Welcome page, with the list of steps on the left: Welcome, License, Install folder, Requirements, Warcraft III, Maps folder, Test maps, Install and Finish." width="820" height="600" loading="lazy" />
  </a>
  <figcaption>Each step of the setup is listed on the left; run it again later to update or remove Tinkerbench.<span class="screenshot-notice">Image subject to change.</span></figcaption>
</figure>

## Other Linux packages

The releases page also has packages for people who prefer them. They install only Tinkerbench,
so connect the game on the first launch, as described below.

| Package     | Install                                                          |
| ----------- | ---------------------------------------------------------------- |
| `.deb`      | `sudo apt install ./<file>.deb` (Debian, Ubuntu, Mint)           |
| `.AppImage` | Make it executable with `chmod +x <file>.AppImage`, then run it. |

## Connect the game

Tinkerbench Setup connects the game for you. Tinkerbench's own setup screen does it after a
`.deb` or AppImage install, or when the game has moved:
1. Start Tinkerbench.
2. Under **1. Warcraft III folder**, check the folder Tinkerbench found. When it finds several
   installations, choose yours under **Found on this computer**. Otherwise choose **Browse…**
   and select the installation directory containing `Data/` and `.build.info`. A Maps folder
   is not a game installation.
3. Choose whether to enable **Use HD (Reforged) art values and icons**, then select **Load game data**.
4. Wait for the unit count to confirm the game data loaded and the **2. Map** section to appear.
5. Choose **Open a map…** to open a copy of a `.w3m` or `.w3x` map, or **Browse game data only**
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
