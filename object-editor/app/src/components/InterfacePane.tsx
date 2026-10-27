import { open } from "@tauri-apps/plugin-dialog";
import { Plus } from "@untitledui/icons";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { api, errorMessage } from "../api";
import { luaNames } from "../interface";
import type { Design, DialogDesign, IconChoice, MultiboardDesign } from "../types";
import { classes } from "../ui";
import { Button } from "./Button";
import { DialogEditor } from "./DialogEditor";
import { GameText } from "./GameText";
import { MultiboardEditor } from "./MultiboardEditor";

/** Typing pauses this long before the window re-reads whether the map is unsaved. */
const MAP_STATUS_DELAY_MS = 300;

type InterfacePaneProps = {
  editable: boolean;
  /** Called after every change, so the window can show the map as unsaved. */
  onMapChanged: () => void;
  onError: (message: string | null) => void;
};

/** "1 button", "3 buttons". */
function counted(count: number, word: string): string {
  if (count === 1) {
    return `1 ${word}`;
  }

  return `${count} ${word}s`;
}

/** The designs with one swapped for its new version. */
function replaced(designs: Design[], changed: Design): Design[] {
  return designs.map((d) => {
    if (d.id === changed.id) {
      return changed;
    }

    return d;
  });
}

/** A scoreboard to start from: a header row, then a row for each player. */
function newMultiboard(count: number): MultiboardDesign {
  const cell = (text: string, color: string | null = null) => ({ text, color, icon: null });
  return {
    kind: "multiboard",
    id: crypto.randomUUID(),
    name: `Multiboard ${count + 1}`,
    title: "Scoreboard",
    titleColor: null,
    columns: [
      { name: "Player", width: 12 },
      { name: "Kills", width: 5 },
      { name: "Deaths", width: 5 },
    ],
    rows: [
      {
        name: "Header",
        perPlayer: false,
        cells: [cell("Player", "ffcc00"), cell("Kills", "ffcc00"), cell("Deaths", "ffcc00")],
      },
      { name: "Players", perPlayer: true, cells: [cell("{player}"), cell("0"), cell("0")] },
    ],
    showAtStart: true,
    minimized: false,
  };
}

/** The list's second line for a design. */
function summary(design: Design): string {
  if (design.kind === "multiboard") {
    return `Multiboard · ${counted(design.columns.length, "column")}`;
  }

  return `Dialog · ${counted(design.buttons.length, "button")}`;
}

function newDialog(count: number): DialogDesign {
  return {
    kind: "dialog",
    id: crypto.randomUUID(),
    name: `Dialog ${count + 1}`,
    title: "Choose a difficulty",
    titleColor: null,
    buttons: [
      { text: "Normal", hotkey: "N", highlightHotkey: true, action: { kind: "close" } },
      { text: "Hard", hotkey: "H", highlightHotkey: true, action: { kind: "close" } },
    ],
    showAtStart: true,
  };
}

/**
 * The Interface tab: the map's in-game UI designs (dialogs so far) on the
 * left, the selected one's editor and preview in the middle, and the Lua
 * they become, with the test copy, along the bottom. Designs live in
 * `<map> UI.json` next to the map and are written with it on Save.
 */
