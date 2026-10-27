import { open } from "@tauri-apps/plugin-dialog";
import { useEffect, useState } from "react";
import { api, errorMessage } from "../api";
import type {
  AbilityDetail,
  Field,
  LevelField,
  SpellTemplate,
  TemplateParam,
  TemplateValues,
  TestCopy,
} from "../types";
import { capitalized } from "../ui";
import { Button } from "./Button";
import { CommitInput } from "./CommitInput";
import { LevelTable } from "./LevelTable";
import { LevelText } from "./LevelText";
import { PlaceholderValues } from "./TextPreview";
import { UnitIcon } from "./UnitIcon";

/** Each parameter's default: its values, or the first choice. */
export function defaultValues(template: SpellTemplate): TemplateValues {
  return Object.fromEntries(
    template.params.map((p) => {
      if (p.choices.length > 0) {
        return [p.id, [0]];
      }

      return [p.id, p.values];
    }),
  );
}

type TemplateChoicesProps = {
  templates: SpellTemplate[];
  busy: boolean;
  onPick: (template: SpellTemplate) => void;
};

/** The wizard's triggered spells, under the kinds of spell. */
export function TemplateChoices({ templates, busy, onPick }: TemplateChoicesProps) {
  return (
    <section aria-labelledby="triggered-spells" className="flex flex-col gap-2">
      <h3 id="triggered-spells" className="font-semibold text-gold-bright">
        Triggered spells
      </h3>
      <p className="text-sm text-muted">
        For effects no game ability has. The spell is a copy of Channel, and Tinkerbench writes the
        Lua code that runs the effect when it is cast.
      </p>
      <ul aria-label="Triggered spells" className="grid grid-cols-2 gap-2">
        {templates.map((template) => (
          <li key={template.id}>
            <button
              type="button"
              disabled={busy}
              onClick={() => onPick(template)}
              className="flex h-full w-full items-start gap-3 rounded-[3px] border border-line bg-well px-3 py-2 text-left outline-none hover:border-gold focus-visible:border-glow disabled:opacity-40"
            >
              <UnitIcon path={template.icon} size="small" />
              <span className="min-w-0 flex-1">
                <span className="block font-semibold text-gold-bright">{template.label}</span>
                <span className="block text-sm text-muted">{template.description}</span>
              </span>
            </button>
          </li>
        ))}
      </ul>
    </section>
  );
}

type EffectStepProps = {
  detail: AbilityDetail;
  template: SpellTemplate;
  values: TemplateValues;
  name: Field | undefined;
  /** The spell's own values that matter for this template (cooldown, mana, ...). */
  fields: LevelField[];
  tooltip: LevelField | undefined;
  onRename: (name: string) => void;
  onValue: (param: string, values: number[]) => void;
  onChange: (field: string, level: number, value: string) => void;
  onFill: (field: string, fromLevel: number, value: string) => void;
  onReset: (field: string, level: number) => void;
};

