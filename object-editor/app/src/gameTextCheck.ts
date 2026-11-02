const COLOR_START = /^\|c[0-9a-f]{8}/i;
const COLOR_CODE_LENGTH = 10;
const SHORT_CODE_LENGTH = 2;
const PLACEHOLDER = /^<([^<>,]+),([^<>,]+)(,%)?>/;
const FILLABLE = /<[^<>,]+,[^<>,]+(,%)?>/g;
const SNIPPET_LENGTH = 12;

function snippet(text: string): string {
  return text.slice(0, SNIPPET_LENGTH);
}

function codeProblem(rest: string, open: boolean): { problem: string | null; skip: number } {
  const code = rest.slice(0, SHORT_CODE_LENGTH).toLowerCase();

  if (code === "|c") {
    if (COLOR_START.test(rest)) {
      return { problem: null, skip: COLOR_CODE_LENGTH };
    }

    return {
      problem: `“${snippet(rest)}”: a color needs 8 hex digits after |c, like |cffffcc00.`,
      skip: SHORT_CODE_LENGTH,
    };
  }

  if (code === "|r") {
    if (open) {
      return { problem: null, skip: SHORT_CODE_LENGTH };
    }

    return { problem: "“|r” ends a color, but no color is open there.", skip: SHORT_CODE_LENGTH };
  }

  if (code === "|n") {
    return { problem: null, skip: SHORT_CODE_LENGTH };
  }

  return {
    problem: `“${snippet(code)}” is not a text code; the codes are |c, |r and |n.`,
    skip: 1,
  };
}

function placeholderProblem(rest: string): { problem: string | null; skip: number } {
  const match = PLACEHOLDER.exec(rest);

  if (match) {
    return { problem: null, skip: match[0].length };
  }

  if (!rest.includes(">")) {
    return { problem: "“<” starts a placeholder that is never closed with “>”.", skip: 1 };
  }

  return {
    problem: `“${snippet(rest)}” is not a placeholder like <AHbz,DataA1> or <AHbz,DataA1,%>.`,
    skip: 1,
  };
}

export function textProblems(text: string): string[] {
  const problems: string[] = [];
  let open = false;
  let i = 0;

  while (i < text.length) {
    const rest = text.slice(i);
    let found: { problem: string | null; skip: number } = { problem: null, skip: 1 };

    if (rest.startsWith("|")) {
      found = codeProblem(rest, open);
      const code = rest.slice(0, SHORT_CODE_LENGTH).toLowerCase();

      if (code === "|c" && found.problem === null) {
        open = true;
      }

      if (code === "|r") {
        open = false;
      }
    } else if (rest.startsWith("<")) {
      found = placeholderProblem(rest);
    }

    if (found.problem) {
      problems.push(found.problem);
    }

    i += found.skip;
  }

  if (open) {
    problems.push("A color is never ended with |r, so it runs to the end of the text.");
  }

  return [...new Set(problems)];
}

export function unfilledPlaceholders(filled: string): string[] {
  return [...new Set(filled.match(FILLABLE) ?? [])];
}
