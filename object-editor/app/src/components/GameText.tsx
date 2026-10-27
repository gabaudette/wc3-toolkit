import type { ReactNode } from "react";

/** Warcraft III text codes: `|cAARRGGBB` starts a color, `|r` ends it, `|n` breaks a line. */
const COLOR_CODE = /^\|c[0-9a-f]{2}([0-9a-f]{6})/i;
const COLOR_CODE_LENGTH = 10;
const SHORT_CODE_LENGTH = 2;

type Run = {
  text: string;
  color: string | null;
};

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

export function plainGameText(text: string): string {
  return parseGameText(text)
    .map((run) => run.text)
    .join("")
    .replace(/\n/g, " ");
}

const MIN_CONTRAST = 4.5;
const LIGHTEN_STEP = 0.1;
const MAX_STEPS = 10;
const CHANNEL_MAX = 255;
const HEX_RADIX = 16;
const HEX_PAIR = 2;
/** sRGB to linear light (WCAG relative luminance). */
const LINEAR_LIMIT = 0.039_28;
const LINEAR_SCALE = 12.92;
const GAMMA_OFFSET = 0.055;
const GAMMA_SCALE = 1.055;
const GAMMA = 2.4;
const RED_WEIGHT = 0.2126;
const GREEN_WEIGHT = 0.7152;
const BLUE_WEIGHT = 0.0722;
const CONTRAST_OFFSET = 0.05;

function channels(hex: string): [number, number, number] {
  const value = (start: number) => Number.parseInt(hex.slice(start, start + HEX_PAIR), HEX_RADIX);

  return [value(1), value(1 + HEX_PAIR), value(1 + 2 * HEX_PAIR)];
}

function linear(channel: number): number {
  const c = channel / CHANNEL_MAX;

  if (c <= LINEAR_LIMIT) {
    return c / LINEAR_SCALE;
  }

  return ((c + GAMMA_OFFSET) / GAMMA_SCALE) ** GAMMA;
}

function luminance(hex: string): number {
  const [r, g, b] = channels(hex);

  return RED_WEIGHT * linear(r) + GREEN_WEIGHT * linear(g) + BLUE_WEIGHT * linear(b);
}

const BACKGROUND_LUMINANCE = luminance("#142039");

function contrast(hex: string): number {
  return (luminance(hex) + CONTRAST_OFFSET) / (BACKGROUND_LUMINANCE + CONTRAST_OFFSET);
}

function toHex(rgb: number[]): string {
  return `#${rgb.map((c) => Math.round(c).toString(HEX_RADIX).padStart(HEX_PAIR, "0")).join("")}`;
}

export function readableColor(hex: string): string {
  let rgb: number[] = channels(hex);
  let color = hex;

  for (let step = 0; step < MAX_STEPS && contrast(color) < MIN_CONTRAST; step++) {
    rgb = rgb.map((c) => c + (CHANNEL_MAX - c) * LIGHTEN_STEP);
    color = toHex(rgb);
  }

  return color;
}

type GameTextProps = {
  text: string;
  singleLine?: boolean;
};

export function GameText({ text, singleLine = false }: GameTextProps): ReactNode {
  let className = "whitespace-pre-line";

  if (singleLine) {
    className = "";
  }

  return (
    <span className={className}>
      {parseGameText(text).map((run, index) => {
        const key = `${index}-${run.text.length}`;
        let shown = run.text;

        if (singleLine) {
          shown = shown.replace(/\n/g, " ");
        }

        if (run.color) {
          return (
            <span key={key} style={{ color: readableColor(run.color) }}>
              {shown}
            </span>
          );
        }

        return <span key={key}>{shown}</span>;
      })}
    </span>
  );
}
