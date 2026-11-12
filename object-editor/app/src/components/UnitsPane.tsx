import { useEffect, useMemo, useState } from "react";
import { api } from "../api";
import type { Result } from "../result";
import type { Field, HotkeyClash, UnitDetail, UnitItem } from "../types";
import { replaceField } from "../ui";
import { OpenAbilityContext } from "./openAbility";
import { UNIT_LIST, UnitList } from "./UnitList";
import { UnitPage } from "./UnitPage";

type UnitsPaneProps = {
  units: UnitItem[];
  gameUnits: number;
  editable: boolean;
  selected: string | null;
  onSelect: (id: string) => void;
  onOpenAbility: (id: string) => void;
  onMapChanged: () => void;
  onUnitsChanged: () => void;
  onError: (message: string | null) => void;
};

export function UnitsPane({
  units,
  gameUnits,
  editable,
  selected,
  onSelect,
  onOpenAbility,
  onMapChanged,
  onUnitsChanged,
  onError,
}: UnitsPaneProps) {
  const [detail, setDetail] = useState<UnitDetail | null>(null);
  const [clashes, setClashes] = useState<HotkeyClash[]>([]);
  const item = useMemo(() => units.find((u) => u.id === selected), [units, selected]);

  useEffect(() => {
    if (!selected) {
      setDetail(null);
    }
  }, [selected]);

  useEffect(() => {
    if (!item) {
      return;
    }

    let current = true;

    void Promise.all([api.unitDetail(item.id), api.hotkeyClashes(item.id)]).then(
      ([loaded, found]) => {
        if (!current) {
          return;
        }

        if (!loaded.ok) {
          onError(loaded.error);
          return;
        }

        if (!found.ok) {
          onError(found.error);
          return;
        }

        setDetail(loaded.value);
        setClashes(found.value);
      },
    );

    return () => {
      current = false;
    };
  }, [item, onError]);

  async function applyField(edit: Promise<Result<Field>>) {
    const updated = await edit;

    if (!updated.ok) {
      onError(updated.error);
      return;
    }

    setDetail((current) => current && replaceField(current, updated.value));
    onError(null);
    onMapChanged();
    onUnitsChanged();
  }

  async function createCopy() {
    if (!detail) {
      return;
    }

    const created = await api.createUnit(detail.item.base);

    if (!created.ok) {
      onError(created.error);
      return;
    }

    onMapChanged();
    onUnitsChanged();
    onSelect(created.value.id);
  }

  return (
    <div className="flex min-h-0 flex-1 gap-2.5">
      <UnitList
        config={UNIT_LIST}
        units={units}
        selected={selected}
        onSelect={onSelect}
        onError={onError}
      />
      {detail && (
        <OpenAbilityContext value={onOpenAbility}>
          <UnitPage
            key={detail.item.id}
            detail={detail}
            clashes={clashes}
            editable={editable}
            onChange={(field, value) => applyField(api.setField(detail.item.id, field, value))}
            onReset={(field) => applyField(api.resetField(detail.item.id, field))}
            onCreateCopy={createCopy}
          />
        </OpenAbilityContext>
      )}
      {!detail && (
        <p className="m-auto text-muted">Pick a unit on the left. {gameUnits} game units loaded.</p>
      )}
    </div>
  );
}
