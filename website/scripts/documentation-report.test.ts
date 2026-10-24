import { expect, test } from "bun:test";
import { documentationReportUrl } from "../src/documentationReport";

test("an authored page opens a draft with its exact title, canonical URL and useful prompts", () => {
  const title = "JASS & Lua: #1 + 50% / été?";
  const report = new URL(
    documentationReportUrl({
      title,
      canonicalUrl: "https://docs.example.test/guide/?private=ignored#heading",
    }),
  );
  expect(report.origin + report.pathname).toBe(
    "https://github.com/gabaudette/wc3-toolkit/issues/new",
  );
  expect([...report.searchParams.keys()]).toEqual(["title", "body"]);
  expect(report.searchParams.get("title")).toBe(`Docs: ${title}`);
  const body = report.searchParams.get("body");
  expect(body).toContain(`Page: ${title}\nURL: https://docs.example.test/guide/`);
  expect(body).not.toContain("private");
  expect(body).not.toContain("#heading");
  expect(body).not.toContain("Code language:");
  expect(body).toContain("## What is wrong or unclear?");
  expect(body).toContain("## What did you expect?");
  expect(body).toContain("exact error text");
});

test("reference reports retain API qualification and the selected language", () => {
  for (const [language, label] of [
    ["lua", "Lua"],
    ["jass", "JASS"],
  ] as const) {
    const report = new URL(
      documentationReportUrl({
        title: "CreateTimer",
        canonicalUrl: "https://docs.example.test/reference/api/common/native/createtimer/",
        reference: "API: CreateTimer (common.j, native)",
        language,
      }),
    );
    expect(report.searchParams.get("body")).toContain("API: CreateTimer (common.j, native)");
    expect(report.searchParams.get("body")).toContain(`Code language: ${label}`);
  }
});

test("field reports preserve case-sensitive raw codes and deployment subpaths", () => {
  const report = new URL(
    documentationReportUrl({
      title: "Data - Follow Through Time · Ncl1",
      canonicalUrl: new URL("https://docs.example.test/docs/reference/fields/ability/ncl1/"),
      reference: "Field: Ncl1 (ability)",
    }),
  );
  const body = report.searchParams.get("body");
  expect(body).toContain("URL: https://docs.example.test/docs/reference/fields/ability/ncl1/");
  expect(body).toContain("Field: Ncl1 (ability)");
  expect(body).not.toContain("Code language:");
});
