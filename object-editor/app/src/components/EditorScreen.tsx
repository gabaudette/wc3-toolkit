import { getCurrentWindow } from "@tauri-apps/api/window";
import { SearchLg } from "@untitledui/icons";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { api, report } from "../api";
import { choosePath, confirm } from "../dialogs";
import { attempt, type Result, settle } from "../result";
import type {
  Catalog,
  Field,
  GameSummary,
  HotkeyClash,
  LeveledCatalog,
  MapSummary,
  TransferObject,
  UnitDetail,
  UnitItem,
} from "../types";
import { classes, MAP_FILTERS, MAP_PATH_KEY, remembered, replaceField } from "../ui";
import { AbilitiesPane } from "./AbilitiesPane";
import { AppVersion } from "./AppVersion";
import { Button } from "./Button";
import { CommandPalette, type PaletteCommand } from "./CommandPalette";
import { ImportDialog } from "./ImportDialog";
import { InterfacePane } from "./InterfacePane";
import { LeveledPane } from "./LeveledPane";
import { OpenAbilityContext } from "./openAbility";
import { PlainPane } from "./PlainPane";
import { ShareDialog } from "./ShareDialog";
import {
  BUFF_LIST,
  DESTRUCTABLE_LIST,
  DOODAD_LIST,
  ITEM_LIST,
  type ListConfig,
  UNIT_LIST,
  UnitList,
  UPGRADE_LIST,
} from "./UnitList";
import { UnitPage } from "./UnitPage";

type Tab = "units" | "abilities" | LeveledCatalog | Catalog | "interface";

const TABS: { id: Tab; label: string }[] = [
  { id: "units", label: "Units" },
  { id: "abilities", label: "Abilities" },
  { id: "items", label: "Items" },
  { id: "upgrades", label: "Upgrades" },
  { id: "buffs", label: "Buffs" },
  { id: "destructables", label: "Destructables" },
  { id: "doodads", label: "Doodads" },
  { id: "interface", label: "Interface" },
];

const LEVELED_TABS: { catalog: LeveledCatalog; list: ListConfig }[] = [
  { catalog: "upgrades", list: UPGRADE_LIST },
  { catalog: "doodads", list: DOODAD_LIST },
];

const NO_LEVELED_SELECTION: Record<LeveledCatalog, string | null> = {
  upgrades: null,
  doodads: null,
};

const PLAIN_TABS: { catalog: Catalog; list: ListConfig }[] = [
  { catalog: "items", list: ITEM_LIST },
  { catalog: "buffs", list: BUFF_LIST },
  { catalog: "destructables", list: DESTRUCTABLE_LIST },
];

const NO_PLAIN_SELECTION: Record<Catalog, string | null> = {
  items: null,
  buffs: null,
  destructables: null,
};

type EditorScreenProps = {
  game: GameSummary;
  initialMap: MapSummary | null;
};

