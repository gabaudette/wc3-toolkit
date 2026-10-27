import { useEffect, useState } from "react";
import { api } from "../api";
import { choosePath } from "../dialogs";
import { whenOk } from "../result";
import type { GameSummary, MapSummary } from "../types";
import { GAME_DIR_KEY, HD_KEY, MAP_FILTERS, MAP_PATH_KEY, remembered } from "../ui";
import { AppVersion } from "./AppVersion";
import { Button } from "./Button";
import { Checkbox } from "./Checkbox";

type SetupScreenProps = {
  onReady: (game: GameSummary, map: MapSummary | null) => void;
};

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

    void whenOk(api.detectGameDir(), (found) => {
      if (found) {
        setDir(found);
      }
    });
  }, [dir]);

  async function browseGame() {
    const picked = await choosePath({ directory: true, title: "Warcraft III folder" });

    if (!picked.ok) {
      setError(picked.error);
      return;
    }

    if (picked.value) {
      setDir(picked.value);
    }
  }

  async function openGame() {
    setBusy(true);
    setError(null);
    const summary = await api.openGame(dir, hd);

    setBusy(false);
    if (!summary.ok) {
      setError(summary.error);
      return;
    }

    remembered.set(GAME_DIR_KEY, dir);
    remembered.set(HD_KEY, String(hd));
    setGame(summary.value);
  }

  async function openMap(path: string, summary: GameSummary) {
    setBusy(true);
    setError(null);
    const map = await api.openMap(path);

    if (!map.ok) {
      setError(map.error);
      setBusy(false);
      return;
    }

    remembered.set(MAP_PATH_KEY, path);
    onReady(summary, map.value);
  }

  async function browseMap(summary: GameSummary) {
    const lastMap = remembered.get(MAP_PATH_KEY);
    const picked = await choosePath({
      title: "Open a map",
      filters: MAP_FILTERS,
      ...(lastMap && { defaultPath: lastMap }),
      ...(!lastMap && summary.mapsDir && { defaultPath: summary.mapsDir }),
    });

    if (!picked.ok) {
      setError(picked.error);
      return;
    }

    if (picked.value) {
      await openMap(picked.value, summary);
    }
  }

  const lastMap = remembered.get(MAP_PATH_KEY);

  return (
    <main className="mx-auto flex h-full max-w-xl flex-col justify-center px-6">
      <div className="frame flex flex-col gap-8 bg-surface-1 p-8">
        <header>
          <h1 className="flex items-baseline gap-3 font-display text-4xl font-bold text-gold">
            Tinkerbench
            <AppVersion />
          </h1>
          <p className="mt-1 text-muted">
            An Object Editor for Warcraft III Reforged maps. It reads the game data from your own
            Warcraft III install; nothing is uploaded anywhere.
          </p>
        </header>

        <section className="flex flex-col gap-3">
          <h2 className="font-display text-lg font-bold text-gold">1. Warcraft III folder</h2>
          <div className="flex gap-2">
            <input
              value={dir}
              onChange={(e) => setDir(e.target.value)}
              placeholder="C:\Program Files (x86)\Warcraft III"
              aria-label="Warcraft III folder"
              className="min-w-0 flex-1 rounded-[3px] border border-field bg-well px-3 py-1.5 font-mono text-xs outline-none focus:border-focus"
            />
            <Button onPress={browseGame}>Browse…</Button>
          </div>
          <Checkbox isSelected={hd} onChange={setHd}>
            Use HD (Reforged) art values and icons
          </Checkbox>
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
            <h2 className="font-display text-lg font-bold text-gold">2. Map</h2>
            <p className="text-muted">
              Close the map in the World Editor before saving here, or the editor will overwrite
              your changes on its next save.
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
            className="rounded-[3px] border border-danger/40 bg-danger/10 px-3 py-2 text-danger"
          >
            {error}
          </p>
        )}
      </div>
    </main>
  );
}
