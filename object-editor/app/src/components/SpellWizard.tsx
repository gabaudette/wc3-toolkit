import { ArrowLeft } from "@untitledui/icons";
import { useEffect, useRef, useState } from "react";
import { api } from "../api";
import { ok, type Result, settle } from "../result";
import type {
  AbilityDetail,
  Field,
  IconChoice,
  LevelField,
  SpellBase,
  SpellKind,
  SpellTemplate,
  TemplateValues,
  UnitDetail,
  UnitItem,
} from "../types";
import { levelCount } from "../ui";
import { AbilityPicker } from "./AbilityPicker";
import { Button } from "./Button";
import { CommitInput } from "./CommitInput";
import { DialogShell } from "./DialogShell";
import { GameText } from "./GameText";
import { LevelTable } from "./LevelTable";
import { LevelText } from "./LevelText";
import { type BuffChoice, SpellBuffStep } from "./SpellBuffStep";
import { SpellButtonStep } from "./SpellButtonStep";
import { CodeStep, defaultValues, EffectStep, TemplateChoices, TestStep } from "./SpellTriggered";
import { SpellUnits } from "./SpellUnits";
import { PlaceholderValues } from "./TextPreview";
import { UnitIcon } from "./UnitIcon";

type SpellWizardProps = {
  open: boolean;
  onClose: () => void;
  onCreated: (id: string) => void;
  onMapChanged: () => void;
  onUnitsChanged: () => void;
};

type Step = "kind" | "base" | "values" | "buff" | "effect" | "button" | "units" | "code" | "test";

const NAME = "anam";
const LEVELS = "alev";
const COMMON_VALUES = ["atar", "aran", "acdn", "amcs", "aare", "adur", "ahdu"];
const SPELL_DATA = "data";
/** The tooltip title and text, per level. */
const TOOLTIPS = ["atp1", "aub1"];
/** The tooltip text, which a triggered spell's values fill in. */
const TOOLTIP_TEXT = "aub1";
/** Tooltip texts that show values: normal, turned off, and the hero's learn text. */
const PLACEHOLDER_TEXTS = ["aub1", "auu1", "arut"];
/** Texts that usually repeat the spell's name: tooltip titles and the learn title. */
const NAMED_TEXTS = ["atp1", "aret"];
const NO_LEVEL = 0;
/** The buffs a spell leaves on its target, per level, as a comma list of IDs. */
const BUFFS = "abuf";
/** Fields of a spell's own buff named after it: the editor's name and the tooltip title. */
const BUFF_NAMES = ["fnam", "ftip"];
const BASE_BUFF: BuffChoice = { kind: "base" };
const ICON = "aart";
/** A hero spell's icon in its learn menu; it follows the button's icon. */
const LEARN_ICON = "arar";
/** Texts that may show the hotkey highlighted, as older tooltips do: `(|cffffcc00T|r)`. */
const HOTKEY_TEXTS = ["atp1", "aret", "aut1"];
const HOTKEY = "ahky";
/** A hero spell's hotkey in its learn menu; it follows the cast hotkey when they matched. */
const LEARN_HOTKEY = "arhk";

const STEP_TITLES: Record<Step, string> = {
  kind: "New spell: what does it do?",
  base: "New spell: start from",
  values: "New spell: values",
  buff: "New spell: buff",
  button: "New spell: button",
  units: "New spell: give it to units",
  effect: "New spell: effect",
  code: "New spell: Lua code",
  test: "New spell: try it",
};

function triggeredFields(detail: AbilityDetail, template: SpellTemplate): LevelField[] {
  const ids = ["acdn", "amcs"];

  if (template.target !== "none") {
    ids.push("aran", "atar");
  }

  if (template.area > 0) {
    ids.push("aare");
  }

  return ids.flatMap((id) => {
    const field = levelField(detail, id);

    if (field) {
      return [field];
    }

    return [];
  });
}

