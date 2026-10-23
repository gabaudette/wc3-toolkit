import { mkdir, readdir, rm } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { API_CATEGORIES, categoryFor } from "./api-categories";
import { guideKey, validateApiGuides } from "./api-guides";
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
type Example = {
  title: string;
  category: string;
  description: string;
  usage: string;
  jass: string;
  lua: string;
};
type ConceptComparison = {
  title: string;
  anchor: string;
  apis: string[];
  unitFields: string[];
};

// Selected reference pages link to the authored comparisons, outside generated directories.
const conceptComparisons: ConceptComparison[] = [
  {
    title: "Armor Type versus Defense Type",
    anchor: "armor-type-versus-defense-type",
    apis: [],
    unitFields: ["uarm", "udty", "udef"],
  },
  {
    title: "Removing versus killing a unit",
    anchor: "removing-versus-killing-a-unit",
    apis: ["RemoveUnit", "KillUnit", "GroupRemoveUnit"],
    unitFields: [],
  },
  {
    title: "Coordinates versus location handles",
    anchor: "coordinates-versus-location-handles",
    apis: ["GetUnitX", "GetUnitY", "GetUnitLoc", "Location", "RemoveLocation"],
    unitFields: [],
  },
];

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

async function page(
  path: string,
  title: string,
  description: string,
  body: string,
  practicalGuide = false,
) {
  const url = new URL(`${path}.md`, output);
  await mkdir(new URL(".", url), { recursive: true });
  let practicalMetadata = "";
  if (practicalGuide) {
    practicalMetadata = "practicalGuide: true\n";
  }
  await Bun.write(
    url,
    `---\ntitle: ${JSON.stringify(title)}\ndescription: ${JSON.stringify(description)}\neditUrl: false\n${practicalMetadata}---\n\n${body}\n`,
  );
}

function languageSection(language: "jass" | "lua", body: string) {
  return `<section data-example-language="${language}">\n\n${body}\n\n</section>`;
}

