import { useCallback, useMemo } from "react";
import { api } from "../api";
import type { Result } from "../result";
import type { Catalog, Field } from "../types";
import { replaceField } from "../ui";
import { useObjectPane } from "./objectPane";
import { type ListConfig, UnitList } from "./UnitList";
import { UnitPage } from "./UnitPage";

type PlainPaneProps = {
  catalog: Catalog;
  list: ListConfig;
  editable: boolean;
  selected: string | null;
  onSelect: (id: string) => void;
  onMapChanged: () => void;
  onError: (message: string | null) => void;
};

export function PlainPane({
  catalog,
  list,
  editable,
  selected,
  onSelect,
  onMapChanged,
  onError,
}: PlainPaneProps) {
  const source = useMemo(
    () => ({
      list: () => api.listPlain(catalog),
      detail: (id: string) => api.plainDetail(catalog, id),
      create: (base: string) => api.createPlain(catalog, base),
    }),
    [catalog],
  );
  const pane = useObjectPane(source, { selected, onSelect, onMapChanged, onError });
  const { detail, setDetail } = pane;
  const objects = pane.items;

  const listError = useCallback((message: string) => onError(message), [onError]);

  function run(id: string, edit: () => Promise<Result<Field>>) {
    pane.run(id, edit, (field) => {
      setDetail((current) => current && replaceField(current, field));
      return pane.reload(id);
    });
  }

  return (
    <div className="flex min-h-0 flex-1 gap-2.5">
      <UnitList
        config={list}
        units={objects}
        selected={selected}
        onSelect={onSelect}
        onError={listError}
      />
      {detail && (
        <UnitPage
          key={detail.item.id}
          detail={detail}
          editable={editable}
          onChange={(field, value) =>
            run(detail.item.id, () => api.setPlainField(catalog, detail.item.id, field, value))
          }
          onReset={(field) =>
            run(detail.item.id, () => api.resetPlainField(catalog, detail.item.id, field))
          }
          onCreateCopy={() => pane.createCopy(detail.item.base)}
        />
      )}
      {!detail && (
        <p className="m-auto text-muted">
          Pick {list.article} {list.singular} on the left. {objects.length} {list.noun} loaded.
        </p>
      )}
    </div>
  );
}
