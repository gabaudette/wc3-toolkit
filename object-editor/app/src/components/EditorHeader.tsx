import { SearchLg } from "@untitledui/icons";
import type { MapSummary } from "../types";
import { AppVersion } from "./AppVersion";
import { Button } from "./Button";

type EditorHeaderProps = {
  map: MapSummary | null;
  saving: boolean;
  canShare: boolean;
  onSearch: () => void;
  onImport: () => void;
  onShare: () => void;
  onOpenMap: () => void;
  onSave: () => void;
};

export function EditorHeader({
  map,
  saving,
  canShare,
  onSearch,
  onImport,
  onShare,
  onOpenMap,
  onSave,
}: EditorHeaderProps) {
  return (
    <header className="frame flex flex-wrap items-center gap-3 bg-surface-1 px-4 py-2">
      <h1 className="font-display text-2xl font-bold tracking-wide text-gold">Tinkerbench</h1>
      <AppVersion />
      <span className="flex-1" />
      <button
        type="button"
        onClick={onSearch}
        className="flex w-[22rem] max-w-full items-center gap-2.5 rounded-[3px] border border-field bg-well px-3 py-1.5 text-left text-muted hover:border-frame"
      >
        <SearchLg className="size-4" aria-hidden="true" />
        <span className="flex-1 truncate">Search objects and commands…</span>
        <kbd className="rounded-[3px] border border-line-strong px-1.5 font-sans text-xs text-ink">
          Ctrl K
        </kbd>
      </button>
      <span className="flex-1" />
      {map && (
        <span
          aria-live="polite"
          className="min-w-0 truncate font-mono text-xs text-muted"
          title={map.path}
        >
          {map.name}
          {map.unsaved && <span className="ml-2 font-sans text-gold">● unsaved</span>}
        </span>
      )}
      {!map && <span className="text-muted">Game data (read-only)</span>}
      {map && (
        <>
          <Button onPress={onImport}>Import…</Button>
          <Button disabled={!canShare} onPress={onShare}>
            Share…
          </Button>
        </>
      )}
      <Button onPress={onOpenMap}>Open map…</Button>
      {map && (
        <Button primary disabled={!map.unsaved || saving} onPress={onSave}>
          Save
        </Button>
      )}
    </header>
  );
}

type ErrorBannerProps = { error: string | null; onDismiss: () => void };

export function ErrorBanner({ error, onDismiss }: ErrorBannerProps) {
  if (!error) {
    return null;
  }

  return (
    <div
      role="alert"
      className="flex items-center gap-3 rounded-[3px] border border-danger/40 bg-danger/10 px-4 py-2 text-danger"
    >
      <span className="flex-1">{error}</span>
      <Button small onPress={onDismiss}>
        Dismiss
      </Button>
    </div>
  );
}
