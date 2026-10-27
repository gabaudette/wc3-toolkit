import { useCallback, useEffect, useRef, useState } from "react";
import { api } from "../api";
import { type Result, settle } from "../result";
import type { AbilityDetail, AbilitySummary } from "../types";
import { AbilityList } from "./AbilityList";
import { AbilityPage } from "./AbilityPage";
import { Button } from "./Button";
import { DialogShell } from "./DialogShell";
import { plainGameText } from "./GameText";
import { SpellUnits } from "./SpellUnits";
import { SpellWizard } from "./SpellWizard";

type AbilitiesPaneProps = {
  editable: boolean;
  selected: string | null;
  onSelect: (id: string) => void;
  onMapChanged: () => void;
  onUnitsChanged: () => void;
  onError: (message: string | null) => void;
};

export function AbilitiesPane({
  editable,
  selected,
  onSelect,
  onMapChanged,
  onUnitsChanged,
  onError,
}: AbilitiesPaneProps) {
  const [abilities, setAbilities] = useState<AbilitySummary[]>([]);
  const [detail, setDetail] = useState<AbilityDetail | null>(null);
  const [wizardOpen, setWizardOpen] = useState(false);
  const [givingOpen, setGivingOpen] = useState(false);
  const queue = useRef<Promise<void>>(Promise.resolve());

  const listError = useCallback((message: string) => onError(message), [onError]);

  const refreshList = useCallback(() => {
    void settle(api.listAbilities(), setAbilities, onError);
  }, [onError]);

  useEffect(() => {
    refreshList();
  }, [refreshList]);

  useEffect(() => {
    if (!selected) {
      setDetail(null);
      return;
    }

    void settle(api.abilityDetail(selected), setDetail, onError);
  }, [selected, onError]);

  function run(edit: () => Promise<Result<AbilityDetail>>) {
    queue.current = queue.current.then(async () => {
      const updated = await edit();

      if (!updated.ok) {
        onError(updated.error);
        return;
      }

      setDetail(updated.value);
      onError(null);
      onMapChanged();
      refreshList();
    });
  }

  function change(id: string, field: string, level: number, value: string) {
    run(() => api.setAbilityField(id, field, level, value));
  }

  function fill(id: string, field: string, fromLevel: number, value: string) {
    run(async () => {
      const levels = detail?.levels ?? fromLevel;
      let updated = await api.setAbilityField(id, field, fromLevel, value);

      for (let level = fromLevel + 1; level <= levels && updated.ok; level++) {
        updated = await api.setAbilityField(id, field, level, value);
      }

      return updated;
    });
  }

  function reset(id: string, field: string, level: number) {
    run(() => api.resetAbilityField(id, field, level));
  }

  async function createCopy() {
    if (!detail) {
      return;
    }

    const created = await api.createAbility(detail.card.summary.base);

    if (!created.ok) {
      onError(created.error);
      return;
    }

    onMapChanged();
    refreshList();
    onSelect(created.value.id);
  }

  return (
    <div className="flex min-h-0 flex-1 gap-2.5">
      <AbilityList
        abilities={abilities}
        selected={selected}
        onSelect={onSelect}
        onError={listError}
        {...(editable && { onNewSpell: () => setWizardOpen(true) })}
      />
      {wizardOpen && (
        <SpellWizard
          open={wizardOpen}
          onClose={() => setWizardOpen(false)}
          onCreated={(id) => {
            setWizardOpen(false);
            refreshList();
            onSelect(id);
          }}
          onMapChanged={onMapChanged}
          onUnitsChanged={onUnitsChanged}
        />
      )}
      {givingOpen && detail && (
        <DialogShell
          open={givingOpen}
          title={`Units with ${plainGameText(detail.card.summary.name)}`}
          onClose={() => setGivingOpen(false)}
          footer={
            <Button primary onPress={() => setGivingOpen(false)}>
              Done
            </Button>
          }
          wide
        >
          <SpellUnits
            spell={detail.card.summary.id}
            onChanged={() => {
              onMapChanged();
              onUnitsChanged();
            }}
          />
        </DialogShell>
      )}
      {detail && (
        <AbilityPage
          key={detail.card.summary.id}
          detail={detail}
          editable={editable}
          onChange={(field, level, value) => change(detail.card.summary.id, field, level, value)}
          onFill={(field, level, value) => fill(detail.card.summary.id, field, level, value)}
          onReset={(field, level) => reset(detail.card.summary.id, field, level)}
          onCreateCopy={createCopy}
          onGiveToUnits={() => setGivingOpen(true)}
        />
      )}
      {!detail && (
        <p className="m-auto text-muted">
          Pick an ability on the left. {abilities.length} abilities loaded.
        </p>
      )}
    </div>
  );
}
