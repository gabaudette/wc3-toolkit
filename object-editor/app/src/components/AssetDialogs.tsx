import { useEffect, useMemo, useState } from "react";
import { api, iconUrl } from "../api";
import { assetKind, extensionOf, fileName, modelFieldValue, withExtension } from "../assets";
import { chooseSavePath } from "../dialogs";
import type { UnitItem } from "../types";
import { classes, PNG_FILTERS } from "../ui";
import { Button } from "./Button";
import { DialogShell } from "./DialogShell";
import { GameText, plainGameText } from "./GameText";
import { ModelViewer } from "./ModelViewer";
import { Select } from "./Select";
import { UnitIcon } from "./UnitIcon";

const MAX_BASES = 200;
const NO_ICON = "";

type CreateUnitDialogProps = {
  model: string | null;
  images: string[];
  units: UnitItem[];
  onCreated: (id: string) => void;
  onClose: () => void;
};

export function CreateUnitDialog({
  model,
  images,
  units,
  onCreated,
  onClose,
}: CreateUnitDialogProps) {
  const [query, setQuery] = useState("");
  const [base, setBase] = useState<string | null>(null);
  const [icon, setIcon] = useState(NO_ICON);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    if (model) {
      setError(null);
      setIcon(NO_ICON);
    }
  }, [model]);

  const bases = useMemo(() => {
    const needle = query.trim().toLowerCase();

    return units
      .filter((u) => !u.custom)
      .filter(
        (u) =>
          !needle ||
          plainGameText(u.name).toLowerCase().includes(needle) ||
          u.id.toLowerCase().includes(needle),
      )
      .slice(0, MAX_BASES);
  }, [units, query]);

  const selectedIcon = units.find((u) => u.id === base)?.icon ?? null;

  async function create() {
    if (!model || !base) {
      return;
    }

    setBusy(true);
    setError(null);
    const stop = (message: string) => {
      setError(message);
      setBusy(false);
    };
    const created = await api.createUnit(base);

    if (!created.ok) {
      stop(created.error);
      return;
    }

    const id = created.value.id;
    const modelSet = await api.setField(id, "umdl", modelFieldValue(model));

    if (!modelSet.ok) {
      stop(modelSet.error);
      return;
    }

    if (icon !== NO_ICON) {
      const iconSet = await api.setField(id, "uico", icon);

      if (!iconSet.ok) {
        stop(iconSet.error);
        return;
      }
    }

    setBusy(false);
    onCreated(id);
  }

  return (
    <DialogShell
      open={model !== null}
      title="Create a unit from this model"
      onClose={onClose}
      footer={
        <>
          <Button onPress={onClose}>Cancel</Button>
          <Button primary disabled={busy || !base} onPress={create}>
            Create unit
          </Button>
        </>
      }
    >
      <p className="text-sm text-muted">
        The new unit copies the base unit's stats and abilities and uses{" "}
        <span className="font-mono text-ink">{fileName(model ?? "")}</span> as its model.
      </p>
      <label className="flex flex-col gap-1 text-sm text-muted">
        Base unit
        <input
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder="Search by name or ID"
          className="rounded-[3px] border border-field bg-well px-2.5 py-1.5 text-sm text-ink outline-none focus:border-focus"
        />
      </label>
      <ul aria-label="Base units" className="h-64 overflow-y-auto rounded-[3px] border border-line">
        {bases.map((u) => (
          <li key={u.id}>
            <button
              type="button"
              onClick={() => setBase(u.id)}
              aria-pressed={base === u.id}
              className={classes(
                "flex w-full items-center gap-2 px-2 py-1 text-left hover:bg-surface-2",
                base === u.id && "bg-surface-2 shadow-[inset_2px_0_0_var(--color-glow)]",
              )}
            >
              <UnitIcon path={u.icon} size="small" />
              <span className="min-w-0 flex-1 truncate">
                <GameText text={u.name} singleLine />
              </span>
              <span className="font-mono text-xs text-faint">{u.id}</span>
            </button>
          </li>
        ))}
      </ul>
      {images.length > 0 && (
        <div className="flex flex-col gap-1 text-sm text-muted">
          <span aria-hidden="true">Icon</span>
          <div className="flex items-center gap-2">
            <UnitIcon key={icon} path={icon || selectedIcon} size="small" />
            <div className="min-w-0 flex-1">
              <Select
                label="Icon"
                value={icon}
                onChange={setIcon}
                options={[
                  { value: NO_ICON, label: "The base unit's icon" },
                  ...images.map((path) => ({ value: path, label: path })),
                ]}
              />
            </div>
          </div>
        </div>
      )}
      {error && (
        <p role="alert" className="text-sm text-danger">
          {error}
        </p>
      )}
    </DialogShell>
  );
}

