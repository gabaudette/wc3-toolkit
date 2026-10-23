import { useEffect, useMemo, useState } from "react";
import { api, errorMessage } from "../api";
import type { AbilitySummary, SearchHit } from "../types";
import { classes } from "../ui";
import { Chip, toggle } from "./Chip";
import { GameText } from "./GameText";
import { ListFold, useFolded } from "./ListFold";
import { UnitIcon } from "./UnitIcon";

type AbilityListProps = {
  abilities: AbilitySummary[];
  selected: string | null;
  onSelect: (id: string) => void;
  onError: (message: string) => void;
};

type Kind = "hero" | "unit" | "item";
type Origin = "map" | "game";

const SEARCH_DELAY_MS = 200;

const KINDS: { id: Kind; label: string }[] = [
  { id: "hero", label: "Hero" },
  { id: "unit", label: "Unit" },
  { id: "item", label: "Item" },
];

const ORIGINS: { id: Origin; label: string }[] = [
  { id: "map", label: "This map" },
  { id: "game", label: "Game" },
];

export function kindOf(ability: AbilitySummary): Kind {
  if (ability.hero) {
    return "hero";
  }

  if (ability.item) {
    return "item";
  }

  return "unit";
}

function originOf(ability: AbilitySummary): Origin {
  if (ability.inMap) {
    return "map";
  }

  return "game";
}

export function AbilityList({ abilities, selected, onSelect, onError }: AbilityListProps) {
  const [query, setQuery] = useState("");
  const [hits, setHits] = useState<SearchHit[] | null>(null);
  const [kinds, setKinds] = useState<Kind[]>(["hero", "unit", "item"]);
  const [origins, setOrigins] = useState<Origin[]>(["map", "game"]);
  const [folded, toggleFolded] = useFolded("wc3-object-editor.fold.abilities");

  // Searches again when the list changes, so edited and new abilities show up.
  useEffect(() => {
    const trimmed = query.trim();
    if (!trimmed || abilities.length === 0) {
      setHits(null);
      return;
    }

    const timer = window.setTimeout(() => {
      api
        .searchAbilities(trimmed)
        .then(setHits)
        .catch((e) => onError(errorMessage(e)));
    }, SEARCH_DELAY_MS);
    return () => window.clearTimeout(timer);
  }, [query, abilities, onError]);

  const shown = useMemo(() => {
    const byId = new Map(abilities.map((a) => [a.id, a]));
    const kept = (ability: AbilitySummary) =>
      kinds.includes(kindOf(ability)) && origins.includes(originOf(ability));

    if (hits) {
      return hits
        .map((hit) => ({ ability: byId.get(hit.id), reason: hit.reason }))
        .filter((row): row is { ability: AbilitySummary; reason: string } => Boolean(row.ability))
        .filter((row) => kept(row.ability));
    }

    return abilities.filter(kept).map((ability) => ({ ability, reason: "" }));
  }, [abilities, hits, kinds, origins]);

  // While searching every match shows; otherwise the map's own abilities come
  // first and the game's fold away under their header.
  const own = shown.filter((r) => Boolean(hits) || r.ability.inMap);
  const game = shown.filter((r) => !hits && !r.ability.inMap);
  const mapHasNone = !abilities.some((a) => a.inMap);
  let visible = own.length + game.length;
  if (folded) {
    visible = own.length;
  }

  function row({ ability, reason }: { ability: AbilitySummary; reason: string }) {
    return (
      <li key={ability.id}>
        <button
          type="button"
          onClick={() => onSelect(ability.id)}
          aria-current={selected === ability.id}
          className={classes(
            "flex w-full items-center gap-2 px-3 py-1.5 text-left hover:bg-surface-2",
            selected === ability.id && "bg-surface-2 shadow-[inset_2px_0_0_var(--color-glow)]",
          )}
        >
          <UnitIcon path={ability.icon} size="small" />
          <span className="min-w-0 flex-1">
            <span className="block truncate">
              <GameText text={ability.name} singleLine />
              {ability.suffix && (
                <span className="ml-1 text-faint">
                  <GameText text={ability.suffix} singleLine />
                </span>
              )}
            </span>
            <span className="block truncate font-mono text-xs text-faint">
              {ability.id}
              {ability.custom && ` · based on ${ability.base}`}
              {reason && ` · ${reason}`}
            </span>
          </span>
          {ability.changedFields > 0 && (
            <span
              className="rounded bg-changed-soft px-1.5 text-xs text-changed"
              title={`${ability.changedFields} changed values`}
            >
              {ability.changedFields}
            </span>
          )}
        </button>
      </li>
    );
  }

  return (
    <aside
      aria-label="Ability list"
      className="panel flex min-h-0 w-72 shrink-0 flex-col rounded-[3px] bg-surface-1"
    >
      <div className="flex flex-col gap-2 border-b border-line p-3">
        <input
          type="search"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder="Search abilities: Holy Light, AHhb, 75…"
          aria-label="Search abilities"
          className="rounded-[3px] border border-field bg-well px-3 py-1.5 outline-none focus:border-glow"
        />
        <div className="flex flex-wrap gap-1">
          {KINDS.map((k) => (
            <Chip
              key={k.id}
              label={k.label}
              on={kinds.includes(k.id)}
              onPress={() => setKinds(toggle(kinds, k.id))}
            />
          ))}
          <span className="mx-1 border-l border-line" />
          {ORIGINS.map((o) => (
            <Chip
              key={o.id}
              label={o.label}
              on={origins.includes(o.id)}
              onPress={() => setOrigins(toggle(origins, o.id))}
            />
          ))}
        </div>
        <p className="text-xs text-faint">
          {visible} shown
          {folded && game.length > 0 && ` · ${game.length} game abilities folded`}
        </p>
      </div>

      <ul className="min-h-0 flex-1 overflow-y-auto" aria-label="Abilities">
        {own.map(row)}
        {mapHasNone && folded && !hits && (
          <li className="px-3 py-4 text-sm text-muted">
            This map adds or changes no abilities yet.
          </li>
        )}
        {game.length > 0 && (
          <ListFold
            label="Game abilities"
            count={game.length}
            folded={folded}
            onToggle={toggleFolded}
          />
        )}
        {!folded && game.map(row)}
      </ul>
    </aside>
  );
}
