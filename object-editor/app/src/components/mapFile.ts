import { getCurrentWindow } from "@tauri-apps/api/window";
import { useCallback, useEffect, useRef, useState } from "react";
import { api, report } from "../api";
import { choosePath, confirm } from "../dialogs";
import { attempt, settle } from "../result";
import type { GameSummary, MapSummary } from "../types";
import { MAP_FILTERS, MAP_PATH_KEY, remembered } from "../ui";

async function confirmDiscard(map: MapSummary): Promise<boolean> {
  const answer = await confirm(`${map.name} has unsaved changes. Discard them?`, {
    title: "Unsaved changes",
    kind: "warning",
    okLabel: "Discard changes",
    cancelLabel: "Keep editing",
  });

  return answer.ok && answer.value;
}

function useCloseGuard(map: MapSummary | null, onError: (message: string) => void): void {
  const mapRef = useRef(map);

  mapRef.current = map;

  useEffect(() => {
    const appWindow = getCurrentWindow();
    const unlisten = appWindow.onCloseRequested(async (event) => {
      const current = mapRef.current;
      if (!current?.unsaved) {
        return;
      }

      event.preventDefault();
      if (!(await confirmDiscard(current))) {
        return;
      }

      const closed = await attempt(() => appWindow.destroy());
      if (!closed.ok) {
        report("error", "closing the window", closed.error);
        onError(closed.error);
      }
    });

    return () => {
      void attempt(() => unlisten).then((listening) => {
        if (listening.ok) {
          listening.value();
          return;
        }

        report("error", "close prompt", listening.error);
      });
    };
  }, [onError]);
}

type MapFileOptions = {
  game: GameSummary;
  initialMap: MapSummary | null;
  onOpened: () => void;
  onError: (message: string | null) => void;
};

export function useMapFile({ game, initialMap, onOpened, onError }: MapFileOptions) {
  const [map, setMap] = useState(initialMap);
  const [saving, setSaving] = useState(false);
  const [changes, setChanges] = useState(0);

  useCloseGuard(map, onError);

  const refreshMap = useCallback(() => {
    setChanges((count) => count + 1);
    void settle(api.mapStatus(), setMap, onError);
  }, [onError]);

  const save = useCallback(async () => {
    setSaving(true);
    const saved = await api.saveMap();
    setSaving(false);
    if (!saved.ok) {
      onError(saved.error);
      return;
    }

    setMap(saved.value);
    onError(null);
  }, [onError]);

  async function mayLeave(): Promise<boolean> {
    return !map?.unsaved || confirmDiscard(map);
  }

  async function openOther() {
    if (!(await mayLeave())) {
      return;
    }

    const picked = await choosePath({
      title: "Open a map",
      filters: MAP_FILTERS,
      ...(game.mapsDir && { defaultPath: game.mapsDir }),
    });

    if (!picked.ok) {
      onError(picked.error);
      return;
    }

    const path = picked.value;

    if (!path) {
      return;
    }

    const opened = await api.openMap(path);

    if (!opened.ok) {
      onError(opened.error);
      return;
    }

    setMap(opened.value);
    remembered.set(MAP_PATH_KEY, path);
    onOpened();
  }

  return { map, saving, changes, refreshMap, save, mayLeave, openOther };
}
