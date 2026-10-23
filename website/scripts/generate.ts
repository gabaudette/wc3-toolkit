import { mkdir, readdir, rm } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import type { Declaration } from "./jass";
import { signatures } from "./jass";

type ApiSnapshot = {
  version: string;
  sources: { name: string; sha256: string }[];
  declarations: Declaration[];
};
type Field = {
  id: string;
  name: string;
  value_type: string;
  min?: string;
  max?: string;
  per_level: boolean;
  cosmetic: boolean;
  applies_to: string[];
  excludes: string[];
  choices: string[];
};
type GuideEntry = { ids: string[]; description: string; advanced?: boolean; unverified?: boolean };
type Guide = { field: GuideEntry[] };
type Example = { description: string; jass: string; lua: string };

const root = new URL("../", import.meta.url);
const output = new URL("src/content/docs/reference/", root);
const guides = new URL("../object-editor/crates/wc3-data/data/", root);
const api: ApiSnapshot = await Bun.file(new URL("data/api.json", root)).json();
const fields = Bun.TOML.parse(await Bun.file(new URL("data/fields.toml", root)).text()) as Record<
  string,
  Field[]
>;
const examples: Record<string, Example> = await Bun.file(
  new URL("data/examples.json", root),
).json();
const catalogs = ["unit", "ability", "item", "upgrade", "buff", "destructable", "doodad"];
const markdown = (value: string) =>
  value.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/\|/g, "\\|");
const code = (value: string) => `\`${value.replace(/`/g, "")}\``;
const slug = (declaration: Declaration) =>
  `${declaration.source.replace(".j", "")}/${declaration.kind}/${declaration.name.toLowerCase()}`;
const apiPath = (declaration: Declaration) => `reference/api/${slug(declaration)}`;

async function page(path: string, title: string, description: string, body: string) {
  const url = new URL(`${path}.md`, output);
  await mkdir(new URL(".", url), { recursive: true });
  await Bun.write(
    url,
    `---\ntitle: ${JSON.stringify(title)}\ndescription: ${JSON.stringify(description)}\neditUrl: false\n---\n\n${body}\n`,
  );
}

// All generated links are relative, so previews, custom domains and Pages subpaths agree.
function link(from: string, to: string) {
  const fromParts = from.split("/");
  const toParts = to.split("/");
  while (fromParts.length && fromParts[0] === toParts[0]) {
    fromParts.shift();
    toParts.shift();
  }
  return `${"../".repeat(fromParts.length)}${toParts.join("/")}/`;
}

const types = new Map(
  api.declarations.filter((entry) => entry.kind === "type").map((entry) => [entry.name, entry]),
);
function typeLink(from: string, name: string) {
  const target = types.get(name);
  if (target) {
    return `[${code(name)}](${link(from, apiPath(target))})`;
  }
  return code(name);
}

// Detect collisions before touching the previous output.
const paths = new Set<string>();
for (const declaration of api.declarations) {
  const path = slug(declaration);
  if (paths.has(path)) {
    throw new Error(`API URL collision: ${path}`);
  }
  paths.add(path);
}
for (const name of Object.keys(examples)) {
  if (!api.declarations.some((declaration) => declaration.name === name)) {
    throw new Error(`Example refers to an unknown API: ${name}`);
  }
}

await mkdir(output, { recursive: true });
for (const directory of ["api", "fields"]) {
  // These two ignored directories are owned entirely by this generator.
  await rm(new URL(`${directory}/`, output), { force: true, recursive: true });
}

const apiIndex = [
  "This reference is generated from declarations in your installed game's `common.j` and `blizzard.j`. It includes native functions, Blizzard helpers, handle types and constants. Search by symbol name with **Ctrl K** (or **⌘ K**).",
  `**Source snapshot:** ${markdown(api.version)}. ${api.declarations.length.toLocaleString("en")} declarations. A newer game version may add or change APIs.`,
  "Function names and argument order carry over to Lua. JASS types describe the engine's API; they do not become Lua type annotations. In JASS, pass callbacks as `function Name`; in Lua, pass a function value. Blizzard helpers can have their own behavior and overhead; inspect your local script when implementation details matter.",
  "[Read the JASS and Lua introduction](../../scripting/languages/). Generated call syntax uses parameter names as placeholders; it is not a runnable example by itself. Authored examples are labeled separately.",
];

