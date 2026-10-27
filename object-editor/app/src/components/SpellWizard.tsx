import { ArrowLeft } from "@untitledui/icons";
import { useEffect, useRef, useState } from "react";
import { api, errorMessage } from "../api";
import type {
  AbilityDetail,
  Field,
  IconChoice,
  LevelField,
  SpellBase,
  SpellKind,
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
import { SpellUnits } from "./SpellUnits";
import { PlaceholderValues } from "./TextPreview";
import { UnitIcon } from "./UnitIcon";

type SpellWizardProps = {
  open: boolean;
  onClose: () => void;
  /** The spell was created; its page opens. */
  onCreated: (id: string) => void;
  /** Called after every change, so the window can show the map as unsaved. */
  onMapChanged: () => void;
  /** The spell was given to a unit or taken away, or the units that had it lost it. */
  onUnitsChanged: () => void;
};

type Step = "kind" | "base" | "values" | "buff" | "button" | "units";

const NAME = "anam";
const LEVELS = "alev";
/** Values every spell has, in the order the values step shows them. */
const COMMON_VALUES = ["atar", "aran", "acdn", "amcs", "aare", "adur", "ahdu"];
/** The category holding a base ability's own values (damage, stun time, ...). */
const SPELL_DATA = "data";
/** The tooltip title and text, per level. */
const TOOLTIPS = ["atp1", "aub1"];
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
};

/** The fields the values step edits: the common ones first, then the base ability's own. */
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
async function ownValues(id: string, base: string, detail: AbilityDetail): Promise<AbilityDetail> {
  const reference = new RegExp(`<${base},`, "gi");
  let updated = detail;
  for (const fieldId of PLACEHOLDER_TEXTS) {
    for (const { raw, level } of namedTexts(detail, fieldId)) {
      const own = raw.replace(reference, `<${id},`);
      if (own !== raw) {
        updated = await api.setAbilityField(id, fieldId, level, own);
      }
    }
  }

  return updated;
}

/** A text field's values with their levels (0 for a field without levels). */
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

/** The IDs in a comma list such as `BPSE,BHbd`. */
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

/** The buff the step shows: the spell's own, the one picked, or the base's first. */
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

