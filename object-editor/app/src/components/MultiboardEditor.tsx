import { ArrowDown, ArrowLeft, ArrowRight, ArrowUp, Plus, XClose } from "@untitledui/icons";
import { useState } from "react";
import { iconUrl } from "../api";
import type { BoardCell, BoardColumn, BoardRow, IconChoice, MultiboardDesign } from "../types";
import { classes } from "../ui";
import { Button } from "./Button";
import { Checkbox } from "./Checkbox";
import { ColorSwatches } from "./ColorSwatches";
import { DesignHeader } from "./DesignBasics";
import { GameText } from "./GameText";
import { IconPicker } from "./SpellButtonStep";
import { UnitIcon } from "./UnitIcon";

type MultiboardEditorProps = {
  board: MultiboardDesign;
  icons: IconChoice[];
  luaName: string;
  editable: boolean;
  onChange: (board: MultiboardDesign) => void;
  onDuplicate: () => void;
  onDelete: () => void;
};

type Selection =
  | { kind: "cell"; row: number; column: number }
  | { kind: "row"; row: number }
  | { kind: "column"; column: number };

const inputClass =
  "rounded-[3px] border border-field bg-well px-2 py-1 text-ink outline-none focus:border-focus disabled:opacity-60";
const EMPTY_CELL: BoardCell = { text: "", color: null, icon: null };
const PLAYER_NAME = "{player}";

type SamplePlayer = { name: string; color: string };

const SAMPLE_PLAYERS: SamplePlayer[] = [
  { name: "Player 1", color: "ff0303" },
  { name: "Player 2", color: "0042ff" },
];
const PREVIEW_WIDTH_SCALE = 0.94;

function cellAt(row: BoardRow, column: number): BoardCell {
  return row.cells[column] ?? EMPTY_CELL;
}

function covered(board: MultiboardDesign, selection: Selection): [number, number][] {
  if (selection.kind === "cell") {
    return [[selection.row, selection.column]];
  }

  if (selection.kind === "row") {
    return board.columns.map((_, column) => [selection.row, column]);
  }

  return board.rows.map((_, row) => [row, selection.column]);
}

function restyled(
  board: MultiboardDesign,
  selection: Selection,
  change: Partial<BoardCell>,
): MultiboardDesign {
  const targets = covered(board, selection);

  return {
    ...board,
    rows: board.rows.map((row, r) => ({
      ...row,
      cells: board.columns.map((_, c) => {
        const cell = cellAt(row, c);

        if (targets.some(([tr, tc]) => tr === r && tc === c)) {
          return { ...cell, ...change };
        }

        return cell;
      }),
    })),
  };
}

function shared<K extends keyof BoardCell>(
  board: MultiboardDesign,
  selection: Selection,
  key: K,
): BoardCell[K] | null {
  const values = covered(board, selection).map(([r, c]) => {
    const row = board.rows[r];
    if (!row) {
      return null;
    }

    return cellAt(row, c)[key];
  });
  const [first] = values;

  if (first === undefined || values.some((v) => v !== first)) {
    return null;
  }

  return first;
}

function selectionLabel(board: MultiboardDesign, selection: Selection): string {
  if (selection.kind === "row") {
    return `The whole row "${board.rows[selection.row]?.name ?? ""}"`;
  }

  if (selection.kind === "column") {
    return `The whole column "${board.columns[selection.column]?.name ?? ""}"`;
  }

  const row = board.rows[selection.row]?.name ?? "";
  const column = board.columns[selection.column]?.name ?? "";

  return `The cell in row "${row}", column "${column}"`;
}

function moved<T>(list: T[], index: number, by: number): T[] {
  const target = index + by;

  if (target < 0 || target >= list.length) {
    return list;
  }

  const next = [...list];
  const [item] = next.splice(index, 1);

  if (item !== undefined) {
    next.splice(target, 0, item);
  }

  return next;
}

