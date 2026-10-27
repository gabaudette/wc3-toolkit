import { useEffect, useMemo, useState } from "react";
import { Dialog, Heading, Modal, ModalOverlay } from "react-aria-components";
import { api } from "../api";
import { settle } from "../result";
import type { AbilitySummary } from "../types";
import { capitalized, classes, levelCount } from "../ui";
import { Button } from "./Button";
import { GameText, plainGameText } from "./GameText";
import { UnitIcon } from "./UnitIcon";

type Kind = "hero" | "unit" | "item";

type AbilityPickerProps = {
  open: boolean;
  heroOnly: boolean;
  exclude: string[];
  onPick: (id: string) => void;
  onClose: () => void;
};

/** Results shown at once; typing narrows the list further. */
const MAX_RESULTS = 200;

function kindOf(ability: AbilitySummary): Kind {
  if (ability.hero) {
    return "hero";
  }

  if (ability.item) {
    return "item";
  }

  return "unit";
}

const KINDS: { id: Kind; label: string }[] = [
  { id: "hero", label: "Hero" },
  { id: "unit", label: "Unit" },
  { id: "item", label: "Item" },
];

export function AbilityPicker({ open, heroOnly, exclude, onPick, onClose }: AbilityPickerProps) {
  const [abilities, setAbilities] = useState<AbilitySummary[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [query, setQuery] = useState("");
  const [kind, setKind] = useState<Kind>("hero");

  useEffect(() => {
    if (!open) {
      return;
    }

    // Loaded each time: the map's own abilities change while editing.
    void settle(api.listAbilities(), setAbilities, setError);
  }, [open]);

  useEffect(() => {
    if (heroOnly) {
      setKind("hero");
    }
  }, [heroOnly]);

  const shown = useMemo(() => {
    const needle = query.trim().toLowerCase();
    return abilities
      .filter((a) => kindOf(a) === kind && !exclude.includes(a.id))
      .filter(
        (a) =>
          !needle ||
          plainGameText(a.name).toLowerCase().includes(needle) ||
          a.id.toLowerCase().includes(needle) ||
          (a.suffix ?? "").toLowerCase().includes(needle),
      )
      .slice(0, MAX_RESULTS);
  }, [abilities, kind, query, exclude]);

  return (
    <ModalOverlay
      isOpen={open}
      onOpenChange={(isOpen) => {
        if (!isOpen) {
          onClose();
        }
      }}
      isDismissable
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/60"
    >
      <Modal className="flex max-h-[80vh] w-[36rem] flex-col rounded-lg border border-line bg-surface-1 shadow-2xl">
        <Dialog className="flex min-h-0 flex-1 flex-col outline-none">
          <div className="flex flex-col gap-3 border-b border-line p-4">
            <Heading slot="title" className="text-lg font-semibold">
              Add an ability
            </Heading>
            <input
              type="search"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder="Search abilities: Holy Light, AHhb…"
              aria-label="Search abilities"
              autoFocus
              className="rounded-[3px] border border-field bg-well px-3 py-1.5 outline-none focus:border-focus"
            />
            {!heroOnly && (
              <div className="flex gap-1">
                {KINDS.map((k) => (
                  <button
                    key={k.id}
                    type="button"
                    onClick={() => setKind(k.id)}
                    aria-pressed={kind === k.id}
                    className={classes(
                      "min-h-6 rounded-full border px-2.5 py-1 text-xs",
                      kind === k.id && "border-gold/60 bg-gold-soft text-gold",
                      kind !== k.id && "border-line text-faint hover:text-muted",
                    )}
                  >
                    {k.label} abilities
                  </button>
                ))}
              </div>
            )}
            {heroOnly && <p className="text-xs text-faint">Hero abilities only.</p>}
          </div>

          {error && <p className="p-4 text-danger">{error}</p>}
          <ul className="min-h-0 flex-1 overflow-y-auto" aria-label="Abilities">
            {shown.map((ability) => (
              <li key={ability.id}>
                <button
                  type="button"
                  onClick={() => onPick(ability.id)}
                  className="flex w-full items-center gap-3 px-4 py-1.5 text-left hover:bg-surface-2"
                >
                  <UnitIcon path={ability.icon} size="small" />
                  <span className="min-w-0 flex-1">
                    <span className="block truncate">
                      <GameText text={ability.name} singleLine />
                      {ability.suffix && (
                        <span className="text-faint">
                          {" "}
                          <GameText text={ability.suffix} singleLine />
                        </span>
                      )}
                    </span>
                    <span className="block font-mono text-xs text-faint">
                      {ability.id} · {levelCount(ability.levels)}
                      {ability.race && ` · ${capitalized(ability.race)}`}
                    </span>
                  </span>
                </button>
              </li>
            ))}
            {shown.length === 0 && !error && (
              <li className="p-4 text-muted">No abilities match.</li>
            )}
          </ul>

          <div className="flex justify-end border-t border-line p-3">
            <Button onPress={onClose}>Close</Button>
          </div>
        </Dialog>
      </Modal>
    </ModalOverlay>
  );
}
