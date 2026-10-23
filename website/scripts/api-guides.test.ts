import { expect, test } from "bun:test";
import catalog from "../data/api-guides.json";
import examples from "../data/examples.json";
import { categoryFor } from "./api-categories";
import { guideKey, validateApiGuides } from "./api-guides";
import type { Declaration } from "./jass";

const api: { declarations: Declaration[] } = await Bun.file(
  new URL("../data/api.json", import.meta.url),
).json();
const initial = catalog.guides[0];
function validate(guides: unknown[], declarations = api.declarations, recipes = examples) {
  return validateApiGuides({ verification: catalog.verification, guides }, declarations, recipes);
}

test("initial practical guides cover units, timers, groups, triggers and locations", () => {
  const result = validate(catalog.guides);
  expect(result.guides.size).toBeGreaterThanOrEqual(20);
  const topics = new Set(
    [...result.guides.values()].map((guide) => categoryFor(guide.declaration).id),
  );
  for (const topic of [
    "units-and-heroes",
    "timers",
    "groups-and-forces",
    "triggers-and-events",
    "geometry",
  ]) {
    expect(topics.has(topic)).toBe(true);
  }
  for (const guide of result.guides.values()) {
    expect(result.guides.get(guideKey(guide.declaration))).toBe(guide);
  }
});

test("parameter notes must cover exactly the current signature", () => {
  expect(() => validate([{ ...initial, parameters: {} }])).toThrow();
  expect(() =>
    validate([{ ...initial, parameters: { ...initial.parameters, stale: "Old name" } }]),
  ).toThrow();
  expect(() =>
    validate([{ ...initial, parameters: { ...initial.parameters, id: " " } }]),
  ).toThrow();
});

test("guides must resolve to a single callable symbol in the specified source", () => {
  expect(() => validate([{ ...initial, source: "missing.j" }])).toThrow();
  expect(() => validate([{ ...initial, name: "unit" }])).toThrow();
  const target = api.declarations.find((d) => d.name === initial.name);
  if (!target) {
    throw new Error("Missing fixture");
  }
  expect(() => validate([initial], [...api.declarations, target])).toThrow();
  expect(() => validate([initial, initial])).toThrow("Duplicate API guide");
});

test("related APIs must be known, unambiguous, distinct and not self-links", () => {
  expect(() => validate([{ ...initial, related: ["MissingApi"] }])).toThrow();
  expect(() => validate([{ ...initial, related: [initial.name] }])).toThrow();
  expect(() => validate([{ ...initial, related: ["RemoveUnit", "RemoveUnit"] }])).toThrow();
  const target = api.declarations.find((d) => d.name === "RemoveUnit");
  if (!target) {
    throw new Error("Missing fixture");
  }
  expect(() =>
    validate([initial], [...api.declarations, { ...target, source: "other.j" }]),
  ).toThrow();
});

test("a reused recipe must exist and demonstrate the API in both languages", () => {
  expect(() => validate([{ ...initial, example: "MissingExample" }])).toThrow();
  expect(() => validate([{ ...initial, example: "DestroyTimer" }])).toThrow();
  const recipes = {
    ...examples,
    CreateUnit: { ...examples.CreateUnit, lua: "function Other() end" },
  };
  expect(() => validate([initial], api.declarations, recipes)).toThrow("both languages");
  const result = validate(catalog.guides);
  expect(result.guides.get("common.j/GetEnumUnit")?.example).toBe("ForGroup");
});

test("required text and language notes cannot silently become empty", () => {
  for (const field of ["summary", "returns", "ownership"]) {
    expect(() => validate([{ ...initial, [field]: " " }])).toThrow();
  }
  for (const field of ["usage", "pitfalls", "related"]) {
    expect(() => validate([{ ...initial, [field]: [] }])).toThrow();
  }
  expect(() => validate([{ ...initial, luaNotes: [" "] }])).toThrow();
  expect(() =>
    validateApiGuides({ ...catalog, verification: "" }, api.declarations, examples),
  ).toThrow();
});

test("unknown fields cannot silently discard misspelled guidance", () => {
  expect(() => validate([{ ...initial, luaNote: ["Misspelled field"] }])).toThrow(
    "Unknown API guide field",
  );
  expect(() =>
    validateApiGuides({ ...catalog, unknown: true }, api.declarations, examples),
  ).toThrow();
});