function valueFields(detail: AbilityDetail): LevelField[] {
  const all = detail.sections.flatMap((s) =>
    s.levelFields.map((f) => ({ f, category: s.category })),
  );
  const common = COMMON_VALUES.flatMap((id) => all.filter(({ f }) => f.field.id === id));
  const own = all.filter(
    ({ category, f }) => category === SPELL_DATA && !TOOLTIPS.includes(f.field.id),
  );

  return [...common, ...own].map(({ f }) => f);
}

function spellType(hero: boolean): string {
  if (hero) {
    return "Hero spell";
  }

  return "Unit spell";
}

/**
 * Points the copied tooltips at the spell's own values. Game tooltips name
 * their ability in placeholders (`<AHtb,DataA1>`), so a copy would keep
 * showing the original's numbers, in the editor and in game.
 */
async function ownValues(
  id: string,
  base: string,
  detail: AbilityDetail,
): Promise<Result<AbilityDetail>> {
  const reference = new RegExp(`<${base},`, "gi");
  let updated = ok(detail);

  for (const fieldId of PLACEHOLDER_TEXTS) {
    for (const { raw, level } of namedTexts(detail, fieldId)) {
      const own = raw.replace(reference, `<${id},`);

      if (own !== raw) {
        updated = await api.setAbilityField(id, fieldId, level, own);
        if (!updated.ok) {
          return updated;
        }
      }
    }
  }

  return updated;
}

function namedTexts(detail: AbilityDetail, id: string): { raw: string; level: number }[] {
  const perLevel = levelField(detail, id);

  if (perLevel) {
    return perLevel.cells.map((cell, index) => ({ raw: cell.raw, level: index + 1 }));
  }

  const plain = plainField(detail, id);

  if (plain) {
    return [{ raw: plain.raw, level: NO_LEVEL }];
  }

  return [];
}

function idList(text: string): string[] {
  return text
    .split(",")
    .map((id) => id.trim())
    .filter(Boolean);
}

/**
 * The spell's buff field for a choice, or `null` to keep the base ability's.
 * Its own buff replaces the base's first and keeps any others (a few spells
 * leave two); another buff replaces them all.
 */
function chosenBuffs(choice: BuffChoice, baseBuffs: string[]): string | null {
  if (choice.kind === "own") {
    return [choice.id, ...baseBuffs.slice(1)].join(",");
  }

  if (choice.kind === "other") {
    return choice.id;
  }

  return null;
}

function buffShown(choice: BuffChoice, baseBuffs: string[]): string | null {
  if (choice.kind !== "base") {
    return choice.id;
  }

  return baseBuffs[0] ?? null;
}

function plainField(detail: AbilityDetail, id: string): Field | undefined {
  return detail.sections.flatMap((s) => s.fields).find((f) => f.id === id);
}

function levelField(detail: AbilityDetail, id: string): LevelField | undefined {
  return detail.sections.flatMap((s) => s.levelFields).find((f) => f.field.id === id);
}

