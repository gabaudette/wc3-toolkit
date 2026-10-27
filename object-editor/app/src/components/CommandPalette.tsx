import { SearchLg } from "@untitledui/icons";
import { useEffect, useMemo, useState } from "react";
import {
  Autocomplete,
  Dialog,
  Header,
  Input,
  Menu,
  MenuItem,
  MenuSection,
  Modal,
  ModalOverlay,
  SearchField,
} from "react-aria-components";
import { api } from "../api";
import { settle } from "../result";
import type { AbilitySummary, Catalog, LeveledCatalog, UnitItem } from "../types";
import { GameText, plainGameText } from "./GameText";
import { UnitIcon } from "./UnitIcon";

/** A command the palette can run, such as saving the map. */
export type PaletteCommand = {
  id: string;
  label: string;
  /** Shortcut shown next to the command, if it has one. */
  shortcut?: string;
  run: () => void;
};

type CommandPaletteProps = {
  open: boolean;
  commands: PaletteCommand[];
  units: UnitItem[];
  onOpenUnit: (id: string) => void;
  onOpenAbility: (id: string) => void;
  /** Items, buffs and destructables. */
  onOpenPlain: (catalog: Catalog, id: string) => void;
  /** Upgrades and doodads. */
  onOpenLeveled: (catalog: LeveledCatalog, id: string) => void;
  onClose: () => void;
};

type Entry = {
  id: string;
  label: string;
  detail: string;
  icon: string | null;
  run: () => void;
};

type Group = { name: string; entries: Entry[] };

/** The tabs of objects without levels, in tab order, with their group names. */
const PLAIN_GROUPS: { catalog: Catalog; name: string }[] = [
  { catalog: "items", name: "Items" },
  { catalog: "buffs", name: "Buffs" },
  { catalog: "destructables", name: "Destructables" },
];

const NO_PLAIN: Record<Catalog, UnitItem[]> = { items: [], buffs: [], destructables: [] };

const LEVELED_GROUPS: { catalog: LeveledCatalog; name: string }[] = [
  { catalog: "upgrades", name: "Upgrades" },
  { catalog: "doodads", name: "Doodads" },
];

const NO_LEVELED: Record<LeveledCatalog, UnitItem[]> = { upgrades: [], doodads: [] };

/** Results shown per group; typing narrows them further. */
const MAX_PER_GROUP = 8;

/** Match quality: the start of the text, then the start of a word, then anywhere. */
function rank(text: string, needle: string): number {
  if (text.startsWith(needle)) {
    return 0;
  }

  if (text.includes(` ${needle}`)) {
    return 1;
  }

  return 2;
}

/** Items whose text contains the query, best matches first ("holy" finds Holy Light before Unholy Aura). */
function matching<T>(list: T[], query: string, text: (item: T) => string): T[] {
  const needle = query.trim().toLowerCase();

  if (!needle) {
    return list;
  }

  return list
    .map((item) => ({ item, text: text(item).toLowerCase() }))
    .filter((m) => m.text.includes(needle))
    .map((m) => ({ item: m.item, rank: rank(m.text, needle) }))
    .sort((a, b) => a.rank - b.rank)
    .map((m) => m.item);
}

/**
 * Ctrl K: one search box for commands, units, abilities, items and upgrades. Arrow keys move,
 * Enter runs, Escape closes.
 */
