import { Plus } from "@untitledui/icons";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { api } from "../api";
import { choosePath } from "../dialogs";
import { luaNames } from "../interface";
import { settle, whenOk } from "../result";
import type {
  Design,
  DialogDesign,
  IconChoice,
  LeaderboardDesign,
  MultiboardDesign,
  TimerWindowDesign,
} from "../types";
import { besides, classes, counted, DESIGN_FILTERS } from "../ui";
import { Button } from "./Button";
import { ExportDesignsDialog, ImportDesignsDialog } from "./DesignTransferDialogs";
import { DialogEditor } from "./DialogEditor";
import { GameText } from "./GameText";
import { LeaderboardEditor } from "./LeaderboardEditor";
import { MultiboardEditor } from "./MultiboardEditor";
import { clock, TimerWindowEditor } from "./TimerWindowEditor";

const NEW_TIMER_SECONDS = 90;

const MAP_STATUS_DELAY_MS = 300;

type InterfacePaneProps = {
  editable: boolean;
  mapPath: string | null;
  onMapChanged: () => void;
  onError: (message: string | null) => void;
};

function replaced(designs: Design[], changed: Design): Design[] {
  return designs.map((d) => {
    if (d.id === changed.id) {
      return changed;
    }

    return d;
  });
}

function newMultiboard(count: number): MultiboardDesign {
  const cell = (text: string, color: string | null = null) => ({ text, color, icon: null });

  return {
    kind: "multiboard",
    id: crypto.randomUUID(),
    name: `Multiboard ${count + 1}`,
    title: "Scoreboard",
    titleColor: null,
    columns: [
      { id: crypto.randomUUID(), name: "Player", width: 12 },
      { id: crypto.randomUUID(), name: "Kills", width: 5 },
      { id: crypto.randomUUID(), name: "Deaths", width: 5 },
    ],
    rows: [
      {
        id: crypto.randomUUID(),
        name: "Header",
        perPlayer: false,
        cells: [cell("Player", "ffcc00"), cell("Kills", "ffcc00"), cell("Deaths", "ffcc00")],
      },
      {
        id: crypto.randomUUID(),
        name: "Players",
        perPlayer: true,
        cells: [cell("{player}"), cell("0"), cell("0")],
      },
    ],
    showAtStart: true,
    minimized: false,
  };
}

function summary(design: Design): string {
  if (design.kind === "multiboard") {
    return `Multiboard · ${counted(design.columns.length, "column")}`;
  }

  if (design.kind === "leaderboard") {
    const rows: string[] = [];

    if (design.players) {
      rows.push("players");
    }

    if (design.labels.length > 0) {
      rows.push(counted(design.labels.length, "own row"));
    }

    return ["Leaderboard", ...rows].join(" · ");
  }

  if (design.kind === "timerWindow") {
    return `Timer window · ${clock(design.duration)}`;
  }

  return `Dialog · ${counted(design.buttons.length, "button")}`;
}

function newLeaderboard(count: number): LeaderboardDesign {
  return {
    kind: "leaderboard",
    id: crypto.randomUUID(),
    name: `Leaderboard ${count + 1}`,
    title: "Kills",
    titleColor: null,
    players: true,
    playerValue: 0,
    labels: [],
    sort: "highFirst",
    showAtStart: true,
  };
}

function newTimerWindow(count: number): TimerWindowDesign {
  return {
    kind: "timerWindow",
    id: crypto.randomUUID(),
    name: `Timer ${count + 1}`,
    title: "Next wave",
    titleColor: null,
    duration: NEW_TIMER_SECONDS,
    repeats: true,
    startAtStart: true,
    hideWhenDone: true,
    onDone: "",
  };
}

