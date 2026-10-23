export const GUIDE_SEARCH_WEIGHT = 4;
const REFERENCE_SEARCH_WEIGHT = 1;

export const SEARCH_CONTENT_KINDS = ["Guides", "API", "Object fields"] as const;
type ContentKind = (typeof SEARCH_CONTENT_KINDS)[number];
export type SearchMetadata = {
  filters: ContentKind[];
  label: string;
  weight: number;
};

/** Collection IDs are independent of the deployment's URL base. */
export function searchMetadata(id: string, practicalGuide = false): SearchMetadata {
  const path = id
    .replace(/\.(md|mdx)$/, "")
    .replace(/\/index$/, "")
    .replace(/\/$/, "");
  if (path === "reference/api" || path.startsWith("reference/api/")) {
    if (practicalGuide) {
      return { filters: ["API", "Guides"], label: "API guide", weight: GUIDE_SEARCH_WEIGHT };
    }
    return { filters: ["API"], label: "API reference", weight: REFERENCE_SEARCH_WEIGHT };
  }
  if (path === "reference/fields" || path.startsWith("reference/fields/")) {
    return {
      filters: ["Object fields"],
      label: "Object fields",
      weight: REFERENCE_SEARCH_WEIGHT,
    };
  }
  return { filters: ["Guides"], label: "Guide", weight: GUIDE_SEARCH_WEIGHT };
}
