import { ChevronDown, ChevronRight } from "@untitledui/icons";
import { useCallback, useState } from "react";
import { remembered } from "../ui";

const FOLDED = "1";
const OPEN = "0";

/** Whether a list's game objects are folded, remembered between sessions under `key`. */
export function useFolded(key: string): [boolean, () => void] {
  const [folded, setFolded] = useState(() => remembered.get(key) === FOLDED);

  const toggle = useCallback(() => {
    setFolded((current) => {
      const next = !current;
      let stored = OPEN;
      if (next) {
        stored = FOLDED;
      }

      remembered.set(key, stored);
      return next;
    });
  }, [key]);

  return [folded, toggle];
}

type ListFoldProps = {
  /** "Game units". */
  label: string;
  count: number;
  folded: boolean;
  onToggle: () => void;
};

/**
 * The header between a map's own objects and the game's, in an object list.
 * Folding it leaves only what the map adds or changes.
 */
export function ListFold({ label, count, folded, onToggle }: ListFoldProps) {
  return (
    <li className="sticky top-0 z-10">
      <button
        type="button"
        onClick={onToggle}
        aria-expanded={!folded}
        className="flex w-full items-center gap-2 border-y border-line bg-surface-1 px-3 py-1.5 text-left hover:bg-surface-2"
      >
        {folded && <ChevronRight className="size-4 text-muted" aria-hidden="true" />}
        {!folded && <ChevronDown className="size-4 text-muted" aria-hidden="true" />}
        <span className="font-display text-sm font-bold text-gold">{label}</span>
        <span className="text-xs text-faint">{count}</span>
      </button>
    </li>
  );
}
