import { useMemo, useState } from "react";
import type { AbilityDetail, IconChoice } from "../types";
import { classes } from "../ui";
import { UnitIcon } from "./UnitIcon";

/** A spot on a 4×3 command card, as `abpx`/`abpy` store it. */
type Spot = { x: number; y: number };

/** A command the game puts on the card itself, which a spell's button would hide. */
type Command = Spot & { label: string; hotkey: string };

type SpellButtonStepProps = {
  detail: AbilityDetail;
  icons: IconChoice[];
  /** Sets a field without levels, such as the icon or the button's spot. */
  onSet: (field: string, value: string) => void;
  onIcon: (path: string) => void;
  onHotkey: (field: string, key: string) => void;
};

const COLUMNS = 4;
const ROWS = 3;
/** Icons drawn at once; typing narrows the list further. */
const MAX_ICONS = 120;

/** The commands every unit has: Move, Stop, Hold Position, Attack and Patrol. */
const UNIT_COMMANDS: Command[] = [
  { x: 0, y: 0, label: "Move", hotkey: "M" },
  { x: 1, y: 0, label: "Stop", hotkey: "S" },
  { x: 2, y: 0, label: "Hold Position", hotkey: "H" },
  { x: 3, y: 0, label: "Attack", hotkey: "A" },
  { x: 0, y: 1, label: "Patrol", hotkey: "P" },
];
/** Heroes also have the button that opens their learn menu. */
const HERO_COMMANDS: Command[] = [...UNIT_COMMANDS, { x: 3, y: 1, label: "Learn", hotkey: "O" }];
/** The learn menu's own button: Cancel, bottom right, on Esc. */
const LEARN_COMMANDS: Command[] = [{ x: 3, y: 2, label: "Cancel", hotkey: "Esc" }];

const HOTKEY = "ahky";
const LEARN_HOTKEY = "arhk";
const BUTTON = { x: "abpx", y: "abpy" };
const LEARN_BUTTON = { x: "arpx", y: "arpy" };

function raw(detail: AbilityDetail, id: string): string {
  return detail.sections.flatMap((s) => s.fields).find((f) => f.id === id)?.raw ?? "";
}

function spot(detail: AbilityDetail, fields: { x: string; y: string }): Spot {
  return { x: Number(raw(detail, fields.x)) || 0, y: Number(raw(detail, fields.y)) || 0 };
}

/** What the player would lose, in plain words; nothing when the button fits. */
function conflicts(at: Spot, hotkey: string, commands: Command[]): string[] {
  const found: string[] = [];
  const covered = commands.find((c) => c.x === at.x && c.y === at.y);
  if (covered) {
    found.push(`This spot holds ${covered.label}; the spell's button would hide it.`);
  }

  const shared = commands.find((c) => c.hotkey === hotkey.toUpperCase());
  if (shared) {
    found.push(`${hotkey.toUpperCase()} is already the hotkey of ${shared.label}.`);
  }

  return found;
}

/**
 * The spell wizard's button step: the spell's icon, its hotkey and where its
 * button sits on the command card (and on a hero's learn menu). Warns when
 * the button would hide a command every unit has or share its hotkey.
 */
export function SpellButtonStep({ detail, icons, onSet, onIcon, onHotkey }: SpellButtonStepProps) {
  const { summary } = detail.card;
  const hero = summary.hero;
  const button = spot(detail, BUTTON);
  const learn = spot(detail, LEARN_BUTTON);
  const hotkey = raw(detail, HOTKEY);
  const learnHotkey = raw(detail, LEARN_HOTKEY);
  let commands = UNIT_COMMANDS;
  if (hero) {
    commands = HERO_COMMANDS;
  }

  return (
    <>
      <IconPicker current={summary.icon} icons={icons} onPick={onIcon} />
      <div className="flex flex-wrap gap-8">
        <CardEditor
          title="Command card"
          spot={button}
          icon={summary.icon}
          commands={commands}
          hotkey={hotkey}
          onMove={(at) => {
            onSet(BUTTON.x, String(at.x));
            onSet(BUTTON.y, String(at.y));
          }}
          onHotkey={(key) => onHotkey(HOTKEY, key)}
        />
        {hero && (
          <CardEditor
            title="Learn menu"
            spot={learn}
            icon={summary.icon}
            commands={LEARN_COMMANDS}
            hotkey={learnHotkey}
            onMove={(at) => {
              onSet(LEARN_BUTTON.x, String(at.x));
              onSet(LEARN_BUTTON.y, String(at.y));
            }}
            onHotkey={(key) => onHotkey(LEARN_HOTKEY, key)}
          />
        )}
      </div>
      <p className="text-xs text-faint">
        The next step checks the button against the abilities of each unit you give the spell to.
      </p>
    </>
  );
}