export function SpellWizard({
  open,
  onClose,
  onCreated,
  onMapChanged,
  onUnitsChanged,
}: SpellWizardProps) {
  const [step, setStep] = useState<Step>("kind");
  const [kinds, setKinds] = useState<SpellKind[]>([]);
  const [kind, setKind] = useState<SpellKind | null>(null);
  const [detail, setDetail] = useState<AbilityDetail | null>(null);
  const [picking, setPicking] = useState(false);
  const [buffChoice, setBuffChoice] = useState<BuffChoice>(BASE_BUFF);
  const [baseBuffs, setBaseBuffs] = useState<string[]>([]);
  const [baseName, setBaseName] = useState("");
  const [shownBuff, setShownBuff] = useState<UnitDetail | null>(null);
  const [allBuffs, setAllBuffs] = useState<UnitItem[]>([]);
  const [icons, setIcons] = useState<IconChoice[]>([]);
  const [templates, setTemplates] = useState<SpellTemplate[]>([]);
  const [template, setTemplate] = useState<SpellTemplate | null>(null);
  const [values, setValues] = useState<TemplateValues>({});
  const [folder, setFolder] = useState("");
  const [source, setSource] = useState("");
  const [modulePath, setModulePath] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const queue = useRef<Promise<void>>(Promise.resolve());

  useEffect(() => {
    if (!open) {
      return;
    }

    setStep("kind");
    setKind(null);
    setDetail(null);
    setBuffChoice(BASE_BUFF);
    setTemplate(null);
    setError(null);
    void settle(api.spellKinds(), setKinds, setError);
    void settle(api.spellTemplates(), setTemplates, setError);
  }, [open]);

  const draft = detail?.card.summary.id ?? null;
  const shownBuffId = buffShown(buffChoice, baseBuffs);

  useEffect(() => {
    if (!shownBuffId) {
      setShownBuff(null);
      return;
    }

    void settle(api.plainDetail("buffs", shownBuffId), setShownBuff, setError);
  }, [shownBuffId]);

  async function discard(): Promise<Result<null>> {
    if (!draft) {
      return ok(null);
    }

    await queue.current;
    if (buffChoice.kind === "own") {
      const dropped = await api.discardPlain("buffs", buffChoice.id);

      if (!dropped.ok) {
        return dropped;
      }
    }

    const discarded = await api.discardSpell(draft);

    if (!discarded.ok) {
      return discarded;
    }

    setDetail(null);
    setBuffChoice(BASE_BUFF);
    setTemplate(null);
    onMapChanged();
    onUnitsChanged();
    return ok(null);
  }

  async function close() {
    if (step === "test" && draft) {
      done();
      return;
    }

    const discarded = await discard();

    if (!discarded.ok) {
      setError(discarded.error);
    }

    onClose();
  }

  async function back() {
    setError(null);
    if (step === "code") {
      setStep("units");
      return;
    }

    if (step === "units") {
      setStep("button");
      return;
    }

    if (step === "button" && template) {
      setStep("effect");
      return;
    }

    if (step === "button") {
      setStep("buff");
      return;
    }

    if (step === "buff") {
      setStep("values");
      return;
    }

    if (step === "effect" || step === "values") {
      const discarded = await discard();

      if (!discarded.ok) {
        setError(discarded.error);
        return;
      }

      let previous: Step = "base";

      if (step === "effect") {
        previous = "kind";
      }

      setStep(previous);
      return;
    }

    setStep("kind");
  }

  async function start(base: string) {
    setBusy(true);
    setError(null);
    const made = await copyBase(base);

    setBusy(false);
    if (!made.ok) {
      setError(made.error);
      return;
    }

    setDetail(made.value);
    setStep("values");
  }

  async function copyBase(base: string): Promise<Result<AbilityDetail>> {
    const created = await api.createAbility(base);

    if (!created.ok) {
      return created;
    }

    setBaseName(created.value.name);
    onMapChanged();
    const copied = await api.abilityDetail(created.value.id);

    if (!copied.ok) {
      return copied;
    }

    return ownValues(created.value.id, base, copied.value);
  }

  async function startTriggered(picked: SpellTemplate) {
    setBusy(true);
    setError(null);
    const created = await api.createTriggeredSpell(picked.id);

    if (!created.ok) {
      setError(created.error);
      setBusy(false);
      return;
    }

    onMapChanged();
    setBaseName("Channel");
    setTemplate(picked);
    setValues(defaultValues(picked));
    const made = await api.abilityDetail(created.value.id);

    setBusy(false);
    if (!made.ok) {
      setError(made.error);
      return;
    }

    setDetail(made.value);
    setStep("effect");
  }

  function setValue(param: string, list: number[]) {
    if (!template) {
      return;
    }

    const next = { ...values, [param]: list };

    setValues(next);
    edit((id) => api.setTriggeredValues(id, template.id, next));
  }

  async function toCode() {
    if (!(draft && template)) {
      return;
    }

    setBusy(true);
    setError(null);
    await queue.current;
    if (!folder) {
      const found = await api.defaultLuaFolder();

      if (found.ok) {
        setFolder(found.value ?? "");
      }
    }

    const module = await api.triggeredSpellLua(draft, template.id, values);

    setBusy(false);
    if (!module.ok) {
      setError(module.error);
      return;
    }

    setSource(module.value);
    setStep("code");
  }

  async function createTriggered() {
    if (!(draft && template)) {
      return;
    }

    if (!folder.trim()) {
      setError("Pick a folder for the Lua module.");
      return;
    }

    setBusy(true);
    setError(null);
    await queue.current;
    const written = await api.writeTriggeredSpellLua(draft, template.id, values, folder.trim());

    setBusy(false);
    if (!written.ok) {
      setError(written.error);
      return;
    }

    setModulePath(written.value);
    setStep("test");
  }

  function edit(change: (id: string) => Promise<Result<AbilityDetail> | null>) {
    if (!draft) {
      return;
    }

    queue.current = queue.current.then(async () => {
      const updated = await change(draft);

      if (updated && !updated.ok) {
        setError(updated.error);
        return;
      }

      if (updated) {
        setDetail(updated.value);
      }

      setError(null);
      onMapChanged();
    });
  }

  /**
   * Renames the spell, and its tooltip titles with it: they start as the base
   * ability's ("Storm Bolt - [Level 1]"), which would otherwise stay.
   */
  const rename = (name: string) =>
    edit(async (id) => {
      const before = detail;
      let updated = await api.setAbilityField(id, NAME, NO_LEVEL, name);
      const old = before && plainField(before, NAME)?.raw;
      if (!updated.ok || !before || !old || old === name) {
        return updated;
      }

      for (const fieldId of NAMED_TEXTS) {
        for (const { raw, level } of namedTexts(before, fieldId)) {
          if (raw.includes(old)) {
            updated = await api.setAbilityField(id, fieldId, level, raw.replaceAll(old, name));
            if (!updated.ok) {
              return updated;
            }
          }
        }
      }

      return updated;
    });

  const set = (field: string, level: number, value: string) =>
    edit((id) => api.setAbilityField(id, field, level, value));
  const reset = (field: string, level: number) =>
    edit((id) => api.resetAbilityField(id, field, level));
  const fill = (field: string, fromLevel: number, value: string) =>
    edit(async (id) => {
      const levels = detail?.levels ?? fromLevel;
      let updated = await api.setAbilityField(id, field, fromLevel, value);
      for (let level = fromLevel + 1; level <= levels && updated.ok; level++) {
        updated = await api.setAbilityField(id, field, level, value);
      }

      return updated;
    });

  async function toBuff() {
    if (!detail) {
      return;
    }

    setBusy(true);
    setError(null);
    await queue.current;
    const base = await api.abilityDetail(detail.card.summary.base);

    if (!base.ok) {
      setError(base.error);
      setBusy(false);
      return;
    }

    const buffs = await api.listPlain("buffs");

    setBusy(false);
    if (!buffs.ok) {
      setError(buffs.error);
      return;
    }

    const first = levelField(base.value, BUFFS)?.cells[0]?.raw ?? "";

    setBaseBuffs(idList(first));
    setAllBuffs(buffs.value);
    setStep("buff");
  }

  async function setBuffs(id: string, buffs: string | null): Promise<Result<AbilityDetail> | null> {
    let updated: Result<AbilityDetail> | null = null;

    for (let level = 1; level <= (detail?.levels ?? 1); level++) {
      if (buffs === null) {
        updated = await api.resetAbilityField(id, BUFFS, level);
      } else {
        updated = await api.setAbilityField(id, BUFFS, level, buffs);
      }

      if (!updated.ok) {
        return updated;
      }
    }

    return updated;
  }

  async function dropOwnBuff(): Promise<Result<void>> {
    if (buffChoice.kind === "own") {
      return api.discardPlain("buffs", buffChoice.id);
    }

    return ok(undefined);
  }

  const pickBaseBuff = () =>
    edit(async (id) => {
      const dropped = await dropOwnBuff();
      if (!dropped.ok) {
        return dropped;
      }

      setBuffChoice(BASE_BUFF);
      return setBuffs(id, null);
    });

  const pickOtherBuff = (buff: string) =>
    edit(async (id) => {
      const dropped = await dropOwnBuff();
      if (!dropped.ok) {
        return dropped;
      }

      const other: BuffChoice = { kind: "other", id: buff };
      setBuffChoice(other);
      return setBuffs(id, chosenBuffs(other, baseBuffs));
    });

  const pickOwnBuff = () =>
    edit(async (id) => {
      const [first] = baseBuffs;
      if (buffChoice.kind === "own" || !first || !detail) {
        return null;
      }

      const created = await api.createPlain("buffs", first);
      if (!created.ok) {
        return created;
      }

      const name = plainField(detail, NAME)?.raw ?? created.value.name;
      for (const field of BUFF_NAMES) {
        const named = await api.setPlainField("buffs", created.value.id, field, name);
        if (!named.ok) {
          return named;
        }
      }

      const own: BuffChoice = { kind: "own", id: created.value.id };
      setBuffChoice(own);
      return setBuffs(id, chosenBuffs(own, baseBuffs));
    });

  const editOwnBuff = (field: string, value: string) =>
    edit(async () => {
      if (buffChoice.kind !== "own") {
        return null;
      }

      const changed = await api.setPlainField("buffs", buffChoice.id, field, value);
      if (!changed.ok) {
        return changed;
      }

      const shown = await api.plainDetail("buffs", buffChoice.id);
      if (!shown.ok) {
        return shown;
      }

      setShownBuff(shown.value);
      return null;
    });

  async function toButton() {
    setBusy(true);
    setError(null);
    await queue.current;
    if (icons.length === 0) {
      const loaded = await api.spellIcons();

      if (!loaded.ok) {
        setError(loaded.error);
        setBusy(false);
        return;
      }

      setIcons(loaded.value);
    }

    setBusy(false);
    setStep("button");
  }

  const setIcon = (path: string) =>
    edit(async (id) => {
      let updated = await api.setAbilityField(id, ICON, NO_LEVEL, path);
      if (updated.ok && detail?.card.summary.hero) {
        updated = await api.setAbilityField(id, LEARN_ICON, NO_LEVEL, path);
      }

      return updated;
    });

  /**
   * Sets a hotkey, and moves its highlight in tooltips that show it, as older
   * tooltips do (`Storm Bolt (|cffffcc00T|r)`); Reforged's leave it out. A
   * hero spell's learn hotkey follows the cast hotkey when they were the same.
   */
  const setHotkey = (field: string, key: string) =>
    edit(async (id) => {
      const before = detail;
      let updated = await api.setAbilityField(id, field, NO_LEVEL, key);
      const old = before && plainField(before, field)?.raw;
      if (!updated.ok || !before || !old || old === key) {
        return updated;
      }

      const learn = plainField(before, LEARN_HOTKEY)?.raw;
      if (field === HOTKEY && before.card.summary.hero && learn === old) {
        updated = await api.setAbilityField(id, LEARN_HOTKEY, NO_LEVEL, key);
        if (!updated.ok) {
          return updated;
        }
      }

      const highlight = `(|cffffcc00${old}|r)`;
      for (const fieldId of HOTKEY_TEXTS) {
        for (const { raw, level } of namedTexts(before, fieldId)) {
          if (raw.includes(highlight)) {
            const moved = raw.replaceAll(highlight, `(|cffffcc00${key}|r)`);
            updated = await api.setAbilityField(id, fieldId, level, moved);
            if (!updated.ok) {
              return updated;
            }
          }
        }
      }

      return updated;
    });

  async function toUnits() {
    await queue.current;
    setStep("units");
  }

  async function finish() {
    if (!draft) {
      return;
    }

    await queue.current;
    const buffs = chosenBuffs(buffChoice, baseBuffs);

    if (buffs !== null) {
      const updated = await setBuffs(draft, buffs);

      if (updated && !updated.ok) {
        setError(updated.error);
        return;
      }
    }

    done();
  }

  function done() {
    if (!draft) {
      return;
    }

    const created = draft;

    setDetail(null);
    setTemplate(null);
    onCreated(created);
  }

  const footer = (
    <>
      {step !== "kind" && step !== "test" && (
        <Button onPress={back} disabled={busy}>
          <span className="flex items-center gap-1.5">
            <ArrowLeft className="size-4" aria-hidden="true" />
            Back
          </span>
        </Button>
      )}
      {step !== "test" && <Button onPress={close}>Cancel</Button>}
      {step === "values" && (
        <Button primary onPress={toBuff} disabled={busy}>
          Next: buff
        </Button>
      )}
      {step === "buff" && (
        <Button primary onPress={toButton} disabled={busy}>
          Next: button
        </Button>
      )}
      {step === "button" && (
        <Button primary onPress={toUnits} disabled={busy}>
          Next: units
        </Button>
      )}
      {step === "effect" && (
        <Button primary onPress={toButton} disabled={busy}>
          Next: button
        </Button>
      )}
      {step === "units" && template && (
        <Button primary onPress={toCode} disabled={busy}>
          Next: Lua code
        </Button>
      )}
      {step === "units" && !template && (
        <Button primary onPress={finish} disabled={busy}>
          Create spell
        </Button>
      )}
      {step === "code" && (
        <Button primary onPress={createTriggered} disabled={busy}>
          Create spell
        </Button>
      )}
      {step === "test" && (
        <Button primary onPress={done}>
          Done
        </Button>
      )}
    </>
  );

  return (
    <>
      <DialogShell
        open={open && !picking}
        title={STEP_TITLES[step]}
        page={step}
        dismissable={false}
        onClose={close}
        footer={footer}
        wide={step === "values" || step === "units" || step === "effect" || step === "code"}
      >
        {error && (
          <p role="alert" className="text-danger">
            {error}
          </p>
        )}
        {step === "kind" && (
          <>
            <h3 className="font-semibold text-gold-bright">Based on a game ability</h3>
            <KindStep
              kinds={kinds}
              onPick={(picked) => {
                setKind(picked);
                setStep("base");
              }}
            />
            <TemplateChoices templates={templates} busy={busy} onPick={startTriggered} />
          </>
        )}
        {step === "effect" && detail && template && (
          <EffectStep
            detail={detail}
            template={template}
            values={values}
            name={plainField(detail, NAME)}
            fields={triggeredFields(detail, template)}
            tooltip={levelField(detail, TOOLTIP_TEXT)}
            onRename={rename}
            onValue={setValue}
            onChange={set}
            onFill={fill}
            onReset={reset}
          />
        )}
        {step === "base" && kind && (
          <BaseStep kind={kind} busy={busy} onPick={start} onOther={() => setPicking(true)} />
        )}
        {step === "values" && detail && (
          <ValuesStep
            detail={detail}
            baseName={baseName}
            onRename={rename}
            onChange={set}
            onFill={fill}
            onReset={reset}
          />
        )}
        {step === "buff" && detail && (
          <SpellBuffStep
            baseName={baseName}
            baseBuffs={baseBuffs}
            choice={buffChoice}
            shown={shownBuff}
            allBuffs={allBuffs}
            busy={busy}
            onOwn={pickOwnBuff}
            onBase={pickBaseBuff}
            onOther={pickOtherBuff}
            onEditOwn={editOwnBuff}
          />
        )}
        {step === "button" && detail && (
          <SpellButtonStep
            detail={detail}
            icons={icons}
            onSet={(field, value) => set(field, NO_LEVEL, value)}
            onIcon={setIcon}
            onHotkey={setHotkey}
          />
        )}
        {step === "code" && <CodeStep folder={folder} source={source} onFolder={setFolder} />}
        {step === "test" && detail && (
          <TestStep
            name={detail.card.summary.name}
            modulePath={modulePath}
            folder={folder.trim()}
            onSaved={onMapChanged}
          />
        )}
        {step === "units" && draft && (
          <SpellUnits
            spell={draft}
            onChanged={() => {
              onMapChanged();
              onUnitsChanged();
            }}
          />
        )}
      </DialogShell>
      <AbilityPicker
        open={picking}
        heroOnly={false}
        exclude={[]}
        onPick={(id) => {
          setPicking(false);
          start(id);
        }}
        onClose={() => setPicking(false)}
      />
    </>
  );
}

