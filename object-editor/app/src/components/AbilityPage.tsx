import { Copy01 } from "@untitledui/icons";
import { useMemo, useState } from "react";
import type { AbilityDetail, AbilitySection, Field, LevelField } from "../types";
import { capitalized, levelCount, variationModel } from "../ui";
import { Button } from "./Button";
import { GameText } from "./GameText";
import { Inspector } from "./Inspector";
import { LevelTable } from "./LevelTable";
import { LevelText } from "./LevelText";
import { AdvancedGroup, FieldHeader, FieldRows, matches, SectionToggle, toggled } from "./Sections";
import { UnitIcon } from "./UnitIcon";

/** The object types this page shows: abilities, and upgrades and doodads shaped like them. */
type Noun = "ability" | "upgrade" | "doodad";

const PLURALS: Record<Noun, string> = {
  ability: "abilities",
  upgrade: "upgrades",
  doodad: "doodads",
};
/** What a column of the level tables is called. */
const LEVEL_NAMES: Record<Noun, string> = {
  ability: "Level",
  upgrade: "Level",
  doodad: "Variation",
};
const DOODAD_MODEL = "dfil";
const FILTER_EXAMPLES: Record<Noun, string> = {
  ability: "Filter fields: cooldown, damage, icon…",
  upgrade: "Filter fields: gold, effect, icon…",
  doodad: "Filter fields: model, tint, size…",
};

type AbilityPageProps = {
  detail: AbilityDetail;
  /** Defaults to "ability". */
  noun?: Noun;
  editable: boolean;
  /** `level` is 0 for fields without levels. */
  onChange: (field: string, level: number, value: string) => void;
  onFill: (field: string, fromLevel: number, value: string) => void;
  onReset: (field: string, level: number) => void;
  onCreateCopy: () => void;
};

/** Sections open by default; the rest start folded to keep the page short. */
const OPEN_BY_DEFAULT = ["stats", "data", "text"];
/** Doodads have no stats or data: their art is what people change. */
const DOODAD_OPEN_BY_DEFAULT = ["text", "art"];
const NO_LEVEL = 0;
/** Keys of the level tables, shown under the page header ("level" or "variation"). */
function levelKeys(level: string): [string, string][] {
  return [
    ["Arrows", "move"],
    ["Enter", "edits"],
    ["Enter or Tab", `saves and moves to the next ${level}`],
    ["Ctrl+Enter", `fills the later ${level}s too`],
    ["Delete", "resets"],
    ["Esc", "cancels"],
  ];
}

type Shown = {
  section: AbilitySection;
  fields: Field[];
  levelFields: LevelField[];
};

function isText(field: LevelField): boolean {
  return field.field.control === "text" || field.field.control === "longText";
}

function levelFieldMatches(field: LevelField, filter: string, changedOnly: boolean): boolean {
  const changed = field.cells.some((c) => c.modified);
  return (!changedOnly || changed) && (!filter || matches(field.field, filter));
}

function visible(section: AbilitySection, filter: string, changedOnly: boolean): Shown {
  return {
    section,
    fields: section.fields.filter(
      (f) => (!changedOnly || f.modified) && (!filter || matches(f, filter)),
    ),
    levelFields: section.levelFields.filter((f) => levelFieldMatches(f, filter, changedOnly)),
  };
}

function changedIn(fields: Field[], levelFields: LevelField[]): number {
  const cells = levelFields.flatMap((f) => f.cells).filter((c) => c.modified).length;
  return fields.filter((f) => f.modified).length + cells;
}

/** Header of the base column: the ability a custom one copies, else the game's value. */
function baseLabel(detail: AbilityDetail): string {
  const { summary } = detail.card;
  if (summary.custom) {
    return `Base · ${summary.base}`;
  }

  return "Game value";
}

/** "s" for counts other than 1. */
function plural(count: number): string {
  if (count === 1) {
    return "";
  }

  return "s";
}

