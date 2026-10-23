import { existsSync } from "node:fs";
import { readdir } from "node:fs/promises";
import { resolve } from "node:path";

const directory = resolve(import.meta.dir, "../dist");
const base = process.env.DOCS_BASE || "/";
const MAX_REPORTED_FAILURES = 30;
const origin = "https://docs.invalid";
const files = await readdir(directory, { recursive: true });
const failures = new Set<string>();
let pages = 0;

for (const file of files) {
  if (!file.endsWith(".html") || file === "designs/index.html") {
    continue;
  }
  const url = new URL(`${base.replace(/\/$/, "")}/${file.replace(/index\.html$/, "")}`, origin);
  const rewriter = new HTMLRewriter().on("*", {
    element(element) {
      const href = element.getAttribute("href") ?? element.getAttribute("src");
      if (!href || href.startsWith("#")) {
        return;
      }
      const target = new URL(href, url);
      if (target.origin !== origin) {
        return;
      }
      if (base !== "/" && target.pathname !== base && !target.pathname.startsWith(`${base}/`)) {
        failures.add(`${file}: escapes site base: ${href}`);
        return;
      }
      let localPath = target.pathname;
      if (base !== "/") {
        localPath = localPath.slice(base.length);
      }
      localPath = decodeURIComponent(localPath).replace(/^\//, "");
      if (localPath.endsWith("/") || !localPath) {
        localPath += "index.html";
      }
      if (!existsSync(resolve(directory, localPath))) {
        failures.add(`${file}: missing target: ${href}`);
      }
    },
  });
  await rewriter.transform(new Response(Bun.file(resolve(directory, file)))).text();
  pages += 1;
}

if (failures.size) {
  throw new Error(
    `Broken local links:\n${[...failures].slice(0, MAX_REPORTED_FAILURES).join("\n")}\n${failures.size} failures total.`,
  );
}
process.stdout.write(`Checked local links and assets in ${pages} pages.\n`);
