import { FlipBackward, InfoCircle } from "@untitledui/icons";
import {
  type FocusEvent,
  Fragment,
  type KeyboardEvent,
  type ReactNode,
  useEffect,
  useLayoutEffect,
  useRef,
  useState,
} from "react";
import { type Cell, inside, moved, typesCharacter } from "../grid";
import type { Field, LevelCell, LevelField } from "../types";
import { classes } from "../ui";
import { Description } from "./FieldRow";
import { FlagsInput } from "./FlagsInput";

type LevelTableProps = {
  fields: LevelField[];
  editable: boolean;
  /** `level` is 1-based. */
  onChange: (field: string, level: number, value: string) => void;
  /** Sets `level` and every later level to `value`. */
  onFill: (field: string, level: number, value: string) => void;
  onReset: (field: string, level: number) => void;
  /** What a column is called: "Level", or "Variation" for doodads. */
  levelName?: string;
  /** Shows a row's field in the page's inspector, when the page has one. */
  onInspect?: (field: Field) => void;
};

/**
 * A cell being edited and the text typed so far. `typed` when editing began
 * by typing a character, which then stays instead of being selected.
 */
type Editing = Cell & { draft: string; typed: boolean };

const TRUE = "1";
const FALSE = "0";

const inputClass =
  "w-full min-w-16 rounded border border-gold bg-surface-0 px-1.5 py-0.5 font-mono text-sm outline-none";

/** Only the active cell is in the tab order; arrows move between the others. */
function rovingIndex(active: boolean): number {
  if (active) {
    return 0;
  }

  return -1;
}

function inputMode(control: string): "decimal" | "text" {
  if (control === "number") {
    return "decimal";
  }

  return "text";
}

function cellKey(cell: Cell): string {
  return `${cell.row}:${cell.column}`;
}

/** Choices and flags pick from a list; everything else is typed. */
function isTyped(field: LevelField): boolean {
  return !["bool", "select", "flags", "flagList"].includes(field.field.control);
}

function shownValue(field: LevelField, cell: LevelCell): string {
  const { control } = field.field;
  if (control === "number" && !cell.raw) {
    return "0";
  }

  if (control === "number" || control === "codes" || control === "text") {
    return cell.raw || cell.display;
  }

  return cell.display;
}

function boolMark(cell: LevelCell): string {
  if (cell.raw === TRUE) {
    return "✓";
  }

  return "–";
}

function toggledBool(cell: LevelCell): string {
  if (cell.raw === TRUE) {
    return FALSE;
  }

  return TRUE;
}

type FlagsPopoverProps = {
  /** The cell it opens under. */
  anchor: HTMLElement | undefined;
  label: string;
  onKey: (e: KeyboardEvent) => void;
  onCommit: () => void;
  children: ReactNode;
};

/** Pixels between a cell and the checkboxes under it. */
const POPOVER_GAP = 4;

/** Above or below the cell, whichever side has more room. */
type Placement = { right: number } & ({ top: number } | { bottom: number });

const HALF = 2;

/**
 * Checkboxes under a cell; leaving them applies the choice. Placed on the
 * window, so the table's scrolling area does not cut them off.
 */
function FlagsPopover({ anchor, label, onKey, onCommit, children }: FlagsPopoverProps) {
  const panel = useRef<HTMLDivElement>(null);
  const [placement, setPlacement] = useState<Placement | null>(null);

  useLayoutEffect(() => {
    const cell = anchor?.getBoundingClientRect();
    if (!cell) {
      return;
    }

    const { clientWidth, clientHeight } = document.documentElement;
    const right = clientWidth - cell.right;
    if (cell.bottom > clientHeight / HALF) {
      setPlacement({ right, bottom: clientHeight - cell.top + POPOVER_GAP });
    } else {
      setPlacement({ right, top: cell.bottom + POPOVER_GAP });
    }
  }, [anchor]);

  useEffect(() => {
    panel.current?.querySelector("input")?.focus();
  }, []);

  return (
    <div
      ref={panel}
      role="dialog"
      aria-label={label}
      onKeyDown={onKey}
      onBlur={(e: FocusEvent) => {
        if (!(e.relatedTarget instanceof Node && e.currentTarget.contains(e.relatedTarget))) {
          onCommit();
        }
      }}
      style={placement ?? undefined}
      className="fixed z-30 w-72 rounded-[3px] border border-line bg-surface-1 p-2 text-left shadow-lg"
    >
      {children}
      <p className="mt-1.5 text-xs text-faint">Enter applies, Esc cancels.</p>
    </div>
  );
}