function facts(detail: AbilityDetail, noun: Noun): string {
  const { summary, hotkey, requiredLevel } = detail.card;
  let kind = "Unit ability";
  if (noun === "doodad") {
    const variations = `${detail.levels} variation${plural(detail.levels)}`;
    return ["Doodad", summary.race, variations].filter(Boolean).join(" · ");
  }

  if (noun === "upgrade") {
    kind = "Upgrade";
  } else if (summary.hero) {
    kind = "Hero ability";
  } else if (summary.item) {
    kind = "Item ability";
  }

  return [
    kind,
    summary.race && capitalized(summary.race),
    levelCount(detail.levels),
    summary.hero && requiredLevel > 0 && `learned at hero level ${requiredLevel}`,
    hotkey && `hotkey ${hotkey}`,
  ]
    .filter(Boolean)
    .join(" · ");
}

export function AbilityPage({
  detail,
  noun = "ability",
  editable,
  onChange,
  onFill,
  onReset,
  onCreateCopy,
}: AbilityPageProps) {
  const [filter, setFilter] = useState("");
  const [changedOnly, setChangedOnly] = useState(false);
  const [open, setOpen] = useState<string[]>(() => {
    if (noun === "doodad") {
      return DOODAD_OPEN_BY_DEFAULT;
    }

    return OPEN_BY_DEFAULT;
  });
  const [openAdvanced, setOpenAdvanced] = useState<string[]>([]);
  const [inspected, setInspected] = useState<string | null>(null);
  const { summary, description } = detail.card;
  const single = detail.sections.flatMap((s) => s.fields);
  const perLevel = detail.sections.flatMap((s) => s.levelFields.map((f) => f.field));
  const inspectedField =
    single.find((f) => f.id === inspected) ?? perLevel.find((f) => f.id === inspected) ?? null;
  const inspect = (field: Field) => setInspected(field.id);
  const searching = Boolean(filter) || changedOnly;
  // Doodads show their model; abilities and upgrades have none to show.
  const doodadModel = single.find((f) => f.id === DOODAD_MODEL)?.raw;
  let modelPath: string | null = null;
  if (noun === "doodad" && doodadModel) {
    modelPath = variationModel(doodadModel, detail.levels);
  }
  const changedCount = detail.sections.reduce(
    (sum, s) => sum + changedIn(s.fields, s.levelFields),
    0,
  );

  const sections = useMemo(
    () =>
      detail.sections
        .map((s) => visible(s, filter, changedOnly))
        .filter((s) => s.fields.length + s.levelFields.length > 0),
    [detail, filter, changedOnly],
  );

  /** Single fields, then the per-level table, then per-level text, for one group of fields. */
  function body(fields: Field[], levelFields: LevelField[]) {
    return (
      <>
        <FieldRows
          fields={fields}
          editable={editable}
          damageBonus={null}
          onChange={(field, value) => onChange(field, NO_LEVEL, value)}
          onReset={(field) => onReset(field, NO_LEVEL)}
          onInspect={inspect}
        />
        <LevelTable
          fields={levelFields.filter((f) => !isText(f))}
          levelName={LEVEL_NAMES[noun]}
          editable={editable}
          onChange={onChange}
          onFill={onFill}
          onReset={onReset}
          onInspect={inspect}
        />
        {levelFields.filter(isText).map((field) => (
          <LevelText
            key={field.field.id}
            field={field}
            levelName={LEVEL_NAMES[noun]}
            editable={editable}
            onChange={(level, value) => onChange(field.field.id, level, value)}
            onReset={(level) => onReset(field.field.id, level)}
            onInspect={inspect}
          />
        ))}
      </>
    );
  }

  return (
    <div className="flex min-h-0 min-w-0 flex-1 gap-2.5">
      <section className="frame flex min-h-0 min-w-0 flex-1 flex-col bg-surface-1">
        <header className="flex flex-col gap-4 border-b border-line p-5">
          <div className="flex items-center gap-4">
            <UnitIcon path={summary.icon} size="large" />
            <div className="min-w-0 flex-1">
              <h2 className="truncate font-display text-3xl font-bold text-gold-bright">
                <GameText text={summary.name} singleLine />
                {summary.suffix && (
                  <span className="ml-2 text-base font-normal text-faint">
                    <GameText text={summary.suffix} singleLine />
                  </span>
                )}
              </h2>
              <p className="font-mono text-xs text-faint">
                {summary.id}
                {summary.custom && ` · based on ${summary.base}`}
              </p>
              <p className="text-sm text-muted">{facts(detail, noun)}</p>
            </div>
            {editable && (
              <Button onPress={onCreateCopy}>
                <span className="flex items-center gap-1.5">
                  <Copy01 className="size-4" aria-hidden="true" />
                  New {noun} based on this
                </span>
              </Button>
            )}
          </div>

          {description && (
            <div className="max-h-32 overflow-y-auto rounded-[3px] border border-line bg-well px-3 py-2 text-sm leading-relaxed">
              <GameText text={description} />
            </div>
          )}

          <div className="flex items-center gap-3">
            <input
              type="search"
              value={filter}
              onChange={(e) => setFilter(e.target.value)}
              placeholder={FILTER_EXAMPLES[noun]}
              aria-label="Filter fields"
              className="min-w-0 flex-1 rounded-[3px] border border-field bg-well px-3 py-1.5 outline-none focus:border-glow"
            />
            <label className="flex items-center gap-2 text-muted">
              <input
                type="checkbox"
                checked={changedOnly}
                onChange={(e) => setChangedOnly(e.target.checked)}
              />
              Changed only ({changedCount})
            </label>
          </div>

          {!editable && (
            <p className="text-sm text-faint">
              Browsing game data. Open a map to edit {PLURALS[noun]}.
            </p>
          )}
          {editable && (
            <ul
              aria-label="Level table keys"
              className="flex flex-wrap gap-x-4 gap-y-1 text-xs text-faint"
            >
              {levelKeys(LEVEL_NAMES[noun].toLowerCase()).map(([key, action]) => (
                <li key={key}>
                  <kbd className="rounded-[3px] border border-line px-1 font-sans text-muted">
                    {key}
                  </kbd>{" "}
                  {action}
                </li>
              ))}
            </ul>
          )}
        </header>

        <div className="min-h-0 flex-1 overflow-y-auto">
          <FieldHeader base={baseLabel(detail)} />
          {sections.length === 0 && <p className="p-5 text-muted">No fields match.</p>}
          {sections.map(({ section, fields, levelFields }) => {
            const expanded = searching || open.includes(section.category);
            const advancedFields = fields.filter((f) => f.advanced);
            const advancedLevels = levelFields.filter((f) => f.field.advanced);

            return (
              <div key={section.category}>
                <SectionToggle
                  label={section.label}
                  count={fields.length + levelFields.length}
                  changed={changedIn(fields, levelFields)}
                  expanded={expanded}
                  onToggle={() => setOpen(toggled(open, section.category))}
                />
                {expanded &&
                  body(
                    fields.filter((f) => !f.advanced),
                    levelFields.filter((f) => !f.field.advanced),
                  )}
                {expanded && (
                  <AdvancedGroup
                    count={advancedFields.length + advancedLevels.length}
                    changed={changedIn(advancedFields, advancedLevels)}
                    open={searching || openAdvanced.includes(section.category)}
                    onToggle={() => setOpenAdvanced(toggled(openAdvanced, section.category))}
                  >
                    {body(advancedFields, advancedLevels)}
                  </AdvancedGroup>
                )}
              </div>
            );
          })}
        </div>
      </section>
      <Inspector
        {...(noun === "doodad" && { modelPath })}
        field={inspectedField}
        perLevel={inspectedField !== null && perLevel.includes(inspectedField)}
        editable={editable}
        onReset={(field) => onReset(field, NO_LEVEL)}
      />
    </div>
  );
}
