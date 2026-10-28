import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { api } from "../api";
import { settle } from "../result";
import type { ConstantGroup, ConstantsView, ConstantView } from "../types";
import { classes, counted } from "../ui";
import { Button } from "./Button";
import { Checkbox } from "./Checkbox";
import { Chip } from "./Chip";

const DAMAGE_TABLE = ["ctno", "ctpi", "ctsi", "ctma", "ctch", "ctsp", "cthe"];
const ATTACK_TYPES = ["Normal", "Pierce", "Siege", "Magic", "Chaos", "Spells", "Hero"];
const DEFENSE_TYPES = [
  "Small",
  "Medium",
  "Large",
  "Fortified",
  "Normal",
  "Hero",
  "Divine",
  "Unarmored",
];
const ON = "1";
const OFF = "0";

type ConstantsPanelProps = {
  onMapChanged: () => void;
  onError: (message: string | null) => void;
};

type CommitTextProps = {
  label: string;
  value: string;
  wide?: boolean;
  onCommit: (value: string) => void;
};

export function CommitText({ label, value, wide, onCommit }: CommitTextProps) {
  const [text, setText] = useState(value);

  useEffect(() => setText(value), [value]);

  function commit() {
    if (text.trim() !== value) {
      onCommit(text.trim());
    }
  }

  return (
    <input
      aria-label={label}
      value={text}
      onChange={(e) => setText(e.target.value)}
      onBlur={commit}
      onKeyDown={(e) => {
        if (e.key === "Enter") {
          commit();
        }
      }}
      className={classes(
        "rounded-[3px] border border-field bg-well px-2 py-1 text-sm text-ink outline-none focus:border-focus",
        wide && "w-full",
        !wide && "w-36 text-right font-mono",
      )}
    />
  );
}

function switchValue(on: boolean): string {
  if (on) {
    return ON;
  }

  return OFF;
}

function listItem(value: string, at: number): string {
  return value.split(",")[at]?.trim() ?? "";
}

function withItem(value: string, at: number, item: string, length: number): string {
  const items = Array.from({ length }, (_, i) => listItem(value, i));

  items[at] = item;
  return items.join(",");
}

function matches(constant: ConstantView, needle: string): boolean {
  return (
    !needle ||
    constant.label.toLowerCase().includes(needle) ||
    constant.description.toLowerCase().includes(needle) ||
    constant.id.includes(needle)
  );
}

export function ConstantsPanel({ onMapChanged, onError }: ConstantsPanelProps) {
  const [view, setView] = useState<ConstantsView | null>(null);
  const [query, setQuery] = useState("");
  const [changedOnly, setChangedOnly] = useState(false);
  const queue = useRef<Promise<void>>(Promise.resolve());

  const reload = useCallback(() => {
    void settle(api.gameplayConstants(), setView, onError);
  }, [onError]);

  useEffect(() => {
    reload();
  }, [reload]);

  function run(work: () => Promise<unknown>) {
    queue.current = queue.current.then(async () => {
      await work();
      reload();
      onMapChanged();
    });
  }

  function setValue(id: string, value: string | null) {
    run(() => settle(api.setGameplayConstant(id, value), () => onError(null), onError));
  }

  function setCustom(custom: boolean) {
    run(() => settle(api.setCustomConstants(custom), () => onError(null), onError));
  }

  const needle = query.trim().toLowerCase();
  const groups = useMemo(
    () =>
      (view?.groups ?? [])
        .map((g) => ({
          ...g,
          constants: g.constants.filter((c) => matches(c, needle) && (!changedOnly || c.changed)),
        }))
        .filter((g) => g.constants.length > 0),
    [view, needle, changedOnly],
  );
  const changed = (view?.groups ?? []).flatMap((g) => g.constants).filter((c) => c.changed).length;

  if (!view) {
    return <p className="m-auto text-muted">Loading the gameplay constants…</p>;
  }

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <div className="flex flex-col gap-3 border-b border-line px-5 py-3">
        <Checkbox isSelected={view.custom} onChange={setCustom}>
          <span className="text-ink">Use custom gameplay constants</span>
        </Checkbox>
        <p className="text-xs text-faint">
          {view.custom &&
            `The map changes ${counted(changed, "constant")}; the rest keep the game's values. Turning this off drops the changes when you save.`}
          {!view.custom &&
            "The map uses the game's values. Changing any constant turns this on; only the constants you change are stored in the map."}
        </p>
        <div className="flex flex-wrap items-center gap-2">
          <input
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="Search constants: experience, decay, upkeep…"
            aria-label="Search gameplay constants"
            className="min-w-0 flex-1 rounded-[3px] border border-field bg-well px-2.5 py-1.5 text-sm text-ink outline-none focus:border-focus"
          />
          <Chip
            label="Changed only"
            on={changedOnly}
            onPress={() => setChangedOnly(!changedOnly)}
          />
        </div>
      </div>
      <div className="flex min-h-0 flex-1 flex-col gap-6 overflow-y-auto p-5">
        {groups.length === 0 && <p className="text-sm text-faint">No constant matches.</p>}
        {groups.map((group) => (
          <ConstantSection key={group.id} group={group} onSet={setValue} />
        ))}
      </div>
    </div>
  );
}

type ConstantSectionProps = {
  group: ConstantGroup;
  onSet: (id: string, value: string | null) => void;
};

