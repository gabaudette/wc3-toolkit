import { getCurrentWindow } from "@tauri-apps/api/window";
import { ask, open } from "@tauri-apps/plugin-dialog";
import { SearchLg } from "@untitledui/icons";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { api, errorMessage } from "../api";
import type {
  Field,
  GameSummary,
  HotkeyClash,
  MapSummary,
  TransferObject,
  UnitDetail,
  UnitItem,
} from "../types";
import { classes, MAP_FILTERS, MAP_PATH_KEY, remembered, replaceField } from "../ui";
import { AbilitiesPane } from "./AbilitiesPane";
import { Button } from "./Button";
import { CommandPalette, type PaletteCommand } from "./CommandPalette";
import { ImportDialog } from "./ImportDialog";
import { ItemsPane } from "./ItemsPane";
import { OpenAbilityContext } from "./openAbility";
import { ShareDialog } from "./ShareDialog";
import { UNIT_LIST, UnitList } from "./UnitList";
import { UnitPage } from "./UnitPage";
import { UpgradesPane } from "./UpgradesPane";

type Tab = "units" | "abilities" | "items" | "upgrades";

const TABS: { id: Tab; label: string }[] = [
  { id: "units", label: "Units" },
  { id: "abilities", label: "Abilities" },
  { id: "items", label: "Items" },
  { id: "upgrades", label: "Upgrades" },
];

type EditorScreenProps = {
  game: GameSummary;
  initialMap: MapSummary | null;
};

/** Asks before throwing away unsaved changes. Resolves to `true` to discard them. */
function confirmDiscard(map: MapSummary): Promise<boolean> {
  return ask(`${map.name} has unsaved changes. Discard them?`, {
    title: "Unsaved changes",
    kind: "warning",
    okLabel: "Discard changes",
    cancelLabel: "Keep editing",
  });
}

