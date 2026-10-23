import { useCallback, useEffect, useRef, useState } from "react";
import { api, errorMessage } from "../api";
import type { AbilityDetail, UnitItem } from "../types";
import { AbilityPage } from "./AbilityPage";
import { UnitList, UPGRADE_LIST } from "./UnitList";

type UpgradesPaneProps = {
  editable: boolean;
  selected: string | null;
  onSelect: (id: string) => void;
  /** Called after every change, so the window can show the map as unsaved. */
  onMapChanged: () => void;
  onError: (message: string | null) => void;
};

/** The Upgrades tab: the unit list over upgrades, and pages with a column per level. */
export function UpgradesPane({
  editable,
  selected,
  onSelect,
  onMapChanged,
  onError,
}: UpgradesPaneProps) {
  const [upgrades, setUpgrades] = useState<UnitItem[]>([]);
  const [detail, setDetail] = useState<AbilityDetail | null>(null);
  /** Edits run one after another, so a slow answer never overwrites a newer one. */
  const queue = useRef<Promise<void>>(Promise.resolve());

  const fail = useCallback((e: unknown) => onError(errorMessage(e)), [onError]);
  const listError = useCallback((message: string) => onError(message), [onError]);

  const refreshList = useCallback(() => {
    api.listUpgrades().then(setUpgrades).catch(fail);
  }, [fail]);

  useEffect(() => {
    refreshList();
  }, [refreshList]);

  useEffect(() => {
    if (!selected) {
      setDetail(null);
      return;
    }

    api.upgradeDetail(selected).then(setDetail).catch(fail);
  }, [selected, fail]);

  function run(edit: () => Promise<AbilityDetail | null>) {
    queue.current = queue.current.then(async () => {
      try {
        const updated = await edit();
        if (updated) {
          setDetail(updated);
        }

        onError(null);
        onMapChanged();
        refreshList();
      } catch (e) {
        fail(e);
      }
    });
  }

  function fill(id: string, field: string, fromLevel: number, value: string) {
    run(async () => {
      let updated: AbilityDetail | null = null;
      const levels = detail?.levels ?? fromLevel;
      for (let level = fromLevel; level <= levels; level++) {
        updated = await api.setUpgradeField(id, field, level, value);
      }

      return updated;
    });
  }

  async function createCopy() {
    if (!detail) {
      return;
    }

    try {
      const created = await api.createUpgrade(detail.card.summary.base);
      onMapChanged();
      refreshList();
      onSelect(created.id);
    } catch (e) {
      fail(e);
    }
  }

  return (
    <div className="flex min-h-0 flex-1 gap-2.5">
      <UnitList
        config={UPGRADE_LIST}
        units={upgrades}
        selected={selected}
        onSelect={onSelect}
        onError={listError}
      />
      {detail && (
        <AbilityPage
          key={detail.card.summary.id}
          detail={detail}
          noun="upgrade"
          editable={editable}
          onChange={(field, level, value) =>
            run(() => api.setUpgradeField(detail.card.summary.id, field, level, value))
          }
          onFill={(field, level, value) => fill(detail.card.summary.id, field, level, value)}
          onReset={(field, level) =>
            run(() => api.resetUpgradeField(detail.card.summary.id, field, level))
          }
          onCreateCopy={createCopy}
        />
      )}
      {!detail && (
        <p className="m-auto text-muted">
          Pick an upgrade on the left. {upgrades.length} upgrades loaded.
        </p>
      )}
    </div>
  );
}
