import { ArrowLeft } from "@untitledui/icons";
import { useEffect, useState } from "react";
import { api } from "../api";
import { ok, type Result, settle } from "../result";
import type { IconChoice, SpellKind, SpellTemplate } from "../types";
import { AbilityPicker } from "./AbilityPicker";
import { Button } from "./Button";
import { DialogShell } from "./DialogShell";
import { SpellStepBody, type Step } from "./SpellSteps";
import { useSpellBuff, useSpellDraft, useTriggeredSpell } from "./spellDraft";
import { copyBase } from "./spellEdits";

type SpellWizardProps = {
  open: boolean;
  onClose: () => void;
  onCreated: (id: string) => void;
  onMapChanged: () => void;
  onUnitsChanged: () => void;
};

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

const WIDE_STEPS: Step[] = ["values", "units", "effect", "code"];

const PREVIOUS: Partial<Record<Step, Step>> = {
  code: "units",
  units: "button",
  buff: "values",
  values: "base",
  effect: "kind",
};

function previousStep(step: Step, triggered: boolean): Step {
  if (step === "button" && triggered) {
    return "effect";
  }

  if (step === "button") {
    return "buff";
  }

  return PREVIOUS[step] ?? "kind";
}

type Action = { label: string; run: () => void };

type WizardFooterProps = {
  step: Step;
  busy: boolean;
  next: Action | undefined;
  onBack: () => void;
  onCancel: () => void;
};

function WizardFooter({ step, busy, next, onBack, onCancel }: WizardFooterProps) {
  return (
    <>
      {step !== "kind" && step !== "test" && (
        <Button onPress={onBack} disabled={busy}>
          <span className="flex items-center gap-1.5">
            <ArrowLeft className="size-4" aria-hidden="true" />
            Back
          </span>
        </Button>
      )}
      {step !== "test" && <Button onPress={onCancel}>Cancel</Button>}
      {next && (
        <Button primary onPress={next.run} disabled={busy}>
          {next.label}
        </Button>
      )}
    </>
  );
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
  const [picking, setPicking] = useState(false);
  const [baseName, setBaseName] = useState("");
  const [icons, setIcons] = useState<IconChoice[]>([]);
  const [templates, setTemplates] = useState<SpellTemplate[]>([]);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const spell = useSpellDraft(onMapChanged, setError);
  const buff = useSpellBuff(spell, setError);
  const triggered = useTriggeredSpell(spell);
  const { detail, draft, setDetail } = spell;
  const { reset: resetBuff } = buff;
  const { clear: clearTemplate, template } = triggered;

  useEffect(() => {
    if (!open) {
      return;
    }

    setStep("kind");
    setKind(null);
    setDetail(null);
    resetBuff();
    clearTemplate();
    setError(null);
    void settle(api.spellKinds(), setKinds, setError);
    void settle(api.spellTemplates(), setTemplates, setError);
  }, [open, setDetail, resetBuff, clearTemplate]);

  async function discard(): Promise<Result<null>> {
    if (!draft) {
      return ok(null);
    }

    await spell.settled();
    const dropped = await buff.drop();

    if (!dropped.ok) {
      return dropped;
    }

    const discarded = await api.discardSpell(draft);

    if (!discarded.ok) {
      return discarded;
    }

    setDetail(null);
    resetBuff();
    clearTemplate();
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
    if (step === "effect" || step === "values") {
      const discarded = await discard();

      if (!discarded.ok) {
        setError(discarded.error);
        return;
      }
    }

    setStep(previousStep(step, template !== null));
  }

  async function advance(next: Step, work: () => Promise<Result<unknown>>) {
    setBusy(true);
    setError(null);
    await spell.settled();
    const result = await work();

    setBusy(false);
    if (!result.ok) {
      setError(result.error);
      return;
    }

    setStep(next);
  }

  const start = (base: string) =>
    advance("values", async () => {
      const made = await copyBase(base, onMapChanged);

      if (made.ok) {
        setBaseName(made.value.name);
        setDetail(made.value.detail);
      }

      return made;
    });

  const startTriggered = (picked: SpellTemplate) =>
    advance("effect", () =>
      triggered.begin(picked, () => {
        onMapChanged();
        setBaseName("Channel");
      }),
    );

  const toBuff = () => {
    if (detail) {
      void advance("buff", () => buff.load(detail.card.summary.base));
    }
  };

  const toButton = () =>
    advance("button", async () => {
      if (icons.length > 0) {
        return ok(null);
      }

      const loaded = await api.spellIcons();

      if (loaded.ok) {
        setIcons(loaded.value);
      }

      return loaded;
    });

  async function finish() {
    if (!draft) {
      return;
    }

    await spell.settled();
    if (buff.chosen() !== null) {
      const updated = await buff.apply(draft);

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

    setDetail(null);
    clearTemplate();
    onCreated(draft);
  }

  let unitsNext: Action = { label: "Create spell", run: finish };

  if (template) {
    unitsNext = { label: "Next: Lua code", run: () => advance("code", triggered.module) };
  }

  const next: Partial<Record<Step, Action>> = {
    values: { label: "Next: buff", run: toBuff },
    buff: { label: "Next: button", run: toButton },
    effect: { label: "Next: button", run: toButton },
    button: { label: "Next: units", run: () => advance("units", async () => ok(null)) },
    units: unitsNext,
    code: { label: "Create spell", run: () => advance("test", triggered.write) },
    test: { label: "Done", run: done },
  };

  return (
    <>
      <DialogShell
        open={open && !picking}
        title={STEP_TITLES[step]}
        page={step}
        dismissable={false}
        onClose={close}
        footer={
          <WizardFooter step={step} busy={busy} next={next[step]} onBack={back} onCancel={close} />
        }
        wide={WIDE_STEPS.includes(step)}
      >
        <SpellStepBody
          step={step}
          error={error}
          kinds={kinds}
          kind={kind}
          templates={templates}
          icons={icons}
          baseName={baseName}
          busy={busy}
          spell={spell}
          buff={buff}
          triggered={triggered}
          onKind={(picked) => {
            setKind(picked);
            setStep("base");
          }}
          onTemplate={startTriggered}
          onBase={start}
          onOtherBase={() => setPicking(true)}
          onMapChanged={onMapChanged}
          onUnitsChanged={onUnitsChanged}
        />
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