function newDialog(count: number): DialogDesign {
  return {
    kind: "dialog",
    id: crypto.randomUUID(),
    name: `Dialog ${count + 1}`,
    title: "Choose a difficulty",
    titleColor: null,
    buttons: [
      {
        id: crypto.randomUUID(),
        text: "Normal",
        hotkey: "N",
        highlightHotkey: true,
        action: { kind: "close" },
      },
      {
        id: crypto.randomUUID(),
        text: "Hard",
        hotkey: "H",
        highlightHotkey: true,
        action: { kind: "close" },
      },
    ],
    showAtStart: true,
  };
}

export function InterfacePane({ editable, mapPath, onMapChanged, onError }: InterfacePaneProps) {
  const [designs, setDesigns] = useState<Design[]>([]);
  const [selected, setSelected] = useState<string | null>(null);
  const [icons, setIcons] = useState<IconChoice[]>([]);
  const [exporting, setExporting] = useState(false);
  const [importing, setImporting] = useState<string | null>(null);
  const queue = useRef<Promise<void>>(Promise.resolve());
  const statusTimer = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);
  const flush = useCallback(() => queue.current, []);

  useEffect(() => {
    if (!editable) {
      return;
    }

    void settle(
      api.uiDesigns(),
      (loaded) => {
        setDesigns(loaded);
        setSelected(loaded[0]?.id ?? null);
      },
      onError,
    );
  }, [editable, onError]);

  const names = useMemo(() => luaNames(designs), [designs]);
  const exportChoice = useMemo(() => designs.map((d) => d.id), [designs]);
  const design = designs.find((d) => d.id === selected);

  function update(next: Design[]) {
    setDesigns(next);
    queue.current = queue.current.then(() =>
      settle(api.setUiDesigns(next), () => onError(null), onError),
    );
    clearTimeout(statusTimer.current);
    statusTimer.current = setTimeout(() => {
      queue.current.then(onMapChanged);
    }, MAP_STATUS_DELAY_MS);
  }

  useEffect(() => {
    if (design?.kind !== "multiboard" || icons.length > 0) {
      return;
    }

    void settle(api.spellIcons(), setIcons, onError);
  }, [design?.kind, icons.length, onError]);

  function add(created: Design) {
    update([...designs, created]);
    setSelected(created.id);
  }

  function duplicate(original: Design) {
    const taken = new Set(designs.map((d) => d.name.toLowerCase()));
    let name = `${original.name} copy`;

    for (let n = 2; taken.has(name.toLowerCase()); n++) {
      name = `${original.name} copy ${n}`;
    }

    const copy: Design = { ...structuredClone(original), id: crypto.randomUUID(), name };
    const at = designs.findIndex((d) => d.id === original.id) + 1;

    update([...designs.slice(0, at), copy, ...designs.slice(at)]);
    setSelected(copy.id);
  }

  async function pickDesignFile() {
    const picked = await choosePath({
      title: "Import designs",
      filters: DESIGN_FILTERS,
      ...(mapPath && { defaultPath: besides(mapPath, "") }),
    });

    if (!picked.ok) {
      onError(picked.error);
      return;
    }

    if (picked.value) {
      setImporting(picked.value);
    }
  }

  function imported(next: Design[]) {
    const added = next.find((d) => !designs.some((old) => old.id === d.id));

    setDesigns(next);
    setSelected(added?.id ?? selected);
    onMapChanged();
  }

  function remove(id: string) {
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
        Open a map to design its dialogs, boards and timers. They are saved next to it, in
        "&lt;map&gt; UI.json".
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
            <Button primary onPress={() => add(newLeaderboard(designs.length))}>
              <span className="flex items-center justify-center gap-1.5">
                <Plus className="size-4" aria-hidden="true" />
                Leaderboard
              </span>
            </Button>
            <Button primary onPress={() => add(newTimerWindow(designs.length))}>
              <span className="flex items-center justify-center gap-1.5">
                <Plus className="size-4" aria-hidden="true" />
                Timer
              </span>
            </Button>
          </div>
          <div className="grid grid-cols-2 gap-2">
            <Button small onPress={pickDesignFile}>
              Import…
            </Button>
            <Button small disabled={designs.length === 0} onPress={() => setExporting(true)}>
              Export…
            </Button>
          </div>
          <p className="text-xs text-faint">
            {counted(designs.length, "design")} · saved with the map
          </p>
        </div>
        {designs.length === 0 && (
          <p className="p-3 text-sm text-faint">
            Nothing yet. A dialog asks players to choose (a difficulty, a game mode); a multiboard
            shows a table in the corner (a scoreboard); a leaderboard ranks players or teams; a
            timer counts down (to the next wave).
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
                onDuplicate={() => duplicate(design)}
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
                onDuplicate={() => duplicate(design)}
                onDelete={() => remove(design.id)}
              />
            )}
            {design.kind === "leaderboard" && (
              <LeaderboardEditor
                key={design.id}
                board={design}
                luaName={names.get(design.id) ?? ""}
                editable={editable}
                onChange={(changed) => update(replaced(designs, changed))}
                onDuplicate={() => duplicate(design)}
                onDelete={() => remove(design.id)}
              />
            )}
            {design.kind === "timerWindow" && (
              <TimerWindowEditor
                key={design.id}
                timer={design}
                luaName={names.get(design.id) ?? ""}
                editable={editable}
                onChange={(changed) => update(replaced(designs, changed))}
                onDuplicate={() => duplicate(design)}
                onDelete={() => remove(design.id)}
              />
            )}
          </>
        )}
        {!design && (
          <p className="m-auto text-muted">Make a design with the buttons on the left.</p>
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

      <ExportDesignsDialog
        open={exporting}
        designs={designs}
        mapPath={mapPath}
        initial={exportChoice}
        flush={flush}
        onClose={() => setExporting(false)}
      />
      <ImportDesignsDialog
        path={importing}
        flush={flush}
        onImported={imported}
        onClose={() => setImporting(null)}
      />
    </div>
  );
}

