import { open } from "@tauri-apps/plugin-dialog";
import { useEffect, useState } from "react";
import { api, errorMessage } from "../api";
import type { GameSummary, MapSummary } from "../types";
import { GAME_DIR_KEY, HD_KEY, MAP_PATH_KEY, remembered } from "../ui";
import { Button } from "./Button";

type SetupScreenProps = {
  onReady: (game: GameSummary, map: MapSummary | null) => void;
};

const MAP_FILTERS = [{ name: "Warcraft III maps", extensions: ["w3m", "w3x"] }];

export function SetupScreen({ onReady }: SetupScreenProps) {
  const [dir, setDir] = useState(remembered.get(GAME_DIR_KEY) ?? "");
  const [hd, setHd] = useState(remembered.get(HD_KEY) === "true");
  const [game, setGame] = useState<GameSummary | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (dir) {
      return;
    }

    api
      .detectGameDir()
      .then((found) => {
        if (found) {
          setDir(found);
        }
      })
      .catch(() => {
        // Detection is only a convenience; the user can browse instead.
      });
  }, [dir]);

  async function browseGame() {
    const picked = await open({ directory: true, title: "Warcraft III folder" });
    if (typeof picked === "string") {
      setDir(picked);
    }
  }

  async function openGame() {
    setBusy(true);
    setError(null);
    try {
      const summary = await api.openGame(dir, hd);
      remembered.set(GAME_DIR_KEY, dir);
      remembered.set(HD_KEY, String(hd));
      setGame(summary);
    } catch (e) {
      setError(errorMessage(e));
    } finally {
      setBusy(false);
    }
  }

  async function openMap(path: string, summary: GameSummary) {
    setBusy(true);
    setError(null);
    try {
      const map = await api.openMap(path);
      remembered.set(MAP_PATH_KEY, path);
      onReady(summary, map);
    } catch (e) {
      setError(errorMessage(e));
      setBusy(false);
    }
  }

  async function browseMap(summary: GameSummary) {
    const lastMap = remembered.get(MAP_PATH_KEY);
    const picked = await open({
      title: "Open a map",
      filters: MAP_FILTERS,
      ...(lastMap && { defaultPath: lastMap }),
      ...(!lastMap && summary.mapsDir && { defaultPath: summary.mapsDir }),
    });
    if (typeof picked === "string") {
      await openMap(picked, summary);
    }
  }

  const lastMap = remembered.get(MAP_PATH_KEY);

  return (
    <main className="mx-auto flex h-full max-w-xl flex-col justify-center gap-8 px-6">
      <header>
        <h1 className="text-2xl font-semibold">WC3 Object Editor</h1>
        <p className="mt-1 text-muted">
          Reads the game data from your own Warcraft III install. Nothing is uploaded anywhere.
        </p>
      </header>

      <section className="flex flex-col gap-3">
        <h2 className="text-sm font-semibold uppercase tracking-wide text-faint">
          1. Warcraft III folder
        </h2>
        <div className="flex gap-2">
          <input
            value={dir}
            onChange={(e) => setDir(e.target.value)}
            placeholder="C:\Program Files (x86)\Warcraft III"
            aria-label="Warcraft III folder"
            className="min-w-0 flex-1 rounded-md border border-line bg-surface-1 px-3 py-1.5 font-mono text-xs outline-none focus:border-gold"
          />
          <Button onPress={browseGame}>Browse…</Button>
        </div>
        <label className="flex items-center gap-2 text-muted">
          <input type="checkbox" checked={hd} onChange={(e) => setHd(e.target.checked)} />
          Use HD (Reforged) art values and icons
        </label>
        {!game && (
          <div>
            <Button primary disabled={busy || !dir} onPress={openGame}>
              Load game data
            </Button>
          </div>
        )}
        {game && (
          <p className="text-muted">
            Loaded {game.units} units from <span className="font-mono text-xs">{game.dir}</span>
          </p>
        )}
      </section>

      {game && (
        <section className="flex flex-col gap-3">
          <h2 className="text-sm font-semibold uppercase tracking-wide text-faint">2. Map</h2>
          <p className="text-muted">
            Close the map in the World Editor before saving here, or the editor will overwrite your
            changes on its next save.
          </p>
          <div className="flex flex-wrap gap-2">
            {lastMap && (
              <Button primary disabled={busy} onPress={() => openMap(lastMap, game)}>
                Reopen {lastMap.split(/[\\/]/).pop()}
              </Button>
            )}
            <Button disabled={busy} onPress={() => browseMap(game)}>
              Open a map…
            </Button>
            <Button disabled={busy} onPress={() => onReady(game, null)}>
              Browse game data only
            </Button>
          </div>
        </section>
      )}

      {error && (
        <p
          role="alert"
          className="rounded-md border border-danger/40 bg-danger/10 px-3 py-2 text-danger"
        >
          {error}
        </p>
      )}
    </main>
  );
}
