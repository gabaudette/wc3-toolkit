import { ChevronDown, ChevronRight, Copy01 } from "@untitledui/icons";
import { useMemo, useState } from "react";
import type { Field, Section, UnitDetail } from "../types";
import { classes } from "../ui";
import { Button } from "./Button";
import { FieldRow } from "./FieldRow";
import { UnitIcon } from "./UnitIcon";

type UnitPageProps = {
  detail: UnitDetail;
  editable: boolean;
  onChange: (field: string, value: string) => void;
  onReset: (field: string) => void;
  onCreateCopy: () => void;
};

/** Sections open by default; the rest start folded to keep the page short. */
const OPEN_BY_DEFAULT = ["stats", "combat", "abil", "text"];

function matches(field: Field, filter: string): boolean {
  const needle = filter.toLowerCase();
  return (
    field.name.toLowerCase().includes(needle) ||
    field.id.toLowerCase().includes(needle) ||
    field.display.toLowerCase().includes(needle)
  );
}

function visibleFields(section: Section, filter: string, changedOnly: boolean): Field[] {
  return section.fields.filter(
    (f) => (!changedOnly || f.modified) && (!filter || matches(f, filter)),
  );
}

function kindLabel(detail: UnitDetail): string {
  const { kind, race } = detail.item;
  return [kind, race].filter(Boolean).join(" · ");
}

export function UnitPage({ detail, editable, onChange, onReset, onCreateCopy }: UnitPageProps) {
  const [filter, setFilter] = useState("");
  const [changedOnly, setChangedOnly] = useState(false);
  const [open, setOpen] = useState<string[]>(OPEN_BY_DEFAULT);
  const { item } = detail;
  const changedCount = detail.sections.flatMap((s) => s.fields).filter((f) => f.modified).length;
  const searching = Boolean(filter) || changedOnly;

  const sections = useMemo(
    () =>
      detail.sections
        .map((s) => ({ section: s, fields: visibleFields(s, filter, changedOnly) }))
        .filter((s) => s.fields.length > 0),
    [detail, filter, changedOnly],
  );

  function toggleSection(category: string) {
    if (open.includes(category)) {
      setOpen(open.filter((c) => c !== category));
    } else {
      setOpen([...open, category]);
    }
  }

  return (
    <section className="flex min-h-0 flex-1 flex-col">
      <header className="flex flex-col gap-4 border-b border-line p-5">
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

        <div className="flex items-center gap-3">
          <input
            type="search"
            value={filter}
            onChange={(e) => setFilter(e.target.value)}
            placeholder="Filter fields: damage, speed, model…"
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
      </header>

      <div className="min-h-0 flex-1 overflow-y-auto">
        {sections.length === 0 && <p className="p-5 text-muted">No fields match.</p>}
        {sections.map(({ section, fields }) => {
          const expanded = searching || open.includes(section.category);
          const changed = fields.filter((f) => f.modified).length;

          return (
            <div key={section.category}>
              <button
                type="button"
                onClick={() => toggleSection(section.category)}
                aria-expanded={expanded}
                className="sticky top-0 z-10 flex w-full items-center gap-2 border-b border-line bg-surface-1 px-3 py-2 text-left text-sm font-semibold"
              >
                {expanded && <ChevronDown className="size-4" aria-hidden="true" />}
                {!expanded && <ChevronRight className="size-4" aria-hidden="true" />}
                {section.label}
                <span className="font-normal text-faint">{fields.length} fields</span>
                {changed > 0 && (
                  <span className={classes("font-normal text-changed")}>{changed} changed</span>
                )}
              </button>
              {expanded &&
                fields.map((field) => (
                  <FieldRow
                    key={field.id}
                    field={field}
                    editable={editable}
                    onChange={(value) => onChange(field.id, value)}
                    onReset={() => onReset(field.id)}
                  />
                ))}
            </div>
          );
        })}
      </div>
    </section>
  );
}
