import { useMemo, useState } from "react";
import type { Field, UnitDetail, UnitItem } from "../types";
import { classes } from "../ui";
import { CommitInput } from "./CommitInput";
import { GameText, plainGameText } from "./GameText";
import { UnitIcon } from "./UnitIcon";

/** Which buff the spell leaves on its target. */
export type BuffChoice =
  | { kind: "base" }
  | { kind: "own"; id: string }
  | { kind: "other"; id: string };

type SpellBuffStepProps = {
  /** The base ability's name, as the player knows it. */
  baseName: string;
  /** The base ability's buffs at level 1; empty when it leaves none. */
  baseBuffs: string[];
  choice: BuffChoice;
  /** The buff the spell uses now: the base's, its own, or the one picked. */
  shown: UnitDetail | null;
  /** Every buff, the map's first, for "another buff". */
  allBuffs: UnitItem[];
  busy: boolean;
  onOwn: () => void;
  onBase: () => void;
  onOther: (id: string) => void;
  /** Edits the spell's own buff. */
  onEditOwn: (field: string, value: string) => void;
};

const TITLE = "ftip";
const TEXT = "fube";
/** Buffs listed at once in "another buff"; typing narrows the list. */
const MAX_LISTED = 40;

function buffField(detail: UnitDetail | null, id: string): Field | undefined {
  return detail?.sections.flatMap((s) => s.fields).find((f) => f.id === id);
}

/**
 * The spell wizard's buff step: the mark the spell leaves on its target (its
 * icon in the status bar and its tooltip). A spell can keep the base
 * ability's buff, get its own copy named after it, or use another one.
 */
export function SpellBuffStep({
  baseName,
  baseBuffs,
  choice,
  shown,
  allBuffs,
  busy,
  onOwn,
  onBase,
  onOther,
  onEditOwn,
}: SpellBuffStepProps) {
  const [query, setQuery] = useState("");
  const listed = useMemo(() => {
    const needle = query.trim().toLowerCase();
    return allBuffs
      .filter(
        (b) =>
          !needle ||
          plainGameText(b.name).toLowerCase().includes(needle) ||
          b.id.toLowerCase().includes(needle),
      )
      .slice(0, MAX_LISTED);
  }, [allBuffs, query]);

  if (baseBuffs.length === 0) {
    return (
      <p className="text-muted">
        Spells based on <GameText text={baseName} singleLine /> leave no buff on their target, so
        there is nothing to set here.
      </p>
    );
  }

  const title = buffField(shown, TITLE);
  const text = buffField(shown, TEXT);
  const option = (checked: boolean) =>
    classes(
      "flex cursor-pointer gap-3 rounded-[3px] border px-3 py-2",
      checked && "border-gold bg-surface-2",
      !checked && "border-line bg-well hover:border-gold",
    );

  return (
    <>
      <p className="text-muted">
        A buff is the mark the spell leaves on its target: an icon in the unit's status bar, a
        tooltip, and sometimes art on the unit.
      </p>
      <fieldset className="flex flex-col gap-2" disabled={busy}>
        <legend className="sr-only">Buff</legend>
        <label className={option(choice.kind === "own")}>
          <input type="radio" name="buff" checked={choice.kind === "own"} onChange={onOwn} />
          <span>
            <span className="block font-semibold">Its own buff (recommended)</span>
            <span className="block text-sm text-muted">
              A copy of <GameText text={baseName} singleLine />
              's buff with your spell's name and tooltip, so players can tell them apart.
            </span>
          </span>
        </label>
        <label className={option(choice.kind === "base")}>
          <input type="radio" name="buff" checked={choice.kind === "base"} onChange={onBase} />
          <span>
            <span className="block font-semibold">
              The same buff as <GameText text={baseName} singleLine />
            </span>
            <span className="block text-sm text-muted">
              Units hit show the original spell's icon and tooltip.
            </span>
          </span>
        </label>
        <div className={option(choice.kind === "other")}>
          <span className="flex min-w-0 flex-1 flex-col gap-2">
            <span className="block font-semibold">Another buff</span>
            <input
              type="search"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder="Search buffs: Stunned, Slowed, BPSE…"
              aria-label="Search buffs"
              className="rounded-[3px] border border-field bg-well px-3 py-1.5 outline-none focus:border-glow"
            />
            <ul aria-label="Buffs" className="max-h-40 overflow-y-auto">
              {listed.map((buff) => (
                <li key={buff.id}>
                  <button
                    type="button"
                    onClick={() => onOther(buff.id)}
                    aria-pressed={choice.kind === "other" && choice.id === buff.id}
                    className="flex w-full items-center gap-2 px-2 py-1 text-left hover:bg-surface-2 aria-pressed:bg-surface-2"
                  >
                    <UnitIcon path={buff.icon} size="small" />
                    <span className="min-w-0 flex-1 truncate">
                      <GameText text={buff.name} singleLine />
                    </span>
                    <span className="font-mono text-xs text-faint">{buff.id}</span>
                  </button>
                </li>
              ))}
            </ul>
          </span>
        </div>
      </fieldset>

      {shown && (
        <section
          aria-label="Buff"
          className="flex flex-col gap-2 rounded-[3px] border border-line p-3"
        >
          <div className="flex items-center gap-3">
            <UnitIcon path={shown.item.icon} size="large" />
            <div className="min-w-0">
              <p className="font-semibold">
                <GameText text={shown.item.name} singleLine />
              </p>
              <p className="font-mono text-xs text-faint">{shown.item.id}</p>
            </div>
          </div>
          {choice.kind === "own" && title && text && (
            <>
              <CommitInput
                label="Tooltip title"
                field={title}
                onCommit={(value) => onEditOwn(TITLE, value)}
              />
              <CommitInput
                label="Tooltip text"
                field={text}
                multiline
                onCommit={(value) => onEditOwn(TEXT, value)}
              />
              <p className="text-xs text-faint">
                Its icon and art can be changed on its page in the Buffs tab afterwards.
              </p>
            </>
          )}
          {choice.kind !== "own" && title && (
            <div className="rounded border border-line bg-well px-2.5 py-1.5 text-sm">
              <p className="font-semibold">
                <GameText text={title.raw} singleLine />
              </p>
              {text && <GameText text={text.raw} />}
            </div>
          )}
        </section>
      )}
    </>
  );
}