type CardEditorProps = {
  title: string;
  spot: Spot;
  icon: string | null;
  commands: Command[];
  hotkey: string;
  onMove: (at: Spot) => void;
  onHotkey: (key: string) => void;
};

/** A 4×3 card with the spell's button where it sits, and the hotkey that presses it. */
function CardEditor({ title, spot, icon, commands, hotkey, onMove, onHotkey }: CardEditorProps) {
  const warnings = conflicts(spot, hotkey, commands);
  const cells = Array.from({ length: COLUMNS * ROWS }, (_, i) => ({
    x: i % COLUMNS,
    y: Math.floor(i / COLUMNS),
  }));

  return (
    <section aria-label={title} className="flex flex-col gap-2">
      <h3 className="font-semibold text-gold-bright">{title}</h3>
      <fieldset aria-label={`${title}: the spell's spot`} className="grid w-fit grid-cols-4 gap-1">
        {cells.map((cell) => {
          const here = cell.x === spot.x && cell.y === spot.y;
          const command = commands.find((c) => c.x === cell.x && c.y === cell.y);
          return (
            <button
              key={`${cell.x}-${cell.y}`}
              type="button"
              aria-pressed={here}
              aria-label={cellLabel(cell, here, command)}
              onClick={() => onMove(cell)}
              className={classes(
                "flex size-12 items-center justify-center rounded-[3px] border text-[10px] leading-tight text-faint outline-none focus-visible:border-focus",
                here && "border-gold",
                !here && "border-line bg-well hover:border-gold",
              )}
            >
              {here && <UnitIcon path={icon} size="small" />}
              {!here && command?.label}
            </button>
          );
        })}
      </fieldset>
      <label className="flex items-center gap-2 text-sm text-muted">
        Hotkey
        <input
          value={hotkey}
          maxLength={1}
          onChange={(e) => {
            const key = e.target.value.toUpperCase();
            if (key) {
              onHotkey(key);
            }
          }}
          aria-label={`${title} hotkey`}
          className="w-10 rounded-[3px] border border-field bg-well px-2 py-1 text-center font-mono text-ink outline-none focus:border-focus"
        />
      </label>
      {warnings.map((warning) => (
        <p key={warning} role="status" className="text-sm text-gold">
          {warning}
        </p>
      ))}
    </section>
  );
}

function cellLabel(cell: Spot, here: boolean, command: Command | undefined): string {
  const place = `Column ${cell.x + 1}, row ${cell.y + 1}`;
  if (here) {
    return `${place}: the spell`;
  }

  if (command) {
    return `${place}: ${command.label}`;
  }

  return `${place}: empty`;
}

type IconPickerProps = {
  current: string | null;
  icons: IconChoice[];
  onPick: (path: string) => void;
};

/** The spell's icon, and a searchable grid of the game's and the map's icons. */
function IconPicker({ current, icons, onPick }: IconPickerProps) {
  const [query, setQuery] = useState("");
  /** Icons the game data names but whose image does not load; left out once seen. */
  const [missing, setMissing] = useState<ReadonlySet<string>>(new Set());
  const listed = useMemo(() => {
    const needle = query.trim().toLowerCase();
    return icons
      .filter((i) => !missing.has(i.path) && (!needle || i.name.toLowerCase().includes(needle)))
      .slice(0, MAX_ICONS);
  }, [icons, query, missing]);

  return (
    <section aria-label="Icon" className="flex flex-col gap-2">
      <div className="flex items-center gap-3">
        <UnitIcon path={current} size="large" />
        <input
          type="search"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder="Search icons: fire, bolt, BTNStorm…"
          aria-label="Search icons"
          className="min-w-0 flex-1 rounded-[3px] border border-field bg-well px-3 py-1.5 outline-none focus:border-focus"
        />
      </div>
      <ul
        aria-label="Icons"
        className="grid max-h-48 grid-cols-[repeat(auto-fill,2.25rem)] gap-1 overflow-y-auto"
      >
        {listed.map((icon) => (
          <li key={icon.path}>
            <button
              type="button"
              onClick={() => onPick(icon.path)}
              aria-pressed={icon.path === current}
              title={describe(icon)}
              aria-label={describe(icon)}
              className="rounded-[3px] outline-none hover:ring-2 hover:ring-gold focus-visible:ring-2 focus-visible:ring-focus aria-pressed:ring-2 aria-pressed:ring-gold"
            >
              <UnitIcon
                path={icon.path}
                size="small"
                onMissing={() => setMissing((seen) => new Set(seen).add(icon.path))}
              />
            </button>
          </li>
        ))}
      </ul>
    </section>
  );
}

function describe(icon: IconChoice): string {
  if (icon.fromMap) {
    return `${icon.name} (in this map)`;
  }

  return icon.name;
}
