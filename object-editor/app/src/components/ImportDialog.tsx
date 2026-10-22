import { open as openFile } from "@tauri-apps/plugin-dialog";
import { useEffect, useState } from "react";
import { api, errorMessage } from "../api";
import type { ImportPreview, ImportSource, TransferObject, UnitItem } from "../types";
import { classes, MAP_FILTERS, objectCount } from "../ui";
import { Button } from "./Button";
import { DialogShell } from "./DialogShell";
import { TransferList } from "./TransferList";
import { UnitChecklist } from "./UnitChecklist";

type ImportDialogProps = {
  open: boolean;
  mapsDir: string | null;
  onImported: (objects: TransferObject[]) => void;
  onClose: () => void;
};

type Tab = "code" | "map";

const TABS: { id: Tab; label: string }[] = [
  { id: "code", label: "Share code" },
  { id: "map", label: "Another map" },
];

const CODE_ROWS = 4;

/** Adds custom objects from a share code or another map, after showing what will be added. */
export function ImportDialog({ open, mapsDir, onImported, onClose }: ImportDialogProps) {
  const [tab, setTab] = useState<Tab>("code");
  const [code, setCode] = useState("");
  const [otherMap, setOtherMap] = useState<string | null>(null);
  const [otherUnits, setOtherUnits] = useState<UnitItem[]>([]);
  const [chosen, setChosen] = useState<string[]>([]);
  const [preview, setPreview] = useState<ImportPreview | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    if (open) {
      setPreview(null);
      setError(null);
    }
  }, [open]);

  function source(): ImportSource {
    if (tab === "code") {
      return { kind: "code", code };
    }

    return { kind: "map", path: otherMap ?? "", ids: chosen };
  }

  const ready =
    (tab === "code" && code.trim() !== "") ||
    (tab === "map" && otherMap !== null && chosen.length > 0);

  async function run<T>(work: () => Promise<T>): Promise<T | null> {
    setBusy(true);
    try {
      const result = await work();
      setError(null);
      return result;
    } catch (e) {
      setError(errorMessage(e));
      return null;
    } finally {
      setBusy(false);
    }
  }

  async function pickMap() {
    const picked = await openFile({
      title: "Import from a map",
      filters: MAP_FILTERS,
      ...(mapsDir && { defaultPath: mapsDir }),
    });
    if (typeof picked !== "string") {
      return;
    }

    const units = await run(() => api.mapCustomUnits(picked));
    if (units) {
      setOtherMap(picked);
      setOtherUnits(units);
      setChosen([]);
      setPreview(null);
    }
  }

  async function showPreview() {
    setPreview(await run(() => api.previewImport(source())));
  }

  async function importNow() {
    const added = await run(() => api.importObjects(source()));
    if (added) {
      // The next import starts fresh; the chosen map stays for picking more from it.
      setCode("");
      setChosen([]);
      setPreview(null);
      onImported(added);
    }
  }

  const otherName = otherMap?.split(/[\\/]/).pop();

  return (
    <DialogShell
      open={open}
      title="Import objects"
      onClose={onClose}
      footer={
        <>
          <Button onPress={onClose}>Cancel</Button>
          {!preview && (
            <Button primary disabled={!ready || busy} onPress={showPreview}>
              Preview
            </Button>
          )}
          {preview && (
            <Button primary disabled={busy} onPress={importNow}>
              Import {objectCount(preview.objects.length)}
            </Button>
          )}
        </>
      }
    >
      <div className="flex gap-1" role="tablist" aria-label="Import from">
        {TABS.map((t) => (
          <button
            key={t.id}
            type="button"
            role="tab"
            aria-selected={tab === t.id}
            onClick={() => {
              setTab(t.id);
              setPreview(null);
              setError(null);
            }}
            className={classes(
              "rounded-full border px-3 py-0.5 text-sm",
              tab === t.id && "border-gold/60 bg-gold-soft text-gold",
              tab !== t.id && "border-line text-faint hover:text-muted",
            )}
          >
            {t.label}
          </button>
        ))}
      </div>

      {tab === "code" && (
        <textarea
          value={code}
          onChange={(e) => {
            setCode(e.target.value);
            setPreview(null);
          }}
          rows={CODE_ROWS}
          placeholder="Paste a W3OBJ1:… code"
          aria-label="Share code"
          className="break-all rounded-md border border-line bg-surface-0 px-3 py-2 font-mono text-xs outline-none focus:border-gold"
        />
      )}

      {tab === "map" && (
        <>
          <div className="flex items-center gap-2">
            <Button onPress={pickMap} disabled={busy}>
              Choose map…
            </Button>
            <span className="min-w-0 truncate font-mono text-xs text-muted" title={otherMap ?? ""}>
              {otherName ?? "No map chosen"}
            </span>
          </div>
          {otherMap && (
            <UnitChecklist
              units={otherUnits}
              chosen={chosen}
              onChange={(ids) => {
                setChosen(ids);
                setPreview(null);
              }}
            />
          )}
        </>
      )}

      {error && <p className="text-sm text-danger">{error}</p>}
      {preview && (
        <>
          <TransferList objects={preview.objects} warnings={preview.warnings} />
          <p className="text-xs text-faint">
            IDs already used in this map get the next free one. Nothing is written until you save
            the map.
          </p>
        </>
      )}
    </DialogShell>
  );
}
