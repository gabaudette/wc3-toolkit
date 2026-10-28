import { useCallback, useEffect, useMemo, useState } from "react";
import { api, iconUrl } from "../api";
import {
  assetKind,
  fileName,
  folderOf,
  samePath,
  TEXTURE_STATUS,
  USE_KINDS,
  withExtension,
} from "../assets";
import { chooseFiles, choosePath, chooseSavePath, confirm } from "../dialogs";
import { settle } from "../result";
import type { AssetItem, AssetList, AssetUse, TextureStatus, UnitItem } from "../types";
import { ASSET_FILTERS, besides, classes, counted, fileSize, PNG_FILTERS } from "../ui";
import { CreateUnitDialog, ExportGameFileDialog } from "./AssetDialogs";
import { Button } from "./Button";
import { GameText } from "./GameText";
import { ImportAssetsDialog, type ImportPlan } from "./ImportAssetsDialog";
import { ModelViewer } from "./ModelViewer";

const KIND_LABELS: Record<AssetItem["kind"], string> = {
  model: "Model",
  image: "Texture",
  other: "File",
};

type AssetsPaneProps = {
  editable: boolean;
  mapPath: string | null;
  unsaved: boolean;
  units: UnitItem[];
  onMapChanged: () => void;
  onUnitCreated: (id: string) => void;
  onOpenUse: (use: AssetUse) => void;
  onError: (message: string | null) => void;
};

export function AssetsPane({
  editable,
  mapPath,
  unsaved,
  units,
  onMapChanged,
  onUnitCreated,
  onOpenUse,
  onError,
}: AssetsPaneProps) {
  const [list, setList] = useState<AssetList | null>(null);
  const [selected, setSelected] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [query, setQuery] = useState("");
  const [plan, setPlan] = useState<ImportPlan | null>(null);
  const [exportingGame, setExportingGame] = useState(false);
  const [creating, setCreating] = useState<string | null>(null);

  const reload = useCallback(
    (select?: string) => {
      void settle(
        api.listAssets(),
        (loaded) => {
          setList(loaded);
          setSelected((current) => {
            const wanted = select ?? current;

            return loaded.assets.find((a) => wanted && samePath(a.path, wanted))?.path ?? null;
          });
        },
        onError,
      );
    },
    [onError],
  );

  useEffect(() => {
    if (editable) {
      reload();
    }
  }, [editable, reload]);

  const assets = list?.assets ?? [];
  const shown = useMemo(() => {
    const needle = query.trim().toLowerCase();

    return assets.filter((a) => !needle || a.path.toLowerCase().includes(needle));
  }, [assets, query]);
  const asset = assets.find((a) => a.path === selected);
  const images = useMemo(
    () => assets.filter((a) => a.kind === "image").map((a) => a.path),
    [assets],
  );

  function changed(select?: string) {
    reload(select);
    onMapChanged();
  }

  async function pickFiles() {
    const picked = await chooseFiles({
      title: "Import files",
      filters: ASSET_FILTERS,
      ...(mapPath && { defaultPath: besides(mapPath, "") }),
    });

    if (!picked.ok) {
      onError(picked.error);
      return;
    }

    if (picked.value.length > 0) {
      setPlan({ files: picked.value, folder: null });
    }
  }

  async function pickFolder() {
    const picked = await choosePath({
      title: "Import a folder",
      directory: true,
      ...(mapPath && { defaultPath: besides(mapPath, "") }),
    });

    if (!picked.ok) {
      onError(picked.error);
      return;
    }

    if (picked.value) {
      setPlan({ files: [], folder: picked.value });
    }
  }

  function imported(paths: string[]) {
    setPlan(null);
    setNotice(null);
    changed(paths.find((path) => assetKind(path) === "model") ?? paths[0]);
  }

  if (!editable) {
    return (
      <p className="m-auto text-muted">
        Open a map to import models and textures into it, or to export its files.
      </p>
    );
  }

  return (
    <div className="flex min-h-0 flex-1 gap-2.5">
      <aside
        aria-label="Imported files"
        className="panel flex min-h-0 w-80 shrink-0 flex-col rounded-[3px] bg-surface-1"
      >
        <div className="flex flex-col gap-2 border-b border-line p-3">
          <div className="grid grid-cols-2 gap-2">
            <Button primary onPress={pickFiles}>
              Import files…
            </Button>
            <Button primary onPress={pickFolder}>
              Import folder…
            </Button>
          </div>
          <Button small onPress={() => setExportingGame(true)}>
            Export a game file…
          </Button>
          <input
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="Filter by path"
            aria-label="Filter imported files"
            className="rounded-[3px] border border-field bg-well px-2.5 py-1.5 text-sm text-ink outline-none focus:border-focus"
          />
          <p className="text-xs text-faint">{counted(assets.length, "imported file")}</p>
        </div>
        {list && !list.listed && (
          <p className="p-3 text-sm text-faint">
            This map has no import list (protected maps often drop it), so the files imported before
            can't be shown. Files you import now are listed.
          </p>
        )}
        {list?.listed && assets.length === 0 && (
          <p className="p-3 text-sm text-faint">
            Nothing imported yet. Import a model with its textures, then create a unit from it.
          </p>
        )}
        <ul aria-label="Files" className="min-h-0 flex-1 overflow-y-auto">
          {shown.map((a) => (
            <li key={a.path}>
              <button
                type="button"
                onClick={() => {
                  setSelected(a.path);
                  setNotice(null);
                }}
                aria-current={selected === a.path}
                title={a.path}
                className={classes(
                  "flex w-full flex-col px-3 py-1.5 text-left hover:bg-surface-2",
                  selected === a.path && "bg-surface-2 shadow-[inset_2px_0_0_var(--color-glow)]",
                )}
              >
                <span className="flex items-center gap-2">
                  <span className="min-w-0 flex-1 truncate">{fileName(a.path)}</span>
                  {a.queued && unsaved && (
                    <span className="shrink-0 text-xs text-gold-bright">Not saved</span>
                  )}
                </span>
                <span className="truncate text-xs text-faint">
                  {KIND_LABELS[a.kind]} · {fileSize(a.size)} · {usedBy(a.uses.length)}
                </span>
              </button>
            </li>
          ))}
        </ul>
      </aside>

      <section className="frame flex min-h-0 min-w-0 flex-1 flex-col bg-surface-1">
        {asset && (
          <AssetDetail
            unsaved={unsaved}
            key={asset.path}
            asset={asset}
            assets={assets}
            notice={notice}
            onNotice={setNotice}
            onChanged={changed}
            onCreateUnit={() => setCreating(asset.path)}
            onOpenUse={onOpenUse}
            onError={onError}
          />
        )}
        {!asset && (
          <p className="m-auto max-w-md text-center text-muted">
            Choose a file to preview it, see which objects use it, or rename, replace, export or
            remove it.
          </p>
        )}
      </section>

      <ImportAssetsDialog
        plan={plan}
        existing={assets.map((a) => a.path)}
        onImported={imported}
        onClose={() => setPlan(null)}
      />
      <ExportGameFileDialog open={exportingGame} onClose={() => setExportingGame(false)} />
      <CreateUnitDialog
        model={creating}
        images={images}
        units={units}
        onCreated={(id) => {
          setCreating(null);
          reload();
          onUnitCreated(id);
        }}
        onClose={() => setCreating(null)}
      />
    </div>
  );
}

