import { useCallback, useEffect, useMemo, useState } from "react";
import { api } from "../api";
import { settle } from "../result";
import type { GameSummary, MapSummary, TransferObject, UnitItem } from "../types";
import { AbilitiesPane } from "./AbilitiesPane";
import { AssetsPane } from "./AssetsPane";
import { AssistantPromptDialog } from "./AssistantPromptDialog";
import { CommandPalette } from "./CommandPalette";
import { EditorHeader, ErrorBanner } from "./EditorHeader";
import { TabBar, useEditorTabs } from "./EditorTabs";
import { ExportDialog } from "./ExportDialog";
import { type EditorDialog, editorCommands } from "./editorCommands";
import { ignoreSelection, LEVELED_TABS, PLAIN_TABS, useNavigation } from "./editorNavigation";
import { ExportContext, type ExportScope } from "./exportObjects";
import { ImportDialog } from "./ImportDialog";
import { InterfacePane } from "./InterfacePane";
import { LeveledPane } from "./LeveledPane";
import { LuaFolderContext } from "./luaFolder";
import { MapPane } from "./MapPane";
import { useMapFile } from "./mapFile";
import { PlacedPane } from "./PlacedPane";
import { PlainPane } from "./PlainPane";
import { ShareDialog } from "./ShareDialog";
import { TabPanel } from "./TabPanel";
import { UnitsPane } from "./UnitsPane";

type EditorScreenProps = {
  game: GameSummary;
  initialMap: MapSummary | null;
  onChangeGame: () => void;
};

