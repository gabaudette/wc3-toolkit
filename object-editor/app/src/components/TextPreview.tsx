import { useEffect, useState } from "react";
import { api } from "../api";
import { GameText } from "./GameText";

/** Delay before asking the backend to fill placeholders while typing. */
const FILL_DELAY_MS = 150;
const PLACEHOLDER_START = "<";

/**
 * Shows text the way the game draws it: colors, line breaks and filled-in
 * `<ID,Column>` placeholders. Updates as the user types.
 */
export function TextPreview({ text }: { text: string }) {
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
