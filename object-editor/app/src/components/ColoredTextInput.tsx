import { type FocusEvent, type KeyboardEvent, type SyntheticEvent, useRef, useState } from "react";
import { applyColor, type Edit, removeColor, SWATCHES } from "../colorCodes";

type ColoredTextInputProps = {
  value: string;
  multiline: boolean;
  readOnly: boolean;
  rows: number;
  label: string;
  className: string;
  onDraft: (text: string) => void;
  /** Applies the text, like leaving the field. */
  onCommit: (text: string) => void;
};

type Selection = {
  start: number;
  end: number;
};

const HEX_PREFIX = "#";

/**
 * A text box for game text. Selecting text shows a small menu that wraps the
 * selection in a color code, or removes colors from it.
 */
export function ColoredTextInput({
  value,
  multiline,
  readOnly,
  rows,
  label,
  className,
  onDraft,
  onCommit,
}: ColoredTextInputProps) {
  const inputRef = useRef<HTMLInputElement & HTMLTextAreaElement>(null);
  const menuRef = useRef<HTMLDivElement>(null);
  const [selection, setSelection] = useState<Selection | null>(null);

  function trackSelection(e: SyntheticEvent<HTMLInputElement | HTMLTextAreaElement>) {
    const { selectionStart, selectionEnd } = e.currentTarget;

    if (
      readOnly ||
      selectionStart === null ||
      selectionEnd === null ||
      selectionStart === selectionEnd
    ) {
      setSelection(null);
      return;
    }

    setSelection({ start: selectionStart, end: selectionEnd });
  }

  /** Puts the edited text in place and keeps the new text selected. */
  function show(edit: Edit) {
    onDraft(edit.text);
    setSelection({ start: edit.start, end: edit.end });
    window.requestAnimationFrame(() => {
      inputRef.current?.focus();
      inputRef.current?.setSelectionRange(edit.start, edit.end);
    });
  }

  function color(hex: string) {
    if (selection) {
      show(applyColor(value, selection.start, selection.end, hex));
    }
  }

  function uncolor() {
    if (selection) {
      show(removeColor(value, selection.start, selection.end));
    }
  }

  /** The system color dialog takes focus, so a custom color is applied right away. */
  function customColor(hex: string) {
    if (selection) {
      const edit = applyColor(value, selection.start, selection.end, hex.replace(HEX_PREFIX, ""));

      onDraft(edit.text);
      onCommit(edit.text);
      setSelection(null);
    }
  }

  function blur(e: FocusEvent) {
    // Focus moving into the menu (the custom color picker) keeps it open.
    if (menuRef.current?.contains(e.relatedTarget as Node | null)) {
      return;
    }

    setSelection(null);
    onCommit(value);
  }

  function key(e: KeyboardEvent) {
    if (e.key === "Escape") {
      setSelection(null);
    }

    if (e.key === "Enter" && !multiline) {
      onCommit(value);
    }
  }

  const shared = {
    ref: inputRef,
    value,
    readOnly,
    "aria-label": label,
    className,
    onChange: (e: { target: { value: string } }) => onDraft(e.target.value),
    onSelect: trackSelection,
    onBlur: blur,
    onKeyDown: key,
  };

  return (
    <div className="relative">
      {selection && (
        <div
          ref={menuRef}
          role="toolbar"
          aria-label="Text color"
          // Clicking a swatch must not move focus out of the text box.
          onMouseDown={(e) => {
            if (!(e.target instanceof HTMLInputElement)) {
              e.preventDefault();
            }
          }}
          className="absolute -top-9 right-0 z-20 flex items-center gap-1 rounded-[3px] border border-line bg-surface-2 px-1.5 py-1 shadow-xl"
        >
          {SWATCHES.map((swatch) => (
            <button
              key={swatch.hex}
              type="button"
              title={swatch.name}
              aria-label={`Color ${swatch.name}`}
              onClick={() => color(swatch.hex)}
              style={{ backgroundColor: `${HEX_PREFIX}${swatch.hex}` }}
              className="size-4 rounded-sm border border-black/40 hover:scale-125"
            />
          ))}
          <label
            title="Custom color"
            className="relative ml-1 flex size-5 cursor-pointer items-center justify-center rounded-sm border border-line text-xs text-muted hover:text-ink"
          >
            +
            <input
              type="color"
              aria-label="Custom color"
              onChange={(e) => customColor(e.target.value)}
              className="absolute inset-0 cursor-pointer opacity-0"
            />
          </label>
          <button
            type="button"
            onClick={uncolor}
            className="ml-1 whitespace-nowrap rounded px-1.5 text-xs text-muted hover:bg-surface-1 hover:text-ink"
          >
            Remove color
          </button>
        </div>
      )}
      {multiline && <textarea {...shared} rows={rows} />}
      {!multiline && <input {...shared} />}
    </div>
  );
}
