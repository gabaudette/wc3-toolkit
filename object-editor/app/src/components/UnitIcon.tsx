import { useState } from "react";
import { iconUrl } from "../api";
import { classes } from "../ui";

type UnitIconProps = {
  path: string | null;
  size: "small" | "large";
};

/** A unit's game icon, or a plain square when it has none or it fails to load. */
export function UnitIcon({ path, size }: UnitIconProps) {
  const [failed, setFailed] = useState(false);
  const box = classes(
    "shrink-0 rounded border border-line bg-surface-2",
    size === "small" && "size-8",
    size === "large" && "size-16",
  );

  if (!path || failed) {
    return <div className={box} aria-hidden="true" />;
  }

  return (
    <img
      src={iconUrl(path)}
      alt=""
      loading="lazy"
      draggable={false}
      onError={() => setFailed(true)}
      className={box}
    />
  );
}
