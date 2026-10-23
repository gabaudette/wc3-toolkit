import type { Declaration } from "./jass";

export type ApiGuide = {
  declaration: Declaration;
  summary: string;
  parameters: Record<string, string>;
  returns: string;
  usage: string[];
  ownership: string;
  pitfalls: string[];
  related: Declaration[];
  example: string;
  jassNotes: string[];
  luaNotes: string[];
  unverified: string[];
};

function record(value: unknown): Record<string, unknown> {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    throw new Error("Expected an API guide object");
  }
  return value as Record<string, unknown>;
}

function text(value: unknown): string {
  if (typeof value !== "string" || !value.trim()) {
    throw new Error("API guide text must be nonempty");
  }
  return value;
}

function list(value: unknown, optional = false): string[] {
  if (optional && value === undefined) {
    return [];
  }
  if (!Array.isArray(value) || (!optional && !value.length)) {
    throw new Error("Expected a nonempty API guide list");
  }
  return value.map(text);
}

function knownFields(value: Record<string, unknown>, allowed: string[]) {
  for (const field of Object.keys(value)) {
    if (!allowed.includes(field)) {
      throw new Error(`Unknown API guide field: ${field}`);
    }
  }
}

export function guideKey(declaration: Pick<Declaration, "source" | "name">) {
  return `${declaration.source}/${declaration.name}`;
}

// Authored input is checked before generation removes the previous reference pages.
export function validateApiGuides(
  input: unknown,
  declarations: Declaration[],
  examples: Record<string, { jass: string; lua: string }>,
) {
  const catalog = record(input);
  knownFields(catalog, ["verification", "guides"]);
  const verification = text(catalog.verification);
  if (!Array.isArray(catalog.guides) || !catalog.guides.length) {
    throw new Error("Expected API guides");
  }
  const guides = new Map<string, ApiGuide>();
  for (const value of catalog.guides) {
    const entry = record(value);
    knownFields(entry, [
      "name",
      "source",
      "summary",
      "parameters",
      "returns",
      "usage",
      "ownership",
      "pitfalls",
      "related",
      "example",
      "jassNotes",
      "luaNotes",
      "unverified",
    ]);
    const name = text(entry.name);
    const source = text(entry.source);
    const matches = declarations.filter((item) => item.name === name && item.source === source);
    const declaration = matches[0];
    if (
      matches.length !== 1 ||
      !declaration ||
      (declaration.kind !== "native" && declaration.kind !== "function")
    ) {
      throw new Error(`Unknown or ambiguous callable API guide: ${source}/${name}`);
    }
    const key = guideKey(declaration);
    if (guides.has(key)) {
      throw new Error(`Duplicate API guide: ${key}`);
    }
    const notes = record(entry.parameters);
    const parameters: Record<string, string> = {};
    const names = declaration.parameters.map((parameter) => parameter.name);
    if (
      Object.keys(notes).length !== names.length ||
      Object.keys(notes).some((n) => !names.includes(n))
    ) {
      throw new Error(`Stale parameter notes: ${key}`);
    }
    for (const parameter of names) {
      parameters[parameter] = text(notes[parameter]);
    }
    const relatedNames = list(entry.related);
    const related = relatedNames.map((relatedName) => {
      const targets = declarations.filter((item) => item.name === relatedName);
      const target = targets[0];
      if (targets.length !== 1 || !target || guideKey(target) === key) {
        throw new Error(`Unknown, ambiguous or self-related API: ${relatedName} in ${key}`);
      }
      return target;
    });
    if (new Set(relatedNames).size !== relatedNames.length) {
      throw new Error(`Duplicate related API: ${key}`);
    }
    const example = text(entry.example);
    const recipe = examples[example];
    if (!recipe) {
      throw new Error(`Missing example: ${example} in ${key}`);
    }
    const call = new RegExp(`\\b${name}\\s*\\(`);
    if (!call.test(recipe.jass) || !call.test(recipe.lua)) {
      throw new Error(`Example ${example} must call ${name} in both languages`);
    }
    guides.set(key, {
      declaration,
      summary: text(entry.summary),
      parameters,
      returns: text(entry.returns),
      usage: list(entry.usage),
      ownership: text(entry.ownership),
      pitfalls: list(entry.pitfalls),
      related,
      example,
      jassNotes: list(entry.jassNotes, true),
      luaNotes: list(entry.luaNotes, true),
      unverified: list(entry.unverified, true),
    });
  }
  return { verification, guides };
}