function usedBy(count: number): string {
  if (count === 0) {
    return "unused";
  }

  return `used by ${counted(count, "object")}`;
}

type AssetDetailProps = {
  asset: AssetItem;
  unsaved: boolean;
  assets: AssetItem[];
  notice: string | null;
  onNotice: (notice: string | null) => void;
  onChanged: (select?: string) => void;
  onCreateUnit: () => void;
  onOpenUse: (use: AssetUse) => void;
  onError: (message: string | null) => void;
};

function AssetDetail({
  asset,
  unsaved,
  assets,
  notice,
  onNotice,
  onChanged,
  onCreateUnit,
  onOpenUse,
  onError,
}: AssetDetailProps) {
  const [textures, setTextures] = useState<TextureStatus[]>([]);
  const [newPath, setNewPath] = useState(asset.path);
  const [revision, setRevision] = useState(0);

  useEffect(() => {
    if (asset.kind !== "model") {
      return;
    }

    void settle(api.assetTextures(asset.path), setTextures, onError);
  }, [asset.kind, asset.path, onError]);

  const trimmed = newPath.trim();
  const taken = assets.some((a) => a.path !== asset.path && samePath(a.path, trimmed));

  async function rename() {
    const renamed = await api.renameAsset(asset.path, trimmed);

    if (!renamed.ok) {
      onError(renamed.error);
      return;
    }

    onError(null);
    onNotice(`Renamed and updated ${counted(renamed.value, "field")}.`);
    onChanged(trimmed);
  }

  async function replace() {
    const picked = await choosePath({
      title: `Replace ${fileName(asset.path)}`,
      filters: ASSET_FILTERS,
    });

    if (!picked.ok) {
      onError(picked.error);
      return;
    }

    if (!picked.value) {
      return;
    }

    const replaced = await api.replaceAsset(asset.path, picked.value);

    if (!replaced.ok) {
      onError(replaced.error);
      return;
    }

    onNotice("Replaced. The map gets the new file when you save it.");
    setRevision((r) => r + 1);
    onChanged(asset.path);
  }

  async function remove() {
    const message = [
      `Remove ${asset.path} from the map?`,
      asset.uses.length > 0 && `${usedBy(asset.uses.length)}; they will show without it.`,
    ]
      .filter(Boolean)
      .join(" ");
    const sure = await confirm(message, {
      title: "Remove file",
      kind: "warning",
      okLabel: "Remove file",
      cancelLabel: "Keep it",
    });

    if (!sure.ok) {
      onError(sure.error);
      return;
    }

    if (!sure.value) {
      return;
    }

    const removed = await api.removeAsset(asset.path);

    if (!removed.ok) {
      onError(removed.error);
      return;
    }

    onChanged();
  }

  async function exportFile(png: boolean) {
    let name = fileName(asset.path);

    if (png) {
      name = withExtension(name, "png");
    }

    const dest = await chooseSavePath({
      title: `Export ${fileName(asset.path)}`,
      defaultPath: name,
      ...(png && { filters: PNG_FILTERS }),
    });

    if (!dest.ok) {
      onError(dest.error);
      return;
    }

    if (!dest.value) {
      return;
    }

    const written = await api.exportAsset(asset.path, dest.value, png);

    if (!written.ok) {
      onError(written.error);
      return;
    }

    onNotice(`Saved ${dest.value}.`);
  }

  return (
    <>
      <header className="border-b border-line px-5 py-3">
        <h2 className="truncate font-display text-3xl font-bold text-gold-bright">
          {fileName(asset.path)}
        </h2>
        <p className="truncate font-mono text-xs text-faint">
          {folderOf(asset.path)} · {fileSize(asset.size)}
          {asset.queued && unsaved && " · added when you save the map"}
        </p>
      </header>
      <div className="flex min-h-0 max-w-4xl flex-1 flex-col gap-4 overflow-y-auto p-5">
        {asset.kind === "model" && <ModelViewer key={revision} path={asset.path} />}
        {asset.kind === "image" && (
          <img
            key={revision}
            src={`${iconUrl(asset.path)}?v=${revision}`}
            alt={`Preview of ${fileName(asset.path)}`}
            className="w-64 max-w-full self-start rounded-[3px] border border-line bg-well [image-rendering:pixelated]"
          />
        )}

        <div className="flex flex-wrap gap-2">
          {asset.kind === "model" && (
            <Button primary onPress={onCreateUnit}>
              Create a unit from this model…
            </Button>
          )}
          <Button onPress={replace}>Replace…</Button>
          <Button onPress={() => exportFile(false)}>Export…</Button>
          {asset.kind === "image" && (
            <Button onPress={() => exportFile(true)}>Export as PNG…</Button>
          )}
          <Button onPress={remove}>Remove</Button>
        </div>
        {notice && (
          <p role="status" className="text-sm">
            {notice}
          </p>
        )}

        <form
          className="flex items-end gap-2"
          onSubmit={(e) => {
            e.preventDefault();

            if (trimmed && !taken && trimmed !== asset.path) {
              void rename();
            }
          }}
        >
          <label className="flex min-w-0 flex-1 flex-col gap-1 text-sm text-muted">
            Path in the map
            <input
              value={newPath}
              onChange={(e) => setNewPath(e.target.value)}
              className="rounded-[3px] border border-field bg-well px-2.5 py-1.5 font-mono text-xs text-ink outline-none focus:border-focus"
            />
          </label>
          <Button disabled={!trimmed || taken || trimmed === asset.path} onPress={rename}>
            Rename
          </Button>
        </form>
        {taken && <p className="text-xs text-danger">Another file already has this path.</p>}
        <p className="-mt-2 text-xs text-faint">
          Renaming also changes the fields that point to this file.
        </p>

        {asset.kind === "model" && (
          <section aria-labelledby="asset-textures" className="flex flex-col gap-1">
            <h3 id="asset-textures" className="font-semibold text-gold-bright">
              Textures
            </h3>
            {textures.length === 0 && (
              <p className="text-sm text-faint">This model names no texture files.</p>
            )}
            <ul className="flex flex-col gap-0.5">
              {textures.map((texture) => (
                <li key={texture.path} className="flex items-center gap-3 text-sm">
                  <span className="min-w-0 truncate font-mono text-xs">{texture.path}</span>
                  <span
                    className={classes(
                      "shrink-0 text-xs",
                      texture.status === "missing" && "font-semibold text-danger",
                      texture.status !== "missing" && "text-faint",
                    )}
                  >
                    {TEXTURE_STATUS[texture.status]}
                  </span>
                </li>
              ))}
            </ul>
          </section>
        )}

        <section aria-labelledby="asset-uses" className="flex flex-col gap-1">
          <h3 id="asset-uses" className="font-semibold text-gold-bright">
            Used by
          </h3>
          {asset.uses.length === 0 && (
            <p className="text-sm text-faint">No unit, ability, item or other object uses it.</p>
          )}
          <ul className="flex flex-col">
            {asset.uses.map((use) => (
              <li key={`${use.kind}-${use.id}-${use.field}`}>
                <button
                  type="button"
                  onClick={() => onOpenUse(use)}
                  className="flex w-full items-center gap-2 rounded-[3px] px-2 py-1 text-left text-sm hover:bg-surface-2"
                >
                  <span className="w-24 shrink-0 text-xs text-faint">{USE_KINDS[use.kind]}</span>
                  <span className="min-w-0 flex-1 truncate">
                    <GameText text={use.name} singleLine />
                  </span>
                  <span className="font-mono text-xs text-faint">
                    {use.id} · {use.field}
                  </span>
                </button>
              </li>
            ))}
          </ul>
        </section>
      </div>
    </>
  );
}
