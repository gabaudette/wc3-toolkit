import type { ReactNode } from "react";

/** Warcraft III text codes: `|cAARRGGBB` starts a color, `|r` ends it, `|n` breaks a line. */
const COLOR_CODE = /^\|c[0-9a-f]{2}([0-9a-f]{6})/i;
const COLOR_CODE_LENGTH = 10;
const SHORT_CODE_LENGTH = 2;

type Run = {
  text: string;
  color: string | null;
};

/** Splits text into runs of one color, with "\n" for line breaks. */
export function parseGameText(text: string): Run[] {
  const runs: Run[] = [];
  let color: string | null = null;
  let current = "";
  let i = 0;

  function flush() {
    if (current) {
      runs.push({ text: current, color });
      current = "";
    }
  }

  while (i < text.length) {
    const rest = text.slice(i);
    const colorMatch = COLOR_CODE.exec(rest);
    const lower = rest.slice(0, SHORT_CODE_LENGTH).toLowerCase();

    if (colorMatch?.[1]) {
      flush();
      color = `#${colorMatch[1]}`;
      i += COLOR_CODE_LENGTH;
    } else if (lower === "|r") {
      flush();
      color = null;
      i += SHORT_CODE_LENGTH;
    } else if (lower === "|n") {
      current += "\n";
      i += SHORT_CODE_LENGTH;
    } else {
      current += text[i];
      i += 1;
    }
  }

  flush();
  return runs;
}

/** Renders game text with its colors and line breaks, like in-game tooltips. */
export function GameText({ text }: { text: string }): ReactNode {
  return (
    <span className="whitespace-pre-line">
      {parseGameText(text).map((run, index) => {
        const key = `${index}-${run.text.length}`;
        if (run.color) {
          return (
            <span key={key} style={{ color: run.color }}>
              {run.text}
            </span>
          );
        }

        return <span key={key}>{run.text}</span>;
      })}
    </span>
  );
}