type CellEditorProps = {
  levelName: string;
  anchor: HTMLElement | undefined;
  field: LevelField;
  editing: Editing;
  onDraft: (draft: string) => void;
  onKey: (e: KeyboardEvent) => void;
  onCommit: () => void;
  onChoice: (value: string) => void;
};

function CellEditor({
  anchor,
  field,
  editing,
  onDraft,
  onKey,
  onCommit,
  onChoice,
  levelName,
}: CellEditorProps) {
  const meta = field.field;
  const label = `${meta.name}, ${levelName.toLowerCase()} ${editing.column + 1}`;

  if (meta.control === "select") {
    return (
      <select
        // biome-ignore lint/a11y/noAutofocus: editing starts from the keyboard on this cell.
        autoFocus
        value={editing.draft}
        onChange={(e) => onChoice(e.target.value)}
        onKeyDown={onKey}
        onBlur={onCommit}
        aria-label={label}
        className={inputClass}
      >
        {!meta.options.some((o) => o.value.toLowerCase() === editing.draft.toLowerCase()) && (
          <option value={editing.draft}>{editing.draft || "(none)"}</option>
        )}
        {meta.options.map((o) => (
          <option key={o.value} value={o.value}>
            {o.label}
          </option>
        ))}
      </select>
    );
  }

  if (meta.control === "flags" || meta.control === "flagList") {
    return (
      <FlagsPopover anchor={anchor} label={label} onKey={onKey} onCommit={onCommit}>
        <FlagsInput field={meta} raw={editing.draft} editable onChange={onDraft} />
      </FlagsPopover>
    );
  }

  return (
    <input
      // biome-ignore lint/a11y/noAutofocus: editing starts from the keyboard on this cell.
      autoFocus
      value={editing.draft}
      inputMode={inputMode(meta.control)}
      onChange={(e) => onDraft(e.target.value)}
      onKeyDown={onKey}
      onBlur={onCommit}
      onFocus={(e) => {
        if (!editing.typed) {
          e.target.select();
        }
      }}
      aria-label={label}
      className={inputClass}
    />
  );
}

/**
 * Per-level fields as a table: one row per field, one column per level.
 * Arrow keys move between cells; typing or Enter edits a cell; Enter or Tab
 * applies and moves to the next level; Ctrl+Enter applies to every later
 * level too; Escape cancels; Delete resets the cell to the base value.
 */