export function InterfacePane({ editable, onMapChanged, onError }: InterfacePaneProps) {
  const [designs, setDesigns] = useState<Design[]>([]);
  const [selected, setSelected] = useState<string | null>(null);
  const [icons, setIcons] = useState<IconChoice[]>([]);
  const fail = useCallback((e: unknown) => onError(errorMessage(e)), [onError]);
  /** Updates run one after another, so a slow answer never overwrites a newer one. */
  const queue = useRef<Promise<void>>(Promise.resolve());
  const statusTimer = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);
  const flush = useCallback(() => queue.current, []);

  useEffect(() => {
    if (!editable) {
      return;
    }

    api
      .uiDesigns()
      .then((loaded) => {
        setDesigns(loaded);
        setSelected(loaded[0]?.id ?? null);
      })
      .catch(fail);
  }, [editable, fail]);

  const names = useMemo(() => luaNames(designs), [designs]);
  const design = designs.find((d) => d.id === selected);

  /**
   * Shows `next` at once and hands it to the map. The window's map status
   * (unsaved, Save) is re-read once typing pauses: re-reading it on every
   * keystroke re-renders the whole window and drops keys.
   */
  function update(next: Design[]) {
    setDesigns(next);
    queue.current = queue.current.then(() =>
      api.setUiDesigns(next).then(() => onError(null), fail),
    );
    clearTimeout(statusTimer.current);
    statusTimer.current = setTimeout(() => {
      queue.current.then(onMapChanged);
    }, MAP_STATUS_DELAY_MS);
  }

  useEffect(() => {
    // Cell icons pick from every icon; loaded once, when a board needs them.
    if (design?.kind !== "multiboard" || icons.length > 0) {
      return;
    }

    api.spellIcons().then(setIcons).catch(fail);
  }, [design?.kind, icons.length, fail]);

  function add(created: Design) {
    update([...designs, created]);
    setSelected(created.id);
  }

  function remove(id: string) {
    // Buttons that showed it close their dialog instead.
    const next = designs
      .filter((d) => d.id !== id)
      .map((d) => {
        if (d.kind !== "dialog") {
          return d;
        }

        return {
          ...d,
          buttons: d.buttons.map((b) => {
            if (b.action.kind === "showDialog" && b.action.dialog === id) {
              return { ...b, action: { kind: "close" as const } };
            }

            return b;
          }),
        };
      });
    update(next);
    setSelected(next[0]?.id ?? null);
  }

  if (!editable) {
    return (
      <p className="m-auto text-muted">
        Open a map to design its dialogs and multiboards. They are saved next to it, in "&lt;map&gt;
        UI.json".
      </p>
    );
  }

  return (
    <div className="flex min-h-0 flex-1 gap-2.5">
      <aside
        aria-label="Interface designs"
        className="panel flex min-h-0 w-72 shrink-0 flex-col rounded-[3px] bg-surface-1"
      >
        <div className="flex flex-col gap-2 border-b border-line p-3">
          <div className="grid grid-cols-2 gap-2">
            <Button primary onPress={() => add(newDialog(designs.length))}>
              <span className="flex items-center justify-center gap-1.5">
                <Plus className="size-4" aria-hidden="true" />
                Dialog
              </span>
            </Button>
            <Button primary onPress={() => add(newMultiboard(designs.length))}>
              <span className="flex items-center justify-center gap-1.5">
                <Plus className="size-4" aria-hidden="true" />
                Multiboard
              </span>
            </Button>
          </div>
          <p className="text-xs text-faint">
            {counted(designs.length, "design")} · saved with the map
          </p>
        </div>
        {designs.length === 0 && (
          <p className="p-3 text-sm text-faint">
            Nothing yet. A dialog asks players to choose (a difficulty, a game mode); a multiboard
            shows a table in the corner (a scoreboard).
          </p>
        )}
        <ul aria-label="Designs" className="min-h-0 flex-1 overflow-y-auto">
          {designs.map((d) => (
            <li key={d.id}>
              <button
                type="button"
                onClick={() => setSelected(d.id)}
                aria-current={selected === d.id}
                className={classes(
                  "flex w-full flex-col px-3 py-1.5 text-left hover:bg-surface-2",
                  selected === d.id && "bg-surface-2 shadow-[inset_2px_0_0_var(--color-glow)]",
                )}
              >
                <span className="truncate">{d.name || "(unnamed)"}</span>
                <span className="truncate text-xs text-faint">
                  {summary(d)} · <span className="font-mono">UI.{names.get(d.id)}</span>
                </span>
              </button>
            </li>
          ))}
        </ul>
      </aside>

      <section className="frame flex min-h-0 min-w-0 flex-1 flex-col bg-surface-1">
        {design && (
          <>
            <header className="border-b border-line px-5 py-3">
              <h2 className="truncate font-display text-3xl font-bold text-gold-bright">
                <GameText text={design.name || "(unnamed)"} singleLine />
              </h2>
            </header>
            {design.kind === "dialog" && (
              <DialogEditor
                key={design.id}
                dialog={design}
                designs={designs}
                luaName={names.get(design.id) ?? ""}
                editable={editable}
                onChange={(changed) => update(replaced(designs, changed))}
                onDelete={() => remove(design.id)}
              />
            )}
            {design.kind === "multiboard" && (
              <MultiboardEditor
                key={design.id}
                board={design}
                icons={icons}
                luaName={names.get(design.id) ?? ""}
                editable={editable}
                onChange={(changed) => update(replaced(designs, changed))}
                onDelete={() => remove(design.id)}
              />
            )}
          </>
        )}
        {!design && (
          <p className="m-auto text-muted">
            Make a dialog or a multiboard with the buttons on the left.
          </p>
        )}
        {designs.length > 0 && (
          <LuaBar
            selected={design?.id ?? null}
            designs={designs}
            flush={flush}
            onSaved={onMapChanged}
          />
        )}
      </section>
    </div>
  );
}

