import { useEffect, useMemo, useState } from "react";
import { ITEM_CLASSES, ITEM_LEVELS, randomItemCode } from "../loot";
import type { ItemChoice } from "../types";
import { classes } from "../ui";
import { Button } from "./Button";
import { Chip, toggle } from "./Chip";
import { DialogShell } from "./DialogShell";
import { GameText, plainGameText } from "./GameText";
import { Select } from "./Select";
import { UnitIcon } from "./UnitIcon";

const MAX_RESULTS = 200;
const ANY = "any";

type Group = "map" | "game";

const GROUPS: { id: Group; label: string }[] = [
  { id: "map", label: "This map" },
  { id: "game", label: "Game" },
];

type ItemPickerProps = {
  open: boolean;
  title: string;
  choices: ItemChoice[];
  allowRandom: boolean;
  onPick: (code: string) => void;
  onClose: () => void;
};

function groupOf(choice: ItemChoice): Group {
  if (choice.inMap) {
    return "map";
  }

  return "game";
}

function levelOptions(anyLabel: string) {
  return [
    { value: ANY, label: anyLabel },
    ...ITEM_LEVELS.map((level) => ({ value: String(level), label: `Level ${level}` })),
  ];
}

export function ItemPicker({
  open,
  title,
  choices,
  allowRandom,
  onPick,
  onClose,
}: ItemPickerProps) {
  const [query, setQuery] = useState("");
  const [groups, setGroups] = useState<Group[]>(["map", "game"]);
  const [itemClasses, setItemClasses] = useState<string[]>([]);
  const [level, setLevel] = useState(ANY);
  const [randomClass, setRandomClass] = useState(ANY);
  const [randomLevel, setRandomLevel] = useState("1");

  useEffect(() => {
    if (open) {
      setQuery("");
    }
  }, [open]);

  const filtered = itemClasses.length > 0 || level !== ANY || groups.length < GROUPS.length;

  function clearFilters() {
    setGroups(["map", "game"]);
    setItemClasses([]);
    setLevel(ANY);
  }

  const shown = useMemo(() => {
    const needle = query.trim().toLowerCase();

    return choices
      .filter((c) => groups.includes(groupOf(c)))
      .filter((c) => itemClasses.length === 0 || itemClasses.includes(c.class))
      .filter((c) => level === ANY || c.level === Number(level))
      .filter(
        (c) =>
          !needle ||
          plainGameText(c.name).toLowerCase().includes(needle) ||
          c.id.toLowerCase().includes(needle),
      )
      .slice(0, MAX_RESULTS);
  }, [choices, query, groups, itemClasses, level]);

  function pickRandom() {
    const level = Number(randomLevel);

    if (randomClass === ANY) {
      onPick(randomItemCode({ classId: null, level }));
      return;
    }

    onPick(randomItemCode({ classId: randomClass, level }));
  }

  return (
    <DialogShell
      open={open}
      title={title}
      wide
      onClose={onClose}
      footer={<Button onPress={onClose}>Cancel</Button>}
    >
      <input
        value={query}
        onChange={(e) => setQuery(e.target.value)}
        placeholder="Search items: Claws, ratc…"
        aria-label="Search items"
        className="rounded-[3px] border border-field bg-well px-3 py-1.5 text-ink outline-none focus:border-focus"
      />
      <div className="flex flex-wrap items-center gap-1.5">
        {GROUPS.map((g) => (
          <Chip
            key={g.id}
            label={g.label}
            on={groups.includes(g.id)}
            onPress={() => setGroups(toggle(groups, g.id))}
          />
        ))}
        <span className="mx-1 h-4 w-px bg-line" aria-hidden="true" />
        {ITEM_CLASSES.map((c) => (
          <Chip
            key={c.id}
            label={c.label}
            on={itemClasses.includes(c.id)}
            onPress={() => setItemClasses(toggle(itemClasses, c.id))}
          />
        ))}
        <div className="ml-auto w-36">
          <Select
            label="Item level"
            value={level}
            onChange={setLevel}
            options={levelOptions("Any level")}
          />
        </div>
      </div>
      <ul aria-label="Items" className="h-80 overflow-y-auto rounded-[3px] border border-line">
        {shown.map((c) => (
          <li key={c.id}>
            <button
              type="button"
              onClick={() => onPick(c.id)}
              className="flex w-full items-center gap-2 px-2 py-1 text-left hover:bg-surface-2"
            >
              <UnitIcon path={c.icon} size="small" />
              <span className="min-w-0 flex-1 truncate">
                <GameText text={c.name} singleLine />
              </span>
              <span className="text-xs text-faint">
                {ITEM_CLASSES.find((i) => i.id === c.class)?.label ?? c.class} · level {c.level}
              </span>
              <span className={classes("w-12 text-right font-mono text-xs text-faint")}>
                {c.id}
              </span>
            </button>
          </li>
        ))}
        {shown.length === 0 && (
          <li className="flex items-center gap-3 p-3 text-sm text-faint">
            No item matches.
            {filtered && (
              <Button small onPress={clearFilters}>
                Clear filters
              </Button>
            )}
          </li>
        )}
      </ul>
      {allowRandom && (
        <div className="flex flex-wrap items-end gap-2 border-t border-line pt-3">
          <p className="w-full text-sm text-muted">
            Or a random item: the game picks one of the chosen class and level when it drops.
          </p>
          <div className="w-44">
            <Select
              label="Random item class"
              value={randomClass}
              onChange={setRandomClass}
              options={[
                { value: ANY, label: "Any class" },
                ...ITEM_CLASSES.map((c) => ({ value: c.id, label: c.label })),
              ]}
            />
          </div>
          <div className="w-32">
            <Select
              label="Random item level"
              value={randomLevel}
              onChange={setRandomLevel}
              options={ITEM_LEVELS.map((l) => ({ value: String(l), label: `Level ${l}` }))}
            />
          </div>
          <Button onPress={pickRandom}>Use a random item</Button>
        </div>
      )}
    </DialogShell>
  );
}
