type Recipe = {
  title: string;
  description: string;
  usage: string;
  jass: string;
  lua: string;
};

/** Expand the authored guide from the same paired recipe source as the API pages. */
export function expandLifecycleExamples(template: string, examples: Record<string, Recipe>) {
  const body = template.replace(/<!-- lifecycle-example: (\w+) -->/g, (_, name: string) => {
    const example = examples[name];
    if (!example?.jass.trim() || !example.lua.trim()) {
      throw new Error(`Missing paired lifecycle recipe: ${name}`);
    }
    return [
      '<details class="lifecycle-recipe" data-pagefind-ignore>',
      `<summary>Code: ${example.title}</summary>`,
      example.description,
      example.usage,
      `<section data-example-language="jass">\n\n#### JASS\n\n\`\`\`jass\n${example.jass}\n\`\`\`\n\n</section>`,
      `<section data-example-language="lua">\n\n#### Lua\n\n\`\`\`lua\n${example.lua}\n\`\`\`\n\n</section>`,
      "</details>",
    ].join("\n\n");
  });
  if (body.includes("lifecycle-example")) {
    throw new Error("Malformed lifecycle example directive");
  }
  return body;
}
