import { save } from "@tauri-apps/plugin-dialog";
import { useEffect, useRef, useState } from "react";
import { api, errorMessage } from "../api";
import type { SharePreview, UnitItem } from "../types";
import { BUNDLE_FILTERS } from "../ui";
import { Button } from "./Button";
import { DialogShell } from "./DialogShell";
import { TransferList } from "./TransferList";
import { UnitChecklist } from "./UnitChecklist";

type ShareDialogProps = {
  open: boolean;
  /** The open map's custom units. */
  units: UnitItem[];
  /** Ticked when the dialog opens: the unit on screen, if it is custom. */
  initial: string[];
  onClose: () => void;
};

/** How long the "Copied" confirmation stays, in milliseconds. */
const COPIED_MS = 2000;

const BUNDLE_EXTENSION = ".w3obj";

/**
 * Turns chosen custom units, and every custom object they use, into a text
 * code that any map can import, or into a bundle file that also carries the
 * models, textures and icons imported into this map that they use.
 */
export function ShareDialog({ open, units, initial, onClose }: ShareDialogProps) {
  const [chosen, setChosen] = useState(initial);
  const [preview, setPreview] = useState<SharePreview | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);
  const [saved, setSaved] = useState<string | null>(null);
  const codeRef = useRef<HTMLTextAreaElement>(null);

  useEffect(() => {
    if (open) {
      setChosen(initial);
      setPreview(null);
      setError(null);
      setSaved(null);
    }
  }, [open, initial]);

  function choose(ids: string[]) {
    setChosen(ids);
    setPreview(null);
    setSaved(null);
  }

  async function create() {
    try {
      setPreview(await api.shareCode(chosen));
      setError(null);
    } catch (e) {
      setError(errorMessage(e));
    }
  }

  async function saveBundle() {
    const first = units.find((u) => u.id === chosen[0]);
    const path = await save({
      title: "Save bundle file",
      filters: BUNDLE_FILTERS,
      defaultPath: `${first?.name ?? "units"}${BUNDLE_EXTENSION}`,
    });
    if (!path) {
      return;
    }

    try {
      await api.saveBundle(chosen, path);
      setSaved(path);
      setError(null);
    } catch (e) {
      setError(errorMessage(e));
    }
  }

  async function copy() {
    if (!preview) {
      return;
    }

    try {
      await navigator.clipboard.writeText(preview.code);
      setCopied(true);
      window.setTimeout(() => setCopied(false), COPIED_MS);
    } catch {
      // No clipboard access: select the code so Ctrl+C works.
      codeRef.current?.select();
    }
  }

  return (
    <DialogShell
      open={open}
      title="Share units"
      onClose={onClose}
      footer={
        <>
          <Button onPress={onClose}>Close</Button>
          {!preview && (
            <Button primary disabled={chosen.length === 0} onPress={create}>
              Create share code
            </Button>
          )}
          {preview && <Button onPress={saveBundle}>Save bundle file…</Button>}
          {preview && (
            <Button primary onPress={copy}>
              {copied && "Copied"}
              {!copied && "Copy code"}
            </Button>
          )}
        </>
      }
    >
      <p className="text-sm text-muted">
        Pick custom units. Their custom abilities, buffs and the custom units they train or build
        come along. Paste the code into Import in any map, or save a bundle file, which also carries
        the models, textures and icons imported into this map that they use.
      </p>
      <UnitChecklist units={units} chosen={chosen} onChange={choose} />
      {error && <p className="text-sm text-danger">{error}</p>}
      {preview && (
        <>
          <textarea
            ref={codeRef}
            value={preview.code}
            readOnly
            rows={3}
            aria-label="Share code"
            onFocus={(e) => e.currentTarget.select()}
            className="break-all rounded-md border border-line bg-surface-0 px-3 py-2 font-mono text-xs outline-none focus:border-gold"
          />
          <TransferList
            objects={preview.objects}
            files={preview.files}
            warnings={preview.warnings}
          />
          {saved && <p className="text-sm text-muted">Saved {saved.split(/[\\/]/).pop()}.</p>}
        </>
      )}
    </DialogShell>
  );
}