export function MultiboardEditor({
  board,
  icons,
  luaName,
  editable,
  onChange,
  onDuplicate,
  onDelete,
}: MultiboardEditorProps) {
  const [selection, setSelection] = useState<Selection>({ kind: "cell", row: 0, column: 0 });
  const set = (change: Partial<MultiboardDesign>) => onChange({ ...board, ...change });

  function setColumn(index: number, change: Partial<BoardColumn>) {
    set({
      columns: board.columns.map((column, i) => {
        if (i === index) {
          return { ...column, ...change };
        }

        return column;
      }),
    });
  }

  function setRow(index: number, change: Partial<BoardRow>) {
    set({
      rows: board.rows.map((row, i) => {
        if (i === index) {
          return { ...row, ...change };
        }

        return row;
      }),
    });
  }

  function setCellText(r: number, c: number, text: string) {
    onChange(restyled(board, { kind: "cell", row: r, column: c }, { text }));
  }

  function addColumn() {
    set({
      columns: [
        ...board.columns,
        { id: crypto.randomUUID(), name: `Column ${board.columns.length + 1}`, width: 6 },
      ],
      rows: board.rows.map((row) => ({
        ...row,
        cells: [...board.columns.map((_, c) => cellAt(row, c)), EMPTY_CELL],
      })),
    });
  }

  function removeColumn(index: number) {
    set({
      columns: board.columns.filter((_, i) => i !== index),
      rows: board.rows.map((row) => ({ ...row, cells: row.cells.filter((_, i) => i !== index) })),
    });

    setSelection({ kind: "cell", row: 0, column: 0 });
  }

  function moveColumn(index: number, by: number) {
    set({
      columns: moved(board.columns, index, by),
      rows: board.rows.map((row) => ({
        ...row,
        cells: moved(
          board.columns.map((_, c) => cellAt(row, c)),
          index,
          by,
        ),
      })),
    });
  }

  function addRow() {
    set({
      rows: [
        ...board.rows,
        {
          id: crypto.randomUUID(),
          name: `Row ${board.rows.length + 1}`,
          perPlayer: false,
          cells: board.columns.map(() => EMPTY_CELL),
        },
      ],
    });
  }

  function removeRow(index: number) {
    set({ rows: board.rows.filter((_, i) => i !== index) });
    setSelection({ kind: "cell", row: 0, column: 0 });
  }

  const styleColor = shared(board, selection, "color");
  const styleIcon = shared(board, selection, "icon");

  return (
    <div className="flex min-h-0 flex-1 flex-col gap-5 overflow-y-auto p-5">
      <div className="grid grid-cols-[minmax(0,1fr)_minmax(0,1.1fr)] gap-5">
        <div className="flex min-w-0 flex-col gap-4">
          <DesignHeader
            name={board.name}
            lua={`UI.${luaName}.SetValue(player, "${board.columns[1]?.name ?? "Kills"}", 5)`}
            editable={editable}
            onRename={(name) => set({ name })}
            onDuplicate={onDuplicate}
            onDelete={onDelete}
          />

          <label className="flex flex-col gap-1 text-sm text-muted">
            Title
            <input
              value={board.title}
              disabled={!editable}
              onChange={(e) => set({ title: e.target.value })}
              className={`${inputClass} px-2.5 py-1.5`}
            />
          </label>
          <div className="-mt-2">
            <ColorSwatches
              value={board.titleColor}
              onChange={(titleColor) => set({ titleColor })}
              label="Title color"
              none={{ label: "Game default", swatch: "#ffffff" }}
              disabled={!editable}
            />
          </div>

          <Checkbox
            isSelected={board.showAtStart}
            isDisabled={!editable}
            onChange={(showAtStart) => set({ showAtStart })}
          >
            Show when the game starts
          </Checkbox>
          <Checkbox
            isSelected={board.minimized}
            isDisabled={!editable}
            onChange={(minimized) => set({ minimized })}
          >
            Start folded to its title (players open it with a click)
          </Checkbox>
        </div>

        <BoardPreview board={board} />
      </div>

      <section aria-labelledby="board-cells" className="flex flex-col gap-2">
        <h3 id="board-cells" className="font-semibold text-gold-bright">
          Cells
        </h3>
        <p className="text-xs text-faint">
          Click a cell, or a row or column's style button, to set its color and icon below. On a row
          for each player, <code className="font-mono">{PLAYER_NAME}</code> is that player's name in
          their color.
        </p>
        <div className="overflow-x-auto">
          <table className="border-separate border-spacing-1 text-sm">
            <thead>
              <tr>
                <th className="sr-only" scope="col">
                  Rows
                </th>
                {board.columns.map((column, c) => (
                  <th key={column.id} scope="col" className="min-w-36 align-bottom font-normal">
                    <div className="flex flex-col gap-1">
                      <input
                        value={column.name}
                        disabled={!editable}
                        onChange={(e) => setColumn(c, { name: e.target.value })}
                        aria-label={`Column ${c + 1} name`}
                        className={`${inputClass} font-semibold`}
                      />
                      <div className="flex items-center gap-1 text-xs text-faint">
                        <input
                          type="number"
                          min={1}
                          max={100}
                          step={0.5}
                          value={column.width}
                          disabled={!editable}
                          onChange={(e) => setColumn(c, { width: Number(e.target.value) })}
                          aria-label={`Column ${c + 1} width, percent of the screen`}
                          title="Width, in percent of the screen's width"
                          className={`${inputClass} w-16 text-right`}
                        />
                        %
                        <span className="flex-1" />
                        <SmallButton
                          label={`Style column ${column.name}`}
                          pressed={selection.kind === "column" && selection.column === c}
                          onPress={() => setSelection({ kind: "column", column: c })}
                        >
                          Style
                        </SmallButton>
                        <IconButton
                          label={`Move column ${column.name} left`}
                          disabled={!editable || c === 0}
                          onPress={() => moveColumn(c, -1)}
                        >
                          <ArrowLeft className="size-3.5" aria-hidden="true" />
                        </IconButton>
                        <IconButton
                          label={`Move column ${column.name} right`}
                          disabled={!editable || c === board.columns.length - 1}
                          onPress={() => moveColumn(c, 1)}
                        >
                          <ArrowRight className="size-3.5" aria-hidden="true" />
                        </IconButton>
                        <IconButton
                          label={`Remove column ${column.name}`}
                          disabled={!editable || board.columns.length === 1}
                          onPress={() => removeColumn(c)}
                        >
                          <XClose className="size-3.5" aria-hidden="true" />
                        </IconButton>
                      </div>
                    </div>
                  </th>
                ))}
                <th scope="col" className="align-bottom font-normal">
                  <Button small disabled={!editable} onPress={addColumn}>
                    <span className="flex items-center gap-1">
                      <Plus className="size-3.5" aria-hidden="true" />
                      Column
                    </span>
                  </Button>
                </th>
              </tr>
            </thead>
            <tbody>
              {board.rows.map((row, r) => (
                <tr key={row.id}>
                  <th scope="row" className="min-w-52 text-left align-top font-normal">
                    <div className="flex flex-col gap-1">
                      <input
                        value={row.name}
                        disabled={!editable}
                        onChange={(e) => setRow(r, { name: e.target.value })}
                        aria-label={`Row ${r + 1} name`}
                        className={inputClass}
                      />
                      <div className="flex items-center gap-1 text-xs text-faint">
                        <Checkbox
                          isSelected={row.perPlayer}
                          isDisabled={!editable}
                          onChange={(perPlayer) => setRow(r, { perPlayer })}
                          className="text-xs"
                        >
                          Each player
                        </Checkbox>
                        <span className="flex-1" />
                        <SmallButton
                          label={`Style row ${row.name}`}
                          pressed={selection.kind === "row" && selection.row === r}
                          onPress={() => setSelection({ kind: "row", row: r })}
                        >
                          Style
                        </SmallButton>
                        <IconButton
                          label={`Move row ${row.name} up`}
                          disabled={!editable || r === 0}
                          onPress={() => set({ rows: moved(board.rows, r, -1) })}
                        >
                          <ArrowUp className="size-3.5" aria-hidden="true" />
                        </IconButton>
                        <IconButton
                          label={`Move row ${row.name} down`}
                          disabled={!editable || r === board.rows.length - 1}
                          onPress={() => set({ rows: moved(board.rows, r, 1) })}
                        >
                          <ArrowDown className="size-3.5" aria-hidden="true" />
                        </IconButton>
                        <IconButton
                          label={`Remove row ${row.name}`}
                          disabled={!editable}
                          onPress={() => removeRow(r)}
                        >
                          <XClose className="size-3.5" aria-hidden="true" />
                        </IconButton>
                      </div>
                    </div>
                  </th>
                  {board.columns.map((column, c) => {
                    const cell = cellAt(row, c);
                    const picked =
                      selection.kind === "cell" && selection.row === r && selection.column === c;

                    return (
                      <td key={column.id} className="align-top">
                        <div
                          className={classes(
                            "flex items-center gap-1 rounded-[3px] border p-0.5",
                            picked && "border-gold",
                            !picked && "border-transparent",
                          )}
                        >
                          {cell.icon && <UnitIcon path={cell.icon} size="small" />}
                          <input
                            value={cell.text}
                            disabled={!editable}
                            onChange={(e) => setCellText(r, c, e.target.value)}
                            onFocus={() => setSelection({ kind: "cell", row: r, column: c })}
                            aria-label={`${row.name}, ${column.name}`}
                            style={{ color: `#${cell.color ?? "e8ecf5"}` }}
                            className={`${inputClass} min-w-0 flex-1`}
                          />
                        </div>
                      </td>
                    );
                  })}
                </tr>
              ))}
            </tbody>
          </table>
        </div>
        <div>
          <Button small disabled={!editable} onPress={addRow}>
            <span className="flex items-center gap-1">
              <Plus className="size-3.5" aria-hidden="true" />
              Row
            </span>
          </Button>
        </div>
      </section>

      <section
        aria-labelledby="board-style"
        className="flex flex-col gap-3 rounded-[3px] border border-line p-3"
      >
        <h3 id="board-style" className="font-semibold text-gold-bright">
          Style: <span className="font-normal text-muted">{selectionLabel(board, selection)}</span>
        </h3>
        <ColorSwatches
          value={styleColor}
          onChange={(color) => onChange(restyled(board, selection, { color }))}
          label="Text color"
          none={{ label: "White", swatch: "#ffffff" }}
          disabled={!editable}
        />
        <div className="flex items-center gap-2 text-sm text-muted">
          <span>Icon</span>
          <Button
            small
            disabled={!editable || styleIcon === null}
            onPress={() => onChange(restyled(board, selection, { icon: null }))}
          >
            No icon
          </Button>
        </div>
        <IconPicker
          current={styleIcon}
          icons={icons}
          onPick={(icon) => onChange(restyled(board, selection, { icon }))}
        />
      </section>
    </div>
  );
}

