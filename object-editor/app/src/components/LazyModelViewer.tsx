import { lazy, Suspense } from "react";
import type { ModelViewerProps } from "./ModelViewer";

const Viewer = lazy(async () => ({ default: (await import("./ModelViewer")).ModelViewer }));

function Loading() {
  return (
    <div className="flex w-72 shrink-0 flex-col gap-1.5">
      <div className="relative aspect-square overflow-hidden rounded-[3px] border border-line bg-surface-0">
        <p className="absolute inset-0 flex items-center justify-center text-sm text-muted">
          Loading model…
        </p>
      </div>
    </div>
  );
}

export function LazyModelViewer({ path }: ModelViewerProps) {
  return (
    <Suspense fallback={<Loading />}>
      <Viewer path={path} />
    </Suspense>
  );
}
