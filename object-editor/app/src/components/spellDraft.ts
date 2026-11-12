import { useCallback, useEffect, useRef, useState } from "react";
import { api } from "../api";
import { failed, ok, type Result, settleCurrent } from "../result";
import type { AbilityDetail, SpellTemplate, TemplateValues, UnitDetail, UnitItem } from "../types";
import { useLuaFolder } from "./luaFolder";
import type { BuffChoice } from "./SpellBuffStep";
import { defaultValues } from "./SpellTriggered";
import {
  BUFF_NAMES,
  BUFFS,
  buffShown,
  chosenBuffs,
  filled,
  idList,
  levelField,
  NAME,
  plainField,
  renamed,
  setBuffs,
  withHotkey,
  withIcon,
} from "./spellEdits";

type ErrorHandler = (message: string | null) => void;

export type SpellDraft = {
  detail: AbilityDetail | null;
  setDetail: (detail: AbilityDetail | null) => void;
  draft: string | null;
  edit: (change: (id: string) => Promise<Result<AbilityDetail> | null>) => void;
  settled: () => Promise<void>;
  set: (field: string, level: number, value: string) => void;
  reset: (field: string, level: number) => void;
  fill: (field: string, fromLevel: number, value: string) => void;
  rename: (name: string) => void;
  setIcon: (path: string) => void;
  setHotkey: (field: string, key: string) => void;
};

export function useSpellDraft(onMapChanged: () => void, onError: ErrorHandler): SpellDraft {
  const [detail, setDetail] = useState<AbilityDetail | null>(null);
  const queue = useRef<Promise<void>>(Promise.resolve());
  const draft = detail?.card.summary.id ?? null;

  function edit(change: (id: string) => Promise<Result<AbilityDetail> | null>) {
    if (!draft) {
      return;
    }

    queue.current = queue.current.then(async () => {
      const updated = await change(draft);

      if (updated && !updated.ok) {
        onError(updated.error);
        return;
      }

      if (updated) {
        setDetail(updated.value);
      }

      onError(null);
      onMapChanged();
    });
  }

  return {
    detail,
    setDetail,
    draft,
    edit,
    settled: () => queue.current,
    set: (field, level, value) => edit((id) => api.setAbilityField(id, field, level, value)),
    reset: (field, level) => edit((id) => api.resetAbilityField(id, field, level)),
    fill: (field, fromLevel, value) =>
      edit((id) => filled(id, field, fromLevel, detail?.levels ?? fromLevel, value)),
    rename: (name) => edit((id) => renamed(id, detail, name)),
    setIcon: (path) => edit((id) => withIcon(id, path, detail?.card.summary.hero ?? false)),
    setHotkey: (field, key) => edit((id) => withHotkey(id, detail, field, key)),
  };
}

const BASE_BUFF: BuffChoice = { kind: "base" };

export type SpellBuff = ReturnType<typeof useSpellBuff>;

