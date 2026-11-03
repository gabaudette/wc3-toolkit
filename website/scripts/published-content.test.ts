import { expect, test } from "bun:test";
import { fileURLToPath } from "node:url";
import { Glob } from "bun";

test("published pages keep internal task tracking out of reader content", async () => {
  const pages = new Glob("**/*.{md,mdx,html,astro}");

  for (const root of [
    new URL("../src/", import.meta.url),
    new URL("../public/", import.meta.url),
  ]) {
    for await (const path of pages.scan(fileURLToPath(root))) {
      const content = await Bun.file(new URL(path, root)).text();

      expect(content, path).not.toMatch(/\bAGE-\d+\b|\blinear\.app\//i);
    }
  }
});
