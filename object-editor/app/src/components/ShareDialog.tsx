import { useEffect, useRef, useState } from "react";
import { api } from "../api";
import { chooseSavePath } from "../dialogs";
import { attempt } from "../result";
import type { SharePreview, UnitItem } from "../types";
import { BUNDLE_FILTERS } from "../ui";
import { Button } from "./Button";
import { DialogShell } from "./DialogShell";
import { plainGameText } from "./GameText";
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
    const created = await api.shareCode(chosen);

    if (!created.ok) {
      setError(created.error);
      return;
    }

    setPreview(created.value);
    setError(null);
  }

  async function saveBundle() {
    const first = units.find((u) => u.id === chosen[0]);
    const picked = await chooseSavePath({
      title: "Save bundle file",
      filters: BUNDLE_FILTERS,
      defaultPath: `${plainGameText(first?.name ?? "units")}${BUNDLE_EXTENSION}`,
    });

    if (!picked.ok) {
      setError(picked.error);
      return;
    }

    const path = picked.value;

    if (!path) {
      return;
    }

    const written = await api.saveBundle(chosen, path);

    if (!written.ok) {
      setError(written.error);
      return;
    }

    setSaved(path);
    setError(null);
  }

  async function copy() {
    if (!preview) {
      return;
    }

    const copiedCode = await attempt(() => navigator.clipboard.writeText(preview.code));

    if (!copiedCode.ok) {
      codeRef.current?.select();
      return;
    }

    setCopied(true);
    window.setTimeout(() => setCopied(false), COPIED_MS);
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
            className="break-all rounded-[3px] border border-field bg-well px-3 py-2 font-mono text-xs outline-none focus:border-focus"
          />
          <TransferList
            objects={preview.objects}
            files={preview.files}
            warnings={preview.warnings}
          />
          <p role="status" className="text-sm text-muted">
            {saved && `Saved ${saved.split(/[\\/]/).pop()}.`}
            {copied && <span className="sr-only">Code copied.</span>}
          </p>
        </>
      )}
    </DialogShell>
  );
}