export function CommandPalette({
  open,
  commands,
  units,
  onOpenUnit,
  onOpenAbility,
  onOpenPlain,
  onOpenLeveled,
  onClose,
}: CommandPaletteProps) {
  const [query, setQuery] = useState("");
  const [abilities, setAbilities] = useState<AbilitySummary[]>([]);
  const [plain, setPlain] = useState<Record<Catalog, UnitItem[]>>(NO_PLAIN);
  const [leveled, setLeveled] = useState<Record<LeveledCatalog, UnitItem[]>>(NO_LEVELED);

  useEffect(() => {
    if (!open) {
      return;
    }

    setQuery("");
    void settle(api.listAbilities(), setAbilities, () => setAbilities([]));
    for (const { catalog } of PLAIN_GROUPS) {
      void settle(
        api.listPlain(catalog),
        (list) => setPlain((current) => ({ ...current, [catalog]: list })),
        () => setPlain((current) => ({ ...current, [catalog]: [] })),
      );
    }

    for (const { catalog } of LEVELED_GROUPS) {
      void settle(
        api.listLeveled(catalog),
        (list) => setLeveled((current) => ({ ...current, [catalog]: list })),
        () => setLeveled((current) => ({ ...current, [catalog]: [] })),
      );
    }
  }, [open]);

  const groups = useMemo<Group[]>(() => {
    const done = (run: () => void) => () => {
      onClose();
      run();
    };
    const commandEntries = matching(commands, query, (c) => c.label).map((c) => ({
      id: `command:${c.id}`,
      label: c.label,
      detail: c.shortcut ?? "",
      icon: null,
      run: done(c.run),
    }));
    const unitEntries = matching(units, query, (u) => `${plainGameText(u.name)} ${u.id}`)
      .slice(0, MAX_PER_GROUP)
      .map((u) => ({
        id: `unit:${u.id}`,
        label: u.name,
        detail: u.id,
        icon: u.icon,
        run: done(() => onOpenUnit(u.id)),
      }));
    // Abilities, items and upgrades only show once something is typed: there are hundreds.
    const searched = query.trim() !== "";
    const abilityEntries = matching(abilities, query, (a) => `${plainGameText(a.name)} ${a.id}`)
      .filter(() => searched)
      .slice(0, MAX_PER_GROUP)
      .map((a) => ({
        id: `ability:${a.id}`,
        label: a.name,
        detail: a.id,
        icon: a.icon,
        run: done(() => onOpenAbility(a.id)),
      }));
    const plainGroups = PLAIN_GROUPS.map(({ catalog, name }) => ({
      name,
      entries: matching(plain[catalog], query, (o) => `${plainGameText(o.name)} ${o.id}`)
        .filter(() => searched)
        .slice(0, MAX_PER_GROUP)
        .map((o) => ({
          id: `${catalog}:${o.id}`,
          label: o.name,
          detail: o.id,
          icon: o.icon,
          run: done(() => onOpenPlain(catalog, o.id)),
        })),
    }));
    const leveledGroups = LEVELED_GROUPS.map(({ catalog, name }) => ({
      name,
      entries: matching(leveled[catalog], query, (o) => `${plainGameText(o.name)} ${o.id}`)
        .filter(() => searched)
        .slice(0, MAX_PER_GROUP)
        .map((o) => ({
          id: `${catalog}:${o.id}`,
          label: o.name,
          detail: o.id,
          icon: o.icon,
          run: done(() => onOpenLeveled(catalog, o.id)),
        })),
    }));

    return [
      { name: "Commands", entries: commandEntries },
      { name: "Units", entries: unitEntries },
      { name: "Abilities", entries: abilityEntries },
      ...plainGroups,
      ...leveledGroups,
    ].filter((g) => g.entries.length > 0);
  }, [
    commands,
    units,
    abilities,
    plain,
    leveled,
    query,
    onOpenUnit,
    onOpenAbility,
    onOpenPlain,
    onOpenLeveled,
    onClose,
  ]);

  const byId = useMemo(
    () => new Map(groups.flatMap((g) => g.entries).map((e) => [e.id, e])),
    [groups],
  );

  return (
    <ModalOverlay
      isOpen={open}
      onOpenChange={(isOpen) => {
        if (!isOpen) {
          onClose();
        }
      }}
      isDismissable
      className="fixed inset-0 z-50 flex items-start justify-center bg-black/60 pt-[12vh]"
    >
      <Modal className="frame w-[36rem] max-w-[calc(100vw-2rem)] bg-surface-1 shadow-2xl">
        <Dialog aria-label="Search objects and commands" className="outline-none">
          <Autocomplete inputValue={query} onInputChange={setQuery}>
            <SearchField
              aria-label="Search objects and commands"
              autoFocus
              className="flex items-center gap-2 border-b border-line px-4 py-3"
            >
              <SearchLg className="size-4 text-muted" aria-hidden="true" />
              <Input
                placeholder="Search units, abilities, items, upgrades, commands…"
                className="min-w-0 flex-1 bg-transparent text-base text-ink outline-none placeholder:text-faint"
              />
            </SearchField>
            <Menu
              aria-label="Results"
              className="max-h-[60vh] overflow-y-auto p-1.5 outline-none"
              onAction={(key) => byId.get(String(key))?.run()}
              renderEmptyState={() => <p className="p-4 text-muted">Nothing matches.</p>}
            >
              {groups.map((group) => (
                <MenuSection key={group.name} className="pb-1">
                  <Header className="px-2.5 pt-2 pb-1 font-display text-sm font-semibold text-gold">
                    {group.name}
                  </Header>
                  {group.entries.map((entry) => (
                    <MenuItem
                      key={entry.id}
                      id={entry.id}
                      textValue={plainGameText(entry.label)}
                      className="flex cursor-default items-center gap-3 rounded-[3px] px-2.5 py-1.5 text-ink outline-none data-[focused]:bg-surface-2 data-[focused]:shadow-[inset_2px_0_0_var(--color-glow)]"
                    >
                      {entry.icon !== null && <UnitIcon path={entry.icon} size="small" />}
                      <span className="min-w-0 flex-1 truncate">
                        <GameText text={entry.label} singleLine />
                      </span>
                      <span className="font-mono text-xs text-faint">{entry.detail}</span>
                    </MenuItem>
                  ))}
                </MenuSection>
              ))}
            </Menu>
          </Autocomplete>
        </Dialog>
      </Modal>
    </ModalOverlay>
  );
}