function ConstantSection({ group, onSet }: ConstantSectionProps) {
  const table = group.constants.filter((c) => DAMAGE_TABLE.includes(c.id));
  const rows = group.constants.filter((c) => !DAMAGE_TABLE.includes(c.id));
  const basic = rows.filter((c) => !c.advanced);
  const advanced = rows.filter((c) => c.advanced);

  return (
    <section aria-labelledby={`constants-${group.id}`} className="flex flex-col gap-2">
      <h3 id={`constants-${group.id}`} className="font-display text-xl font-bold text-gold-bright">
        {group.label}
      </h3>
      {table.length > 0 && <DamageTable rows={table} onSet={onSet} />}
      {basic.map((c) => (
        <ConstantRow key={c.id} constant={c} onSet={onSet} />
      ))}
      {advanced.length > 0 && (
        <details className="rounded-[3px] border border-line">
          <summary className="cursor-pointer px-3 py-2 text-sm text-muted">
            Advanced ({advanced.length})
          </summary>
          <div className="flex flex-col gap-2 p-2">
            {advanced.map((c) => (
              <ConstantRow key={c.id} constant={c} onSet={onSet} />
            ))}
          </div>
        </details>
      )}
    </section>
  );
}

type ConstantRowProps = {
  constant: ConstantView;
  onSet: (id: string, value: string | null) => void;
};

function ConstantRow({ constant, onSet }: ConstantRowProps) {
  const textual = constant.valueType === "string" || constant.valueType === "unitList";

  return (
    <div
      className={classes(
        "flex flex-col gap-1 rounded-[3px] px-3 py-2",
        constant.changed && "bg-gold-soft/40",
      )}
    >
      <div className="flex flex-wrap items-center gap-3">
        <span className="min-w-0 flex-1 text-sm text-ink">
          {constant.label}
          {constant.changed && (
            <span className="ml-2 text-xs font-semibold text-gold-bright">changed</span>
          )}
        </span>
        {constant.valueType === "bool" && (
          <Checkbox
            isSelected={constant.value === ON}
            onChange={(on) => onSet(constant.id, switchValue(on))}
            aria-label={constant.label}
          >
            <span className="sr-only">{constant.label}</span>
          </Checkbox>
        )}
        {constant.valueType !== "bool" && !textual && (
          <CommitText
            label={constant.label}
            value={constant.value}
            onCommit={(value) => onSet(constant.id, value)}
          />
        )}
        {constant.changed && (
          <Button small onPress={() => onSet(constant.id, null)}>
            Reset
          </Button>
        )}
      </div>
      {textual && (
        <CommitText
          label={constant.label}
          value={constant.value}
          wide
          onCommit={(value) => onSet(constant.id, value)}
        />
      )}
      <p className="text-xs text-muted">
        {constant.description}
        {constant.unverified && <span className="ml-1 text-faint">(not confirmed in game)</span>}
      </p>
      <p className="font-mono text-xs text-faint">
        Game: {constant.default || "(empty)"} · {constant.id}
      </p>
    </div>
  );
}

type DamageTableProps = {
  rows: ConstantView[];
  onSet: (id: string, value: string | null) => void;
};

function DamageTable({ rows, onSet }: DamageTableProps) {
  const ordered = DAMAGE_TABLE.map((id) => rows.find((r) => r.id === id)).filter(
    (r): r is ConstantView => r !== undefined,
  );
  const description = ordered[0]?.description ?? "";

  return (
    <div className="flex flex-col gap-2 rounded-[3px] border border-line p-3">
      <p className="text-sm text-ink">
        Damage table: attack type (rows) against defense type (columns)
      </p>
      <p className="text-xs text-muted">{description}</p>
      <div className="overflow-x-auto">
        <table className="text-sm">
          <thead>
            <tr>
              <th className="px-2 py-1 text-left text-xs font-normal text-faint">Attack</th>
              {DEFENSE_TYPES.map((d) => (
                <th key={d} scope="col" className="px-1 py-1 text-xs font-normal text-faint">
                  {d}
                </th>
              ))}
              <th aria-label="Reset" />
            </tr>
          </thead>
          <tbody>
            {ordered.map((row, at) => (
              <tr key={row.id} className={classes(row.changed && "bg-gold-soft/40")}>
                <th scope="row" className="px-2 py-1 text-left font-normal text-ink">
                  {ATTACK_TYPES[at]}
                </th>
                {DEFENSE_TYPES.map((d, column) => (
                  <td key={d} className="px-1 py-0.5">
                    <TableCell
                      label={`${ATTACK_TYPES[at]} against ${d}`}
                      value={listItem(row.value, column)}
                      changed={listItem(row.value, column) !== listItem(row.default, column)}
                      onCommit={(item) =>
                        onSet(row.id, withItem(row.value, column, item, DEFENSE_TYPES.length))
                      }
                    />
                  </td>
                ))}
                <td className="px-1">
                  {row.changed && (
                    <Button small onPress={() => onSet(row.id, null)}>
                      Reset
                    </Button>
                  )}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}

type TableCellProps = {
  label: string;
  value: string;
  changed: boolean;
  onCommit: (value: string) => void;
};

function TableCell({ label, value, changed, onCommit }: TableCellProps) {
  const [text, setText] = useState(value);

  useEffect(() => setText(value), [value]);

  function commit() {
    if (text.trim() !== value) {
      onCommit(text.trim());
    }
  }

  return (
    <input
      aria-label={label}
      value={text}
      onChange={(e) => setText(e.target.value)}
      onBlur={commit}
      onKeyDown={(e) => {
        if (e.key === "Enter") {
          commit();
        }
      }}
      className={classes(
        "w-16 rounded-[3px] border bg-well px-1.5 py-0.5 text-right font-mono text-xs outline-none focus:border-focus",
        changed && "border-gold text-gold-bright",
        !changed && "border-field text-ink",
      )}
    />
  );
}
