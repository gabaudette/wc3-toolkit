import { useCallback, useEffect, useRef, useState } from "react";
import { api, errorMessage } from "../api";
import type { Field, UnitDetail, UnitItem } from "../types";
import { replaceField } from "../ui";
import { ITEM_LIST, UnitList } from "./UnitList";
import { UnitPage } from "./UnitPage";

type ItemsPaneProps = {
  editable: boolean;
  selected: string | null;
  onSelect: (id: string) => void;
  /** Called after every change, so the window can show the map as unsaved. */
  onMapChanged: () => void;
  onError: (message: string | null) => void;
};

/** The Items tab: the unit list and page over the item data. */
export function ItemsPane({ editable, selected, onSelect, onMapChanged, onError }: ItemsPaneProps) {
  const [items, setItems] = useState<UnitItem[]>([]);
  const [detail, setDetail] = useState<UnitDetail | null>(null);
  /** Edits run one after another, so a slow answer never overwrites a newer one. */
  const queue = useRef<Promise<void>>(Promise.resolve());

  const fail = useCallback((e: unknown) => onError(errorMessage(e)), [onError]);
  const listError = useCallback((message: string) => onError(message), [onError]);

  const refreshList = useCallback(() => {
    api.listItems().then(setItems).catch(fail);
  }, [fail]);

  useEffect(() => {
    refreshList();
  }, [refreshList]);

  useEffect(() => {
    if (!selected) {
      setDetail(null);
      return;
    }

    api.itemDetail(selected).then(setDetail).catch(fail);
  }, [selected, fail]);

  function run(id: string, edit: () => Promise<Field>) {
    queue.current = queue.current.then(async () => {
      try {
        const updated = await edit();
        setDetail((current) => current && replaceField(current, updated));
        onError(null);
        onMapChanged();
        refreshList();
        // The summary and header can depend on the field, so fetch the page too.
        setDetail(await api.itemDetail(id));
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
      const created = await api.createItem(detail.item.base);
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
        config={ITEM_LIST}
        units={items}
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
            run(detail.item.id, () => api.setItemField(detail.item.id, field, value))
          }
          onReset={(field) => run(detail.item.id, () => api.resetItemField(detail.item.id, field))}
          onCreateCopy={createCopy}
        />
      )}
      {!detail && (
        <p className="m-auto text-muted">Pick an item on the left. {items.length} items loaded.</p>
      )}
    </div>
  );
}