for (const source of api.sources) {
  apiIndex.push(
    `## ${source.name}\n\nSource SHA-256: ${code(source.sha256)}. Implementations and source comments are not included.`,
  );
  const declarations = api.declarations
    .filter((entry) => entry.source === source.name)
    .sort((a, b) => a.name.localeCompare(b.name));
  for (const kind of ["native", "function", "type", "constant"]) {
    const members = declarations.filter((entry) => entry.kind === kind);
    if (!members.length) {
      continue;
    }
    apiIndex.push(
      `### ${kind} (${members.length})\n\n${members.map((entry) => `- [${code(entry.name)}](./${slug(entry)}/)`).join("\n")}`,
    );
  }
}

for (const declaration of api.declarations) {
  const from = apiPath(declaration);
  const { jass, lua } = signatures(declaration);
  const body = [
    `**${declaration.kind}** · ${code(declaration.source)} line ${declaration.line} · [Browse the API](${link(from, "reference/api")})`,
    `## JASS declaration\n\n\`\`\`jass\n${jass}\n\`\`\``,
  ];
  if (declaration.kind === "type") {
    body.push(
      `Parent type: ${typeLink(from, declaration.returns)}. In Lua this is an engine handle, not a table you create yourself.`,
    );
  } else if (declaration.kind === "constant") {
    body.push(
      `Value type: ${typeLink(from, declaration.returns)}. Use ${code(declaration.name)} by name in JASS or Lua. Its initialization value is not included in this declaration reference.`,
    );
  } else {
    body.push(
      `## Lua call syntax\n\n\`\`\`lua\n${lua}\n\`\`\`\n\nParameter names above are placeholders. Supply values of the types below.`,
    );
    body.push("## Parameters");
    if (declaration.parameters.length) {
      body.push(
        `| Name | JASS type |\n| --- | --- |\n${declaration.parameters.map((parameter) => `| ${code(parameter.name)} | ${typeLink(from, parameter.type)} |`).join("\n")}`,
      );
    } else {
      body.push("This function takes no arguments.");
    }
    let returns = typeLink(from, declaration.returns);
    if (declaration.returns === "nothing") {
      returns = "No return value (`nothing` in JASS).";
    }
    body.push(`## Returns\n\n${returns}`);
    const example = examples[declaration.name];
    if (example) {
      body.push(
        `## Example\n\n${example.description}\n\n### JASS\n\n\`\`\`jass\n${example.jass}\n\`\`\`\n\n### Lua\n\n\`\`\`lua\n${example.lua}\n\`\`\``,
      );
    }
  }
  await page(
    `api/${slug(declaration)}`,
    declaration.name,
    `${declaration.kind} from ${declaration.source}: JASS declaration, Lua usage and parameter types.`,
    body.join("\n\n"),
  );
}
await page(
  "api/index",
  "JASS & Lua API",
  "Search native functions, Blizzard helpers, handle types and constants.",
  apiIndex.join("\n\n"),
);

let fieldCount = 0;
const fieldIndex = [
  "Every field in the seven project-authored field guides gets its own searchable page. Metadata adds names, types, ranges and choices from the local installation snapshot. Spell-specific ability data fields are also listed, with a clear note where no authored explanation exists.",
  `**Metadata snapshot:** ${markdown(api.version)}. Guide descriptions are read directly from the editor's TOML files on every build.`,
  "Search by raw code (for example `uarm`) or display name. Start with [Armor Type](./unit/uarm/) versus [Defense Type](./unit/udty/): one changes the hit sound, the other changes combat damage.",
];

