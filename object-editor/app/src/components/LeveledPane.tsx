import { useCallback, useEffect, useRef, useState } from "react";
import { api } from "../api";
import { type Result, settle } from "../result";
import type { AbilityDetail, LeveledCatalog, UnitItem } from "../types";
import { AbilityPage } from "./AbilityPage";
import { type ListConfig, UnitList } from "./UnitList";

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
  onMapChanged: () => void;
  onError: (message: string | null) => void;
};

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
  const queue = useRef<Promise<void>>(Promise.resolve());

  const listError = useCallback((message: string) => onError(message), [onError]);

  const refreshList = useCallback(() => {
    void settle(api.listLeveled(catalog), setUpgrades, onError);
  }, [catalog, onError]);

  useEffect(() => {
    refreshList();
  }, [refreshList]);

  useEffect(() => {
    if (!selected) {
      setDetail(null);
      return;
    }

    void settle(api.leveledDetail(catalog, selected), setDetail, onError);
  }, [catalog, selected, onError]);

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

  function fill(id: string, field: string, fromLevel: number, value: string) {
    run(async () => {
      const levels = detail?.levels ?? fromLevel;
      let updated = await api.setLeveledField(catalog, id, field, fromLevel, value);

      for (let level = fromLevel + 1; level <= levels && updated.ok; level++) {
        updated = await api.setLeveledField(catalog, id, field, level, value);
      }

      return updated;
    });
  }

  async function createCopy() {
    if (!detail) {
      return;
    }

    const created = await api.createLeveled(catalog, detail.card.summary.base);

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