function KindStep({ kinds, onPick }: { kinds: SpellKind[]; onPick: (kind: SpellKind) => void }) {
  return (
    <ul aria-label="Kinds of spell" className="grid grid-cols-2 gap-2">
      {kinds.map((kind) => (
        <li key={kind.id}>
          <button
            type="button"
            onClick={() => onPick(kind)}
            className="flex h-full w-full flex-col gap-0.5 rounded-[3px] border border-line bg-well px-3 py-2 text-left outline-none hover:border-gold focus-visible:border-focus"
          >
            <span className="font-semibold text-gold-bright">{kind.label}</span>
            <span className="text-sm text-muted">{kind.description}</span>
          </button>
        </li>
      ))}
    </ul>
  );
}

type BaseStepProps = {
  kind: SpellKind;
  busy: boolean;
  onPick: (base: string) => void;
  onOther: () => void;
};

function BaseStep({ kind, busy, onPick, onOther }: BaseStepProps) {
  return (
    <>
      <p className="text-muted">
        {kind.label}: start from one of these game abilities. Your spell is a copy you can change
        freely; the original stays as it is.
      </p>
      <ul aria-label="Suggested abilities" className="flex flex-col gap-2">
        {kind.bases.map((base) => (
          <li key={base.card.summary.id}>
            <BaseChoice base={base} busy={busy} onPick={onPick} />
          </li>
        ))}
      </ul>
      <div>
        <Button onPress={onOther} disabled={busy}>
          Another ability…
        </Button>
      </div>
    </>
  );
}

