import { existsSync, readdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";

const APP = resolve(import.meta.dir, "..");
const WORKSPACE = resolve(APP, "..");
const VENDOR = join(WORKSPACE, "vendor");
const OUTPUT = join(APP, "src-tauri", "THIRD-PARTY-NOTICES.txt");
const SETUP_OUTPUT = join(WORKSPACE, "setup", "THIRD-PARTY-NOTICES.txt");
const APP_CRATE = "wc3-object-editor";
const SETUP_CRATE = "tinkerbench-setup";
const LICENSE_FILE = /^(licen[cs]e|copying|notice|copyright|unlicense)/i;
const LICENSE_DIR = /^licen[cs]es$/i;
const RULE_WIDTH = 80;
const RULE = "=".repeat(RULE_WIDTH);
const FAILURE = 1;
const UNKNOWN_LICENSE = "see the project";

type Component = {
  name: string;
  version: string;
  license: string;
  texts: string[];
  repository?: string;
};

type CargoPackage = {
  id: string;
  name: string;
  version: string;
  license: string | null;
  repository: string | null;
  manifest_path: string;
  source: string | null;
};

type CargoNode = { id: string; deps: { pkg: string; dep_kinds: { kind: string | null }[] }[] };

type CargoMetadata = { packages: CargoPackage[]; resolve: { nodes: CargoNode[] } };

type PackageJson = {
  name: string;
  version: string;
  license?: unknown;
  repository?: unknown;
  dependencies?: Record<string, string>;
  optionalDependencies?: Record<string, string>;
  peerDependencies?: Record<string, string>;
};

function read(path: string): string {
  return readFileSync(path, "utf8").replace(/\r\n/g, "\n").trim();
}

function licenseTexts(dir: string): string[] {
  const texts: string[] = [];

  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const path = join(dir, entry.name);

    if (entry.isFile() && LICENSE_FILE.test(entry.name)) {
      texts.push(read(path));
    } else if (entry.isDirectory() && LICENSE_DIR.test(entry.name)) {
      texts.push(...licenseTexts(path));
    }
  }

  return texts.filter(Boolean);
}

function cargoMetadata(): CargoMetadata {
  const manifest = join(WORKSPACE, "Cargo.toml");
  const result = Bun.spawnSync(
    ["cargo", "metadata", "--format-version", "1", "--locked", "--manifest-path", manifest],
    { stderr: "inherit" },
  );

  if (!result.success) {
    process.exit(FAILURE);
  }

  return JSON.parse(result.stdout.toString());
}

function rustComponents(metadata: CargoMetadata, crate: string): Component[] {
  const packages = new Map(metadata.packages.map((p) => [p.id, p]));
  const nodes = new Map(metadata.resolve.nodes.map((n) => [n.id, n]));
  const root = metadata.packages.find((p) => p.name === crate && p.source === null);
  const shipped = new Set<string>();
  const queue: string[] = [];

  if (root) {
    queue.push(root.id);
  }

  for (let id = queue.pop(); id !== undefined; id = queue.pop()) {
    if (shipped.has(id)) {
      continue;
    }

    shipped.add(id);

    for (const dep of nodes.get(id)?.deps ?? []) {
      if (dep.dep_kinds.some(({ kind }) => kind === null)) {
        queue.push(dep.pkg);
      }
    }
  }

  return [...shipped].flatMap((id) => {
    const found = packages.get(id);

    if (!found || found.source === null) {
      return [];
    }

    return [
      {
        name: found.name,
        version: found.version,
        license: found.license ?? UNKNOWN_LICENSE,
        texts: licenseTexts(dirname(found.manifest_path)),
        ...(found.repository && { repository: found.repository }),
      },
    ];
  });
}

function packageDir(name: string, from: string): string | null {
  for (let dir = from; ; dir = dirname(dir)) {
    const candidate = join(dir, "node_modules", name);

    if (existsSync(join(candidate, "package.json"))) {
      return candidate;
    }

    if (dirname(dir) === dir) {
      return null;
    }
  }
}

function repositoryUrl(repository: unknown): string | null {
  if (typeof repository === "string") {
    return repository;
  }

  if (repository && typeof repository === "object" && "url" in repository) {
    return String(repository.url);
  }

  return null;
}

function npmComponents(): Component[] {
  const components = new Map<string, Component>();
  const app: PackageJson = JSON.parse(read(join(APP, "package.json")));
  const queue = Object.keys(app.dependencies ?? {}).map((name) => ({ name, from: APP }));

  for (let next = queue.pop(); next !== undefined; next = queue.pop()) {
    const dir = packageDir(next.name, next.from);

    if (!dir || components.has(dir)) {
      continue;
    }

    const manifest: PackageJson = JSON.parse(read(join(dir, "package.json")));
    let license = UNKNOWN_LICENSE;

    if (typeof manifest.license === "string") {
      license = manifest.license;
    }

    const repository = repositoryUrl(manifest.repository);

    components.set(dir, {
      name: manifest.name,
      version: manifest.version,
      license,
      texts: licenseTexts(dir),
      ...(repository && { repository }),
    });

    const deps = {
      ...manifest.peerDependencies,
      ...manifest.optionalDependencies,
      ...manifest.dependencies,
    };

    queue.push(...Object.keys(deps).map((name) => ({ name, from: dir })));
  }

  return [...components.values()];
}

