import { Plus } from "@untitledui/icons";
import { type ReactNode, useCallback, useEffect, useRef, useState } from "react";
import { api } from "../api";
import { newKey } from "../loot";
import { settle } from "../result";
import type { ForceView, MapInfoView, PlayerView, PropertiesView, ScreenView } from "../types";
import { classes } from "../ui";
import { Button } from "./Button";
import { Checkbox } from "./Checkbox";
import { CommitText, ConstantsPanel } from "./ConstantsPanel";
import { GameText } from "./GameText";
import { Select } from "./Select";

type Section =
  | "description"
  | "screens"
  | "options"
  | "environment"
  | "players"
  | "forces"
  | "constants";

const SECTIONS: { id: Section; label: string }[] = [
  { id: "description", label: "Description" },
  { id: "screens", label: "Loading screen and prologue" },
  { id: "options", label: "Options" },
  { id: "environment", label: "Fog, weather and water" },
  { id: "players", label: "Players" },
  { id: "forces", label: "Forces" },
  { id: "constants", label: "Gameplay constants" },
];

const CONTROLLERS = [
  { value: "1", label: "User" },
  { value: "2", label: "Computer" },
  { value: "3", label: "Neutral" },
  { value: "4", label: "Rescuable" },
];

const RACES = [
  { value: "0", label: "Selectable" },
  { value: "1", label: "Human" },
  { value: "2", label: "Orc" },
  { value: "3", label: "Undead" },
  { value: "4", label: "Night Elf" },
];

const FOG_STYLES = [
  { value: "0", label: "Linear" },
  { value: "1", label: "Exponential" },
  { value: "2", label: "Exponential, steeper" },
];

const FORCE_FLAGS = [
  { flag: 0x01, label: "Allied" },
  { flag: 0x02, label: "Allied victory" },
  { flag: 0x08, label: "Share vision" },
  { flag: 0x10, label: "Share unit control" },
  { flag: 0x20, label: "Share advanced unit control" },
];

const PLAYER_COLORS = [
  "Red",
  "Blue",
  "Teal",
  "Purple",
  "Yellow",
  "Orange",
  "Green",
  "Pink",
  "Gray",
  "Light Blue",
  "Dark Green",
  "Brown",
  "Maroon",
  "Navy",
  "Turquoise",
  "Violet",
  "Wheat",
  "Peach",
  "Mint",
  "Lavender",
  "Coal",
  "Snow",
  "Emerald",
  "Peanut",
];

const NO_WEATHER = "";
const TEXT_ROWS = 4;

type MapPaneProps = {
  editable: boolean;
  onMapChanged: () => void;
  onError: (message: string | null) => void;
};

type KeyedForce = ForceView & { key: string };

function playerLabel(number: number): string {
  const color = PLAYER_COLORS[number];

  if (color) {
    return `Player ${number + 1} (${color})`;
  }

  return `Player ${number + 1}`;
}

function withFlag(flags: number, flag: number, on: boolean): number {
  if (on) {
    return flags | flag;
  }

  return flags & ~flag;
}

type CommitAreaProps = {
  label: string;
  value: string;
  onCommit: (value: string) => void;
};

function CommitArea({ label, value, onCommit }: CommitAreaProps) {
  const [text, setText] = useState(value);

  useEffect(() => setText(value), [value]);

  return (
    <textarea
      aria-label={label}
      value={text}
      rows={TEXT_ROWS}
      onChange={(e) => setText(e.target.value)}
      onBlur={() => {
        if (text !== value) {
          onCommit(text);
        }
      }}
      className="w-full rounded-[3px] border border-field bg-well px-2.5 py-1.5 text-sm text-ink outline-none focus:border-focus"
    />
  );
}

function Labeled({ label, children }: { label: string; children: ReactNode }) {
  return (
    <div className="flex flex-col gap-1">
      <span aria-hidden="true" className="text-sm text-muted">
        {label}
      </span>
      {children}
    </div>
  );
}

