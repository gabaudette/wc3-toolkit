import { useCallback, useEffect, useRef, useState } from "react";
import { api } from "../api";
import { type Result, settle } from "../result";
import type { Catalog, Field, UnitDetail, UnitItem } from "../types";
import { replaceField } from "../ui";
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
  const [objects, setObjects] = useState<UnitItem[]>([]);
  const [detail, setDetail] = useState<UnitDetail | null>(null);
  const queue = useRef<Promise<void>>(Promise.resolve());

  const listError = useCallback((message: string) => onError(message), [onError]);

  const refreshList = useCallback(() => {
    void settle(api.listPlain(catalog), setObjects, onError);
  }, [catalog, onError]);

  useEffect(() => {
    refreshList();
  }, [refreshList]);

  useEffect(() => {
    if (!selected) {
      setDetail(null);
      return;
    }

    void settle(api.plainDetail(catalog, selected), setDetail, onError);
  }, [catalog, selected, onError]);

  function run(id: string, edit: () => Promise<Result<Field>>) {
    queue.current = queue.current.then(async () => {
      const updated = await edit();

      if (!updated.ok) {
        onError(updated.error);
        return;
      }

      setDetail((current) => current && replaceField(current, updated.value));
      onError(null);
      onMapChanged();
      refreshList();
      await settle(api.plainDetail(catalog, id), setDetail, onError);
    });
  }

  async function createCopy() {
    if (!detail) {
      return;
    }

    const created = await api.createPlain(catalog, detail.item.base);

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
          onCreateCopy={createCopy}
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
