import { createContext, useContext, useEffect, useState } from "react";
import { api } from "../api";
import { GameText } from "./GameText";

/** Delay before asking the backend to fill placeholders while typing. */
const FILL_DELAY_MS = 150;
const PLACEHOLDER_START = "<";

/**
 * Changes whenever the values placeholders can show change, such as the page's
 * object after an edit, so previews fill their placeholders again: a tooltip's
 * `<A000,DataA1>` must follow the damage typed a moment ago.
 */
export const PlaceholderValues = createContext<object | null>(null);

const revisions = new WeakMap<object, number>();
let lastRevision = 0;

function revisionOf(values: object | null): number {
  if (values === null) {
    return 0;
  }

  let revision = revisions.get(values);

  if (revision === undefined) {
    lastRevision += 1;
    revision = lastRevision;
    revisions.set(values, revision);
  }

  return revision;
}

/**
 * Shows text the way the game draws it: colors, line breaks and filled-in
 * `<ID,Column>` placeholders. Updates as the user types.
 */
export function TextPreview({ text }: { text: string }) {
  const values = useContext(PlaceholderValues);

  return <FilledText key={revisionOf(values)} text={text} />;
}

function FilledText({ text }: { text: string }) {
  const [filled, setFilled] = useState(text);

  useEffect(() => {
    setFilled(text);
    if (!text.includes(PLACEHOLDER_START)) {
      return;
    }

    const timer = window.setTimeout(() => {
      api
        .fillPlaceholders(text)
        .then(setFilled)
        .catch(() => {
          // The raw text is still shown; a failed lookup only leaves placeholders unfilled.
        });
    }, FILL_DELAY_MS);

    return () => window.clearTimeout(timer);
  }, [text]);

  if (!text.trim()) {
    return null;
  }

  return (
    <div className="mt-1 rounded border border-line bg-well px-2.5 py-1.5 text-sm leading-relaxed text-ink">
      <p className="mb-0.5 text-[10px] uppercase tracking-wide text-faint">In game</p>
      <GameText text={filled} />
    </div>
  );
}
