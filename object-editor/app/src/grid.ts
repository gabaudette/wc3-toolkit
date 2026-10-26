// Keyboard movement in the per-level tables: one row per field, one column per level.

export type Cell = {
  row: number;
  column: number;
};

/**
 * The active cell kept inside a table of `rows` by `columns`, or `null` when
 * it already is. An empty table keeps row and column 0: comparing against a
 * clamped -1 would never settle and loop forever.
 */
export function inside(cell: Cell, rows: number, columns: number): Cell | null {
  const row = Math.max(0, Math.min(cell.row, rows - 1));
  const column = Math.max(0, Math.min(cell.column, columns - 1));
  if (row === cell.row && column === cell.column) {
    return null;
  }

  return { row, column };
}

/** Where an arrow, Home or End key moves the active cell; `null` for other keys. */
export function moved(cell: Cell, key: string, rows: number, columns: number): Cell | null {
  const clamp = (value: number, size: number) => Math.min(size - 1, Math.max(0, value));

  switch (key) {
    case "ArrowUp":
      return { ...cell, row: clamp(cell.row - 1, rows) };
    case "ArrowDown":
      return { ...cell, row: clamp(cell.row + 1, rows) };
    case "ArrowLeft":
      return { ...cell, column: clamp(cell.column - 1, columns) };
    case "ArrowRight":
      return { ...cell, column: clamp(cell.column + 1, columns) };
    case "Home":
      return { ...cell, column: 0 };
    case "End":
      return { ...cell, column: columns - 1 };
    default:
      return null;
  }
}

/** Whether a key press types a character, which starts editing a cell with it. */
export function typesCharacter(key: string, withModifier: boolean): boolean {
  return key.length === 1 && !withModifier;
}
