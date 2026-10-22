import { Copy01 } from "@untitledui/icons";
import { useMemo, useState } from "react";
import type { AbilityDetail, AbilitySection, Field, LevelField } from "../types";
import { capitalized, levelCount } from "../ui";
import { Button } from "./Button";
import { GameText } from "./GameText";
import { LevelTable } from "./LevelTable";
import { LevelText } from "./LevelText";
import { AdvancedGroup, FieldRows, matches, SectionToggle, toggled } from "./Sections";
import { UnitIcon } from "./UnitIcon";

type AbilityPageProps = {
  detail: AbilityDetail;
  editable: boolean;
  /** `level` is 0 for fields without levels. */
  onChange: (field: string, level: number, value: string) => void;
  onFill: (field: string, fromLevel: number, value: string) => void;
  onReset: (field: string, level: number) => void;
  onCreateCopy: () => void;
};

/** Sections open by default; the rest start folded to keep the page short. */
const OPEN_BY_DEFAULT = ["stats", "data", "text"];
const NO_LEVEL = 0;

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

function facts(detail: AbilityDetail): string {
  const { summary, hotkey, requiredLevel } = detail.card;
  let kind = "Unit ability";
  if (summary.hero) {
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
  editable,
  onChange,
  onFill,
  onReset,
  onCreateCopy,
}: AbilityPageProps) {
  const [filter, setFilter] = useState("");
  const [changedOnly, setChangedOnly] = useState(false);
  const [open, setOpen] = useState<string[]>(OPEN_BY_DEFAULT);
  const [openAdvanced, setOpenAdvanced] = useState<string[]>([]);
  const { summary, description } = detail.card;
  const searching = Boolean(filter) || changedOnly;
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
        />
        <LevelTable
          fields={levelFields.filter((f) => !isText(f))}
          editable={editable}
          onChange={onChange}
          onFill={onFill}
          onReset={onReset}
        />
        {levelFields.filter(isText).map((field) => (
          <LevelText
            key={field.field.id}
            field={field}
            editable={editable}
            onChange={(level, value) => onChange(field.field.id, level, value)}
            onReset={(level) => onReset(field.field.id, level)}
          />
        ))}
      </>
    );
  }

  return (
    <section className="flex min-h-0 min-w-0 flex-1 flex-col">
      <header className="flex flex-col gap-4 border-b border-line p-5">
        <div className="flex items-center gap-4">
          <UnitIcon path={summary.icon} size="large" />
          <div className="min-w-0 flex-1">
            <h2 className="truncate text-xl font-semibold">
              {summary.name}
              {summary.suffix && (
                <span className="ml-2 text-base font-normal text-faint">{summary.suffix}</span>
              )}
            </h2>
            <p className="font-mono text-xs text-faint">
              {summary.id}
              {summary.custom && ` · based on ${summary.base}`}
            </p>
            <p className="text-sm text-muted">{facts(detail)}</p>
          </div>
          {editable && (
            <Button onPress={onCreateCopy}>
              <span className="flex items-center gap-1.5">
                <Copy01 className="size-4" aria-hidden="true" />
                New ability based on this
              </span>
            </Button>
          )}
        </div>

        {description && (
          <div className="max-h-32 overflow-y-auto rounded-md border border-line bg-[#05070d] px-3 py-2 text-sm leading-relaxed">
            <GameText text={description} />
          </div>
        )}

        <div className="flex items-center gap-3">
          <input
            type="search"
            value={filter}
            onChange={(e) => setFilter(e.target.value)}
            placeholder="Filter fields by name, code, value or description: cooldown, damage, icon…"
            aria-label="Filter fields"
            className="min-w-0 flex-1 rounded-md border border-line bg-surface-0 px-3 py-1.5 outline-none focus:border-gold"
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
          <p className="text-sm text-faint">Browsing game data. Open a map to edit abilities.</p>
        )}
        {editable && (
          <p className="text-xs text-faint">
            In the level tables: arrows move, type or Enter to edit, Enter or Tab applies and moves
            to the next level, Ctrl+Enter applies to every later level, Delete resets.
          </p>
        )}
      </header>

      <div className="min-h-0 flex-1 overflow-y-auto">
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
  );
}
