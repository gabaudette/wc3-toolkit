import type {
  AbilityDetail,
  IconChoice,
  LevelField,
  SpellBase,
  SpellKind,
  SpellTemplate,
} from "../types";
import { levelCount } from "../ui";
import { Button } from "./Button";
import { CommitInput } from "./CommitInput";
import { GameText } from "./GameText";
import { LevelTable } from "./LevelTable";
import { LevelText } from "./LevelText";
import { SpellBuffStep } from "./SpellBuffStep";
import { SpellButtonStep } from "./SpellButtonStep";
import { CodeStep, EffectStep, TemplateChoices, TestStep } from "./SpellTriggered";
import { SpellUnits } from "./SpellUnits";
import type { SpellBuff, SpellDraft, TriggeredSpell } from "./spellDraft";
import {
  LEVELS,
  levelField,
  NAME,
  NO_LEVEL,
  plainField,
  TOOLTIP_TEXT,
  TOOLTIPS,
  triggeredFields,
  valueFields,
} from "./spellEdits";
import { PlaceholderValues } from "./TextPreview";
import { UnitIcon } from "./UnitIcon";

export type Step =
  | "kind"
  | "base"
  | "values"
  | "buff"
  | "effect"
  | "button"
  | "units"
  | "code"
  | "test";

function spellType(hero: boolean): string {
  if (hero) {
    return "Hero spell";
  }

  return "Unit spell";
}

type KindStepProps = {
  kinds: SpellKind[];
  templates: SpellTemplate[];
  busy: boolean;
  onPick: (kind: SpellKind) => void;
  onTemplate: (template: SpellTemplate) => void;
};

export function KindStep({ kinds, templates, busy, onPick, onTemplate }: KindStepProps) {
  return (
    <>
      <h3 className="font-semibold text-gold-bright">Based on a game ability</h3>
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
      <TemplateChoices templates={templates} busy={busy} onPick={onTemplate} />
    </>
  );
}

type BaseStepProps = {
  kind: SpellKind;
  busy: boolean;
  onPick: (base: string) => void;
  onOther: () => void;
};

export function BaseStep({ kind, busy, onPick, onOther }: BaseStepProps) {
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

export function ValuesStep({
  detail,
  baseName,
  onRename,
  onChange,
  onFill,
  onReset,
}: ValuesStepProps) {
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

type SpellStepBodyProps = {
  step: Step;
  error: string | null;
  kinds: SpellKind[];
  kind: SpellKind | null;
  templates: SpellTemplate[];
  icons: IconChoice[];
  baseName: string;
  busy: boolean;
  spell: SpellDraft;
  buff: SpellBuff;
  triggered: TriggeredSpell;
  onKind: (kind: SpellKind) => void;
  onTemplate: (template: SpellTemplate) => void;
  onBase: (base: string) => void;
  onOtherBase: () => void;
  onMapChanged: () => void;
  onUnitsChanged: () => void;
};

export function SpellStepBody({
  step,
  error,
  kinds,
  kind,
  templates,
  icons,
  baseName,
  busy,
  spell,
  buff,
  triggered,
  onKind,
  onTemplate,
  onBase,
  onOtherBase,
  onMapChanged,
  onUnitsChanged,
}: SpellStepBodyProps) {
  const { detail, draft } = spell;
  const { template } = triggered;

  return (
    <>
      {error && (
        <p role="alert" className="text-danger">
          {error}
        </p>
      )}
      {step === "kind" && (
        <KindStep
          kinds={kinds}
          templates={templates}
          busy={busy}
          onPick={onKind}
          onTemplate={onTemplate}
        />
      )}
      {step === "effect" && detail && template && (
        <EffectStep
          detail={detail}
          template={template}
          values={triggered.values}
          name={plainField(detail, NAME)}
          fields={triggeredFields(detail, template)}
          tooltip={levelField(detail, TOOLTIP_TEXT)}
          onRename={spell.rename}
          onValue={triggered.setValue}
          onChange={spell.set}
          onFill={spell.fill}
          onReset={spell.reset}
        />
      )}
      {step === "base" && kind && (
        <BaseStep kind={kind} busy={busy} onPick={onBase} onOther={onOtherBase} />
      )}
      {step === "values" && detail && (
        <ValuesStep
          detail={detail}
          baseName={baseName}
          onRename={spell.rename}
          onChange={spell.set}
          onFill={spell.fill}
          onReset={spell.reset}
        />
      )}
      {step === "buff" && detail && (
        <SpellBuffStep
          baseName={baseName}
          baseBuffs={buff.baseBuffs}
          choice={buff.choice}
          shown={buff.shown}
          allBuffs={buff.all}
          busy={busy}
          onOwn={buff.pickOwn}
          onBase={buff.pickBase}
          onOther={buff.pickOther}
          onEditOwn={buff.editOwn}
        />
      )}
      {step === "button" && detail && (
        <SpellButtonStep
          detail={detail}
          icons={icons}
          onSet={(field, value) => spell.set(field, NO_LEVEL, value)}
          onIcon={spell.setIcon}
          onHotkey={spell.setHotkey}
        />
      )}
      {step === "code" && (
        <CodeStep
          folder={triggered.folder}
          source={triggered.source}
          onFolder={triggered.setFolder}
        />
      )}
      {step === "test" && detail && (
        <TestStep
          name={detail.card.summary.name}
          modulePath={triggered.modulePath}
          folder={triggered.folder.trim()}
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
    </>
  );
}