export function EditorScreen({ game, initialMap }: EditorScreenProps) {
  const [map, setMap] = useState(initialMap);
  const [units, setUnits] = useState<UnitItem[]>([]);
  const [selected, setSelected] = useState<string | null>(null);
  const [detail, setDetail] = useState<UnitDetail | null>(null);
  const [clashes, setClashes] = useState<HotkeyClash[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const [dialog, setDialog] = useState<"share" | "import" | "search" | null>(null);
  const [tab, setTab] = useState<Tab>("units");
  const [selectedAbility, setSelectedAbility] = useState<string | null>(null);
  const [selectedItem, setSelectedItem] = useState<string | null>(null);
  const [selectedUpgrade, setSelectedUpgrade] = useState<string | null>(null);
  /** Bumped when the map is replaced or imported into, so the ability list reloads. */
  const [revision, setRevision] = useState(0);
  const editable = map !== null;
  const customUnits = useMemo(() => units.filter((u) => u.custom), [units]);
  const shareFirst = useMemo(() => {
    if (selected && customUnits.some((u) => u.id === selected)) {
      return [selected];
    }

    return [];
  }, [selected, customUnits]);

  // The close handler is registered once; it reads the latest map from here.
  const mapRef = useRef(map);
  mapRef.current = map;

  const fail = useCallback((e: unknown) => setError(errorMessage(e)), []);

  const refreshMap = useCallback(() => {
    api.mapStatus().then(setMap).catch(fail);
  }, [fail]);

  const openAbility = useCallback((id: string) => {
    setSelectedAbility(id);
    setTab("abilities");
  }, []);

  const openItem = useCallback((id: string) => {
    setSelectedItem(id);
    setTab("items");
  }, []);

  const openUpgrade = useCallback((id: string) => {
    setSelectedUpgrade(id);
    setTab("upgrades");
  }, []);

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

  async function imported(objects: TransferObject[]) {
    setDialog(null);
    setRevision((r) => r + 1);
    try {
      setMap(await api.mapStatus());
      refreshList();
      const firstUnit = objects.find((o) => o.kind === "unit" && !o.dependency);
      if (firstUnit) {
        setSelected(firstUnit.newId ?? firstUnit.id);
      }
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
      setSelectedAbility(null);
      setSelectedItem(null);
      setSelectedUpgrade(null);
      setRevision((r) => r + 1);
      refreshList();
    } catch (e) {
      fail(e);
    }
  }

  const commands: PaletteCommand[] = [];
  if (map?.unsaved) {
    commands.push({ id: "save", label: "Save the map", shortcut: "Ctrl S", run: save });
  }
  commands.push({ id: "open", label: "Open a map…", run: openOtherMap });
  if (map) {
    commands.push({ id: "import", label: "Import objects…", run: () => setDialog("import") });
  }
  if (map && customUnits.length > 0) {
    commands.push({ id: "share", label: "Share units…", run: () => setDialog("share") });
  }
  for (const t of TABS.filter((other) => other.id !== tab)) {
    commands.push({ id: `tab-${t.id}`, label: `Go to ${t.label}`, run: () => setTab(t.id) });
  }

  function openUnit(id: string) {
    setTab("units");
    setSelected(id);
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

      if ((e.ctrlKey || e.metaKey) && e.key === "k") {
        e.preventDefault();
        setDialog("search");
      }
    }

    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [map, save]);

  return (
    // Panels scroll on their own; the window never does, even when a hidden
    // form element sits past its bottom edge.
    <div className="relative flex h-full flex-col gap-2.5 overflow-hidden p-2.5">
      <header className="frame flex flex-wrap items-center gap-3 bg-surface-1 px-4 py-2">
        <h1 className="font-display text-2xl font-bold tracking-wide text-gold">Object Editor</h1>
        <span className="flex-1" />
        <button
          type="button"
          onClick={() => setDialog("search")}
          className="flex w-[22rem] max-w-full items-center gap-2.5 rounded-[3px] border border-field bg-well px-3 py-1.5 text-left text-muted hover:border-frame"
        >
          <SearchLg className="size-4" aria-hidden="true" />
          <span className="flex-1 truncate">Search objects and commands…</span>
          <kbd className="rounded-[3px] border border-line-strong px-1.5 font-sans text-xs text-ink">
            Ctrl K
          </kbd>
        </button>
        <span className="flex-1" />
        {map && (
          <span
            aria-live="polite"
            className="min-w-0 truncate font-mono text-xs text-muted"
            title={map.path}
          >
            {map.name}
            {map.unsaved && <span className="ml-2 font-sans text-gold">● unsaved</span>}
          </span>
        )}
        {!map && <span className="text-muted">Game data (read-only)</span>}
        {map && (
          <>
            <Button onPress={() => setDialog("import")}>Import…</Button>
            <Button disabled={customUnits.length === 0} onPress={() => setDialog("share")}>
              Share…
            </Button>
          </>
        )}
        <Button onPress={openOtherMap}>Open map…</Button>
        {map && (
          <Button primary disabled={!map.unsaved || saving} onPress={save}>
            Save
          </Button>
        )}
      </header>

      <nav className="flex gap-6 border-b border-line px-2" aria-label="Object types">
        {TABS.map((t) => (
          <button
            key={t.id}
            type="button"
            onClick={() => setTab(t.id)}
            aria-current={tab === t.id && "page"}
            className={classes(
              "py-1.5 font-display text-lg font-bold",
              tab === t.id && "text-gold-bright shadow-[inset_0_-2px_0_var(--color-gold)]",
              tab !== t.id && "text-muted hover:text-ink",
            )}
          >
            {t.label}
          </button>
        ))}
      </nav>

      {error && (
        <div
          role="alert"
          className="flex items-center gap-3 rounded-[3px] border border-danger/40 bg-danger/10 px-4 py-2 text-danger"
        >
          <span className="flex-1">{error}</span>
          <Button small onPress={() => setError(null)}>
            Dismiss
          </Button>
        </div>
      )}

      <ShareDialog
        open={dialog === "share"}
        units={customUnits}
        initial={shareFirst}
        onClose={() => setDialog(null)}
      />
      <ImportDialog
        open={dialog === "import"}
        mapsDir={game.mapsDir}
        onImported={imported}
        onClose={() => setDialog(null)}
      />
      <CommandPalette
        open={dialog === "search"}
        commands={commands}
        units={units}
        onOpenUnit={openUnit}
        onOpenAbility={openAbility}
        onOpenItem={openItem}
        onOpenUpgrade={openUpgrade}
        onClose={() => setDialog(null)}
      />

      <main className="flex min-h-0 flex-1 flex-col">
        {tab === "abilities" && (
          <AbilitiesPane
            key={revision}
            editable={editable}
            selected={selectedAbility}
            onSelect={setSelectedAbility}
            onMapChanged={refreshMap}
            onError={setError}
          />
        )}

        {tab === "upgrades" && (
          <UpgradesPane
            key={revision}
            editable={editable}
            selected={selectedUpgrade}
            onSelect={setSelectedUpgrade}
            onMapChanged={refreshMap}
            onError={setError}
          />
        )}

        {tab === "items" && (
          <ItemsPane
            key={revision}
            editable={editable}
            selected={selectedItem}
            onSelect={setSelectedItem}
            onMapChanged={refreshMap}
            onError={setError}
          />
        )}

        <div className={classes("flex min-h-0 flex-1 gap-2.5", tab !== "units" && "hidden")}>
          <UnitList
            config={UNIT_LIST}
            units={units}
            selected={selected}
            onSelect={setSelected}
            onError={setError}
          />
          {detail && tab === "units" && (
            <OpenAbilityContext value={openAbility}>
              <UnitPage
                key={detail.item.id}
                detail={detail}
                clashes={clashes}
                editable={editable}
                onChange={(field, value) => applyField(api.setField(detail.item.id, field, value))}
                onReset={(field) => applyField(api.resetField(detail.item.id, field))}
                onCreateCopy={createCopy}
              />
            </OpenAbilityContext>
          )}
          {!detail && (
            <p className="m-auto text-muted">
              Pick a unit on the left. {game.units} game units loaded.
            </p>
          )}
        </div>
      </main>
    </div>
  );
}
