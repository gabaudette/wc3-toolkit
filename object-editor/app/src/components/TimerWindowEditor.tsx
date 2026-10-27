import type { TimerWindowDesign } from "../types";
import { Checkbox } from "./Checkbox";
import { DesignBasics, designInputClass, GameScreen } from "./DesignBasics";
import { GameText } from "./GameText";

type TimerWindowEditorProps = {
  timer: TimerWindowDesign;
  luaName: string;
  editable: boolean;
  onChange: (timer: TimerWindowDesign) => void;
  onDelete: () => void;
};

const SECONDS_PER_MINUTE = 60;
/** The longest countdown the Lua accepts: a day. */
const MAX_SECONDS = 86_400;

/** `90` → `1:30`, as the game's timer window counts. */
export function clock(seconds: number): string {
  const whole = Math.max(0, Math.ceil(seconds));
  const minutes = Math.floor(whole / SECONDS_PER_MINUTE);
  const rest = whole % SECONDS_PER_MINUTE;
  return `${minutes}:${String(rest).padStart(2, "0")}`;
}

/** A timer window's title, length, start, repeat and what happens when it runs out. */
export function TimerWindowEditor({
  timer,
  luaName,
  editable,
  onChange,
  onDelete,
}: TimerWindowEditorProps) {
  const set = (change: Partial<TimerWindowDesign>) => onChange({ ...timer, ...change });
  const minutes = Math.floor(timer.duration / SECONDS_PER_MINUTE);
  const seconds = timer.duration % SECONDS_PER_MINUTE;

  function setLength(nextMinutes: number, nextSeconds: number) {
    const total = nextMinutes * SECONDS_PER_MINUTE + nextSeconds;
    if (Number.isFinite(total) && total > 0 && total <= MAX_SECONDS) {
      set({ duration: total });
    }
  }

  let title = timer.title;
  if (timer.titleColor) {
    title = `|cff${timer.titleColor}${timer.title}|r`;
  }

  return (
    <div className="grid min-h-0 flex-1 grid-cols-[minmax(0,1fr)_minmax(0,1.1fr)] gap-5 overflow-y-auto p-5">
      <div className="flex min-w-0 flex-col gap-4">
        <DesignBasics
          name={timer.name}
          title={timer.title}
          titleColor={timer.titleColor}
          lua={`UI.${luaName}.Start()`}
          editable={editable}
          onChange={set}
          onDelete={onDelete}
        />

        <fieldset className="flex items-center gap-2 text-sm text-muted">
          <legend className="mb-1">Length</legend>
          <input
            type="number"
            min={0}
            step={1}
            value={minutes}
            disabled={!editable}
            onChange={(e) => setLength(Math.trunc(Number(e.target.value)), seconds)}
            aria-label="Minutes"
            className={`${designInputClass} w-20 text-right`}
          />
          min
          <input
            type="number"
            min={0}
            max={59}
            step={1}
            value={seconds}
            disabled={!editable}
            onChange={(e) => setLength(minutes, Number(e.target.value))}
            aria-label="Seconds"
            className={`${designInputClass} w-20 text-right`}
          />
          s
        </fieldset>

        <Checkbox
          isSelected={timer.startAtStart}
          isDisabled={!editable}
          onChange={(startAtStart) => set({ startAtStart })}
        >
          Start counting when the game starts
        </Checkbox>
        <Checkbox
          isSelected={timer.repeats}
          isDisabled={!editable}
          onChange={(repeats) => set({ repeats })}
        >
          Start over each time it runs out
        </Checkbox>
        <Checkbox
          isSelected={timer.hideWhenDone}
          isDisabled={!editable || timer.repeats}
          onChange={(hideWhenDone) => set({ hideWhenDone })}
        >
          Hide the window when the time runs out
        </Checkbox>

        <label className="flex flex-col gap-1 text-sm text-muted">
          When the time runs out, run this Lua function
          <input
            value={timer.onDone}
            disabled={!editable}
            onChange={(e) => set({ onDone: e.target.value })}
            placeholder="SpawnWave (optional)"
            className={`${designInputClass} font-mono`}
          />
        </label>
        {timer.onDone.trim() && (
          <p className="-mt-2 text-xs text-faint">
            Called each time, if your code defines{" "}
            <code className="font-mono">function {timer.onDone.trim()}()</code>.
          </p>
        )}
      </div>

      <GameScreen>
        <div className="absolute top-[1.2cqw] right-[1.2cqw] flex items-center gap-[1.5cqw] rounded-[0.3cqw] border-[0.15cqw] border-frame bg-[#0b0d12]/85 px-[0.9cqw] py-[0.5cqw] text-[1.2cqw] leading-tight text-white">
          <GameText text={title} singleLine />
          <span className="font-semibold">{clock(timer.duration)}</span>
        </div>
      </GameScreen>
    </div>
  );
}
