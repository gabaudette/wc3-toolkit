// AGE-WS-001 and AGE-WS-002 (docs/standards/whitespace.md), which rustfmt and Biome leave
// alone. Formatted code holds one statement per line and indents by nesting, so lines at
// the same indentation are siblings. Strings and comments are tokenized first, since a
// blank line inside a multi-line string would change the program.

const OUTSIDE_CODE = -1;
const PLAIN_CHAR_LENGTH = 2;
const ESCAPE_LENGTH = 3;

type Line = { code: string; indent: number };
type Mode = "code" | "string" | "raw" | "template" | "comment";

class Lexer {
  mode: Mode = "code";
  quote = "";
  rawHashes = 0;
  commentDepth = 0;
  braces = 0;
  templateBraces: number[] = [];

  constructor(readonly rust: boolean) {}

  line(raw: string): Line {
    const startsInCode = this.mode === "code";
    let code = "";
    let i = 0;

    while (i < raw.length) {
      const step = this.step(raw, i);

      code += step.code;
      i = step.next;
      if (step.lineComment) {
        break;
      }
    }

    if (!startsInCode) {
      return { code: "", indent: OUTSIDE_CODE };
    }

    return { code: code.trim(), indent: raw.length - raw.trimStart().length };
  }

  step(raw: string, i: number): { code: string; next: number; lineComment?: boolean } {
    const c = raw[i];
    const next = raw[i + 1];

    if (this.mode === "comment") {
      if (c === "*" && next === "/") {
        this.commentDepth -= 1;
        if (this.commentDepth === 0) {
          this.mode = "code";
        }

        return { code: "", next: i + 2 };
      }

      if (this.rust && c === "/" && next === "*") {
        this.commentDepth += 1;
        return { code: "", next: i + 2 };
      }

      return { code: "", next: i + 1 };
    }

    if (this.mode === "string" || this.mode === "template") {
      if (c === "\\") {
        return { code: "", next: i + 2 };
      }

      if (c === this.quote) {
        this.mode = "code";
        return { code: c, next: i + 1 };
      }

      if (this.mode === "template" && c === "$" && next === "{") {
        this.templateBraces.push(this.braces);
        this.braces += 1;
        this.mode = "code";
        return { code: "${", next: i + 2 };
      }

      return { code: "", next: i + 1 };
    }

    if (this.mode === "raw") {
      const hashes = "#".repeat(this.rawHashes);

      if (c === '"' && raw.startsWith(hashes, i + 1)) {
        this.mode = "code";
        return { code: '"', next: i + 1 + this.rawHashes };
      }

      return { code: "", next: i + 1 };
    }

    return this.codeStep(raw, i);
  }

  codeStep(raw: string, i: number): { code: string; next: number; lineComment?: boolean } {
    const c = raw[i];
    const next = raw[i + 1];

    if (c === "/" && next === "/") {
      return { code: "", next: raw.length, lineComment: true };
    }

    if (c === "/" && next === "*") {
      this.mode = "comment";
      this.commentDepth = 1;
      return { code: "", next: i + 2 };
    }

    const rawString = /^b?r(#*)"/.exec(raw.slice(i));
    if (this.rust && rawString && !/\w/.test(raw[i - 1] ?? "")) {
      this.mode = "raw";
      this.rawHashes = rawString[1].length;
      return { code: '"', next: i + rawString[0].length };
    }

    if (c === '"' || (!this.rust && c === "'")) {
      this.mode = "string";
      this.quote = c;
      return { code: c, next: i + 1 };
    }

    if (!this.rust && c === "`") {
      this.mode = "template";
      this.quote = c;
      return { code: c, next: i + 1 };
    }

    if (this.rust && c === "'") {
      const close = charLiteralEnd(raw, i);
      if (close > 0) {
        return { code: "' '", next: close + 1 };
      }
    }

    if (c === "{") {
      this.braces += 1;
    }

    if (c === "}") {
      this.braces -= 1;
      if (this.templateBraces.at(-1) === this.braces) {
        this.templateBraces.pop();
        this.mode = "template";
        this.quote = "`";
      }
    }

    return { code: c, next: i + 1 };
  }
}

function charLiteralEnd(raw: string, i: number): number {
  let close = i + PLAIN_CHAR_LENGTH;
  if (raw[i + 1] === "\\") {
    close = raw.indexOf("'", i + ESCAPE_LENGTH);
  }

  if (raw[close] === "'") {
    return close;
  }

  return 0;
}

const CONTINUATION =
  /^(\}|\)|\]|>|\/>|else\b|catch\b|finally\b|case\b|default\b|\.|\?|:|&&|\|\||\+|=>|\||as\b|where\b)/;