function headerVersion(path: string, name: string): string {
  return read(path).match(new RegExp(`#define ${name}\\s+"([^"]+)"`))?.[1] ?? "";
}

function zlibNotice(header: string): string {
  const lines = read(header).split("\n");
  const first = lines.findIndex((line) => line.includes("Copyright (C)"));
  const last = lines.findIndex((line) => line.includes("madler@"));

  return lines
    .slice(first, last + 1)
    .map((line) => line.replace(/^ {2}/, ""))
    .join("\n");
}

function vendoredComponents(): Component[] {
  const storm = join(VENDOR, "StormLib");
  const casc = join(VENDOR, "CascLib");
  const stormZlib = join(storm, "src", "zlib", "zlib.h");
  const cascZlib = join(casc, "src", "zlib", "zlib.h");
  const publicDomain = (name: string): Component => ({
    name,
    version: "in StormLib",
    license: "public domain",
    texts: [],
  });

  return [
    {
      name: "StormLib",
      version: headerVersion(join(storm, "src", "StormLib.h"), "STORMLIB_VERSION_STRING"),
      license: "MIT",
      texts: [read(join(storm, "LICENSE"))],
    },
    {
      name: "CascLib",
      version: headerVersion(join(casc, "src", "CascLib.h"), "CASCLIB_VERSION_STRING"),
      license: "MIT",
      texts: [read(join(casc, "LICENSE"))],
    },
    {
      name: "zlib",
      version: `${headerVersion(stormZlib, "ZLIB_VERSION")}, in StormLib`,
      license: "Zlib",
      texts: [zlibNotice(stormZlib)],
    },
    {
      name: "zlib",
      version: `${headerVersion(cascZlib, "ZLIB_VERSION")}, in CascLib`,
      license: "Zlib",
      texts: [zlibNotice(cascZlib)],
    },
    {
      name: "bzip2",
      version: "1.0.5, in StormLib",
      license: "bzip2-1.0.5",
      texts: [read(join(APP, "scripts", "notices", "bzip2-1.0.5.txt"))],
    },
    publicDomain("LZMA SDK"),
    publicDomain("LibTomCrypt"),
    publicDomain("LibTomMath"),
    publicDomain("lookup3 (Bob Jenkins)"),
  ];
}

function label(component: Component): string {
  if (!component.version) {
    return `${component.name} (${component.license})`;
  }

  return `${component.name} ${component.version} (${component.license})`;
}

function byLabel(a: string, b: string): number {
  return a.localeCompare(b, "en");
}

function notices(product: string, components: Component[]): string {
  const byText = new Map<string, Set<string>>();
  const withoutText = new Set<string>();

  for (const component of components) {
    if (component.texts.length === 0) {
      withoutText.add([label(component), component.repository].filter(Boolean).join(", "));
    }

    for (const text of component.texts) {
      byText.set(text, (byText.get(text) ?? new Set()).add(label(component)));
    }
  }

  const sections = [...byText.entries()]
    .map(([text, users]) => ({ users: [...users].sort(byLabel), text }))
    .sort((a, b) => byLabel(a.users[0] ?? "", b.users[0] ?? "") || byLabel(a.text, b.text))
    .map(({ users, text }) => [RULE, ...users, RULE, "", text, ""].join("\n"));

  const untexted = [
    RULE,
    "Components whose package includes no license text (license as declared, and source):",
    RULE,
    "",
    ...[...withoutText].sort(byLabel),
    "",
  ].join("\n");

  return [
    `Third-party software in ${product}`,
    "",
    `${product} includes the software below. Each license text follows the list of components`,
    `it applies to. ${product} itself is under the MIT License (LICENSE).`,
    "",
    ...sections,
    untexted,
  ].join("\n");
}

const metadata = cargoMetadata();
const shared = [
  ...vendoredComponents(),
  ...rustComponents(metadata, APP_CRATE),
  ...npmComponents(),
];
// The setup carries the app, so its notices cover the app's components too.
const outputs = [
  { path: OUTPUT, text: notices("Tinkerbench", shared) },
  {
    path: SETUP_OUTPUT,
    text: notices("Tinkerbench Setup", [...shared, ...rustComponents(metadata, SETUP_CRATE)]),
  },
];

if (process.argv.includes("--check")) {
  const stale = outputs.filter(
    ({ path, text }) => !(existsSync(path) && read(path) === text.trim()),
  );

  for (const { path } of stale) {
    process.stderr.write(`${path} is out of date: run \`bun run notices\` in object-editor/app.\n`);
  }

  if (stale.length > 0) {
    process.exit(FAILURE);
  }
} else {
  for (const { path, text } of outputs) {
    writeFileSync(path, text);
  }
}