type SmallButtonProps = {
  label: string;
  pressed: boolean;
  onPress: () => void;
  children: React.ReactNode;
};

function SmallButton({ label, pressed, onPress, children }: SmallButtonProps) {
  return (
    <button
      type="button"
      onClick={onPress}
      aria-pressed={pressed}
      aria-label={label}
      className={classes(
        "rounded-[3px] border px-1.5 py-0.5 text-xs",
        pressed && "border-gold text-gold-bright",
        !pressed && "border-field text-muted hover:text-ink",
      )}
    >
      {children}
    </button>
  );
}

type IconButtonProps = {
  label: string;
  disabled: boolean;
  onPress: () => void;
  children: React.ReactNode;
};

function IconButton({ label, disabled, onPress, children }: IconButtonProps) {
  return (
    <button
      type="button"
      onClick={onPress}
      disabled={disabled}
      aria-label={label}
      title={label}
      className="rounded p-1 text-faint hover:bg-surface-2 hover:text-ink disabled:opacity-30"
    >
      {children}
    </button>
  );
}

function previewText(cell: BoardCell, player: SamplePlayer | null): string {
  let text = cell.text;

  if (player) {
    text = text.replaceAll(PLAYER_NAME, `|cff${player.color}${player.name}|r`);
  }

  if (cell.color) {
    return `|cff${cell.color}${text}|r`;
  }

  return text;
}

