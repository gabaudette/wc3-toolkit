import { AlertTriangle, ChevronDown, ChevronRight, Copy01 } from "@untitledui/icons";
import { type ReactNode, useMemo, useState } from "react";
import { FIELD_GROUPS } from "../fieldGroups";
import type { Field, HotkeyClash, Section, UnitDetail } from "../types";
import { classes } from "../ui";
import { Button } from "./Button";
import { FieldGroupRow } from "./FieldGroupRow";
import { FieldRow } from "./FieldRow";
import { UnitIcon } from "./UnitIcon";

type UnitPageProps = {
  detail: UnitDetail;
  clashes: HotkeyClash[];
  editable: boolean;
  onChange: (field: string, value: string) => void;
  onReset: (field: string) => void;
  onCreateCopy: () => void;
};

/** Sections open by default; the rest start folded to keep the page short. */
const OPEN_BY_DEFAULT = ["stats", "combat", "abil", "text"];

/** Matches names, codes, values and descriptions, so "armor" also finds Defense Type. */
function matches(field: Field, filter: string): boolean {
  const needle = filter.toLowerCase();
  return [field.name, field.id, field.display, field.description ?? ""].some((text) =>
    text.toLowerCase().includes(needle),
  );
}

function visibleFields(section: Section, filter: string, changedOnly: boolean): Field[] {
  return section.fields.filter(
    (f) => (!changedOnly || f.modified) && (!filter || matches(f, filter)),
  );
}

function toggled(list: string[], value: string): string[] {
  if (list.includes(value)) {
    return list.filter((v) => v !== value);
  }

  return [...list, value];
}

type AdvancedGroupProps = {
  fields: Field[];
  open: boolean;
  onToggle: () => void;
  children: ReactNode;
};

/** Rarely needed fields, folded at the end of their section. */
function AdvancedGroup({ fields, open, onToggle, children }: AdvancedGroupProps) {
  if (fields.length === 0) {
    return null;
  }

  const changed = fields.filter((f) => f.modified).length;

  return (
    <div>
      <button
        type="button"
        onClick={onToggle}
        aria-expanded={open}
        className="flex w-full items-center gap-2 border-b border-line/60 px-3 py-1.5 text-left text-sm text-muted hover:text-ink"
      >
        {open && <ChevronDown className="size-4" aria-hidden="true" />}
        {!open && <ChevronRight className="size-4" aria-hidden="true" />}
        Advanced
        <span className="text-faint">{fields.length} rarely used fields</span>
        {changed > 0 && <span className="text-changed">{changed} changed</span>}
      </button>
      {open && <div className="bg-surface-0/40">{children}</div>}
    </div>
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
    setOpen(toggled(open, category));
  }

  function toggleAdvanced(category: string) {
    setOpenAdvanced(toggled(openAdvanced, category));
  }

  /** Field rows, with related fields merged into one row where all of them are shown. */
  function rows(fields: Field[]) {
    const byId = new Map(fields.map((f) => [f.id, f]));
    const grouped = new Set<string>();
    const result = [];

    for (const field of fields) {
      if (grouped.has(field.id)) {
        continue;
      }

      const group = FIELD_GROUPS.find((g) => g.ids.includes(field.id));
      const members = group?.ids.map((id) => byId.get(id));
      if (group && members?.every(Boolean)) {
        const present = members.filter((m): m is Field => m !== undefined);
        for (const id of group.ids) {
          grouped.add(id);
        }

        result.push(
          <FieldGroupRow
            key={group.label}
            group={group}
            fields={present}
            editable={editable}
            damageBonus={detail.damageBonus}
            onChange={onChange}
            onReset={onReset}
          />,
        );
        continue;
      }

      result.push(
        <FieldRow
          key={field.id}
          field={field}
          editable={editable}
          onChange={(value) => onChange(field.id, value)}
          onReset={() => onReset(field.id)}
        />,
      );
    }

    return result;
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
              {expanded && rows(fields.filter((f) => !f.advanced))}
              {expanded && (
                <AdvancedGroup
                  fields={fields.filter((f) => f.advanced)}
                  open={searching || openAdvanced.includes(section.category)}
                  onToggle={() => toggleAdvanced(section.category)}
                >
                  {rows(fields.filter((f) => f.advanced))}
                </AdvancedGroup>
              )}
            </div>
          );
        })}
      </div>
    </section>
  );
}
