import { ChevronDown, ChevronRight } from "@untitledui/icons";
import type { ReactNode } from "react";
import { FIELD_GROUPS } from "../fieldGroups";
import type { DamageBonus, Field } from "../types";
import { classes, FIELD_ROW_GRID } from "../ui";
import { FieldGroupRow } from "./FieldGroupRow";
import { FieldRow } from "./FieldRow";

export function FieldHeader({ base }: { base: string }) {
  return (
    <div
      className={classes(
        FIELD_ROW_GRID,
        "border-b border-line px-4 py-1.5 font-display text-sm font-bold text-muted",
      )}
    >
      <span>Field</span>
      <span>Value</span>
      <span className="truncate">{base}</span>
      <span />
    </div>
  );
}

export function matches(field: Field, filter: string): boolean {
  const needle = filter.toLowerCase();

  return [field.name, field.id, field.display, field.description ?? ""].some((text) =>
    text.toLowerCase().includes(needle),
  );
}

export function toggled(list: string[], value: string): string[] {
  if (list.includes(value)) {
    return list.filter((v) => v !== value);
  }

  return [...list, value];
}

type SectionToggleProps = {
  label: string;
  count: number;
  changed: number;
  expanded: boolean;
  onToggle: () => void;
};

export function SectionToggle({ label, count, changed, expanded, onToggle }: SectionToggleProps) {
  return (
    <button
      type="button"
      onClick={onToggle}
      aria-expanded={expanded}
      className="sticky top-0 z-20 flex w-full items-center gap-2 border-b border-line bg-surface-1 px-4 py-2 text-left"
    >
      {expanded && <ChevronDown className="size-4" aria-hidden="true" />}
      {!expanded && <ChevronRight className="size-4" aria-hidden="true" />}
      <span className="font-display text-lg font-bold text-gold">{label}</span>
      <span className="text-sm text-faint">{count} fields</span>
      {changed > 0 && <span className="text-sm text-changed">{changed} changed</span>}
    </button>
  );
}

type AdvancedGroupProps = {
  count: number;
  changed: number;
  open: boolean;
  onToggle: () => void;
  children: ReactNode;
};

export function AdvancedGroup({ count, changed, open, onToggle, children }: AdvancedGroupProps) {
  if (count === 0) {
    return null;
  }

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
        <span className="text-faint">{count} rarely used fields</span>
        {changed > 0 && <span className="text-changed">{changed} changed</span>}
      </button>
      {open && <div className="bg-surface-0/40">{children}</div>}
    </div>
  );
}

type FieldRowsProps = {
  fields: Field[];
  editable: boolean;
  damageBonus: DamageBonus | null;
  onChange: (field: string, value: string) => void;
  onReset: (field: string) => void;
  onInspect?: (field: Field) => void;
};

export function FieldRows({
  fields,
  editable,
  damageBonus,
  onChange,
  onReset,
  onInspect,
}: FieldRowsProps) {
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
          damageBonus={damageBonus}
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
        {...(onInspect && { onInspect })}
      />,
    );
  }

  return result;
}