/**
 * Builds a spell step by step: what it does, the game ability it starts from,
 * then its values. The spell is a custom ability created when the base is
 * picked, so the values step edits the real thing with a live tooltip;
 * cancelling or going back removes it again.
 */
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
  /** The base ability's name: the spell's own name may have changed by now. */
  const [baseName, setBaseName] = useState("");
  const [shownBuff, setShownBuff] = useState<UnitDetail | null>(null);
  const [allBuffs, setAllBuffs] = useState<UnitItem[]>([]);
  const [icons, setIcons] = useState<IconChoice[]>([]);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  /** Edits run one after another, so a slow answer never overwrites a newer one. */
  const queue = useRef<Promise<void>>(Promise.resolve());

  useEffect(() => {
    if (!open) {
      return;
    }

    setStep("kind");
    setKind(null);
    setDetail(null);
    setBuffChoice(BASE_BUFF);
    setError(null);
    api
      .spellKinds()
      .then(setKinds)
      .catch((e: unknown) => setError(errorMessage(e)));
  }, [open]);

  const draft = detail?.card.summary.id ?? null;
  const shownBuffId = buffShown(buffChoice, baseBuffs);

  useEffect(() => {
    if (!shownBuffId) {
      setShownBuff(null);
      return;
    }

    api
      .plainDetail("buffs", shownBuffId)
      .then(setShownBuff)
      .catch((e: unknown) => setError(errorMessage(e)));
  }, [shownBuffId]);

  /** Removes the spell and its own buff, as if the wizard had never run. */
  async function discard() {
    if (!draft) {
      return;
    }

    await queue.current;
    if (buffChoice.kind === "own") {
      await api.discardPlain("buffs", buffChoice.id);
    }

    // Also takes the spell from any unit it was given to.
    await api.discardSpell(draft);
    setDetail(null);
    setBuffChoice(BASE_BUFF);
    onMapChanged();
    onUnitsChanged();
  }

  async function close() {
    try {
      await discard();
    } catch (e) {
      setError(errorMessage(e));
    }

    onClose();
  }

  async function back() {
    setError(null);
    try {
      if (step === "units") {
        setStep("button");
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

      if (step === "values") {
        await discard();
        setStep("base");
        return;
      }

      setStep("kind");
    } catch (e) {
      setError(errorMessage(e));
    }
  }

  async function start(base: string) {
    setBusy(true);
    setError(null);
    try {
      const created = await api.createAbility(base);
      // A new copy still has the base ability's name.
      setBaseName(created.name);
      onMapChanged();
      setDetail(await ownValues(created.id, base, await api.abilityDetail(created.id)));
      setStep("values");
    } catch (e) {
      setError(errorMessage(e));
    } finally {
      setBusy(false);
    }
  }

  function edit(change: (id: string) => Promise<AbilityDetail | null>) {
    if (!draft) {
      return;
    }

    queue.current = queue.current.then(async () => {
      try {
        const updated = await change(draft);
        if (updated) {
          setDetail(updated);
        }

        setError(null);
        onMapChanged();
      } catch (e) {
        setError(errorMessage(e));
      }
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
      if (!before || !old || old === name) {
        return updated;
      }

      for (const fieldId of NAMED_TEXTS) {
        for (const { raw, level } of namedTexts(before, fieldId)) {
          if (raw.includes(old)) {
            updated = await api.setAbilityField(id, fieldId, level, raw.replaceAll(old, name));
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
      let updated: AbilityDetail | null = null;
      for (let level = fromLevel; level <= (detail?.levels ?? fromLevel); level++) {
        updated = await api.setAbilityField(id, field, level, value);
      }

      return updated;
    });

  /** Shows the buff step, with the base ability's buffs and every buff to pick from. */
  async function toBuff() {
    if (!detail) {
      return;
    }

    setBusy(true);
    setError(null);
    try {
      await queue.current;
      const base = await api.abilityDetail(detail.card.summary.base);
      const first = levelField(base, BUFFS)?.cells[0]?.raw ?? "";
      setBaseBuffs(idList(first));
      setAllBuffs(await api.listPlain("buffs"));
      setStep("buff");
    } catch (e) {
      setError(errorMessage(e));
    } finally {
      setBusy(false);
    }
  }

  /** The spell's buffs at every level, or the base ability's again for `null`. */
  async function setBuffs(id: string, buffs: string | null): Promise<AbilityDetail | null> {
    let updated: AbilityDetail | null = null;
    for (let level = 1; level <= (detail?.levels ?? 1); level++) {
      if (buffs === null) {
        updated = await api.resetAbilityField(id, BUFFS, level);
      } else {
        updated = await api.setAbilityField(id, BUFFS, level, buffs);
      }
    }

    return updated;
  }

  /** Drops the spell's own buff when another choice replaces it. */
  async function dropOwnBuff() {
    if (buffChoice.kind === "own") {
      await api.discardPlain("buffs", buffChoice.id);
    }
  }

  const pickBaseBuff = () =>
    edit(async (id) => {
      await dropOwnBuff();
      setBuffChoice(BASE_BUFF);
      return setBuffs(id, null);
    });

  const pickOtherBuff = (buff: string) =>
    edit(async (id) => {
      await dropOwnBuff();
      const other: BuffChoice = { kind: "other", id: buff };
      setBuffChoice(other);
      return setBuffs(id, chosenBuffs(other, baseBuffs));
    });

  /** Copies the base ability's buff, names it after the spell, and puts it first. */
  const pickOwnBuff = () =>
    edit(async (id) => {
      const [first] = baseBuffs;
      if (buffChoice.kind === "own" || !first || !detail) {
        return null;
      }

      const created = await api.createPlain("buffs", first);
      const name = plainField(detail, NAME)?.raw ?? created.name;
      for (const field of BUFF_NAMES) {
        await api.setPlainField("buffs", created.id, field, name);
      }

      const own: BuffChoice = { kind: "own", id: created.id };
      setBuffChoice(own);
      return setBuffs(id, chosenBuffs(own, baseBuffs));
    });

  const editOwnBuff = (field: string, value: string) =>
    edit(async () => {
      if (buffChoice.kind !== "own") {
        return null;
      }

      await api.setPlainField("buffs", buffChoice.id, field, value);
      setShownBuff(await api.plainDetail("buffs", buffChoice.id));
      return null;
    });

  /** Shows the button step, with every icon to pick from. */
  async function toButton() {
    setBusy(true);
    setError(null);
    try {
      await queue.current;
      if (icons.length === 0) {
        setIcons(await api.spellIcons());
      }

      setStep("button");
    } catch (e) {
      setError(errorMessage(e));
    } finally {
      setBusy(false);
    }
  }

  /** Sets the button's icon, and a hero spell's learn icon with it. */
  const setIcon = (path: string) =>
    edit(async (id) => {
      let updated = await api.setAbilityField(id, ICON, NO_LEVEL, path);
      if (detail?.card.summary.hero) {
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
      if (!before || !old || old === key) {
        return updated;
      }

      const learn = plainField(before, LEARN_HOTKEY)?.raw;
      if (field === HOTKEY && before.card.summary.hero && learn === old) {
        updated = await api.setAbilityField(id, LEARN_HOTKEY, NO_LEVEL, key);
      }

      const highlight = `(|cffffcc00${old}|r)`;
      for (const fieldId of HOTKEY_TEXTS) {
        for (const { raw, level } of namedTexts(before, fieldId)) {
          if (raw.includes(highlight)) {
            const moved = raw.replaceAll(highlight, `(|cffffcc00${key}|r)`);
            updated = await api.setAbilityField(id, fieldId, level, moved);
          }
        }
      }

      return updated;
    });

  /** Shows the units step, once the button's last edit is in, so its warnings use it. */
  async function toUnits() {
    await queue.current;
    setStep("units");
  }

  async function finish() {
    if (!draft) {
      return;
    }

    await queue.current;
    // Levels may have been added after the buff was picked; they get it too.
    const buffs = chosenBuffs(buffChoice, baseBuffs);
    if (buffs !== null) {
      await setBuffs(draft, buffs);
    }

    const created = draft;
    setDetail(null);
    onCreated(created);
  }

  const footer = (
    <>
      {step !== "kind" && (
        <Button onPress={back} disabled={busy}>
          <span className="flex items-center gap-1.5">
            <ArrowLeft className="size-4" aria-hidden="true" />
            Back
          </span>
        </Button>
      )}
      <Button onPress={close}>Cancel</Button>
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
      {step === "units" && (
        <Button primary onPress={finish} disabled={busy}>
          Create spell
        </Button>
      )}
    </>
  );

  return (
    <>
      <DialogShell
        open={open && !picking}
        title={STEP_TITLES[step]}
        onClose={close}
        footer={footer}
        wide={step === "values" || step === "units"}
      >
        {error && (
          <p role="alert" className="text-danger">
            {error}
          </p>
        )}
        {step === "kind" && (
          <KindStep
            kinds={kinds}
            onPick={(picked) => {
              setKind(picked);
              setStep("base");
            }}
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
            className="flex h-full w-full flex-col gap-0.5 rounded-[3px] border border-line bg-well px-3 py-2 text-left outline-none hover:border-gold focus-visible:border-glow"
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
      className="flex w-full items-start gap-3 rounded-[3px] border border-line bg-well px-3 py-2 text-left outline-none hover:border-gold focus-visible:border-glow disabled:opacity-40"
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
  /** The base ability's name: the spell's own name may have changed. */
  baseName: string;
  /** Renames the spell, and the tooltip titles that repeat its name. */
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
