import { useCallback, useMemo } from "react";
import { api } from "../api";
import type { Result } from "../result";
import type { AbilityDetail, LeveledCatalog } from "../types";
import { AbilityPage } from "./AbilityPage";
import { fillLevels, useObjectPane } from "./objectPane";
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
  const source = useMemo(
    () => ({
      list: () => api.listLeveled(catalog),
      detail: (id: string) => api.leveledDetail(catalog, id),
      create: (base: string) => api.createLeveled(catalog, base),
    }),
    [catalog],
  );
  const pane = useObjectPane(source, { selected, onSelect, onMapChanged, onError });
  const { detail, setDetail } = pane;
  const upgrades = pane.items;

  const listError = useCallback((message: string) => onError(message), [onError]);

  function run(id: string, edit: () => Promise<Result<AbilityDetail>>) {
    pane.run(id, edit, async (updated) => setDetail(updated));
  }

  function fill(id: string, field: string, fromLevel: number, value: string) {
    const levels = detail?.levels ?? fromLevel;

    run(id, () =>
      fillLevels(
        (level) => api.setLeveledField(catalog, id, field, level, value),
        fromLevel,
        levels,
      ),
    );
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
            run(detail.card.summary.id, () =>
              api.setLeveledField(catalog, detail.card.summary.id, field, level, value),
            )
          }
          onFill={(field, level, value) => fill(detail.card.summary.id, field, level, value)}
          onReset={(field, level) =>
            run(detail.card.summary.id, () =>
              api.resetLeveledField(catalog, detail.card.summary.id, field, level),
            )
          }
          onCreateCopy={() => pane.createCopy(detail.card.summary.base)}
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
