import template from "../../../docs/assistant-prompt.md?raw";
import type { AssistantContext, MapContext } from "./types";

const GOAL = "{{goal}}";
const MAP = "{{map}}";
const NO_GOAL = "(Not written yet: ask me what kind of map I want to make.)";
const NO_MAP = "No map is open in Tinkerbench yet; I may start from a new one.";

function objectLine(map: MapContext): string {
  const parts = map.objects
    .filter((o) => o.custom > 0 || o.changed > 0)
    .map((o) => `${o.kind} ${o.custom} custom and ${o.changed} changed`);

  if (parts.length === 0) {
    return "- Objects: none added or changed yet.";
  }

  return `- Objects: ${parts.join("; ")}.`;
}

function listLines(title: string, items: string[]): string[] {
  if (items.length === 0) {
    return [];
  }

  return [`- ${title}:`, ...items.map((item) => `  - ${item}`)];
}

export function mapSummary(context: AssistantContext): string {
  const map = context.map;

  if (!map) {
    return NO_MAP;
  }

  const lines = [`- File: ${map.name}`];

  if (map.script) {
    lines.push(`- Script language: ${map.script}`);
  }

  if (context.balance) {
    lines.push(`- Game balance data: ${context.balance}`);
  }

  lines.push(...listLines("Players", map.players));
  lines.push(...listLines("Forces", map.forces));
  lines.push(objectLine(map));
  lines.push(`- Units and items placed on the map: ${map.placed}`);
  lines.push(`- Imported files: ${map.importedFiles}`);
  return lines.join("\n");
}

export function assistantPrompt(goal: string, context: AssistantContext | null): string {
  const summary = mapSummary(context ?? { balance: null, map: null });
  const wanted = goal.trim() || NO_GOAL;

  return template.replace(GOAL, wanted).replace(MAP, summary);
}
