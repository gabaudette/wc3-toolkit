import { AlertTriangle, Copy01 } from "@untitledui/icons";
import { useMemo, useState } from "react";
import type { Field, HotkeyClash, Section, UnitDetail } from "../types";
import { Button } from "./Button";
import { ModelViewer } from "./ModelViewer";
import { AdvancedGroup, FieldRows, matches, SectionToggle, toggled } from "./Sections";
import { UnitIcon } from "./UnitIcon";

type UnitPageProps = {
  detail: UnitDetail;
  clashes: HotkeyClash[];
  editable: boolean;
  onChange: (field: string, value: string) => void;
  onReset: (field: string) => void;
  onCreateCopy: () => void;
};

const MODEL_FIELD = "umdl";

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

function ClashWarning({ clashes }: { clashes: HotkeyClash[] }) {
  if (clashes.length === 0) {
    return null;
  }

  return (
    <div
      role="status"
      className="flex gap-2 rounded-md border border-gold/40 bg-gold-soft px-3 py-2 text-sm text-gold"
    >
      <AlertTriangle className="mt-0.5 size-4 shrink-0" aria-hidden="true" />
      <ul>
        {clashes.map((clash) => (
          <li key={`${clash.menu}-${clash.key}`}>
            Hotkey <kbd className="rounded bg-surface-0 px-1 font-mono text-ink">{clash.key}</kbd>{" "}
            is used by {clash.names.join(" and ")} ({clash.menu.toLowerCase()}). Only one of them
            will react to the key.
          </li>
        ))}
      </ul>
    </div>
  );
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
  const { item } = detail;
  const allFields = detail.sections.flatMap((s) => s.fields);
  const changedCount = allFields.filter((f) => f.modified).length;
  const modelPath = allFields.find((f) => f.id === MODEL_FIELD)?.raw || null;
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
      />
    );
  }

  return (
    <section className="flex min-h-0 flex-1 flex-col">
      <header className="flex gap-5 border-b border-line p-5">
        <div className="flex min-w-0 flex-1 flex-col gap-4">
          <div className="flex items-center gap-4">
            <UnitIcon path={item.icon} size="large" />
            <div className="min-w-0 flex-1">
              <h2 className="truncate text-xl font-semibold">{item.name}</h2>
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
                  New unit based on this
                </span>
              </Button>
            )}
          </div>

          <dl className="grid grid-cols-[repeat(auto-fit,minmax(9rem,1fr))] gap-2">
            {detail.summary.map((stat) => (
              <div key={stat.label} className="rounded-md bg-surface-1 px-3 py-2">
                <dt className="text-xs text-faint">{stat.label}</dt>
                <dd className="text-lg font-medium">{stat.value}</dd>
              </div>
            ))}
          </dl>

          <ClashWarning clashes={clashes} />

          <div className="flex items-center gap-3">
            <input
              type="search"
              value={filter}
              onChange={(e) => setFilter(e.target.value)}
              placeholder="Filter fields by name, code, value or description: armor, speed, model…"
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
            <p className="text-sm text-faint">Browsing game data. Open a map to edit units.</p>
          )}
        </div>
        <ModelViewer path={modelPath} />
      </header>

      <div className="min-h-0 flex-1 overflow-y-auto">
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
  );
}
