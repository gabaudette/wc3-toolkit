import { useEffect, useRef, useState } from "react";
import type { Catalog, LeveledCatalog } from "../types";
import { classes } from "../ui";

export type Tab =
  | "units"
  | "abilities"
  | LeveledCatalog
  | Catalog
  | "placed"
  | "map"
  | "interface"
  | "assets";

export const TABS: { id: Tab; label: string }[] = [
  { id: "units", label: "Units" },
  { id: "abilities", label: "Abilities" },
  { id: "items", label: "Items" },
  { id: "upgrades", label: "Upgrades" },
  { id: "buffs", label: "Buffs" },
  { id: "destructables", label: "Destructables" },
  { id: "doodads", label: "Doodads" },
  { id: "placed", label: "Placed units" },
  { id: "map", label: "Map" },
  { id: "interface", label: "Interface" },
  { id: "assets", label: "Assets" },
];

export type EditorTabs = {
  tab: Tab;
  setTab: (tab: Tab) => void;
  visited: Tab[];
  paneKey: (pane: Tab) => string;
};

export function useEditorTabs(changes: number, revision: number): EditorTabs {
  const [tab, setTab] = useState<Tab>("units");
  const [visited, setVisited] = useState<Tab[]>(["units"]);
  const [reloads, setReloads] = useState<Partial<Record<Tab, number>>>({});
  const seenChanges = useRef<Partial<Record<Tab, number>>>({});
  const shownTab = useRef<Tab>(tab);

  useEffect(() => {
    const seen = seenChanges.current[tab];

    if (shownTab.current !== tab && seen !== undefined && seen < changes) {
      setReloads((current) => ({ ...current, [tab]: (current[tab] ?? 0) + 1 }));
    }

    seenChanges.current[tab] = changes;
    shownTab.current = tab;
    setVisited((current) => {
      if (current.includes(tab)) {
        return current;
      }

      return [...current, tab];
    });
  }, [tab, changes]);

  function paneKey(pane: Tab): string {
    return `${revision}-${reloads[pane] ?? 0}`;
  }

  return { tab, setTab, visited, paneKey };
}

type TabBarProps = { tab: Tab; onSelect: (tab: Tab) => void };

export function TabBar({ tab, onSelect }: TabBarProps) {
  return (
    <nav className="flex gap-6 border-b border-line px-2" aria-label="Object types">
      {TABS.map((t) => (
        <button
          key={t.id}
          type="button"
          onClick={() => onSelect(t.id)}
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
  );
}
