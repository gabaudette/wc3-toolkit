import { describe, expect, test } from "bun:test";
import {
  assetKind,
  folderOf,
  mdxFromMdl,
  modelFieldValue,
  samePath,
  withExtension,
} from "./assets";

describe("asset paths", () => {
  test("model fields name the text model like the World Editor", () => {
    expect(modelFieldValue("war3mapImported\\Knight.mdx")).toBe("war3mapImported\\Knight.mdl");
    expect(modelFieldValue("war3mapImported\\Knight")).toBe("war3mapImported\\Knight.mdl");
  });

  test("extensions change only in the file name", () => {
    expect(withExtension("war3mapImported\\v1.2\\Knight.blp", "png")).toBe(
      "war3mapImported\\v1.2\\Knight.png",
    );
    expect(folderOf("war3mapImported\\Textures\\Knight.blp")).toBe("war3mapImported\\Textures");
  });

  test("kinds come from the extension", () => {
    expect(assetKind("A.MDX")).toBe("model");
    expect(assetKind("a.tga")).toBe("image");
    expect(assetKind("a.wav")).toBe("other");
  });

  test("paths compare without case or separator style", () => {
    expect(samePath("war3mapImported/Knight.blp", "WAR3MAPIMPORTED\\knight.blp")).toBe(true);
  });
});

describe("text models", () => {
  test("convert to binary models", () => {
    const converted = mdxFromMdl(
      [
        "Version {",
        "\tFormatVersion 800,",
        "}",
        'Model "Box" {',
        "\tBlendTime 150,",
        "\tMinimumExtent { -1, -1, -1 },",
        "\tMaximumExtent { 1, 1, 1 },",
        "}",
      ].join("\n"),
    );

    expect(converted.ok).toBe(true);

    if (converted.ok) {
      expect(atob(converted.value).startsWith("MDLX")).toBe(true);
    }
  });

  test("report what they cannot read", () => {
    expect(mdxFromMdl("Model {").ok).toBe(false);
  });
});
