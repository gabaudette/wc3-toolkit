import { expect, test } from "bun:test";
import { parse as parseLua } from "luaparse";
import examples from "../data/examples.json";
import { type Declaration, parseJass } from "./jass";

const api: { declarations: Declaration[] } = await Bun.file(
  new URL("../data/api.json", import.meta.url),
).json();

test.each(Object.entries(examples))(
  "%s has a documented, complete JASS/Lua recipe",
  (name, example) => {
    const declaration = api.declarations.find((entry) => entry.name === name);
    expect(declaration?.kind === "native" || declaration?.kind === "function").toBe(true);
    for (const field of [example.title, example.category, example.description, example.usage]) {
      expect(field.trim().length).toBeGreaterThan(0);
    }

    const entryPoint = `Example${name}`;
    const jass = parseJass(example.jass, `${name}.j`);
    expect(jass.some((entry) => entry.name === entryPoint)).toBe(true);
    const lua = parseLua(example.lua, { luaVersion: "5.3" });
    expect(
      lua.body.some(
        (node) =>
          node.type === "FunctionDeclaration" &&
          node.identifier?.type === "Identifier" &&
          node.identifier.name === entryPoint,
      ),
    ).toBe(true);
  },
);

test("recipes can be combined without duplicate JASS function names", () => {
  const names = new Set<string>();
  for (const [name, example] of Object.entries(examples)) {
    for (const declaration of parseJass(example.jass, `${name}.j`)) {
      expect(names.has(declaration.name)).toBe(false);
      names.add(declaration.name);
    }
  }
});
