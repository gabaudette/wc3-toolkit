import { createContext, useContext, useEffect, useMemo, useState } from "react";
import { api } from "../api";
import { textProblems, unfilledPlaceholders } from "../gameTextCheck";
import { whenOk } from "../result";
import { GameText } from "./GameText";

const FILL_DELAY_MS = 150;
const PLACEHOLDER_START = "<";

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

export function TextPreview({ text }: { text: string }) {
  const values = useContext(PlaceholderValues);
  const [help, setHelp] = useState(false);

  return (
    <FilledText
      key={revisionOf(values)}
      text={text}
      help={help}
      onToggleHelp={() => setHelp(!help)}
    />
  );
}

function FormattingHelp() {
  return (
    <ul className="mb-1.5 flex flex-col gap-0.5 text-xs text-muted">
      <li>
        <code className="font-mono text-ink">|cffRRGGBB</code> …{" "}
        <code className="font-mono text-ink">|r</code> colors text; select text in the box to pick a
        color.
      </li>
      <li>
        <code className="font-mono text-ink">|n</code> starts a new line.
      </li>
      <li>
        <code className="font-mono text-ink">&lt;AHbz,DataA1&gt;</code> shows a value from the game
        data: an object ID and a column. Add <code className="font-mono text-ink">,%</code> for a
        percent.
      </li>
    </ul>
  );
}

type FilledTextProps = {
  text: string;
  help: boolean;
  onToggleHelp: () => void;
};

function FilledText({ text, help, onToggleHelp }: FilledTextProps) {
  const [filled, setFilled] = useState({ text, from: text });
  const problems = useMemo(() => textProblems(text), [text]);
  let unfilled: string[] = [];

  if (filled.from === text && text.includes(PLACEHOLDER_START)) {
    unfilled = unfilledPlaceholders(filled.text);
  }

  useEffect(() => {
    setFilled({ text, from: "" });
    if (!text.includes(PLACEHOLDER_START)) {
      setFilled({ text, from: text });
      return;
    }

    const timer = window.setTimeout(() => {
      void whenOk(api.fillPlaceholders(text), (result) => setFilled({ text: result, from: text }));
    }, FILL_DELAY_MS);

    return () => window.clearTimeout(timer);
  }, [text]);

  if (!text.trim()) {
    return null;
  }

  return (
    <div className="mt-1 rounded border border-line bg-well px-2.5 py-1.5 text-sm leading-relaxed text-ink">
      <div className="mb-0.5 flex items-center gap-2">
        <p className="flex-1 text-[10px] uppercase tracking-wide text-faint">In game</p>
        <button
          type="button"
          onMouseDown={(e) => e.preventDefault()}
          onClick={onToggleHelp}
          aria-expanded={help}
          className="-my-1 rounded px-1.5 py-1 text-[11px] text-faint hover:text-ink"
        >
          Formatting help
        </button>
      </div>
      {help && <FormattingHelp />}
      <GameText text={filled.text} />
      {(problems.length > 0 || unfilled.length > 0) && (
        <ul aria-live="polite" className="mt-1.5 flex flex-col gap-0.5 text-xs text-danger">
          {problems.map((problem) => (
            <li key={problem}>{problem}</li>
          ))}
          {unfilled.map((placeholder) => (
            <li key={placeholder}>
              Tinkerbench cannot fill {placeholder}: check the object ID and the column.
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