async function confirmDiscard(map: MapSummary): Promise<boolean> {
  const answer = await confirm(`${map.name} has unsaved changes. Discard them?`, {
    title: "Unsaved changes",
    kind: "warning",
    okLabel: "Discard changes",
    cancelLabel: "Keep editing",
  });

  return answer.ok && answer.value;
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
  const [selectedPlain, setSelectedPlain] = useState(NO_PLAIN_SELECTION);
  const [selectedLeveled, setSelectedLeveled] = useState(NO_LEVELED_SELECTION);
  const [revision, setRevision] = useState(0);
  const editable = map !== null;
  const customUnits = useMemo(() => units.filter((u) => u.custom), [units]);
  const shareFirst = useMemo(() => {
    if (selected && customUnits.some((u) => u.id === selected)) {
      return [selected];
    }

    return [];
  }, [selected, customUnits]);

  const mapRef = useRef(map);

  mapRef.current = map;

  const refreshMap = useCallback(() => {
    void settle(api.mapStatus(), setMap, setError);
  }, []);

  const openAbility = useCallback((id: string) => {
    setSelectedAbility(id);
    setTab("abilities");
  }, []);

  const openPlain = useCallback((catalog: Catalog, id: string) => {
    setSelectedPlain((current) => ({ ...current, [catalog]: id }));
    setTab(catalog);
  }, []);

  const openLeveled = useCallback((catalog: LeveledCatalog, id: string) => {
    setSelectedLeveled((current) => ({ ...current, [catalog]: id }));
    setTab(catalog);
  }, []);

  const refreshList = useCallback(() => {
    void settle(api.listUnits(), setUnits, setError);
  }, []);

  const refreshDetail = useCallback((id: string) => {
    void settle(api.unitDetail(id), setDetail, setError);
    void settle(api.hotkeyClashes(id), setClashes, setError);
  }, []);

  const refreshUnits = useCallback(() => {
    refreshList();
    if (selected) {
      refreshDetail(selected);
    }
  }, [refreshList, refreshDetail, selected]);

  useEffect(() => {
    refreshList();
  }, [refreshList]);

  useEffect(() => {
    if (selected) {
      refreshDetail(selected);
    }
  }, [selected, refreshDetail]);

  async function applyField(edit: Promise<Result<Field>>) {
    const updated = await edit;

    if (!updated.ok) {
      setError(updated.error);
      return;
    }

    setDetail((current) => current && replaceField(current, updated.value));
    setError(null);
    refreshMap();
    refreshList();
    if (selected) {
      refreshDetail(selected);
    }
  }

  async function createCopy() {
    if (!detail) {
      return;
    }

    const created = await api.createUnit(detail.item.base);

    if (!created.ok) {
      setError(created.error);
      return;
    }

    refreshMap();
    refreshList();
    setSelected(created.value.id);
  }

  async function imported(objects: TransferObject[]) {
    setDialog(null);
    setRevision((r) => r + 1);
    refreshMap();
    refreshList();
    const firstUnit = objects.find((o) => o.kind === "unit" && !o.dependency);

    if (firstUnit) {
      setSelected(firstUnit.newId ?? firstUnit.id);
    }
  }

  const save = useCallback(async () => {
    setSaving(true);
    const saved = await api.saveMap();
    setSaving(false);
    if (!saved.ok) {
      setError(saved.error);
      return;
    }

    setMap(saved.value);
    setError(null);
  }, []);

  async function openOtherMap() {
    if (map?.unsaved && !(await confirmDiscard(map))) {
      return;
    }

    const picked = await choosePath({
      title: "Open a map",
      filters: MAP_FILTERS,
      ...(game.mapsDir && { defaultPath: game.mapsDir }),
    });

    if (!picked.ok) {
      setError(picked.error);
      return;
    }

    const path = picked.value;

    if (!path) {
      return;
    }

    const opened = await api.openMap(path);

    if (!opened.ok) {
      setError(opened.error);
      return;
    }

    setMap(opened.value);
    remembered.set(MAP_PATH_KEY, path);
    setSelected(null);
    setDetail(null);
    setSelectedAbility(null);
    setSelectedPlain(NO_PLAIN_SELECTION);
    setSelectedLeveled(NO_LEVELED_SELECTION);
    setRevision((r) => r + 1);
    refreshList();
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
      if (!(await confirmDiscard(current))) {
        return;
      }

      const closed = await attempt(() => appWindow.destroy());
      if (!closed.ok) {
        report("error", "closing the window", closed.error);
        setError(closed.error);
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
  }, []);

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
    <div className="relative flex h-full flex-col gap-2.5 overflow-hidden p-2.5">
      <header className="frame flex flex-wrap items-center gap-3 bg-surface-1 px-4 py-2">
        <h1 className="font-display text-2xl font-bold tracking-wide text-gold">Tinkerbench</h1>
        <AppVersion />
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
        onOpenPlain={openPlain}
        onOpenLeveled={openLeveled}
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
            onUnitsChanged={refreshUnits}
            onError={setError}
          />
        )}

        {tab === "interface" && (
          <InterfacePane
            key={revision}
            editable={editable}
            mapPath={map?.path ?? null}
            onMapChanged={refreshMap}
            onError={setError}
          />
        )}

        {LEVELED_TABS.filter(({ catalog }) => tab === catalog).map(({ catalog, list }) => (
          <LeveledPane
            key={`${catalog}-${revision}`}
            catalog={catalog}
            list={list}
            editable={editable}
            selected={selectedLeveled[catalog]}
            onSelect={(id) => openLeveled(catalog, id)}
            onMapChanged={refreshMap}
            onError={setError}
          />
        ))}

        {PLAIN_TABS.filter(({ catalog }) => tab === catalog).map(({ catalog, list }) => (
          <PlainPane
            key={`${catalog}-${revision}`}
            catalog={catalog}
            list={list}
            editable={editable}
            selected={selectedPlain[catalog]}
            onSelect={(id) => openPlain(catalog, id)}
            onMapChanged={refreshMap}
            onError={setError}
          />
        ))}

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
