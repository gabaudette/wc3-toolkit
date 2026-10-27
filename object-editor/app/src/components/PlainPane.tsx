import { useCallback, useEffect, useRef, useState } from "react";
import { api, errorMessage } from "../api";
import type { Catalog, Field, UnitDetail, UnitItem } from "../types";
import { replaceField } from "../ui";
import { type ListConfig, UnitList } from "./UnitList";
import { UnitPage } from "./UnitPage";

type PlainPaneProps = {
  /** Which objects: items, buffs or destructables. */
  catalog: Catalog;
  list: ListConfig;
  editable: boolean;
  selected: string | null;
  onSelect: (id: string) => void;
  /** Called after every change, so the window can show the map as unsaved. */
  onMapChanged: () => void;
  onError: (message: string | null) => void;
};

/** A tab of objects without levels (items, buffs, destructables): the unit list and page over them. */
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
  /** Edits run one after another, so a slow answer never overwrites a newer one. */
  const queue = useRef<Promise<void>>(Promise.resolve());

  const fail = useCallback((e: unknown) => onError(errorMessage(e)), [onError]);
  const listError = useCallback((message: string) => onError(message), [onError]);

  const refreshList = useCallback(() => {
    api.listPlain(catalog).then(setObjects).catch(fail);
  }, [catalog, fail]);

  useEffect(() => {
    refreshList();
  }, [refreshList]);

  useEffect(() => {
    if (!selected) {
      setDetail(null);
      return;
    }

    api.plainDetail(catalog, selected).then(setDetail).catch(fail);
  }, [catalog, selected, fail]);

  function run(id: string, edit: () => Promise<Field>) {
    queue.current = queue.current.then(async () => {
      try {
        const updated = await edit();

        setDetail((current) => current && replaceField(current, updated));
        onError(null);
        onMapChanged();
        refreshList();
        // The summary and header can depend on the field, so fetch the page too.
        setDetail(await api.plainDetail(catalog, id));
      } catch (e) {
        fail(e);
      }
    });
  }

  async function createCopy() {
    if (!detail) {
      return;
    }

    try {
      const created = await api.createPlain(catalog, detail.item.base);

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