export function MapPane({ editable, onMapChanged, onError }: MapPaneProps) {
  const [section, setSection] = useState<Section>("description");
  const [info, setInfo] = useState<MapInfoView | null>(null);
  const [failure, setFailure] = useState<string | null>(null);
  const [forceKeys, setForceKeys] = useState<string[]>([]);
  const queue = useRef<Promise<void>>(Promise.resolve());

  const reload = useCallback(() => {
    void settle(
      api.mapInfo(),
      (loaded) => {
        setInfo(loaded);
        setForceKeys(loaded.properties.forces.map(() => newKey()));
        setFailure(null);
      },
      setFailure,
    );
  }, []);

  useEffect(() => {
    if (editable) {
      reload();
    }
  }, [editable, reload]);

  function update(properties: PropertiesView) {
    if (!info) {
      return;
    }

    setInfo({ ...info, properties });
    queue.current = queue.current.then(() =>
      settle(api.setMapInfo(properties), () => onError(null), onError).then(onMapChanged),
    );
  }

  if (!editable) {
    return <p className="m-auto text-muted">Open a map to edit its properties and constants.</p>;
  }

  return (
    <div className="flex min-h-0 flex-1 gap-2.5">
      <nav
        aria-label="Map sections"
        className="panel flex w-64 shrink-0 flex-col gap-0.5 rounded-[3px] bg-surface-1 p-2"
      >
        {SECTIONS.map((s) => (
          <button
            key={s.id}
            type="button"
            onClick={() => setSection(s.id)}
            aria-current={section === s.id}
            className={classes(
              "rounded-[3px] px-3 py-2 text-left text-sm hover:bg-surface-2",
              section === s.id &&
                "bg-surface-2 text-gold-bright shadow-[inset_2px_0_0_var(--color-glow)]",
              section !== s.id && "text-ink",
            )}
          >
            {s.label}
          </button>
        ))}
        {info && (
          <p className="mt-auto px-3 pb-1 text-xs text-faint">
            Map format {info.version} · {info.lua && "Lua"}
            {!info.lua && "JASS"} script
          </p>
        )}
      </nav>

      <section className="frame flex min-h-0 min-w-0 flex-1 flex-col bg-surface-1">
        {section === "constants" && (
          <ConstantsPanel onMapChanged={onMapChanged} onError={onError} />
        )}
        {section !== "constants" && failure && (
          <p className="m-auto max-w-md text-center text-danger">{failure}</p>
        )}
        {section !== "constants" && info && (
          <div className="flex min-h-0 max-w-4xl flex-1 flex-col gap-4 overflow-y-auto p-5">
            <h2 className="font-display text-3xl font-bold text-gold-bright">
              {SECTIONS.find((s) => s.id === section)?.label}
            </h2>
            {section === "description" && (
              <DescriptionSection properties={info.properties} onChange={update} />
            )}
            {section === "screens" && (
              <ScreensSection properties={info.properties} onChange={update} />
            )}
            {section === "options" && (
              <OptionsSection properties={info.properties} onChange={update} />
            )}
            {section === "environment" && <EnvironmentSection info={info} onChange={update} />}
            {section === "players" && (
              <PlayersSection properties={info.properties} onChange={update} />
            )}
            {section === "forces" && (
              <ForcesSection
                properties={info.properties}
                keys={forceKeys}
                onKeys={setForceKeys}
                onChange={update}
              />
            )}
          </div>
        )}
      </section>
    </div>
  );
}

type SectionProps = {
  properties: PropertiesView;
  onChange: (properties: PropertiesView) => void;
};

function DescriptionSection({ properties, onChange }: SectionProps) {
  return (
    <>
      <Labeled label="Name">
        <CommitText
          label="Map name"
          value={properties.name}
          wide
          onCommit={(name) => onChange({ ...properties, name })}
        />
        <span className="text-xs text-faint">
          Shown in the game's map list: <GameText text={properties.name} singleLine />
        </span>
      </Labeled>
      <Labeled label="Author">
        <CommitText
          label="Author"
          value={properties.author}
          wide
          onCommit={(author) => onChange({ ...properties, author })}
        />
      </Labeled>
      <Labeled label="Suggested players">
        <CommitText
          label="Suggested players"
          value={properties.suggestedPlayers}
          wide
          onCommit={(suggestedPlayers) => onChange({ ...properties, suggestedPlayers })}
        />
      </Labeled>
      <Labeled label="Description">
        <CommitArea
          label="Description"
          value={properties.description}
          onCommit={(description) => onChange({ ...properties, description })}
        />
      </Labeled>
    </>
  );
}