type LuaBarProps = {
  selected: string | null;
  /** Re-reads the selected design's Lua when any design changes. */
  designs: Design[];
  /** Waits for the designs' updates to reach the map. */
  flush: () => Promise<void>;
  onSaved: () => void;
};

/** The Lua folder, the selected design's module, and the test copy. */
function LuaBar({ selected, designs, flush, onSaved }: LuaBarProps) {
  const [folder, setFolder] = useState("");
  const [showCode, setShowCode] = useState(false);
  const [code, setCode] = useState<{ text: string; error: boolean }>({ text: "", error: false });
  const [busy, setBusy] = useState(false);
  const [status, setStatus] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    api
      .defaultLuaFolder()
      .then((dir) => setFolder((current) => current || dir || ""))
      .catch(() => {});
  }, []);

  useEffect(() => {
    // The designs are only a trigger: the module is read from the backend.
    if (!showCode || !selected || designs.length === 0) {
      return;
    }

    flush()
      .then(() => api.uiLua(selected))
      .then((text) => setCode({ text, error: false }))
      .catch((e: unknown) => setCode({ text: errorMessage(e), error: true }));
  }, [showCode, selected, designs, flush]);

  async function browse() {
    const picked = await open({ directory: true, ...(folder && { defaultPath: folder }) });
    if (typeof picked === "string") {
      setFolder(picked);
    }
  }

  async function build() {
    setBusy(true);
    setError(null);
    setStatus(null);
    try {
      const dir = folder.trim();
      await flush();
      const written = await api.writeUiLua(dir);
      await api.saveMap();
      onSaved();
      const copy = await api.buildTestCopy(dir);
      setStatus(
        `Wrote ${written.join(", ")} and built ${copy.path} with ${copy.modules.length} module(s).`,
      );
    } catch (e) {
      setError(errorMessage(e));
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="flex flex-col gap-2 border-t border-line p-4">
      <div className="flex flex-wrap items-center gap-2">
        <label className="flex min-w-0 flex-1 items-center gap-2 text-sm text-muted">
          Lua folder
          <input
            value={folder}
            onChange={(e) => setFolder(e.target.value)}
            className="min-w-0 flex-1 rounded-[3px] border border-field bg-well px-2.5 py-1.5 font-mono text-xs text-ink outline-none focus:border-focus"
          />
        </label>
        <Button onPress={browse}>Browse…</Button>
        <Button onPress={() => setShowCode(!showCode)}>
          {showCode && "Hide Lua"}
          {!showCode && "Show Lua"}
        </Button>
        <Button primary onPress={build} disabled={busy || !folder.trim()}>
          Save and build test copy
        </Button>
      </div>
      <p className="text-xs text-faint">
        Writes a Lua module per design into the folder (the one triggered spells use), saves the map
        and its designs, then builds the "[DEV]" copy next to the map with every module. The map
        must use Lua.
      </p>
      {showCode && (
        <pre
          className={classes(
            "max-h-64 overflow-auto rounded-[3px] border border-line bg-well p-3 font-mono text-xs leading-relaxed",
            code.error && "text-danger",
          )}
        >
          {code.text}
        </pre>
      )}
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
    </div>
  );
}
