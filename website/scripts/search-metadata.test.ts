import { expect, test } from "bun:test";
import { GUIDE_SEARCH_WEIGHT, searchMetadata } from "../src/searchMetadata";

test.each([
  "index",
  "editor/troubleshooting",
  "guides/hero",
  "contributing/documentation",
  "formats/object-data",
  "reference/field-tasks",
  "reference/examples",
])("%s is discoverable as a guide", (id) => {
  expect(searchMetadata(id)).toEqual({
    filters: ["Guides"],
    label: "Guide",
    weight: GUIDE_SEARCH_WEIGHT,
  });
});

test.each([
  "reference/api",
  "reference/api/index.md",
  "reference/api/alphabetical",
  "reference/api/topics/timers",
  "reference/api/common/native/createunit",
  "reference/api/common/type/unit",
  "reference/api/blizzard/function/triggerregisteranyuniteventbj",
])("%s belongs to the API filter", (id) => {
  expect(searchMetadata(id).filters).toEqual(["API"]);
});

test("authored API explanations remain available in both Guides and API", () => {
  for (const id of ["reference/api/common/native/createunit", "reference/api/practical"]) {
    const result = searchMetadata(id, true);
    expect(result.filters).toEqual(["API", "Guides"]);
    expect(result.label).toBe("API guide");
    expect(result.weight).toBeGreaterThan(searchMetadata(id).weight);
  }
});

test.each([
  "reference/fields",
  "reference/fields/unit/index.md",
  "reference/fields/unit/uarm",
  "reference/fields/item/unam",
  "reference/fields/doodad/dvr1",
])("%s belongs to the object-field filter", (id) => {
  expect(searchMetadata(id).filters).toEqual(["Object fields"]);
});

test("classification respects route boundaries and ignores misplaced practical flags", () => {
  expect(searchMetadata("reference/fields/unit/uarm", true).filters).toEqual(["Object fields"]);
  expect(searchMetadata("reference/api-notes").filters).toEqual(["Guides"]);
  expect(searchMetadata("reference/fields-guide").filters).toEqual(["Guides"]);
});