const DECLARATION = /^(export\s+)?(pub(\([^)]*\))?\s+)?(let|const|static)\b/;
const IMPORT = /^(pub(\([^)]*\))?\s+)?(use|mod|import)\s/;
const MEMBER = /^["']?[\w$-]+["']?\??:/;
const JSX = /^([<{]|[\w-]+=|[\w-]+$)/;

export class Checker {
  readonly lines: Line[];

  constructor(
    readonly raw: string[],
    readonly rust: boolean,
    readonly tsx: boolean,
  ) {
    const lexer = new Lexer(rust);
    this.lines = raw.map((line) => lexer.line(line));
  }

  nextCode(from: number): number {
    for (let j = from; j < this.lines.length; j++) {
      const { code, indent } = this.lines[j];
      if (indent !== OUTSIDE_CODE && (code !== "" || this.raw[j].trim() === "")) {
        return j;
      }
    }

    return -1;
  }

  unseparatedSibling(end: number, indent: number): number {
    const after = end + 1;
    if (after >= this.raw.length || this.raw[after].trim() === "") {
      return -1;
    }

    const j = this.nextCode(after);
    if (j < 0 || this.lines[j].indent !== indent) {
      return -1;
    }

    const code = this.lines[j].code;
    const continues = code === "" || CONTINUATION.test(code) || MEMBER.test(code);
    const arm = this.rust && code.includes("=>");
    const jsx = this.tsx && JSX.test(code);
    if (continues || arm || jsx || IMPORT.test(code)) {
      return -1;
    }

    return j;
  }

  declarationEnd(start: number): number {
    let depth = 0;
    for (let j = start; j < this.lines.length; j++) {
      const { code, indent } = this.lines[j];
      if (indent === OUTSIDE_CODE) {
        continue;
      }

      depth += (code.match(/[([{]/g) ?? []).length - (code.match(/[)\]}]/g) ?? []).length;
      if (depth <= 0 && code.endsWith(";")) {
        return j;
      }

      if (depth < 0) {
        return -1;
      }
    }

    return -1;
  }

  violations(): Map<number, string> {
    const found = new Map<number, string>();
    for (let i = 0; i < this.lines.length; i++) {
      const { code, indent } = this.lines[i];
      if (indent === OUTSIDE_CODE || code === "") {
        continue;
      }

      const closesBlock = code.startsWith("}") && !code.endsWith(",") && !code.endsWith("{");
      if (closesBlock && !code.startsWith("} from") && this.unseparatedSibling(i, indent) >= 0) {
        found.set(i + 1, "AGE-WS-001: a blank line after a block");
        continue;
      }

      if (!DECLARATION.test(code)) {
        continue;
      }

      const end = this.declarationEnd(i);
      if (end < 0) {
        continue;
      }

      const next = this.unseparatedSibling(end, indent);
      if (next >= 0 && !DECLARATION.test(this.lines[next].code)) {
        found.set(end + 1, "AGE-WS-002: a blank line after the last declaration");
      }

      i = end;
    }

    return found;
  }
}

export function missingBlankLines(text: string, path: string): Map<number, string> {
  return new Checker(text.split("\n"), path.endsWith(".rs"), path.endsWith(".tsx")).violations();
}