function BaseChoice({
  base,
  busy,
  onPick,
}: {
  base: SpellBase;
  busy: boolean;
  onPick: (base: string) => void;
}) {
  const { summary } = base.card;

  return (
    <button
      type="button"
      disabled={busy}
      onClick={() => onPick(summary.id)}
      className="flex w-full items-start gap-3 rounded-[3px] border border-line bg-well px-3 py-2 text-left outline-none hover:border-gold focus-visible:border-focus disabled:opacity-40"
    >
      <UnitIcon path={summary.icon} size="small" />
      <span className="min-w-0 flex-1">
        <span className="block font-semibold">
          <GameText text={summary.name} singleLine />
          {summary.suffix && (
            <span className="ml-1.5 font-normal text-faint">
              <GameText text={summary.suffix} singleLine />
            </span>
          )}
        </span>
        <span className="block text-sm text-muted">{base.why}</span>
        <span className="block text-xs text-faint">
          {spellType(summary.hero)} · {levelCount(summary.levels)} ·{" "}
          <span className="font-mono">{summary.id}</span>
        </span>
      </span>
    </button>
  );
}

type ValuesStepProps = {
  detail: AbilityDetail;
  baseName: string;
  onRename: (name: string) => void;
  onChange: (field: string, level: number, value: string) => void;
  onFill: (field: string, fromLevel: number, value: string) => void;
  onReset: (field: string, level: number) => void;
};

