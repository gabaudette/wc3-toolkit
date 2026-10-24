import { expect, test } from "bun:test";
import examples from "../data/examples.json";
import { expandLifecycleExamples } from "./lifecycle-examples";

const template = await Bun.file(new URL("../data/lifecycles.md", import.meta.url)).text();

test("the guide embeds both languages directly from existing validated recipes", () => {
  const body = expandLifecycleExamples(template, examples);
  const names = [...template.matchAll(/<!-- lifecycle-example: (\w+) -->/g)].map(
    (match) => match[1],
  );
  expect(names.length).toBeGreaterThan(0);
  expect(new Set(names).size).toBe(names.length);
  for (const name of names) {
    const recipe = examples[name as keyof typeof examples];
    expect(recipe).toBeDefined();
    for (const language of ["jass", "lua"] as const) {
      expect(body).toContain(`\`\`\`${language}\n${recipe[language]}\n\`\`\``);
    }
    expect(body).toContain(recipe.usage);
  }
  // Folded duplicate code is excluded from search; the canonical API examples stay indexed.
  expect(body.match(/data-pagefind-ignore/g)?.length).toBe(names.length);
  expect(body).not.toContain("lifecycle-example");
});

test("unknown, incomplete or malformed recipe directives fail before generation", () => {
  expect(() => expandLifecycleExamples("<!-- lifecycle-example: Missing -->", examples)).toThrow();
  expect(() =>
    expandLifecycleExamples("<!-- lifecycle-example: TimerStart -->", {
      TimerStart: { ...examples.TimerStart, lua: " " },
    }),
  ).toThrow();
  expect(() => expandLifecycleExamples("<!-- lifecycle-example: -->", examples)).toThrow();
});