for (const kind of catalogs) {
  const guide = Bun.TOML.parse(
    await Bun.file(new URL(`${kind}-fields.toml`, guides)).text(),
  ) as Guide;
  const descriptions = new Map<string, GuideEntry>();
  for (const entry of guide.field) {
    for (const id of entry.ids) {
      if (descriptions.has(id)) {
        throw new Error(`${kind}-fields.toml: duplicate guide ID ${id}`);
      }
      descriptions.set(id, entry);
    }
  }
  const catalog = new Map<string, Field>();
  for (const field of fields[kind] ?? []) {
    if (catalog.has(field.id)) {
      throw new Error(`Duplicate ${kind} metadata: ${field.id}`);
    }
    catalog.set(field.id, field);
  }
  const ids = [...new Set([...catalog.keys(), ...descriptions.keys()])].sort();
  if (!ids.length) {
    throw new Error(`No fields for ${kind}`);
  }

  const index = [
    "Generated from the editor's field guide and the versioned metadata snapshot. **Unverified** means the explanation has not been confirmed in game.",
    "| Field | Raw code |\n| --- | --- |",
  ];
  for (const id of ids) {
    const field = catalog.get(id);
    const entry = descriptions.get(id);
    let name = id;
    if (field) {
      name = field.name;
    }
    const body = [
      `**Raw code:** ${code(id)} · **Object type:** ${kind} · [All ${kind} fields](../)`,
    ];
    if (entry) {
      body.push(entry.description);
      if (entry.unverified) {
        body.push(
          ":::caution[Unverified]\nThis explanation has not been confirmed in game. Test it in a small map before relying on it.\n:::",
        );
      }
      if (entry.advanced) {
        body.push("**Advanced field:** folded under Advanced in the editor.");
      }
    } else {
      body.push(
        "This field is present in the game's metadata. There is no project-authored explanation yet. For spell-specific data, the name and applicable base ability are the starting point; test its behavior in game.",
      );
    }
    if (field) {
      body.push(
        `## Values\n\n- **Metadata type:** ${code(field.value_type)}\n- **Minimum:** ${code(field.min ?? "not specified")}\n- **Maximum:** ${code(field.max ?? "not specified")}\n- **Per level:** ${String(field.per_level)}\n- **Cosmetic (netsafe):** ${String(field.cosmetic)}`,
      );
      body.push(
        "Metadata ranges describe the editor's accepted values; they do not guarantee that every value behaves as expected in game.",
      );
      if (field.applies_to.length) {
        body.push(`**Applies to base ability codes:** ${field.applies_to.map(code).join(", ")}.`);
      }
      if (field.excludes.length) {
        body.push(`**Excluded base ability codes:** ${field.excludes.map(code).join(", ")}.`);
      }
      if (field.choices.length) {
        body.push(
          `### Choices\n\n${field.choices.map((choice) => `- ${markdown(choice)}`).join("\n")}`,
        );
      }
    } else {
      body.push(
        "This field is covered by the guide but absent from the metadata snapshot. Its name, range and choices cannot be verified for this installation.",
      );
    }
    body.push(
      `## Source\n\n[Edit the ${kind} field guide](https://github.com/gabaudette/wc3-toolkit/blob/main/object-editor/crates/wc3-data/data/${kind}-fields.toml). Generated pages are rebuilt automatically; change the guide rather than this page.`,
    );
    await page(
      `fields/${kind}/${id.toLowerCase()}`,
      `${name} · ${id}`,
      `${kind} field ${id}: ${entry?.description ?? name}`,
      body.join("\n\n"),
    );
    index.push(`| [${markdown(name)}](./${id.toLowerCase()}/) | ${code(id)} |`);
  }
  await page(
    `fields/${kind}/index`,
    `${kind[0].toUpperCase()}${kind.slice(1)} fields`,
    `${ids.length} ${kind} fields with raw codes and descriptions.`,
    index.join("\n"),
  );
  fieldIndex.push(
    `- [${kind[0].toUpperCase()}${kind.slice(1)} fields](./${kind}/) — ${ids.length} fields`,
  );
  fieldCount += ids.length;
}
await page(
  "fields/index",
  "Object field reference",
  "Every project field guide, with names, values and gotchas.",
  fieldIndex.join("\n\n"),
);

const generated = await readdir(fileURLToPath(output));
if (!generated.includes("api") || !generated.includes("fields")) {
  throw new Error("Reference output is incomplete");
}
process.stdout.write(
  `Generated ${api.declarations.length} API pages and ${fieldCount} field pages.\n`,
);