function ValuesStep({ detail, baseName, onRename, onChange, onFill, onReset }: ValuesStepProps) {
  const name = plainField(detail, NAME);
  const levels = plainField(detail, LEVELS);
  const tooltips = TOOLTIPS.map((id) => levelField(detail, id)).filter(
    (field): field is LevelField => field !== undefined,
  );
  const { summary } = detail.card;

  return (
    <PlaceholderValues.Provider value={detail}>
      <p className="text-sm text-muted">
        Based on <GameText text={baseName} singleLine /> ({summary.base}). Its icon, hotkey and
        everything else can be changed on its page afterwards.
      </p>
      <div className="flex flex-wrap gap-4">
        {name && <CommitInput label="Name" field={name} onCommit={onRename} />}
        {levels && (
          <CommitInput
            label="Levels"
            field={levels}
            numeric
            onCommit={(value) => onChange(LEVELS, NO_LEVEL, value)}
          />
        )}
      </div>
      <LevelTable
        fields={valueFields(detail)}
        editable
        onChange={onChange}
        onFill={onFill}
        onReset={onReset}
      />
      {tooltips.map((field) => (
        <LevelText
          key={field.field.id}
          field={field}
          editable
          onChange={(level, value) => onChange(field.field.id, level, value)}
          onReset={(level) => onReset(field.field.id, level)}
        />
      ))}
    </PlaceholderValues.Provider>
  );
}