type ExportGameFileDialogProps = {
  open: boolean;
  onClose: () => void;
};

function exportName(path: string, png: boolean): string {
  const name = fileName(path);

  if (png) {
    return withExtension(name, "png");
  }

  if (assetKind(path) === "image") {
    return withExtension(name, "dds");
  }

  if (extensionOf(path) === "") {
    return withExtension(name, "mdx");
  }

  return name;
}

export function ExportGameFileDialog({ open, onClose }: ExportGameFileDialogProps) {
  const [path, setPath] = useState("");
  const [shown, setShown] = useState<string | null>(null);
  const [status, setStatus] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const trimmed = path.trim();
  const image = assetKind(trimmed) === "image";

  async function save(png: boolean) {
    setStatus(null);
    setError(null);
    const dest = await chooseSavePath({
      title: "Export a game file",
      defaultPath: exportName(trimmed, png),
      ...(png && { filters: PNG_FILTERS }),
    });

    if (!dest.ok) {
      setError(dest.error);
      return;
    }

    if (!dest.value) {
      return;
    }

    const written = await api.exportGameFile(trimmed, dest.value, png);

    if (!written.ok) {
      setError(written.error);
      return;
    }

    setStatus(`Saved ${dest.value}.`);
  }

  return (
    <DialogShell
      open={open}
      title="Export a game file"
      onClose={onClose}
      footer={<Button onPress={onClose}>Close</Button>}
    >
      <p className="text-sm text-muted">
        Copies a model, texture or icon out of the game, as it is in the data (a unit's Art - Model
        File or Art - Icon). Game files are for your own use only: don't share them.
      </p>
      <form
        className="flex items-end gap-2"
        onSubmit={(e) => {
          e.preventDefault();
          setShown(trimmed || null);
        }}
      >
        <label className="flex min-w-0 flex-1 flex-col gap-1 text-sm text-muted">
          Game path
          <input
            value={path}
            onChange={(e) => {
              setPath(e.target.value);
              setStatus(null);
            }}
            placeholder="ReplaceableTextures\CommandButtons\BTNFootman.blp"
            className="rounded-[3px] border border-field bg-well px-2.5 py-1.5 font-mono text-xs text-ink outline-none focus:border-focus"
          />
        </label>
        <Button disabled={!trimmed} onPress={() => setShown(trimmed)}>
          Preview
        </Button>
      </form>
      {shown && assetKind(shown) === "image" && (
        <img
          key={shown}
          src={iconUrl(shown)}
          alt={`Preview of ${shown}`}
          className="mx-auto max-h-64 max-w-full rounded-[3px] border border-line bg-well"
        />
      )}
      {shown && assetKind(shown) !== "image" && <ModelViewer path={shown} />}
      <div className="flex flex-wrap gap-2">
        <Button primary disabled={!trimmed} onPress={() => save(false)}>
          Export…
        </Button>
        {image && (
          <Button disabled={!trimmed} onPress={() => save(true)}>
            Export as PNG…
          </Button>
        )}
      </div>
      {status && (
        <p role="status" className="text-sm">
          {status}
        </p>
      )}
      {error && (
        <p role="alert" className="text-sm text-danger">
          {error}
        </p>
      )}
    </DialogShell>
  );
}