type LuaBarProps = {
  selected: string | null;
  designs: Design[];
  flush: () => Promise<void>;
  onSaved: () => void;
};

function LuaBar({ selected, designs, flush, onSaved }: LuaBarProps) {
  const [folder, setFolder] = useState("");
  const [showCode, setShowCode] = useState(false);
  const [code, setCode] = useState<{ text: string; error: boolean }>({ text: "", error: false });
  const [busy, setBusy] = useState(false);
  const [status, setStatus] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    void whenOk(api.defaultLuaFolder(), (dir) => setFolder((current) => current || dir || ""));
  }, []);

  useEffect(() => {
    if (!showCode || !selected || designs.length === 0) {
      return;
    }

    void flush()
      .then(() => api.uiLua(selected))
      .then((module) => {
        if (module.ok) {
          setCode({ text: module.value, error: false });
          return;
        }

        setCode({ text: module.error, error: true });
      });
  }, [showCode, selected, designs, flush]);

  async function browse() {
    const picked = await choosePath({ directory: true, ...(folder && { defaultPath: folder }) });

    if (!picked.ok) {
      setError(picked.error);
      return;
    }

    if (picked.value) {
      setFolder(picked.value);
    }
  }

  async function build() {
    setBusy(true);
    setError(null);
    setStatus(null);
    const stop = (message: string) => {
      setError(message);
      setBusy(false);
    };
    const dir = folder.trim();

    await flush();
    const written = await api.writeUiLua(dir);

    if (!written.ok) {
      stop(written.error);
      return;
    }

    const saved = await api.saveMap();

    if (!saved.ok) {
      stop(saved.error);
      return;
    }

    onSaved();
    const copy = await api.buildTestCopy(dir);

    if (!copy.ok) {
      stop(copy.error);
      return;
    }

    setBusy(false);
    setStatus(
      `Wrote ${written.value.join(", ")} and built ${copy.value.path} with ${copy.value.modules.length} module(s).`,
    );
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