type ScreenEditorProps = {
  title: string;
  screen: ScreenView;
  modelHint: string;
  onChange: (screen: ScreenView) => void;
};

function ScreenEditor({ title, screen, modelHint, onChange }: ScreenEditorProps) {
  return (
    <section className="flex flex-col gap-3 rounded-[3px] border border-line p-4">
      <h3 className="font-semibold text-gold-bright">{title}</h3>
      <Labeled label="Title">
        <CommitText
          label={`${title} title`}
          value={screen.title}
          wide
          onCommit={(value) => onChange({ ...screen, title: value })}
        />
      </Labeled>
      <Labeled label="Subtitle">
        <CommitText
          label={`${title} subtitle`}
          value={screen.subtitle}
          wide
          onCommit={(subtitle) => onChange({ ...screen, subtitle })}
        />
      </Labeled>
      <Labeled label="Text">
        <CommitArea
          label={`${title} text`}
          value={screen.text}
          onCommit={(text) => onChange({ ...screen, text })}
        />
      </Labeled>
      <Labeled label="Model">
        <CommitText
          label={`${title} model`}
          value={screen.model}
          wide
          onCommit={(model) => onChange({ ...screen, model })}
        />
        <span className="text-xs text-faint">{modelHint}</span>
      </Labeled>
    </section>
  );
}

function ScreensSection({ properties, onChange }: SectionProps) {
  return (
    <>
      <ScreenEditor
        title="Loading screen"
        screen={properties.loadingScreen}
        modelHint="An imported loading screen model, such as war3mapImported\LoadingScreen.mdx. Empty for the default screen."
        onChange={(loadingScreen) => onChange({ ...properties, loadingScreen })}
      />
      <ScreenEditor
        title="Prologue"
        screen={properties.prologue}
        modelHint="Shown before campaign maps start. Most maps leave it empty."
        onChange={(prologue) => onChange({ ...properties, prologue })}
      />
    </>
  );
}

function OptionsSection({ properties, onChange }: SectionProps) {
  return (
    <div className="flex flex-col gap-2">
      {properties.options.map((option) => (
        <Checkbox
          key={option.flag}
          isSelected={option.on}
          onChange={(on) =>
            onChange({
              ...properties,
              options: properties.options.map((o) => {
                if (o.flag === option.flag) {
                  return { ...o, on };
                }

                return o;
              }),
            })
          }
        >
          <span className="text-ink">{option.label}</span>
        </Checkbox>
      ))}
    </div>
  );
}

type EnvironmentSectionProps = {
  info: MapInfoView;
  onChange: (properties: PropertiesView) => void;
};

