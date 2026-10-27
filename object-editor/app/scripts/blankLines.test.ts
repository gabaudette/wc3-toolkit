import { describe, expect, test } from "bun:test";
import { missingBlankLines } from "./blankLines";

const lines = (...text: string[]) => text.join("\n");

describe("missingBlankLines", () => {
  test("wants a blank line after a block and after the last declaration", () => {
    const rust = lines(
      "fn f() {",
      "    let a = 1;",
      "    let b = 2;",
      "    if a > b {",
      "        g();",
      "    }",
      "    h();",
      "}",
    );

    expect([...missingBlankLines(rust, "f.rs").keys()]).toEqual([3, 6]);
  });

  test("leaves continuations, match arms and JSX alone", () => {
    const rust = lines(
      "fn f(x: bool) -> u8 {",
      "    if x {",
      "        1",
      "    } else {",
      "        2",
      "    };",
      "",
      "    match x {",
      "        true => {",
      "            g();",
      "        }",
      "        false => h(),",
      "    }",
      "}",
    );

    expect(missingBlankLines(rust, "f.rs").size).toBe(0);

    const tsx = lines(
      "<div>",
      "  {open && (",
      "    <p>Hi</p>",
      "  )}",
      "  {list.map((item) => {",
      "    return item;",
      "  })}",
      "  <span />",
      "</div>",
    );

    expect(missingBlankLines(tsx, "f.tsx").size).toBe(0);
  });

  test("never counts lines inside strings or comments", () => {
    const rust = lines(
      "fn f() -> String {",
      '    let lua = r#"',
      "    if x then",
      "    end",
      "    y()",
      '"#;',
      "",
      "    /* if x {",
      "    }",
      "    y(); */",
      "    lua.to_owned()",
      "}",
    );

    expect(missingBlankLines(rust, "f.rs").size).toBe(0);

    const ts = lines("const code = `", "function f() {", "}", "f();`;", "", "run(code);");

    expect(missingBlankLines(ts, "f.ts").size).toBe(0);
  });

  test("ends a declaration where its brackets close", () => {
    const ts = lines(
      "function f() {",
      "  const next = items",
      "    .filter((i) => i)",
      "    .map((i) => {",
      "      return i;",
      "    });",
      "  use(next);",
      "}",
    );

    expect([...missingBlankLines(ts, "f.ts").keys()]).toEqual([6]);
  });
});
