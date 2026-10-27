import { Plus, XClose } from "@untitledui/icons";
import type { LeaderboardDesign, LeaderboardLabel } from "../types";
import { Button } from "./Button";
import { Checkbox } from "./Checkbox";
import { ColorSwatches } from "./ColorSwatches";
import { DesignBasics, designInputClass, GameScreen } from "./DesignBasics";
import { GameText } from "./GameText";
import { Select } from "./Select";

type LeaderboardEditorProps = {
  board: LeaderboardDesign;
  luaName: string;
  editable: boolean;
  onChange: (board: LeaderboardDesign) => void;
  onDuplicate: () => void;
  onDelete: () => void;
};

/** The game gives every row a player; rows of the map's own use the four neutral ones. */
const MAX_LABELS = 4;
const SORTS: { value: LeaderboardDesign["sort"]; label: string }[] = [
  { value: "highFirst", label: "Highest first" },
  { value: "lowFirst", label: "Lowest first" },
  { value: "none", label: "As listed" },
];
const SAMPLE_PLAYERS = [
  { name: "Player 1", color: "ff0303", value: 3 },
  { name: "Player 2", color: "0042ff", value: 7 },
];

export function LeaderboardEditor({
  board,
  luaName,
  editable,
  onChange,
  onDuplicate,
  onDelete,
}: LeaderboardEditorProps) {
  const set = (change: Partial<LeaderboardDesign>) => onChange({ ...board, ...change });
  const setLabel = (index: number, change: Partial<LeaderboardLabel>) =>
    set({
      labels: board.labels.map((label, i) => {
        if (i === index) {
          return { ...label, ...change };
        }

        return label;
      }),
    });
  let example = `"${board.labels[0]?.name ?? "Alliance"}"`;

  if (board.players) {
    example = "player";
  }

  return (
    <div className="grid min-h-0 flex-1 grid-cols-[minmax(0,1fr)_minmax(0,1.1fr)] gap-5 overflow-y-auto p-5">
      <div className="flex min-w-0 flex-col gap-4">
        <DesignBasics
          name={board.name}
          title={board.title}
          titleColor={board.titleColor}
          lua={`UI.${luaName}.AddValue(${example}, 1)`}
          editable={editable}
          onChange={set}
          onDuplicate={onDuplicate}
          onDelete={onDelete}
        />

        <div className="flex flex-wrap items-center gap-3">
          <Checkbox
            isSelected={board.players}
            isDisabled={!editable}
            onChange={(players) => set({ players })}
          >
            A row for each player, starting at
          </Checkbox>
          <input
            type="number"
            step={1}
            value={board.playerValue}
            disabled={!editable || !board.players}
            onChange={(e) => set({ playerValue: Math.trunc(Number(e.target.value)) })}
            aria-label="Players' starting value"
            className={`${designInputClass} w-24 text-right`}
          />
        </div>

        <section aria-labelledby="leaderboard-rows" className="flex flex-col gap-2">
          <h3 id="leaderboard-rows" className="font-semibold text-gold-bright">
            Rows of your own
          </h3>
          <p className="text-xs text-faint">
            Such as teams; at most {MAX_LABELS}, since the game gives every row a player and these
            use the neutral ones.
          </p>
          <ol className="flex flex-col gap-2">
            {board.labels.map((label, index) => (
              <li
                key={label.id}
                className="flex flex-col gap-1.5 rounded-[3px] border border-line bg-surface-2/40 p-2"
              >
                <div className="flex items-center gap-2">
                  <input
                    value={label.name}
                    disabled={!editable}
                    onChange={(e) => setLabel(index, { name: e.target.value })}
                    aria-label={`Row ${index + 1} name`}
                    className={`${designInputClass} min-w-0 flex-1`}
                  />
                  <input
                    type="number"
                    step={1}
                    value={label.value}
                    disabled={!editable}
                    onChange={(e) => setLabel(index, { value: Math.trunc(Number(e.target.value)) })}
                    aria-label={`Row ${index + 1} starting value`}
                    title="Starting value"
                    className={`${designInputClass} w-24 text-right`}
                  />
                  <button
                    type="button"
                    disabled={!editable}
                    onClick={() => set({ labels: board.labels.filter((_, i) => i !== index) })}
                    aria-label={`Remove row ${label.name}`}
                    title="Remove"
                    className="rounded p-1.5 text-faint hover:bg-surface-2 hover:text-ink disabled:opacity-30"
                  >
                    <XClose className="size-4" aria-hidden="true" />
                  </button>
                </div>
                <ColorSwatches
                  value={label.color}
                  onChange={(color) => setLabel(index, { color })}
                  label={`Row ${index + 1} color`}
                  none={{ label: "White", swatch: "#ffffff" }}
                  disabled={!editable}
                />
              </li>
            ))}
          </ol>
          <div>
            <Button
              small
              disabled={!editable || board.labels.length >= MAX_LABELS}
              onPress={() =>
                set({
                  labels: [
                    ...board.labels,
                    {
                      id: crypto.randomUUID(),
                      name: `Team ${board.labels.length + 1}`,
                      color: null,
                      value: 0,
                    },
                  ],
                })
              }
            >
              <span className="flex items-center gap-1">
                <Plus className="size-3.5" aria-hidden="true" />
                Add row
              </span>
            </Button>
          </div>
        </section>

        <div className="flex items-center gap-2 text-sm text-muted">
          <span aria-hidden="true">Order</span>
          <Select
            value={board.sort}
            options={SORTS}
            onChange={(sort) => set({ sort: sort as LeaderboardDesign["sort"] })}
            label="Order"
            isDisabled={!editable}
          />
        </div>
        <Checkbox
          isSelected={board.showAtStart}
          isDisabled={!editable}
          onChange={(showAtStart) => set({ showAtStart })}
        >
          Show when the game starts
        </Checkbox>
      </div>

      <LeaderboardPreview board={board} />
    </div>
  );
}

