import { createHash } from "node:crypto";
import { resolve } from "node:path";
import { parseJass } from "./jass";

const [directory, version] = Bun.argv.slice(2);
if (!directory || !version) {
  throw new Error('Usage: bun run refresh:api /path/to/local/exports "game version"');
}

const sources = [];
const declarations = [];
for (const name of ["common.j", "blizzard.j"]) {
  const content = await Bun.file(resolve(directory, name)).text();
  sources.push({ name, sha256: createHash("sha256").update(content).digest("hex") });
  declarations.push(...parseJass(content, name));
}

const names = new Set<string>();
for (const declaration of declarations) {
  // Source filenames are part of the URL, preserving any cross-file collisions.
  const key = `${declaration.source}/${declaration.name}`;
  if (names.has(key)) {
    throw new Error(`Duplicate API key: ${key}`);
  }
  names.add(key);
}

await Bun.write(
  new URL("../data/api.json", import.meta.url),
  `${JSON.stringify({ version, sources, declarations }, null, 2)}\n`,
);
await Bun.write(
  new URL("../data/fields.toml", import.meta.url),
  Bun.file(resolve(directory, "fields.toml")),
);
process.stdout.write(
  `Saved ${declarations.length} declarations. No game implementations were retained.\n`,
);