export function LevelTable({
  fields,
  editable,
  onChange,
  onFill,
  onReset,
  levelName = "Level",
  onInspect,
}: LevelTableProps) {
  const [active, setActive] = useState<Cell>({ row: 0, column: 0 });
  const [editing, setEditing] = useState<Editing | null>(null);
  const [info, setInfo] = useState<string[]>([]);
  const cells = useRef(new Map<string, HTMLDivElement>());
  /** Set while the active cell should take focus after a move or an edit. */
  const focusActive = useRef(false);
  // The commit handlers run from events and must see the latest edit.
  const editingRef = useRef(editing);
  editingRef.current = editing;

  const levels = Math.max(0, ...fields.map((f) => f.cells.length));
  const levelNumbers = Array.from({ length: levels }, (_, i) => i + 1);

  // Keeps the active cell inside the table, also when it has no rows (an
  // ability's text section), which used to re-render forever.
  useEffect(() => {
    const kept = inside(active, fields.length, levels);
    if (kept) {
      setActive(kept);
    }
  }, [active, fields.length, levels]);

  useLayoutEffect(() => {
    if (focusActive.current && !editing) {
      focusActive.current = false;
      cells.current.get(cellKey(active))?.focus();
    }
  });

  function moveTo(cell: Cell) {
    focusActive.current = true;
    setActive(cell);
  }

  function apply(cell: Cell, value: string, fill: boolean) {
    const field = fields[cell.row];
    const current = field?.cells[cell.column];
    if (!field || !current) {
      return;
    }

    const level = cell.column + 1;
    if (fill) {
      onFill(field.field.id, level, value);
    } else if (value !== current.raw) {
      onChange(field.field.id, level, value);
    }
  }

  /** Applies the edit in progress, once. */
  function commit(fill = false) {
    const edit = editingRef.current;
    if (!edit) {
      return;
    }

    editingRef.current = null;
    setEditing(null);
    focusActive.current = true;
    apply(edit, edit.draft, fill);
  }

  function cancel() {
    editingRef.current = null;
    setEditing(null);
    focusActive.current = true;
  }

  function startEditing(cell: Cell, draft: string | null) {
    const field = fields[cell.row];
    const current = field?.cells[cell.column];
    if (!editable || !field || !current) {
      return;
    }

    if (field.field.control === "bool") {
      apply(cell, toggledBool(current), false);
      return;
    }

    setActive(cell);
    setEditing({ ...cell, draft: draft ?? current.raw, typed: draft !== null });
  }

  function cellKeyDown(e: KeyboardEvent, cell: Cell) {
    const field = fields[cell.row];
    const current = field?.cells[cell.column];
    if (!field || !current) {
      return;
    }

    const target = moved(cell, e.key, fields.length, levels);
    if (target) {
      e.preventDefault();
      moveTo(target);
      return;
    }

    if (e.key === "Enter" || e.key === "F2" || (e.key === " " && !isTyped(field))) {
      e.preventDefault();
      startEditing(cell, null);
      return;
    }

    if ((e.key === "Delete" || e.key === "Backspace") && current.modified && editable) {
      e.preventDefault();
      onReset(field.field.id, cell.column + 1);
      return;
    }

    const withModifier = e.ctrlKey || e.metaKey || e.altKey;
    if (isTyped(field) && typesCharacter(e.key, withModifier)) {
      e.preventDefault();
      startEditing(cell, e.key);
    }
  }

  function editorKeyDown(e: KeyboardEvent) {
    const edit = editingRef.current;
    if (!edit) {
      return;
    }

    if (e.key === "Escape") {
      e.preventDefault();
      cancel();
      return;
    }

    if (e.key === "Enter" && (e.ctrlKey || e.metaKey)) {
      e.preventDefault();
      commit(true);
      return;
    }

    const next = { row: edit.row, column: Math.min(levels - 1, edit.column + 1) };
    const previous = { row: edit.row, column: Math.max(0, edit.column - 1) };
    if (e.key === "Enter" || (e.key === "Tab" && !e.shiftKey)) {
      e.preventDefault();
      commit();
      setActive(next);
      return;
    }

    if (e.key === "Tab" && e.shiftKey) {
      e.preventDefault();
      commit();
      setActive(previous);
    }
  }

  function toggleInfo(field: Field) {
    if (onInspect) {
      onInspect(field);
      return;
    }

    const id = field.id;
    if (info.includes(id)) {
      setInfo(info.filter((i) => i !== id));
    } else {
      setInfo([...info, id]);
    }
  }

  if (fields.length === 0) {
    return null;
  }

  return (
    <div className="overflow-x-auto border-b border-line/60">
      <table aria-label="Values per level" className="border-collapse text-sm">
        <thead>
          <tr className="text-xs text-faint">
            <th className="sticky left-0 z-10 w-72 min-w-72 bg-surface-0 px-3 py-1 text-left font-normal">
              Per {levelName.toLowerCase()}
            </th>
            {levelNumbers.map((level) => (
              <th key={level} className="min-w-20 px-2 py-1 text-right font-normal">
                {levelName} {level}
              </th>
            ))}
            <th className="w-8">
              <span className="sr-only">Reset</span>
            </th>
          </tr>
        </thead>
        <tbody>
          {fields.map((field, row) => {
            const meta = field.field;
            const changed = field.cells.filter((c) => c.modified).length;
            const showInfo = info.includes(meta.id);

            return (
              <Fragment key={meta.id}>
                <tr className="border-t border-line/60">
                  <th
                    scope="row"
                    className={classes(
                      "sticky left-0 z-10 bg-surface-0 px-3 py-1 text-left font-normal",
                      changed > 0 && "border-l-2 border-l-changed",
                    )}
                  >
                    <span className="flex items-baseline gap-2">
                      <span title={meta.name}>{meta.name}</span>
                      <span className="shrink-0 font-mono text-xs text-faint">{meta.id}</span>
                      {meta.description && (
                        <button
                          type="button"
                          onClick={() => toggleInfo(meta)}
                          aria-expanded={showInfo}
                          aria-label={`What ${meta.name} does`}
                          title="What this field does"
                          className={classes(
                            "shrink-0 self-center -my-1 rounded p-1.5 hover:text-ink",
                            showInfo && "text-gold",
                            !showInfo && "text-faint",
                          )}
                        >
                          <InfoCircle className="size-3.5" aria-hidden="true" />
                        </button>
                      )}
                    </span>
                  </th>
                  {field.cells.map((cell, column) => {
                    const position = { row, column };
                    const isActive = active.row === row && active.column === column;
                    const isEditing = editing?.row === row && editing.column === column;
                    const shown = shownValue(field, cell);

                    return (
                      <td key={levelNumbers[column]} className="relative px-1 py-0.5 text-right">
                        {isEditing && meta.control !== "flags" && meta.control !== "flagList" && (
                          <CellEditor
                            levelName={levelName}
                            anchor={cells.current.get(cellKey(position))}
                            field={field}
                            editing={editing}
                            onDraft={(draft) => setEditing({ ...editing, draft })}
                            onKey={editorKeyDown}
                            onCommit={() => commit()}
                            onChoice={(value) => {
                              setEditing({ ...editing, draft: value });
                              editingRef.current = { ...editing, draft: value };
                              commit();
                            }}
                          />
                        )}
                        {!(isEditing && meta.control === "select") && (
                          // biome-ignore lint/a11y/useSemanticElements: a grid cell with roving focus, not a form control.
                          <div
                            ref={(element) => {
                              if (element) {
                                cells.current.set(cellKey(position), element);
                              } else {
                                cells.current.delete(cellKey(position));
                              }
                            }}
                            role="button"
                            tabIndex={rovingIndex(isActive)}
                            hidden={isEditing && isTyped(field)}
                            onFocus={() => {
                              setActive(position);
                              onInspect?.(meta);
                            }}
                            onClick={() => startEditing(position, null)}
                            onKeyDown={(e) => cellKeyDown(e, position)}
                            aria-label={`${meta.name}, level ${column + 1}: ${cell.display}`}
                            title={cell.display}
                            className={classes(
                              "max-w-36 cursor-default truncate rounded px-1.5 py-0.5 font-mono outline-none",
                              "focus:ring-2 focus:ring-gold/70",
                              cell.modified && "bg-changed-soft text-changed",
                              !cell.raw && !cell.modified && "text-faint",
                            )}
                          >
                            {meta.control === "bool" && boolMark(cell)}
                            {meta.control !== "bool" && (shown || "–")}
                          </div>
                        )}
                        {isEditing && (meta.control === "flags" || meta.control === "flagList") && (
                          <CellEditor
                            levelName={levelName}
                            anchor={cells.current.get(cellKey(position))}
                            field={field}
                            editing={editing}
                            onDraft={(draft) => {
                              setEditing({ ...editing, draft });
                              editingRef.current = { ...editing, draft };
                            }}
                            onKey={(e) => {
                              if (e.key === "Enter") {
                                e.preventDefault();
                                commit();
                                return;
                              }

                              editorKeyDown(e);
                            }}
                            onCommit={() => commit()}
                            onChoice={() => commit()}
                          />
                        )}
                      </td>
                    );
                  })}
                  <td className="px-1">
                    {changed > 0 && editable && (
                      <button
                        type="button"
                        onClick={() => {
                          for (const [column, cell] of field.cells.entries()) {
                            if (cell.modified) {
                              onReset(meta.id, column + 1);
                            }
                          }
                        }}
                        title="Reset every level to the base value"
                        aria-label={`Reset ${meta.name}`}
                        className="rounded p-1 text-faint hover:bg-surface-2 hover:text-ink"
                      >
                        <FlipBackward className="size-4" aria-hidden="true" />
                      </button>
                    )}
                  </td>
                </tr>
                {showInfo && meta.description && (
                  <tr>
                    <td colSpan={levels + 2} className="px-3">
                      <Description field={meta} />
                    </td>
                  </tr>
                )}
              </Fragment>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}
