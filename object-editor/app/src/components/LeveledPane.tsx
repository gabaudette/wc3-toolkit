import { useCallback, useEffect, useRef, useState } from "react";
import { api, errorMessage } from "../api";
import type { AbilityDetail, LeveledCatalog, UnitItem } from "../types";
import { AbilityPage } from "./AbilityPage";
import { type ListConfig, UnitList } from "./UnitList";

/** What the ability page calls each catalog's objects. */
const NOUNS: Record<LeveledCatalog, "upgrade" | "doodad"> = {
  upgrades: "upgrade",
  doodads: "doodad",
};

type LeveledPaneProps = {
  catalog: LeveledCatalog;
  list: ListConfig;
  editable: boolean;
  selected: string | null;
  onSelect: (id: string) => void;
  /** Called after every change, so the window can show the map as unsaved. */
  onMapChanged: () => void;
  onError: (message: string | null) => void;
};

/** The Upgrades and Doodads tabs: the unit list over them, and pages with a column per level or variation. */
export function LeveledPane({
  catalog,
  list,
  editable,
  selected,
  onSelect,
  onMapChanged,
  onError,
}: LeveledPaneProps) {
  const [upgrades, setUpgrades] = useState<UnitItem[]>([]);
  const [detail, setDetail] = useState<AbilityDetail | null>(null);
  /** Edits run one after another, so a slow answer never overwrites a newer one. */
  const queue = useRef<Promise<void>>(Promise.resolve());

  const fail = useCallback((e: unknown) => onError(errorMessage(e)), [onError]);
  const listError = useCallback((message: string) => onError(message), [onError]);

  const refreshList = useCallback(() => {
    api.listLeveled(catalog).then(setUpgrades).catch(fail);
  }, [catalog, fail]);

  useEffect(() => {
    refreshList();
  }, [refreshList]);

  useEffect(() => {
    if (!selected) {
      setDetail(null);
      return;
    }

    api.leveledDetail(catalog, selected).then(setDetail).catch(fail);
  }, [catalog, selected, fail]);

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
        updated = await api.setLeveledField(catalog, id, field, level, value);
      }

      return updated;
    });
  }

  async function createCopy() {
    if (!detail) {
      return;
    }

    try {
      const created = await api.createLeveled(catalog, detail.card.summary.base);

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
        config={list}
        units={upgrades}
        selected={selected}
        onSelect={onSelect}
        onError={listError}
      />
      {detail && (
        <AbilityPage
          key={detail.card.summary.id}
          detail={detail}
          noun={NOUNS[catalog]}
          editable={editable}
          onChange={(field, level, value) =>
            run(() => api.setLeveledField(catalog, detail.card.summary.id, field, level, value))
          }
          onFill={(field, level, value) => fill(detail.card.summary.id, field, level, value)}
          onReset={(field, level) =>
            run(() => api.resetLeveledField(catalog, detail.card.summary.id, field, level))
          }
          onCreateCopy={createCopy}
        />
      )}
      {!detail && (
        <p className="m-auto text-muted">
          Pick {list.article} {list.singular} on the left. {upgrades.length} {list.noun} loaded.
        </p>
      )}
    </div>
  );
}
