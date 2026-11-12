import { useCallback, useEffect, useRef, useState } from "react";
import { type Result, settle, settleCurrent } from "../result";

export type ObjectSource<Item, Detail> = {
  list: () => Promise<Result<Item[]>>;
  detail: (id: string) => Promise<Result<Detail>>;
  create: (base: string) => Promise<Result<{ id: string }>>;
};

type ObjectPaneOptions = {
  selected: string | null;
  onSelect: (id: string) => void;
  onMapChanged: () => void;
  onError: (message: string | null) => void;
};

export async function fillLevels<T>(
  set: (level: number) => Promise<Result<T>>,
  fromLevel: number,
  levels: number,
): Promise<Result<T>> {
  let updated = await set(fromLevel);

  for (let level = fromLevel + 1; level <= levels && updated.ok; level++) {
    updated = await set(level);
  }

  return updated;
}

/**
 * The list, the selected object's detail and the edit queue of a pane that
 * edits one object type. `source` must keep its identity between renders.
 */
export function useObjectPane<Item, Detail>(
  source: ObjectSource<Item, Detail>,
  { selected, onSelect, onMapChanged, onError }: ObjectPaneOptions,
) {
  const [items, setItems] = useState<Item[]>([]);
  const [detail, setDetail] = useState<Detail | null>(null);
  const queue = useRef<Promise<void>>(Promise.resolve());
  const shown = useRef(selected);
  const { list, detail: load, create } = source;

  shown.current = selected;

  const refreshList = useCallback(() => {
    void settle(list(), setItems, onError);
  }, [list, onError]);

  useEffect(() => {
    refreshList();
  }, [refreshList]);

  useEffect(() => {
    if (!selected) {
      setDetail(null);
      return;
    }

    return settleCurrent(load(selected), setDetail, onError);
  }, [load, selected, onError]);

  /**
   * Queues an edit of object `id`; `apply` shows its result unless another
   * object was picked while the edit waited or ran.
   */
  function run<T>(id: string, edit: () => Promise<Result<T>>, apply: (value: T) => Promise<void>) {
    queue.current = queue.current.then(async () => {
      const updated = await edit();

      if (!updated.ok) {
        onError(updated.error);
        return;
      }

      if (shown.current === id) {
        await apply(updated.value);
      }

      onError(null);
      onMapChanged();
      refreshList();
    });
  }

  async function reload(id: string) {
    await settle(
      load(id),
      (loaded) => {
        if (shown.current === id) {
          setDetail(loaded);
        }
      },
      onError,
    );
  }

  async function createCopy(base: string) {
    const created = await create(base);

    if (!created.ok) {
      onError(created.error);
      return;
    }

    onMapChanged();
    refreshList();
    onSelect(created.value.id);
  }

  return { items, detail, setDetail, refreshList, run, reload, createCopy };
}