function BoardPreview({ board }: { board: MultiboardDesign }) {
  const shownRows = board.rows.flatMap<{ row: BoardRow; player: SamplePlayer | null }>((row) => {
    if (row.perPlayer) {
      return SAMPLE_PLAYERS.map((player) => ({ row, player }));
    }

    return [{ row, player: null }];
  });
  let title = board.title;

  if (board.titleColor) {
    title = `|cff${board.titleColor}${board.title}|r`;
  }

  return (
    <figure className="flex min-w-0 flex-col gap-1.5">
      <figcaption className="text-xs text-faint">Preview, as the game shows it</figcaption>
      <div className="@container relative aspect-video overflow-hidden rounded-[3px] border border-line bg-[radial-gradient(ellipse_at_center,#2b3a2a_0%,#141c14_70%)]">
        <div className="absolute top-[1.2cqw] right-[1.2cqw] rounded-[0.3cqw] border-[0.15cqw] border-frame bg-[#0b0d12]/85 px-[0.8cqw] py-[0.6cqw] text-[1.15cqw] leading-tight text-white">
          <p className="mb-[0.4cqw] border-b-[0.1cqw] border-frame/60 pb-[0.3cqw] text-center font-semibold">
            <GameText text={title} singleLine />
          </p>
          {!board.minimized &&
            shownRows.map(({ row, player }) => (
              <div key={`${row.id}-${player?.name}`} className="flex">
                {board.columns.map((column, c) => {
                  const cell = cellAt(row, c);

                  return (
                    <div
                      key={column.id}
                      className="flex shrink-0 items-center gap-[0.3cqw] overflow-hidden py-[0.15cqw] whitespace-nowrap"
                      style={{ width: `${column.width * PREVIEW_WIDTH_SCALE}cqw` }}
                    >
                      {cell.icon && (
                        <img
                          src={iconUrl(cell.icon)}
                          alt=""
                          className="size-[1.3cqw] shrink-0"
                          draggable={false}
                        />
                      )}
                      <GameText text={previewText(cell, player)} singleLine />
                    </div>
                  );
                })}
              </div>
            ))}
        </div>
      </div>
    </figure>
  );
}