export function useSpellBuff(spell: SpellDraft, onError: ErrorHandler) {
  const [choice, setChoice] = useState<BuffChoice>(BASE_BUFF);
  const [baseBuffs, setBaseBuffs] = useState<string[]>([]);
  const [shown, setShown] = useState<UnitDetail | null>(null);
  const [all, setAll] = useState<UnitItem[]>([]);
  const shownId = buffShown(choice, baseBuffs);
  const levels = spell.detail?.levels ?? 1;
  const reset = useCallback(() => setChoice(BASE_BUFF), []);

  useEffect(() => {
    if (!shownId) {
      setShown(null);
      return;
    }

    return settleCurrent(api.plainDetail("buffs", shownId), setShown, onError);
  }, [shownId, onError]);

  async function load(base: string): Promise<Result<null>> {
    const ability = await api.abilityDetail(base);

    if (!ability.ok) {
      return ability;
    }

    const buffs = await api.listPlain("buffs");

    if (!buffs.ok) {
      return buffs;
    }

    setBaseBuffs(idList(levelField(ability.value, BUFFS)?.cells[0]?.raw ?? ""));
    setAll(buffs.value);
    return ok(null);
  }

  async function drop(): Promise<Result<void>> {
    if (choice.kind === "own") {
      return api.discardPlain("buffs", choice.id);
    }

    return ok(undefined);
  }

  function pick(next: BuffChoice) {
    spell.edit(async (id) => {
      const dropped = await drop();

      if (!dropped.ok) {
        return dropped;
      }

      setChoice(next);
      return setBuffs(id, chosenBuffs(next, baseBuffs), levels);
    });
  }

  function pickOwn() {
    const { detail } = spell;

    spell.edit(async (id) => {
      const [first] = baseBuffs;

      if (choice.kind === "own" || !first || !detail) {
        return null;
      }

      const created = await api.createPlain("buffs", first);

      if (!created.ok) {
        return created;
      }

      const name = plainField(detail, NAME)?.raw ?? created.value.name;

      for (const field of BUFF_NAMES) {
        const named = await api.setPlainField("buffs", created.value.id, field, name);

        if (!named.ok) {
          return named;
        }
      }

      const own: BuffChoice = { kind: "own", id: created.value.id };

      setChoice(own);
      return setBuffs(id, chosenBuffs(own, baseBuffs), levels);
    });
  }

  function editOwn(field: string, value: string) {
    spell.edit(async () => {
      if (choice.kind !== "own") {
        return null;
      }

      const changed = await api.setPlainField("buffs", choice.id, field, value);

      if (!changed.ok) {
        return changed;
      }

      const updated = await api.plainDetail("buffs", choice.id);

      if (!updated.ok) {
        return updated;
      }

      setShown(updated.value);
      return null;
    });
  }

  return {
    choice,
    baseBuffs,
    shown,
    all,
    load,
    drop,
    pickBase: () => pick(BASE_BUFF),
    pickOther: (buff: string) => pick({ kind: "other", id: buff }),
    pickOwn,
    editOwn,
    reset,
    apply: (id: string) => setBuffs(id, chosenBuffs(choice, baseBuffs), levels),
    chosen: () => chosenBuffs(choice, baseBuffs),
  };
}

export type TriggeredSpell = ReturnType<typeof useTriggeredSpell>;

export function useTriggeredSpell(spell: SpellDraft) {
  const [template, setTemplate] = useState<SpellTemplate | null>(null);
  const [values, setValues] = useState<TemplateValues>({});
  const [source, setSource] = useState("");
  const [modulePath, setModulePath] = useState("");
  const { folder, setFolder } = useLuaFolder();
  const clear = useCallback(() => setTemplate(null), []);

  async function begin(picked: SpellTemplate, onCreated: () => void): Promise<Result<null>> {
    const created = await api.createTriggeredSpell(picked.id);

    if (!created.ok) {
      return created;
    }

    onCreated();
    setTemplate(picked);
    setValues(defaultValues(picked));
    const made = await api.abilityDetail(created.value.id);

    if (!made.ok) {
      return made;
    }

    spell.setDetail(made.value);
    return ok(null);
  }

  function setValue(param: string, list: number[]) {
    if (!template) {
      return;
    }

    const next = { ...values, [param]: list };

    setValues(next);
    spell.edit((id) => api.setTriggeredValues(id, template.id, next));
  }

  async function module(): Promise<Result<null>> {
    if (!(spell.draft && template)) {
      return ok(null);
    }

    if (!folder) {
      const found = await api.defaultLuaFolder();

      if (found.ok) {
        setFolder(found.value ?? "");
      }
    }

    const made = await api.triggeredSpellLua(spell.draft, template.id, values);

    if (!made.ok) {
      return made;
    }

    setSource(made.value);
    return ok(null);
  }

  async function write(): Promise<Result<null>> {
    if (!(spell.draft && template)) {
      return ok(null);
    }

    if (!folder.trim()) {
      return failed("Pick a folder for the Lua module.");
    }

    const written = await api.writeTriggeredSpellLua(
      spell.draft,
      template.id,
      values,
      folder.trim(),
    );

    if (!written.ok) {
      return written;
    }

    setModulePath(written.value);
    return ok(null);
  }

  return {
    template,
    values,
    source,
    modulePath,
    folder,
    setFolder,
    begin,
    setValue,
    module,
    write,
    clear,
  };
}