// All generated links are relative, so previews, custom domains and Pages subpaths agree.
function link(from: string, to: string) {
  const fromParts = from.split("/");
  const toParts = to.split("/");
  while (fromParts.length && fromParts[0] === toParts[0]) {
    fromParts.shift();
    toParts.shift();
  }
  const relative = `${"../".repeat(fromParts.length)}${toParts.join("/")}`;
  if (!relative) {
    return "./";
  }
  if (relative.endsWith("/")) {
    return relative;
  }
  return `${relative}/`;
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
const practical = validateApiGuides(
  await Bun.file(new URL("data/api-guides.json", root)).json(),
  api.declarations,
  examples,
);

await mkdir(output, { recursive: true });
for (const directory of ["api", "fields", "examples"]) {
  // These ignored directories are owned entirely by this generator.
  await rm(new URL(`${directory}/`, output), { force: true, recursive: true });
}

const apiIndex = [
  "This reference is generated from declarations in your installed game's `common.j` and `blizzard.j`. It includes native functions, Blizzard helpers, handle types and constants. Search by symbol name with **Ctrl K** (or **⌘ K**).",
  `**Source snapshot:** ${markdown(api.version)}. ${api.declarations.length.toLocaleString("en")} declarations. A newer game version may add or change APIs.`,
  languageSection(
    "jass",
    "Pass callbacks as `function Name`, and declare each callback before the function that uses it. Declarations include parameter types and return types.",
  ),
  languageSection(
    "lua",
    "Use the function names and argument order shown in the call syntax, and pass callbacks as function values. Engine types describe expected values; they are not type annotations to paste into your code.",
  ),
  "Blizzard helpers can have their own behavior and overhead; inspect your local script when implementation details matter.",
  "[Read scripting basics](../../scripting/languages/). Generated call syntax uses parameter names as placeholders; it is not a runnable example by itself. Authored examples are labeled separately.",
  "Need a definition? The [map-making glossary](../glossary/) explains raw codes, natives, Blizzard helpers and handle ownership.",
];

const alphabeticalIndex: string[] = [
  "[Browse by topic](../). This complete index groups declarations by script and kind, then sorts each group alphabetically.",
];
for (const source of api.sources) {
  alphabeticalIndex.push(
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
    alphabeticalIndex.push(
      `### ${kind} (${members.length})\n\n${members.map((entry) => `- [${code(entry.name)}](${link("reference/api/alphabetical", apiPath(entry))})`).join("\n")}`,
    );
  }
}

const categoryPath = (id: string) => `reference/api/topics/${id}`;
const sortedCategories = [...API_CATEGORIES].sort((a, b) => a.title.localeCompare(b.title));
const categories = sortedCategories.map((category) => ({
  ...category,
  members: api.declarations
    .filter((entry) => categoryFor(entry).id === category.id)
    .sort((a, b) => a.name.localeCompare(b.name)),
}));
const kindLabels = {
  native: "Native functions",
  function: "Blizzard helpers",
  type: "Handle types",
  constant: "Constants",
};

function countLabel(count: number, singular: string) {
  let label = singular;
  if (count !== 1) {
    label += "s";
  }
  return `${count.toLocaleString("en")} ${label}`;
}

for (const category of categories) {
  const from = categoryPath(category.id);
  const body = [
    category.description,
    `**${category.members.length.toLocaleString("en")} declarations** · [All API topics](${link(from, "reference/api")}) · [Full alphabetical index](${link(from, "reference/api/alphabetical")})`,
    "Functions, types and constants are grouped below. Select a symbol for its declaration, parameter types and Lua call syntax.",
  ];
  const explained = category.members.filter((entry) => practical.guides.has(guideKey(entry)));
  if (explained.length) {
    body.push(
      `## Practical guides\n\n${explained.map((entry) => `- [${code(entry.name)}](${link(from, apiPath(entry))}) — ${markdown(practical.guides.get(guideKey(entry))?.summary ?? "")}`).join("\n")}`,
    );
  }
  for (const kind of ["native", "function", "type", "constant"] as const) {
    const members = category.members.filter((entry) => entry.kind === kind);
    if (!members.length) {
      continue;
    }
    body.push(`## ${kindLabels[kind]} (${members.length})`);
    // Large constant catalogs remain available without burying the callable APIs.
    let list = members
      .map((entry) => `- [${code(entry.name)}](${link(from, apiPath(entry))})`)
      .join("\n");
    if (kind === "constant") {
      list = `<details>\n<summary>Browse ${members.length} constants</summary>\n\n${list}\n\n</details>`;
    }
    body.push(list);
  }
  await page(
    `api/topics/${category.id}/index`,
    category.title,
    category.description,
    body.join("\n\n"),
  );
}

const topicCards = categories.map((category) => {
  const callableCount = category.members.filter(
    (entry) => entry.kind === "native" || entry.kind === "function",
  ).length;
  const constantCount = category.members.filter((entry) => entry.kind === "constant").length;
  const typeCount = category.members.filter((entry) => entry.kind === "type").length;
  const counts: string[] = [];
  if (callableCount) {
    counts.push(countLabel(callableCount, "function"));
  }
  if (typeCount) {
    counts.push(countLabel(typeCount, "type"));
  }
  if (constantCount) {
    counts.push(countLabel(constantCount, "constant"));
  }
  return `<li><a class="api-topic-card" href="${link("reference/api", categoryPath(category.id))}"><strong>${markdown(category.title)}</strong><span>${markdown(category.description)}</span><small>${counts.join(" · ")}</small></a></li>`;
});
const categorizedIndex = [
  "Find native functions, Blizzard helpers, handle types and constants by what you are working on. Each topic brings declarations from `common.j` and `blizzard.j` together.",
  `${api.declarations.length.toLocaleString("en")} declarations · ${categories.length} topics · [Full alphabetical index](./alphabetical/) · [Code examples](../examples/)`,
  `[Start with ${practical.guides.size} practical API guides](./practical/) for parameter meanings, ownership, cleanup and common mistakes. These project-authored explanations supplement the extracted declarations.`,
  "## Browse by topic",
  `<ul class="api-topic-grid" role="list">\n${topicCards.join("\n")}\n</ul>`,
  "## Using the reference",
  ...apiIndex,
  "## Source snapshots",
  ...api.sources.map((source) => `- ${code(source.name)} · SHA-256: ${code(source.sha256)}`),
];

const practicalIndex = [
  `This initial set of **${practical.guides.size} practical guides** covers units, timers, groups, triggers and temporary locations. Each includes parameter meanings, return behavior, usage, ownership, common mistakes, related APIs and a paired JASS/Lua example.`,
  practical.verification,
  `**Source snapshot:** ${markdown(api.version)}. [All API topics](../) · [Full alphabetical index](../alphabetical/) · [Code examples](../../examples/)`,
  "Choose Lua or JASS in the top bar to switch syntax and language-specific guidance. Shared parameter and cleanup notes apply to both languages.",
];
for (const category of categories) {
  const members = category.members
    .map((entry) => practical.guides.get(guideKey(entry)))
    .filter((entry) => entry !== undefined);
  if (members.length) {
    practicalIndex.push(
      `## ${category.title}\n\n${members.map((entry) => `- [${code(entry.declaration.name)}](${link("reference/api/practical", apiPath(entry.declaration))}) — ${markdown(entry.summary)}`).join("\n")}`,
    );
  }
}
await page(
  "api/practical/index",
  "Practical API guides",
  `Practical explanations and JASS/Lua examples for ${practical.guides.size} common APIs.`,
  practicalIndex.join("\n\n"),
  true,
);

for (const declaration of api.declarations) {
  const from = apiPath(declaration);
  const { jass, lua } = signatures(declaration);
  const category = categoryFor(declaration);
  const guide = practical.guides.get(guideKey(declaration));
  const body = [
    `**${declaration.kind}** · ${code(declaration.source)} line ${declaration.line} · [Browse the API](${link(from, "reference/api")})`,
    `Topic: [${category.title}](${link(from, categoryPath(category.id))}).`,
  ];
  const comparison = conceptComparisons.find((entry) => entry.apis.includes(declaration.name));
  if (comparison) {
    body.push(
      `Compare: [${comparison.title}](${link(from, "reference/common-confusions")}#${comparison.anchor}).`,
    );
  }
  if (guide) {
    body.push(
      guide.summary,
      `**Review status:** ${practical.verification}\n\n**Source snapshot:** ${markdown(api.version)}. [Practical guide index](${link(from, "reference/api/practical")}).`,
    );
  }
  body.push(languageSection("jass", `## JASS declaration\n\n\`\`\`jass\n${jass}\n\`\`\``));
  let luaHeading = "Lua call syntax";
  if (declaration.kind === "type" || declaration.kind === "constant") {
    luaHeading = "Lua representation";
  }
  body.push(languageSection("lua", `## ${luaHeading}\n\n\`\`\`lua\n${lua}\n\`\`\``));
  if (declaration.kind === "type") {
    body.push(`Parent type: ${typeLink(from, declaration.returns)}.`);
    body.push(languageSection("lua", "This is an engine handle, not a table you create yourself."));
  } else if (declaration.kind === "constant") {
    body.push(
      `Value type: ${typeLink(from, declaration.returns)}. Use ${code(declaration.name)} by name. Its initialization value is not included in this declaration reference.`,
    );
  } else {
    body.push(
      "Parameter names in the call syntax are placeholders. Supply values of the types below.",
    );
    body.push("## Parameters");
    if (declaration.parameters.length) {
      if (guide) {
        body.push(
          `| Name | Engine type | Meaning |\n| --- | --- | --- |\n${declaration.parameters.map((parameter) => `| ${code(parameter.name)} | ${typeLink(from, parameter.type)} | ${markdown(guide.parameters[parameter.name])} |`).join("\n")}`,
        );
      } else {
        body.push(
          `| Name | Engine type |\n| --- | --- |\n${declaration.parameters.map((parameter) => `| ${code(parameter.name)} | ${typeLink(from, parameter.type)} |`).join("\n")}`,
        );
      }
    } else {
      body.push("This function takes no arguments.");
    }
    let returns = typeLink(from, declaration.returns);
    if (declaration.returns === "nothing") {
      returns = "No return value.";
    }
    body.push(`## Returns\n\n${returns}`);
    if (guide) {
      body.push(
        guide.returns,
        `## How to use it\n\n${guide.usage.map((note) => `- ${markdown(note)}`).join("\n")}`,
        `## Ownership and cleanup\n\n${markdown(guide.ownership)}`,
        `## Common mistakes\n\n${guide.pitfalls.map((note) => `- ${markdown(note)}`).join("\n")}`,
      );
      for (const [language, notes] of [
        ["jass", guide.jassNotes],
        ["lua", guide.luaNotes],
      ] as const) {
        if (notes.length) {
          const noteHeadings = { jass: "JASS notes", lua: "Lua notes" };
          body.push(
            languageSection(
              language,
              `### ${noteHeadings[language]}\n\n${notes.map((note) => `- ${markdown(note)}`).join("\n")}`,
            ),
          );
        }
      }
      if (guide.unverified.length) {
        body.push(
          `## Unverified behavior\n\n${guide.unverified.map((note) => `- ${markdown(note)}`).join("\n")}`,
        );
      }
      body.push(
        `## Related APIs\n\n${guide.related.map((entry) => `- [${code(entry.name)}](${link(from, apiPath(entry))})`).join("\n")}`,
      );
    }
    let example = examples[declaration.name];
    if (guide) {
      example = examples[guide.example];
    }
    if (example) {
      body.push(`## Example\n\n**${example.title}**\n\n${example.description}\n\n${example.usage}`);
      body.push(languageSection("jass", `### JASS\n\n\`\`\`jass\n${example.jass}\n\`\`\``));
      body.push(languageSection("lua", `### Lua\n\n\`\`\`lua\n${example.lua}\n\`\`\``));
      body.push(`[Browse all code examples](${link(from, "reference/examples")})`);
    }
  }
  await page(
    `api/${slug(declaration)}`,
    declaration.name,
    guide?.summary ??
      `${declaration.kind} from ${declaration.source}: JASS declaration, Lua usage and parameter types.`,
    body.join("\n\n"),
    Boolean(guide),
  );
}
await page(
  "api/index",
  "API reference",
  "Browse Warcraft III native functions, Blizzard helpers, handle types and constants by topic.",
  categorizedIndex.join("\n\n"),
);
await page(
  "api/alphabetical/index",
  "API alphabetical index",
  "All Warcraft III API declarations, alphabetically grouped by source file and kind.",
  alphabeticalIndex.join("\n\n"),
);

const exampleIndex = [
  "Choose **Lua** or **JASS** in the top bar (in the menu on mobile). Your choice applies to guides and API examples and is remembered across pages.",
  "Each linked recipe includes setup requirements and cleanup where it owns temporary handles. Call the named entry point after map initialization.",
  languageSection(
    "lua",
    "In the Lua harness, register a no-argument entry point with `Dev.OnStart(ExampleName)`; pass arguments inside a callback when needed.",
  ),
  languageSection(
    "jass",
    "Add functions in declaration order, then call the named entry point from a map initialization trigger. Pass arguments when the recipe requires them.",
  ),
  "Run gameplay changes from synchronized map code. Event-response functions need the event shown in their example. Replace custom raw codes such as `A000` with your map's actual codes. A sample trigger described as map-lifetime should be registered once.",
];
for (const category of new Set(Object.values(examples).map((example) => example.category))) {
  exampleIndex.push(`## ${category}`);
  for (const [name, example] of Object.entries(examples)) {
    if (example.category !== category) {
      continue;
    }
    const declaration = api.declarations.find((entry) => entry.name === name);
    if (!declaration) {
      throw new Error(`Missing API declaration for example ${name}`);
    }
    exampleIndex.push(
      `- [${example.title}](${link("reference/examples", apiPath(declaration))}#example) — ${code(name)}`,
    );
  }
}
await page(
  "examples/index",
  "Code examples",
  "Practical JASS and Lua recipes for units, spells, timers, groups, effects and game systems.",
  exampleIndex.join("\n\n"),
);

let fieldCount = 0;
const fieldIndex = [
  "Every field in the seven project-authored field guides gets its own searchable page. Metadata adds names, types, ranges and choices from the local installation snapshot. Spell-specific ability data fields are also listed, with a clear note where no authored explanation exists.",
  "Know the goal but not the field? [Which field do I change?](../field-tasks/) maps common tasks to exact field labels and raw codes, including companion settings to check.",
  `**Metadata snapshot:** ${markdown(api.version)}. Guide descriptions are read directly from the editor's TOML files on every build.`,
  "Search by raw code (for example `uarm`) or display name. Start with [Armor Type](./unit/uarm/) versus [Defense Type](./unit/udty/): one changes the hit sound, the other changes combat damage.",
  "[Compare armor fields, unit removal/death, and coordinates/location handles](../common-confusions/) for decision guidance and cleanup examples.",
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
    if (kind === "unit") {
      const comparison = conceptComparisons.find((entry) => entry.unitFields.includes(id));
      if (comparison) {
        body.push(
          `Compare: [${comparison.title}](${link(`reference/fields/${kind}/${id.toLowerCase()}`, "reference/common-confusions")}#${comparison.anchor}).`,
        );
      }
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