/** A triggered spell's values: the module's, then the spell's own, and the tooltip they make. */
export function EffectStep({
  detail,
  template,
  values,
  name,
  fields,
  tooltip,
  onRename,
  onValue,
  onChange,
  onFill,
  onReset,
}: EffectStepProps) {
  const levels = Math.max(1, ...template.params.map((p) => p.values.length));
  const levelNumbers = Array.from({ length: levels }, (_, i) => i + 1);
  const numbers = template.params.filter((p) => p.choices.length === 0);
  const choices = template.params.filter((p) => p.choices.length > 0);

  return (
    <PlaceholderValues.Provider value={detail}>
      <p className="text-sm text-muted">{template.description}</p>
      {name && <CommitInput label="Name" field={name} onCommit={onRename} />}
      {choices.map((param) => (
        <ChoiceInput key={param.id} param={param} values={values} onValue={onValue} />
      ))}
      <table className="w-fit text-sm">
        <caption className="mb-1 text-left text-xs text-faint">
          Values the Lua code uses, per level
        </caption>
        <thead>
          <tr>
            <th scope="col" className="sr-only">
              Value
            </th>
            {levelNumbers.map((level) => (
              <th key={level} scope="col" className="px-1 text-right font-normal text-faint">
                Level {level}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {numbers.map((param) => (
            <tr key={param.id}>
              <th scope="row" className="pr-4 text-left font-normal">
                {param.label}
              </th>
              {levelNumbers.map((level) => (
                <td key={level} className="p-1">
                  <NumberCell
                    param={param}
                    level={level}
                    values={values[param.id] ?? param.values}
                    onValue={onValue}
                  />
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
      <LevelTable fields={fields} editable onChange={onChange} onFill={onFill} onReset={onReset} />
      {tooltip && (
        <LevelText field={tooltip} editable={false} onChange={() => {}} onReset={() => {}} />
      )}
      <p className="text-xs text-faint">
        The tooltip follows these values. Everything else, such as the art, can be changed on the
        spell's page afterwards.
      </p>
    </PlaceholderValues.Provider>
  );
}

type ChoiceInputProps = {
  param: TemplateParam;
  values: TemplateValues;
  onValue: (param: string, values: number[]) => void;
};

function ChoiceInput({ param, values, onValue }: ChoiceInputProps) {
  return (
    <label className="flex items-center gap-2 text-sm text-muted">
      {param.label}
      <select
        value={values[param.id]?.[0] ?? 0}
        onChange={(e) => onValue(param.id, [Number(e.target.value)])}
        className="rounded-[3px] border border-field bg-well px-2 py-1 text-ink outline-none focus:border-glow"
      >
        {param.choices.map((choice, i) => (
          <option key={choice.label} value={i}>
            {capitalized(choice.label)}
          </option>
        ))}
      </select>
    </label>
  );
}

type NumberCellProps = {
  param: TemplateParam;
  level: number;
  /** The parameter's values for every level. */
  values: number[];
  onValue: (param: string, values: number[]) => void;
};

/** One level's value; saved on leaving the box or Enter, ignored when not a number. */
function NumberCell({ param, level, values, onValue }: NumberCellProps) {
  const value = values[level - 1] ?? values[values.length - 1] ?? 0;
  const [text, setText] = useState(String(value));
  useEffect(() => setText(String(value)), [value]);

  function commit() {
    const parsed = Number(text.trim());
    const valid =
      text.trim() !== "" && Number.isFinite(parsed) && (!param.whole || Number.isInteger(parsed));
    if (!valid) {
      setText(String(value));
      return;
    }

    if (parsed !== value) {
      const next = [...values];
      next[level - 1] = parsed;
      onValue(param.id, next);
    }
  }

  return (
    <input
      value={text}
      inputMode="decimal"
      aria-label={`${param.label}, level ${level}`}
      onChange={(e) => setText(e.target.value)}
      onBlur={commit}
      onKeyDown={(e) => {
        if (e.key === "Enter") {
          commit();
        }
      }}
      className="w-20 rounded-[3px] border border-field bg-well px-2 py-1 text-right font-mono text-ink outline-none focus:border-glow"
    />
  );
}

type CodeStepProps = {
  folder: string;
  source: string;
  onFolder: (folder: string) => void;
};

/** Where the spell's Lua module goes, and the module itself. */
export function CodeStep({ folder, source, onFolder }: CodeStepProps) {
  async function browse() {
    const picked = await open({ directory: true, ...(folder && { defaultPath: folder }) });
    if (typeof picked === "string") {
      onFolder(picked);
    }
  }

  return (
    <>
      <p className="text-sm text-muted">
        The effect is a Lua module. "Create spell" writes it into this folder, where the test copy
        of the map picks up every module. Keep one folder per map.
      </p>
      <div className="flex gap-2">
        <input
          value={folder}
          onChange={(e) => onFolder(e.target.value)}
          aria-label="Folder for Lua modules"
          className="min-w-0 flex-1 rounded-[3px] border border-field bg-well px-3 py-1.5 font-mono text-sm text-ink outline-none focus:border-glow"
        />
        <Button onPress={browse}>Browse…</Button>
      </div>
      <figure className="flex flex-col gap-1">
        <figcaption className="text-xs text-faint">
          The module, checked as Lua 5.3 like the game runs it
        </figcaption>
        <pre className="max-h-72 overflow-auto rounded-[3px] border border-line bg-well p-3 font-mono text-xs leading-relaxed">
          {source}
        </pre>
      </figure>
    </>
  );
}

type TestStepProps = {
  name: string;
  modulePath: string;
  folder: string;
  /** The map was saved: the window shows it as saved again. */
  onSaved: () => void;
};

/** How to try the new spell: save, build the `[DEV]` copy, play it. */
export function TestStep({ name, modulePath, folder, onSaved }: TestStepProps) {
  const [busy, setBusy] = useState(false);
  const [copy, setCopy] = useState<TestCopy | null>(null);
  const [error, setError] = useState<string | null>(null);

  async function build() {
    setBusy(true);
    setError(null);
    try {
      await api.saveMap();
      onSaved();
      setCopy(await api.buildTestCopy(folder));
    } catch (e) {
      setError(errorMessage(e));
    } finally {
      setBusy(false);
    }
  }

  return (
    <>
      <p>
        {name} is ready. Its Lua code is in{" "}
        <code className="font-mono text-sm break-all text-gold">{modulePath}</code>.
      </p>
      <ol className="flex list-decimal flex-col gap-1 pl-5 text-sm text-muted">
        <li>
          Build the test copy: the map is saved, then copied next to itself as “[DEV] …” with every
          module in the folder added to the copy's Lua script.
        </li>
        <li>
          In Warcraft III, start the copy as a custom game and cast the spell with a unit that has
          it.
        </li>
        <li>
          Build the copy again after changing the map or the modules: the World Editor rewrites the
          map's script on every save, so the code lives only in the copy.
        </li>
      </ol>
      <div>
        <Button primary onPress={build} disabled={busy}>
          Save and build test copy
        </Button>
      </div>
      {error && (
        <p role="alert" className="text-danger">
          {error}
        </p>
      )}
      {copy && (
        <p role="status" className="text-sm">
          Built <code className="font-mono break-all text-gold">{copy.path}</code> with{" "}
          {copy.modules.join(", ")}.
        </p>
      )}
      <p className="text-xs text-faint">
        The map's script language must be Lua (World Editor: Scenario, Map Options).
      </p>
    </>
  );
}
