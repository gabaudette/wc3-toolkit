import { getCurrentWindow } from "@tauri-apps/api/window";
import { ask, open } from "@tauri-apps/plugin-dialog";
import { useCallback, useEffect, useRef, useState } from "react";
import { api, errorMessage } from "../api";
import type { Field, GameSummary, HotkeyClash, MapSummary, UnitDetail, UnitItem } from "../types";
import { MAP_PATH_KEY, remembered } from "../ui";
import { Button } from "./Button";
import { UnitList } from "./UnitList";
import { UnitPage } from "./UnitPage";

type EditorScreenProps = {
  game: GameSummary;
  initialMap: MapSummary | null;
};

const MAP_FILTERS = [{ name: "Warcraft III maps", extensions: ["w3m", "w3x"] }];

/** Asks before throwing away unsaved changes. Resolves to `true` to discard them. */
function confirmDiscard(map: MapSummary): Promise<boolean> {
  return ask(`${map.name} has unsaved changes. Discard them?`, {
    title: "Unsaved changes",
    kind: "warning",
    okLabel: "Discard changes",
    cancelLabel: "Keep editing",
  });
}

function replaceField(detail: UnitDetail, updated: Field): UnitDetail {
  return {
    ...detail,
    sections: detail.sections.map((section) => ({
      ...section,
      fields: section.fields.map((f) => {
        if (f.id === updated.id) {
          return updated;
        }

        return f;
      }),
    })),
  };
}

export function EditorScreen({ game, initialMap }: EditorScreenProps) {
  const [map, setMap] = useState(initialMap);
  const [units, setUnits] = useState<UnitItem[]>([]);
  const [selected, setSelected] = useState<string | null>(null);
  const [detail, setDetail] = useState<UnitDetail | null>(null);
  const [clashes, setClashes] = useState<HotkeyClash[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const editable = map !== null;

  // The close handler is registered once; it reads the latest map from here.
  const mapRef = useRef(map);
  mapRef.current = map;

  const fail = useCallback((e: unknown) => setError(errorMessage(e)), []);

  const refreshList = useCallback(() => {
    api.listUnits().then(setUnits).catch(fail);
  }, [fail]);

  const refreshDetail = useCallback(
    (id: string) => {
      api.unitDetail(id).then(setDetail).catch(fail);
      api.hotkeyClashes(id).then(setClashes).catch(fail);
    },
    [fail],
  );

  useEffect(() => {
    refreshList();
  }, [refreshList]);

  useEffect(() => {
    if (selected) {
      refreshDetail(selected);
    }
  }, [selected, refreshDetail]);

  async function applyField(edit: Promise<Field>) {
    try {
      const updated = await edit;
      setDetail((current) => current && replaceField(current, updated));
      setMap(await api.mapStatus());
      setError(null);
      refreshList();
      if (selected) {
        refreshDetail(selected);
      }
    } catch (e) {
      fail(e);
    }
  }

  async function createCopy() {
    if (!detail) {
      return;
    }

    try {
      const created = await api.createUnit(detail.item.base);
      setMap(await api.mapStatus());
      refreshList();
      setSelected(created.id);
    } catch (e) {
      fail(e);
    }
  }

  const save = useCallback(async () => {
    setSaving(true);
    try {
      setMap(await api.saveMap());
      setError(null);
    } catch (e) {
      fail(e);
    } finally {
      setSaving(false);
    }
  }, [fail]);

  async function openOtherMap() {
    if (map?.unsaved && !(await confirmDiscard(map))) {
      return;
    }

    const picked = await open({
      title: "Open a map",
      filters: MAP_FILTERS,
      ...(game.mapsDir && { defaultPath: game.mapsDir }),
    });
    if (typeof picked !== "string") {
      return;
    }

    try {
      setMap(await api.openMap(picked));
      remembered.set(MAP_PATH_KEY, picked);
      setSelected(null);
      setDetail(null);
      refreshList();
    } catch (e) {
      fail(e);
    }
  }

  useEffect(() => {
    const appWindow = getCurrentWindow();
    const unlisten = appWindow.onCloseRequested(async (event) => {
      const current = mapRef.current;
      if (!current?.unsaved) {
        return;
      }

      event.preventDefault();
      if (await confirmDiscard(current)) {
        await appWindow.destroy();
      }
    });

    return () => {
      unlisten.then((stop) => stop()).catch(fail);
    };
  }, [fail]);

  useEffect(() => {
    function onKey(e: KeyboardEvent) {
      if ((e.ctrlKey || e.metaKey) && e.key === "s" && map?.unsaved) {
        e.preventDefault();
        save();
      }
    }

    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [map, save]);

  return (
    <div className="flex h-full flex-col">
      <header className="flex items-center gap-3 border-b border-line bg-surface-1 px-4 py-2">
        <h1 className="font-semibold">WC3 Object Editor</h1>
        <span className="text-faint">·</span>
        {map && (
          <span className="min-w-0 truncate font-mono text-xs text-muted" title={map.path}>
            {map.name}
            {map.unsaved && <span className="ml-2 text-gold">● unsaved</span>}
          </span>
        )}
        {!map && <span className="text-muted">Game data (read-only)</span>}
        <span className="flex-1" />
        <Button onPress={openOtherMap}>Open map…</Button>
        {map && (
          <Button primary disabled={!map.unsaved || saving} onPress={save}>
            Save
          </Button>
        )}
      </header>

      {error && (
        <div
          role="alert"
          className="flex items-center gap-3 border-b border-danger/40 bg-danger/10 px-4 py-2 text-danger"
        >
          <span className="flex-1">{error}</span>
          <Button small onPress={() => setError(null)}>
            Dismiss
          </Button>
        </div>
      )}

      <div className="flex min-h-0 flex-1">
        <UnitList units={units} selected={selected} onSelect={setSelected} onError={setError} />
        {detail && (
          <UnitPage
            key={detail.item.id}
            detail={detail}
            clashes={clashes}
            editable={editable}
            onChange={(field, value) => applyField(api.setField(detail.item.id, field, value))}
            onReset={(field) => applyField(api.resetField(detail.item.id, field))}
            onCreateCopy={createCopy}
          />
        )}
        {!detail && (
          <p className="m-auto text-muted">
            Pick a unit on the left. {game.units} game units loaded.
          </p>
        )}
      </div>
    </div>
  );
}