function EnvironmentSection({ info, onChange }: EnvironmentSectionProps) {
  const properties = info.properties;
  const fog = properties.fog;

  return (
    <>
      <section className="flex flex-col gap-3 rounded-[3px] border border-line p-4">
        <Checkbox
          isSelected={fog.enabled}
          onChange={(enabled) => onChange({ ...properties, fog: { ...fog, enabled } })}
        >
          <span className="font-semibold text-gold-bright">Terrain fog</span>
        </Checkbox>
        <div className="grid grid-cols-2 gap-3">
          <Labeled label="Style">
            <Select
              label="Fog style"
              value={String(fog.style)}
              options={FOG_STYLES}
              onChange={(style) =>
                onChange({ ...properties, fog: { ...fog, style: Number(style) } })
              }
            />
          </Labeled>
          <Labeled label="Color">
            <input
              type="color"
              aria-label="Fog color"
              value={`#${fog.color}`}
              onChange={(e) =>
                onChange({ ...properties, fog: { ...fog, color: e.target.value.slice(1) } })
              }
              className="h-9 w-20 rounded-[3px] border border-field bg-well"
            />
          </Labeled>
          <Labeled label="Starts at height">
            <CommitText
              label="Fog start height"
              value={String(fog.start)}
              onCommit={(v) => onChange({ ...properties, fog: { ...fog, start: Number(v) } })}
            />
          </Labeled>
          <Labeled label="Ends at height">
            <CommitText
              label="Fog end height"
              value={String(fog.end)}
              onCommit={(v) => onChange({ ...properties, fog: { ...fog, end: Number(v) } })}
            />
          </Labeled>
          <Labeled label="Density">
            <CommitText
              label="Fog density"
              value={String(fog.density)}
              onCommit={(v) => onChange({ ...properties, fog: { ...fog, density: Number(v) } })}
            />
          </Labeled>
        </div>
      </section>
      <Labeled label="Weather">
        <div className="w-80">
          <Select
            label="Weather"
            value={properties.weather}
            options={[
              { value: NO_WEATHER, label: "None" },
              ...info.weather.map((w) => ({ value: w.id, label: w.name })),
            ]}
            onChange={(weather) => onChange({ ...properties, weather })}
          />
        </div>
      </Labeled>
      <Labeled label="Sound environment">
        <CommitText
          label="Sound environment"
          value={properties.soundEnvironment}
          wide
          onCommit={(soundEnvironment) => onChange({ ...properties, soundEnvironment })}
        />
        <span className="text-xs text-faint">
          The reverb for the map's sounds, such as "cave" or "forest". Empty for the default.
        </span>
      </Labeled>
      <section className="flex flex-col gap-2 rounded-[3px] border border-line p-4">
        <Checkbox
          isSelected={properties.waterTinting}
          onChange={(waterTinting) => onChange({ ...properties, waterTinting })}
        >
          <span className="font-semibold text-gold-bright">Tint the water</span>
        </Checkbox>
        <input
          type="color"
          aria-label="Water color"
          value={`#${properties.waterColor}`}
          onChange={(e) => onChange({ ...properties, waterColor: e.target.value.slice(1) })}
          className="h-9 w-20 rounded-[3px] border border-field bg-well"
        />
      </section>
    </>
  );
}