function useShortcuts(unsaved: boolean, save: () => void, search: () => void): void {
  useEffect(() => {
    function onKey(e: KeyboardEvent) {
      if ((e.ctrlKey || e.metaKey) && e.key === "s" && unsaved) {
        e.preventDefault();
        save();
      }

      if ((e.ctrlKey || e.metaKey) && e.key === "k") {
        e.preventDefault();
        search();
      }
    }

    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [unsaved, save, search]);
}

export function EditorScreen({ game, initialMap, onChangeGame }: EditorScreenProps) {
  const [units, setUnits] = useState<UnitItem[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [dialog, setDialog] = useState<EditorDialog | null>(null);
  const [exportScope, setExportScope] = useState<ExportScope | null>(null);
  const [luaFolder, setLuaFolder] = useState("");
  const [revision, setRevision] = useState(0);
  const sharedLuaFolder = useMemo(
    () => ({ folder: luaFolder, setFolder: setLuaFolder }),
    [luaFolder],
  );
  const file = useMapFile({ game, initialMap, onOpened: opened, onError: setError });
  const { map } = file;
  const { tab, setTab, visited, paneKey } = useEditorTabs(file.changes, revision);
  const nav = useNavigation(setTab);
  const editable = map !== null;
  const customUnits = useMemo(() => units.filter((u) => u.custom), [units]);
  const shareFirst = useMemo(() => {
    if (nav.unit && customUnits.some((u) => u.id === nav.unit)) {
      return [nav.unit];
    }

    return [];
  }, [nav.unit, customUnits]);
  const closeDialog = useCallback(() => setDialog(null), []);
  const openSearch = useCallback(() => setDialog("search"), []);

  const refreshList = useCallback(() => {
    void settle(api.listUnits(), setUnits, setError);
  }, []);

  useEffect(() => {
    refreshList();
  }, [refreshList]);

  useShortcuts(map?.unsaved ?? false, file.save, openSearch);

  function opened() {
    setLuaFolder("");
    nav.clear();
    setRevision((r) => r + 1);
    refreshList();
  }

  function imported(objects: TransferObject[]) {
    setDialog(null);
    setRevision((r) => r + 1);
    file.refreshMap();
    refreshList();
    const firstUnit = objects.find((o) => o.kind === "unit" && !o.dependency);

    if (firstUnit) {
      nav.setUnit(firstUnit.newId ?? firstUnit.id);
    }
  }

  async function changeGame() {
    if (await file.mayLeave()) {
      onChangeGame();
    }
  }

  function unitCreated(id: string) {
    file.refreshMap();
    refreshList();
    nav.openUnit(id);
  }

  const commands = editorCommands({
    map,
    canShare: customUnits.length > 0,
    tab,
    save: file.save,
    openMap: file.openOther,
    openDialog: setDialog,
    changeGame: () => void changeGame(),
    exportObjects: () => setExportScope({ type: null, shown: [], selected: null }),
    goTo: setTab,
  });

  return (
    <div className="relative flex h-full flex-col gap-2.5 overflow-hidden p-2.5">
      <EditorHeader
        map={map}
        saving={file.saving}
        canShare={customUnits.length > 0}
        onSearch={openSearch}
        onImport={() => setDialog("import")}
        onShare={() => setDialog("share")}
        onOpenMap={file.openOther}
        onSave={file.save}
      />
      <TabBar tab={tab} onSelect={setTab} />
      <ErrorBanner error={error} onDismiss={() => setError(null)} />

      <ExportDialog
        scope={exportScope}
        mapName={map?.name ?? null}
        onClose={() => setExportScope(null)}
        onError={setError}
      />
      <AssistantPromptDialog open={dialog === "assistant"} onClose={closeDialog} />
      <ShareDialog
        open={dialog === "share"}
        units={customUnits}
        initial={shareFirst}
        onClose={closeDialog}
      />
      <ImportDialog
        open={dialog === "import"}
        mapsDir={game.mapsDir}
        onImported={imported}
        onClose={closeDialog}
      />
      <CommandPalette
        open={dialog === "search"}
        commands={commands}
        units={units}
        onOpenUnit={nav.openUnit}
        onOpenAbility={nav.openAbility}
        onOpenPlain={nav.openPlain}
        onOpenLeveled={nav.openLeveled}
        onClose={closeDialog}
      />

      <ExportContext value={setExportScope}>
        <LuaFolderContext value={sharedLuaFolder}>
          <main className="grid min-h-0 flex-1 grid-cols-1 grid-rows-1 overflow-hidden">
            {visited.includes("abilities") && (
              <TabPanel shown={tab === "abilities"}>
                <AbilitiesPane
                  key={paneKey("abilities")}
                  editable={editable}
                  selected={nav.ability}
                  onSelect={nav.setAbility}
                  onMapChanged={file.refreshMap}
                  onUnitsChanged={refreshList}
                  onError={setError}
                />
              </TabPanel>
            )}

            {visited.includes("interface") && (
              <TabPanel shown={tab === "interface"}>
                <InterfacePane
                  key={paneKey("interface")}
                  editable={editable}
                  mapPath={map?.path ?? null}
                  onMapChanged={file.refreshMap}
                  onError={setError}
                />
              </TabPanel>
            )}

            {visited.includes("map") && (
              <TabPanel shown={tab === "map"}>
                <MapPane
                  key={paneKey("map")}
                  editable={editable}
                  onMapChanged={file.refreshMap}
                  onError={setError}
                />
              </TabPanel>
            )}

            {visited.includes("placed") && (
              <TabPanel shown={tab === "placed"}>
                <PlacedPane
                  key={paneKey("placed")}
                  editable={editable}
                  units={units}
                  onMapChanged={file.refreshMap}
                  onOpenUnit={nav.openUnit}
                  onError={setError}
                />
              </TabPanel>
            )}

            {visited.includes("assets") && (
              <TabPanel shown={tab === "assets"}>
                <AssetsPane
                  key={paneKey("assets")}
                  editable={editable}
                  mapPath={map?.path ?? null}
                  unsaved={map?.unsaved ?? false}
                  units={units}
                  onMapChanged={file.refreshMap}
                  onUnitCreated={unitCreated}
                  onOpenUse={nav.openUse}
                  onError={setError}
                />
              </TabPanel>
            )}

            {LEVELED_TABS.filter(({ catalog }) => visited.includes(catalog)).map(
              ({ catalog, list }) => (
                <TabPanel key={catalog} shown={tab === catalog}>
                  <LeveledPane
                    key={paneKey(catalog)}
                    catalog={catalog}
                    list={list}
                    editable={editable}
                    selected={nav.leveled[catalog]}
                    onSelect={nav.selectLeveled.get(catalog) ?? ignoreSelection}
                    onMapChanged={file.refreshMap}
                    onError={setError}
                  />
                </TabPanel>
              ),
            )}

            {PLAIN_TABS.filter(({ catalog }) => visited.includes(catalog)).map(
              ({ catalog, list }) => (
                <TabPanel key={catalog} shown={tab === catalog}>
                  <PlainPane
                    key={paneKey(catalog)}
                    catalog={catalog}
                    list={list}
                    editable={editable}
                    selected={nav.plain[catalog]}
                    onSelect={nav.selectPlain.get(catalog) ?? ignoreSelection}
                    onMapChanged={file.refreshMap}
                    onError={setError}
                  />
                </TabPanel>
              ),
            )}

            <TabPanel shown={tab === "units"}>
              <UnitsPane
                units={units}
                gameUnits={game.units}
                editable={editable}
                selected={nav.unit}
                onSelect={nav.setUnit}
                onOpenAbility={nav.openAbility}
                onMapChanged={file.refreshMap}
                onUnitsChanged={refreshList}
                onError={setError}
              />
            </TabPanel>
          </main>
        </LuaFolderContext>
      </ExportContext>
    </div>
  );
}
