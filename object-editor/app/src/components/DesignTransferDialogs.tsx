import { save } from "@tauri-apps/plugin-dialog";
import { useEffect, useState } from "react";
import { api, errorMessage } from "../api";
import type { Design, DesignTransfer } from "../types";
import { besides, counted, DESIGN_FILTERS } from "../ui";
import { Button } from "./Button";
import { Checkbox } from "./Checkbox";
import { DialogShell } from "./DialogShell";
import { TransferFiles } from "./TransferList";

const DESIGN_EXTENSION = ".w3ui";

/** The designs a file holds or would hold, the files with them and what they need. */
function DesignTransferList({ transfer }: { transfer: DesignTransfer }) {
  return (
    <div className="flex flex-col gap-2">
      <ul
        className="max-h-64 overflow-y-auto rounded-[3px] border border-line"
        aria-label="Designs"
      >
        {transfer.designs.map((design) => (
          <li key={design.luaName} className="flex items-baseline gap-3 px-3 py-1 text-sm">
            <span className="min-w-0 flex-1 truncate">
              {design.name}
              {design.shownBy && (
                <span className="text-faint"> · brought along, {design.shownBy} shows it</span>
              )}
              {design.renamedFrom && (
                <span className="text-gold"> · was {design.renamedFrom}, a name this map has</span>
              )}
            </span>
            <span className="text-xs text-faint">{design.kind}</span>
            <code className="font-mono text-xs text-muted">UI.{design.luaName}</code>
          </li>
        ))}
      </ul>
      <TransferFiles files={transfer.files} />
      {transfer.needs.length > 0 && (
        <p className="text-xs text-muted">
          Buttons or timers call{" "}
          {transfer.needs.map((name, i) => (
            <span key={name}>
              {i > 0 && ", "}
              <code className="font-mono">{name}</code>
            </span>
          ))}
          . Those calls do nothing until the map's own Lua defines them.
        </p>
      )}
    </div>
  );
}

type ExportDesignsDialogProps = {
  open: boolean;
  designs: Design[];
  /** The open map's file: the design file is saved beside it. */
  mapPath: string | null;
  /** Ticked when the window opens. */
  initial: string[];
  /** Waits for changes still on their way to the map. */
  flush: () => Promise<void>;
  onClose: () => void;
};

/**
 * Saves chosen designs as a design file for another map's Interface tab,
 * with the dialogs their buttons show and the icons imported into this map
 * that they use.
 */
export function ExportDesignsDialog({
  open,
  designs,
  mapPath,
  initial,
  flush,
  onClose,
}: ExportDesignsDialogProps) {
  const [chosen, setChosen] = useState(initial);
  const [preview, setPreview] = useState<DesignTransfer | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [saved, setSaved] = useState<string | null>(null);

  useEffect(() => {
    if (open) {
      setChosen(initial);
      setSaved(null);
      setError(null);
    }
  }, [open, initial]);

  useEffect(() => {
    if (!open) {
      return;
    }

    let current = true;
    flush()
      .then(() => api.uiExportPreview(chosen))
      .then((next) => {
        if (current) {
          setPreview(next);
        }
      })
      .catch((e: unknown) => setError(errorMessage(e)));
    return () => {
      current = false;
    };
  }, [open, chosen, flush]);

  function toggle(id: string, on: boolean) {
    setSaved(null);
    if (on) {
      setChosen([...chosen, id]);
      return;
    }

    setChosen(chosen.filter((c) => c !== id));
  }

  async function saveFile() {
    const first = designs.find((d) => d.id === chosen[0]);
    let name = "designs";
    if (chosen.length === 1 && first) {
      name = first.name;
    }

    let defaultPath = `${name}${DESIGN_EXTENSION}`;
    if (mapPath) {
      defaultPath = besides(mapPath, defaultPath);
    }

    const path = await save({ title: "Save design file", filters: DESIGN_FILTERS, defaultPath });
    if (!path) {
      return;
    }

    try {
      await flush();
      await api.saveUiFile(chosen, path);
      setSaved(path);
      setError(null);
    } catch (e) {
      setError(errorMessage(e));
    }
  }

  return (
    <DialogShell
      open={open}
      title="Export designs"
      onClose={onClose}
      footer={
        <>
          <Button onPress={onClose}>Close</Button>
          <Button primary disabled={chosen.length === 0} onPress={saveFile}>
            Save design file…
          </Button>
        </>
      }
    >
      <p className="text-sm text-muted">
        Pick designs to take to another map: open the file there with Import in the Interface tab.
        Dialogs their buttons show come along, and so do icons imported into this map that they use.
      </p>
      <fieldset className="flex flex-col gap-1.5">
        <legend className="sr-only">Designs to export</legend>
        {designs.map((design) => (
          <Checkbox
            key={design.id}
            isSelected={chosen.includes(design.id)}
            onChange={(on) => toggle(design.id, on)}
          >
            {design.name}
          </Checkbox>
        ))}
      </fieldset>
      {chosen.length > 0 && preview && <DesignTransferList transfer={preview} />}
      {error && <p className="text-sm text-danger">{error}</p>}
      <p role="status" className="text-sm text-muted">
        {saved && `Saved ${saved.split(/[\\/]/).pop()}.`}
      </p>
    </DialogShell>
  );
}

type ImportDesignsDialogProps = {
  /** The design file to import; the window is open while set. */
  path: string | null;
  flush: () => Promise<void>;
  onImported: (designs: Design[]) => void;
  onClose: () => void;
};

/** Shows what a design file adds to this map, renamed where names clash, then adds it. */
export function ImportDesignsDialog({
  path,
  flush,
  onImported,
  onClose,
}: ImportDesignsDialogProps) {
  const [preview, setPreview] = useState<DesignTransfer | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    setPreview(null);
    setError(null);
    if (!path) {
      return;
    }

    let current = true;
    flush()
      .then(() => api.uiImportPreview(path))
      .then((next) => {
        if (current) {
          setPreview(next);
        }
      })
      .catch((e: unknown) => setError(errorMessage(e)));
    return () => {
      current = false;
    };
  }, [path, flush]);

  async function confirm() {
    if (!path) {
      return;
    }

    try {
      await flush();
      onImported(await api.importUiFile(path));
      onClose();
    } catch (e) {
      setError(errorMessage(e));
    }
  }

  let count = "";
  if (preview) {
    count = counted(preview.designs.length, "design");
  }

  return (
    <DialogShell
      open={path !== null}
      title="Import designs"
      onClose={onClose}
      footer={
        <>
          <Button onPress={onClose}>Cancel</Button>
          <Button primary disabled={!preview} onPress={confirm}>
            Import {count}
          </Button>
        </>
      }
    >
      <p className="text-sm text-muted">
        From <span className="font-mono">{path?.split(/[\\/]/).pop()}</span>. They are added to this
        map's designs and saved with it; designs with a name this map already has get a number.
      </p>
      {preview && <DesignTransferList transfer={preview} />}
      {error && <p className="text-sm text-danger">{error}</p>}
    </DialogShell>
  );
}