type PreviewRow = { key: string; text: string; value: number };

function previewRows(board: LeaderboardDesign): PreviewRow[] {
  const rows: PreviewRow[] = [];

  if (board.players) {
    for (const player of SAMPLE_PLAYERS) {
      rows.push({
        key: player.name,
        text: `|cff${player.color}${player.name}|r`,
        value: board.playerValue + player.value,
      });
    }
  }

  for (const label of board.labels) {
    let text = label.name;

    if (label.color) {
      text = `|cff${label.color}${label.name}|r`;
    }

    rows.push({ key: label.id, text, value: label.value });
  }

  if (board.sort === "highFirst") {
    rows.sort((a, b) => b.value - a.value);
  } else if (board.sort === "lowFirst") {
    rows.sort((a, b) => a.value - b.value);
  }

  return rows;
}

function LeaderboardPreview({ board }: { board: LeaderboardDesign }) {
  const rows = previewRows(board);
  let title = board.title;

  if (board.titleColor) {
    title = `|cff${board.titleColor}${board.title}|r`;
  }

  return (
    <GameScreen>
      <div className="absolute top-[1.2cqw] right-[1.2cqw] min-w-[16cqw] rounded-[0.3cqw] border-[0.15cqw] border-frame bg-[#0b0d12]/85 px-[0.8cqw] py-[0.6cqw] text-[1.15cqw] leading-tight text-white">
        <p className="mb-[0.4cqw] border-b-[0.1cqw] border-frame/60 pb-[0.3cqw] text-center font-semibold">
          <GameText text={title} singleLine />
        </p>
        {rows.map((row) => (
          <div key={row.key} className="flex justify-between gap-[1.5cqw] py-[0.15cqw]">
            <GameText text={row.text} singleLine />
            <span>{row.value}</span>
          </div>
        ))}
      </div>
    </GameScreen>
  );
}
