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
import type { AbilitySummary, UnitItem } from "../types";
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

/** Results shown per group; typing narrows them further. */
const MAX_PER_GROUP = 8;

function matching<T>(list: T[], query: string, text: (item: T) => string): T[] {
  const needle = query.trim().toLowerCase();
  return list.filter((item) => !needle || text(item).toLowerCase().includes(needle));
}

/**
 * Ctrl K: one search box for commands, units and abilities. Arrow keys move,
 * Enter runs, Escape closes.
 */
export function CommandPalette({
  open,
  commands,
  units,
  onOpenUnit,
  onOpenAbility,
  onClose,
}: CommandPaletteProps) {
  const [query, setQuery] = useState("");
  const [abilities, setAbilities] = useState<AbilitySummary[]>([]);

  useEffect(() => {
    if (!open) {
      return;
    }

    setQuery("");
    // Abilities are only needed while searching; a failure leaves the group empty.
    api
      .listAbilities()
      .then(setAbilities)
      .catch(() => setAbilities([]));
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
    const unitEntries = matching(units, query, (u) => `${u.name} ${u.id}`)
      .slice(0, MAX_PER_GROUP)
      .map((u) => ({
        id: `unit:${u.id}`,
        label: u.name,
        detail: u.id,
        icon: u.icon,
        run: done(() => onOpenUnit(u.id)),
      }));
    // Abilities only show once something is typed: there are over a thousand.
    const searched = query.trim() !== "";
    const abilityEntries = matching(abilities, query, (a) => `${a.name} ${a.id}`)
      .filter(() => searched)
      .slice(0, MAX_PER_GROUP)
      .map((a) => ({
        id: `ability:${a.id}`,
        label: a.name,
        detail: a.id,
        icon: a.icon,
        run: done(() => onOpenAbility(a.id)),
      }));

    return [
      { name: "Commands", entries: commandEntries },
      { name: "Units", entries: unitEntries },
      { name: "Abilities", entries: abilityEntries },
    ].filter((g) => g.entries.length > 0);
  }, [commands, units, abilities, query, onOpenUnit, onOpenAbility, onClose]);

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
        <Dialog aria-label="Search units, abilities and commands" className="outline-none">
          <Autocomplete inputValue={query} onInputChange={setQuery}>
            <SearchField
              aria-label="Search units, abilities and commands"
              autoFocus
              className="flex items-center gap-2 border-b border-line px-4 py-3"
            >
              <SearchLg className="size-4 text-muted" aria-hidden="true" />
              <Input
                placeholder="Search units, abilities, commands…"
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
                      textValue={entry.label}
                      className="flex cursor-default items-center gap-3 rounded-[3px] px-2.5 py-1.5 text-ink outline-none data-[focused]:bg-surface-2 data-[focused]:shadow-[inset_2px_0_0_var(--color-glow)]"
                    >
                      {entry.icon !== null && <UnitIcon path={entry.icon} size="small" />}
                      <span className="min-w-0 flex-1 truncate">{entry.label}</span>
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