function PlayersSection({ properties, onChange }: SectionProps) {
  function setPlayer(player: PlayerView) {
    onChange({
      ...properties,
      players: properties.players.map((p) => {
        if (p.index === player.index) {
          return player;
        }

        return p;
      }),
    });
  }

  return (
    <>
      <Checkbox
        isSelected={properties.fixedPlayerSettings}
        onChange={(fixedPlayerSettings) => onChange({ ...properties, fixedPlayerSettings })}
      >
        <span className="text-ink">Fixed player settings (players cannot change race or team)</span>
      </Checkbox>
      <p className="text-xs text-faint">
        Adding or removing players needs start locations on the map; use the World Editor for that.
      </p>
      <table className="text-sm">
        <thead>
          <tr className="text-left text-xs text-faint">
            <th className="px-2 py-1 font-normal">Slot</th>
            <th className="px-2 py-1 font-normal">Name</th>
            <th className="px-2 py-1 font-normal">Controller</th>
            <th className="px-2 py-1 font-normal">Race</th>
            <th className="px-2 py-1 font-normal">Fixed start</th>
          </tr>
        </thead>
        <tbody>
          {properties.players.map((player) => (
            <tr key={player.index} className="border-t border-line">
              <td className="px-2 py-1.5 text-ink">{playerLabel(player.number)}</td>
              <td className="px-2 py-1.5">
                <CommitText
                  label={`${playerLabel(player.number)} name`}
                  value={player.name}
                  wide
                  onCommit={(name) => setPlayer({ ...player, name })}
                />
              </td>
              <td className="w-36 px-2 py-1.5">
                <Select
                  label={`${playerLabel(player.number)} controller`}
                  value={String(player.controller)}
                  options={CONTROLLERS}
                  onChange={(v) => setPlayer({ ...player, controller: Number(v) })}
                />
              </td>
              <td className="w-36 px-2 py-1.5">
                <Select
                  label={`${playerLabel(player.number)} race`}
                  value={String(player.race)}
                  options={RACES}
                  onChange={(v) => setPlayer({ ...player, race: Number(v) })}
                />
              </td>
              <td className="px-2 py-1.5">
                <Checkbox
                  isSelected={player.fixedStart}
                  onChange={(fixedStart) => setPlayer({ ...player, fixedStart })}
                  aria-label={`${playerLabel(player.number)} fixed start location`}
                >
                  <span className="sr-only">Fixed start location</span>
                </Checkbox>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </>
  );
}

type ForcesSectionProps = SectionProps & {
  keys: string[];
  onKeys: (keys: string[]) => void;
};

function ForcesSection({ properties, keys, onKeys, onChange }: ForcesSectionProps) {
  const forces: KeyedForce[] = properties.forces.map((f, at) => ({
    ...f,
    key: keys[at] ?? String(at),
  }));

  function setForces(next: KeyedForce[]) {
    onKeys(next.map((f) => f.key));
    onChange({
      ...properties,
      forces: next.map(({ name, flags, players }) => ({ name, flags, players })),
    });
  }

  function setForce(force: KeyedForce) {
    setForces(
      forces.map((f) => {
        if (f.key === force.key) {
          return force;
        }

        return f;
      }),
    );
  }

  function moveTo(target: KeyedForce, number: number) {
    setForces(
      forces.map((f) => {
        const others = f.players.filter((p) => p !== number);

        if (f.key === target.key) {
          return { ...f, players: [...others, number].sort((a, b) => a - b) };
        }

        return { ...f, players: others };
      }),
    );
  }

  return (
    <>
      <Checkbox
        isSelected={properties.customForces}
        onChange={(customForces) => onChange({ ...properties, customForces })}
      >
        <span className="text-ink">Use custom forces (teams set here instead of in the lobby)</span>
      </Checkbox>
      {forces.map((force) => (
        <section
          key={force.key}
          className="flex flex-col gap-3 rounded-[3px] border border-line p-4"
        >
          <div className="flex items-center gap-2">
            <CommitText
              label="Force name"
              value={force.name}
              wide
              onCommit={(name) => setForce({ ...force, name })}
            />
            <Button
              small
              disabled={forces.length < 2}
              onPress={() => setForces(forces.filter((f) => f.key !== force.key))}
            >
              Remove force
            </Button>
          </div>
          <div className="flex flex-wrap gap-x-4 gap-y-1">
            {FORCE_FLAGS.map((f) => (
              <Checkbox
                key={f.flag}
                isSelected={(force.flags & f.flag) !== 0}
                onChange={(on) => setForce({ ...force, flags: withFlag(force.flags, f.flag, on) })}
              >
                <span className="text-sm text-ink">{f.label}</span>
              </Checkbox>
            ))}
          </div>
          <div className="flex flex-wrap gap-x-4 gap-y-1">
            {properties.players.map((player) => (
              <Checkbox
                key={player.index}
                isSelected={force.players.includes(player.number)}
                onChange={() => moveTo(force, player.number)}
              >
                <span className="text-sm text-muted">
                  {playerLabel(player.number)}
                  {player.name && `: ${player.name}`}
                </span>
              </Checkbox>
            ))}
          </div>
        </section>
      ))}
      <div>
        <Button
          onPress={() =>
            setForces([
              ...forces,
              { key: newKey(), name: `Force ${forces.length + 1}`, flags: 0, players: [] },
            ])
          }
        >
          <span className="flex items-center gap-1.5">
            <Plus className="size-4" aria-hidden="true" />
            Add force
          </span>
        </Button>
      </div>
      <p className="text-xs text-faint">
        Each player belongs to one force: ticking a player moves it to that force.
      </p>
    </>
  );
}
