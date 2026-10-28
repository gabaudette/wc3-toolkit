import { generateMDX, parseMDL } from "war3-model";
import { attemptSync, type Result } from "./result";
import type { AssetKind, AssetUseKind, TextureStatusKind } from "./types";

const MODEL_EXTENSIONS = ["mdx", "mdl"];
const IMAGE_EXTENSIONS = ["blp", "dds", "tga"];
const BASE64_CHUNK = 0x80_00;

export const TEXTURE_STATUS: Record<TextureStatusKind, string> = {
  batch: "In this import",
  map: "In the map",
  game: "In the game",
  missing: "Missing",
};

export const USE_KINDS: Record<AssetUseKind, string> = {
  unit: "Unit",
  ability: "Ability",
  item: "Item",
  buff: "Buff",
  destructable: "Destructable",
  upgrade: "Upgrade",
  doodad: "Doodad",
};

export function fileName(path: string): string {
  return path.split(/[\\/]/).pop() ?? path;
}

export function folderOf(path: string): string {
  const name = fileName(path);

  return path.slice(0, path.length - name.length).replace(/[\\/]$/, "");
}

export function extensionOf(path: string): string {
  const name = fileName(path);
  const dot = name.lastIndexOf(".");

  if (dot < 0) {
    return "";
  }

  return name.slice(dot + 1).toLowerCase();
}

export function withExtension(path: string, extension: string): string {
  const name = fileName(path);
  const dot = name.lastIndexOf(".");

  if (dot < 0) {
    return `${path}.${extension}`;
  }

  return `${path.slice(0, path.length - name.length + dot)}.${extension}`;
}

export function assetKind(path: string): AssetKind {
  const extension = extensionOf(path);

  if (MODEL_EXTENSIONS.includes(extension)) {
    return "model";
  }

  if (IMAGE_EXTENSIONS.includes(extension)) {
    return "image";
  }

  return "other";
}

// The World Editor writes model fields with the text model's extension and the
// game loads the `.mdx` beside that name.
export function modelFieldValue(path: string): string {
  return withExtension(path, "mdl");
}

function base64(bytes: Uint8Array): string {
  const parts: string[] = [];

  for (let at = 0; at < bytes.length; at += BASE64_CHUNK) {
    parts.push(String.fromCharCode(...bytes.subarray(at, at + BASE64_CHUNK)));
  }

  return btoa(parts.join(""));
}

export function mdxFromMdl(text: string): Result<string> {
  return attemptSync(() => base64(new Uint8Array(generateMDX(parseMDL(text)))));
}

export function samePath(a: string, b: string): boolean {
  return a.replaceAll("/", "\\").toLowerCase() === b.replaceAll("/", "\\").toLowerCase();
}
