import { Copy01 } from "@untitledui/icons";
import { useMemo, useState } from "react";
import type { Field, HotkeyClash, Section, UnitDetail } from "../types";
import { variationModel } from "../ui";
import { Button } from "./Button";
import { Checkbox } from "./Checkbox";
import { GameText } from "./GameText";
import { Inspector } from "./Inspector";
import { AdvancedGroup, FieldHeader, FieldRows, matches, SectionToggle, toggled } from "./Sections";
import { PlaceholderValues } from "./TextPreview";
import { UnitIcon } from "./UnitIcon";

type UnitPageProps = {
  detail: UnitDetail;
  /** Hotkey clashes; leave out for objects the check does not cover. */
  clashes?: HotkeyClash[];
  editable: boolean;
  onChange: (field: string, value: string) => void;
  onReset: (field: string) => void;
  onCreateCopy: () => void;
};

/** The field holding the model, by noun. */
const MODEL_FIELD: Record<string, string> = { unit: "umdl", item: "ifil", destructable: "bfil" };

const VARIATIONS_FIELD = "bvar";

/** The model the preview loads; destructables number their variations. */
function modelFile(noun: string, fields: Field[]): string | null {
  const raw = fields.find((f) => f.id === MODEL_FIELD[noun])?.raw;
  if (!raw || noun !== "destructable") {
    return raw || null;
  }

  const variations = Number(fields.find((f) => f.id === VARIATIONS_FIELD)?.raw ?? "1");
  return variationModel(raw, variations);
}

/** Kinds that name themselves; heroes, units and buildings are all "unit". */
const OWN_NOUNS = ["item", "buff", "destructable"];

/** What the page calls the object: "unit", "item", "buff" or "destructable". */
function nounOf(item: UnitDetail["item"]): string {
  if (OWN_NOUNS.includes(item.kind)) {
    return item.kind;
  }

  return "unit";
}

/** Sections open by default; the rest start folded to keep the page short. */
const OPEN_BY_DEFAULT = ["stats", "combat", "abil", "text"];

function visibleFields(section: Section, filter: string, changedOnly: boolean): Field[] {
  return section.fields.filter(
    (f) => (!changedOnly || f.modified) && (!filter || matches(f, filter)),
  );
}

function kindLabel(detail: UnitDetail): string {
  const { kind, race } = detail.item;
  return [kind, race].filter(Boolean).join(" · ");
}

/** Header of the base column: the unit a custom unit copies, else the game's value. */
function baseLabel(item: UnitDetail["item"]): string {
  if (item.custom) {
    return `Base · ${item.base}`;
  }

  return "Game value";
}

export function UnitPage({
  detail,
  clashes,
  editable,
  onChange,
  onReset,
  onCreateCopy,
}: UnitPageProps) {
  const [filter, setFilter] = useState("");
  const [changedOnly, setChangedOnly] = useState(false);
  const [open, setOpen] = useState<string[]>(OPEN_BY_DEFAULT);
  const [openAdvanced, setOpenAdvanced] = useState<string[]>([]);
  const [inspected, setInspected] = useState<string | null>(null);
  const { item } = detail;
  const noun = nounOf(item);
  const allFields = detail.sections.flatMap((s) => s.fields);
  const inspectedField = allFields.find((f) => f.id === inspected) ?? null;
  const changedCount = allFields.filter((f) => f.modified).length;
  const modelPath = modelFile(noun, allFields);
  const searching = Boolean(filter) || changedOnly;

  const sections = useMemo(
    () =>
      detail.sections
        .map((s) => ({ section: s, fields: visibleFields(s, filter, changedOnly) }))
        .filter((s) => s.fields.length > 0),
    [detail, filter, changedOnly],
  );

  function toggleSection(category: string) {
    setOpen(toggled(open, category));
  }

  function toggleAdvanced(category: string) {
    setOpenAdvanced(toggled(openAdvanced, category));
  }

  function rows(fields: Field[]) {
    return (
      <FieldRows
        fields={fields}
        editable={editable}
        damageBonus={detail.damageBonus}
        onChange={onChange}
        onReset={onReset}
        onInspect={(field) => setInspected(field.id)}
      />
    );
  }

  return (
    <PlaceholderValues.Provider value={detail}>
      <div className="flex min-h-0 min-w-0 flex-1 gap-2.5">
        <section className="frame flex min-h-0 min-w-0 flex-1 flex-col bg-surface-1">
          <header className="flex gap-5 border-b border-line p-5">
            <div className="flex min-w-0 flex-1 flex-col gap-4">
              <div className="flex items-center gap-4">
                <UnitIcon path={item.icon} size="large" />
                <div className="min-w-0 flex-1">
                  <h2 className="truncate font-display text-3xl font-bold text-gold-bright">
                    <GameText text={item.name} singleLine />
                  </h2>
                  <p className="font-mono text-xs text-faint">
                    {item.id}
                    {item.custom && ` · based on ${item.base}`}
                  </p>
                  <p className="text-sm capitalize text-muted">{kindLabel(detail)}</p>
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

              <dl className="grid grid-cols-[repeat(auto-fit,minmax(9rem,1fr))] gap-2">
                {detail.summary.map((stat) => (
                  <div
                    key={stat.label}
                    className="rounded-[3px] border border-line bg-well px-3 py-2"
                  >
                    <dt className="font-display text-sm font-bold text-muted">{stat.label}</dt>
                    <dd className="text-lg font-semibold">{stat.value}</dd>
                  </div>
                ))}
              </dl>

              <div className="flex items-center gap-3">
                <input
                  type="search"
                  value={filter}
                  onChange={(e) => setFilter(e.target.value)}
                  placeholder="Filter fields: armor, speed, model…"
                  aria-label="Filter fields"
                  className="min-w-0 flex-1 rounded-[3px] border border-field bg-well px-3 py-1.5 outline-none focus:border-focus"
                />
                <Checkbox isSelected={changedOnly} onChange={setChangedOnly}>
                  Changed only ({changedCount})
                </Checkbox>
              </div>

              {!editable && (
                <p className="text-sm text-faint">
                  Browsing game data. Open a map to edit {noun}s.
                </p>
              )}
            </div>
          </header>

          <div className="min-h-0 flex-1 overflow-y-auto">
            <FieldHeader base={baseLabel(item)} />
            {sections.length === 0 && <p className="p-5 text-muted">No fields match.</p>}
            {sections.map(({ section, fields }) => {
              const expanded = searching || open.includes(section.category);
              const changed = fields.filter((f) => f.modified).length;
              const advanced = fields.filter((f) => f.advanced);

              return (
                <div key={section.category}>
                  <SectionToggle
                    label={section.label}
                    count={fields.length}
                    changed={changed}
                    expanded={expanded}
                    onToggle={() => toggleSection(section.category)}
                  />
                  {expanded && rows(fields.filter((f) => !f.advanced))}
                  {expanded && (
                    <AdvancedGroup
                      count={advanced.length}
                      changed={advanced.filter((f) => f.modified).length}
                      open={searching || openAdvanced.includes(section.category)}
                      onToggle={() => toggleAdvanced(section.category)}
                    >
                      {rows(advanced)}
                    </AdvancedGroup>
                  )}
                </div>
              );
            })}
          </div>
        </section>
        <Inspector
          modelPath={modelPath}
          field={inspectedField}
          editable={editable}
          {...(clashes && { clashes })}
          onReset={onReset}
        />
      </div>
    </PlaceholderValues.Provider>
  );
}
