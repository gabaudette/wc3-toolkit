import { useCallback, useState } from "react";
import { api } from "../api";
import type { Result } from "../result";
import type { AbilityDetail } from "../types";
import { AbilityList } from "./AbilityList";
import { AbilityPage } from "./AbilityPage";
import { Button } from "./Button";
import { DialogShell } from "./DialogShell";
import { plainGameText } from "./GameText";
import { fillLevels, useObjectPane } from "./objectPane";
import { SpellUnits } from "./SpellUnits";
import { SpellWizard } from "./SpellWizard";

const ABILITIES = {
  list: api.listAbilities,
  detail: api.abilityDetail,
  create: api.createAbility,
};

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
  const [wizardOpen, setWizardOpen] = useState(false);
  const [givingOpen, setGivingOpen] = useState(false);
  const pane = useObjectPane(ABILITIES, { selected, onSelect, onMapChanged, onError });
  const { detail, setDetail, refreshList } = pane;
  const abilities = pane.items;

  const listError = useCallback((message: string) => onError(message), [onError]);
  const openWizard = useCallback(() => setWizardOpen(true), []);

  function run(id: string, edit: () => Promise<Result<AbilityDetail>>) {
    pane.run(id, edit, async (updated) => setDetail(updated));
  }

  function change(id: string, field: string, level: number, value: string) {
    run(id, () => api.setAbilityField(id, field, level, value));
  }

  function fill(id: string, field: string, fromLevel: number, value: string) {
    const levels = detail?.levels ?? fromLevel;

    run(id, () =>
      fillLevels((level) => api.setAbilityField(id, field, level, value), fromLevel, levels),
    );
  }

  function reset(id: string, field: string, level: number) {
    run(id, () => api.resetAbilityField(id, field, level));
  }

  return (
    <div className="flex min-h-0 flex-1 gap-2.5">
      <AbilityList
        abilities={abilities}
        selected={selected}
        onSelect={onSelect}
        onError={listError}
        {...(editable && { onNewSpell: openWizard })}
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
          onCreateCopy={() => pane.createCopy(detail.card.summary.base)}
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
