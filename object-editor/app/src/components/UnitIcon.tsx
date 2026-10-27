import { useState } from "react";
import { iconUrl } from "../api";
import { classes } from "../ui";

type UnitIconProps = {
  path: string | null;
  size: "small" | "large";
  onMissing?: () => void;
};

export function UnitIcon({ path, size, onMissing }: UnitIconProps) {
  const [failed, setFailed] = useState(false);
  const [pixelated, setPixelated] = useState(false);
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
      onError={() => {
        setFailed(true);
        onMissing?.();
      }}
      onLoad={(e) => setPixelated(e.currentTarget.naturalWidth < e.currentTarget.clientWidth)}
      className={classes(box, pixelated && "[image-rendering:pixelated]")}
    />
  );
}
