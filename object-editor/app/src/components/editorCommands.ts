import type { MapSummary } from "../types";
import type { PaletteCommand } from "./CommandPalette";
import { TABS, type Tab } from "./EditorTabs";

export type EditorDialog = "share" | "import" | "search" | "assistant";

type CommandOptions = {
  map: MapSummary | null;
  canShare: boolean;
  tab: Tab;
  save: () => void;
  openMap: () => void;
  openDialog: (dialog: EditorDialog) => void;
  changeGame: () => void;
  exportObjects: () => void;
  goTo: (tab: Tab) => void;
};

export function editorCommands(options: CommandOptions): PaletteCommand[] {
  const { map, canShare, tab, openDialog } = options;
  const commands: PaletteCommand[] = [];

  if (map?.unsaved) {
    commands.push({ id: "save", label: "Save the map", shortcut: "Ctrl S", run: options.save });
  }

  commands.push({ id: "open", label: "Open a map…", run: options.openMap });
  if (map) {
    commands.push({ id: "import", label: "Import objects…", run: () => openDialog("import") });
  }

  if (map && canShare) {
    commands.push({ id: "share", label: "Share units…", run: () => openDialog("share") });
  }

  commands.push(
    { id: "change-game", label: "Change game folder…", run: options.changeGame },
    {
      id: "assistant",
      label: "Prompt for an AI assistant…",
      run: () => openDialog("assistant"),
    },
    { id: "export", label: "Export objects…", run: options.exportObjects },
  );

  for (const t of TABS.filter((other) => other.id !== tab)) {
    commands.push({ id: `tab-${t.id}`, label: `Go to ${t.label}`, run: () => options.goTo(t.id) });
  }

  return commands;
}
