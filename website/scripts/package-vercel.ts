import { cp, mkdir, readdir, rm } from "node:fs/promises";

const source = new URL("../dist/", import.meta.url);
const output = new URL("../../.vercel/output/", import.meta.url);
const required = [
  "index.html",
  "favicon.ico",
  "pagefind/pagefind.js",
  "reference/fields/unit/uarm/index.html",
  "reference/fields/doodad/index.html",
  "reference/api/common/native/createunit/index.html",
];

for (const file of required) {
  if (!(await Bun.file(new URL(file, source)).exists())) {
    throw new Error(`Cannot package documentation: missing ${file}. Run the build first.`);
  }
}

// This directory is exclusively owned by the Build Output API package step.
await rm(output, { recursive: true, force: true });
await mkdir(output, { recursive: true });
await cp(source, new URL("static/", output), { recursive: true });
await Bun.write(
  new URL("config.json", output),
  `${JSON.stringify(
    {
      version: 3,
      routes: [
        { src: "^/$", dest: "/index.html" },
        { src: "^/((?:[^./]+/)*[^./]+)$", status: 308, headers: { Location: "/$1/" } },
        { src: "^/(.*)/$", dest: "/$1/index.html", check: true },
        { handle: "filesystem" },
        { src: "^/.*$", dest: "/404.html", status: 404 },
      ],
    },
    null,
    2,
  )}\n`,
);

const sourceFiles = await readdir(source, { recursive: true });
const outputFiles = await readdir(new URL("static/", output), { recursive: true });
if (sourceFiles.length !== outputFiles.length) {
  throw new Error("The static deployment package does not match the built site.");
}
process.stdout.write(`Packaged ${outputFiles.length} static entries for Vercel.\n`);
