import { ArrowDown, ArrowUp, Plus, XClose } from "@untitledui/icons";
import { buttonText, titleText } from "../interface";
import type { ButtonAction, Design, DialogButton, DialogDesign } from "../types";
import { Button } from "./Button";
import { Checkbox } from "./Checkbox";
import { ColorSwatches } from "./ColorSwatches";
import { DesignHeader, designInputClass } from "./DesignBasics";
import { GameText } from "./GameText";
import { Select } from "./Select";

type DialogEditorProps = {
  dialog: DialogDesign;
  designs: Design[];
  luaName: string;
  editable: boolean;
  onChange: (dialog: DialogDesign) => void;
  onDuplicate: () => void;
  onDelete: () => void;
};

function newButton(): DialogButton {
  return {
    id: crypto.randomUUID(),
    text: "New button",
    hotkey: "",
    highlightHotkey: true,
    action: { kind: "close" },
  };
}

const ACTION_LABELS: Record<ButtonAction["kind"], string> = {
  close: "Close the dialog",
  showDialog: "Show another dialog",
  runFunction: "Run a Lua function",
};

function actionOfKind(kind: ButtonAction["kind"], others: Design[]): ButtonAction {
  if (kind === "showDialog") {
    return { kind, dialog: others[0]?.id ?? "" };
  }

  if (kind === "runFunction") {
    return { kind, function: "" };
  }

  return { kind: "close" };
}

export function DialogEditor({
  dialog,
  designs,
  luaName,
  editable,
  onChange,
  onDuplicate,
  onDelete,
}: DialogEditorProps) {
  const others = designs.filter((d) => d.id !== dialog.id);
  const set = (change: Partial<DialogDesign>) => onChange({ ...dialog, ...change });
  const setButton = (index: number, change: Partial<DialogButton>) =>
    set({
      buttons: dialog.buttons.map((b, i) => {
        if (i === index) {
          return { ...b, ...change };
        }

        return b;
      }),
    });
  const move = (index: number, by: number) => {
    const buttons = [...dialog.buttons];
    const [moved] = buttons.splice(index, 1);
    if (moved) {
      buttons.splice(index + by, 0, moved);
      set({ buttons });
    }
  };

  return (
    <div className="grid min-h-0 flex-1 grid-cols-[minmax(0,1fr)_minmax(0,1.1fr)] gap-5 overflow-y-auto p-5">
      <div className="flex min-w-0 flex-col gap-4">
        <DesignHeader
          name={dialog.name}
          lua={`UI.${luaName}.Show(player)`}
          editable={editable}
          onRename={(name) => set({ name })}
          onDuplicate={onDuplicate}
          onDelete={onDelete}
        />

        <label className="flex flex-col gap-1 text-sm text-muted">
          Message
          <textarea
            value={dialog.title}
            rows={2}
            disabled={!editable}
            onChange={(e) => set({ title: e.target.value })}
            className={designInputClass}
          />
        </label>
        <div className="-mt-2">
          <ColorSwatches
            value={dialog.titleColor}
            onChange={(titleColor) => set({ titleColor })}
            label="Message color"
            none={{ label: "Game gold", swatch: "#ffcc00" }}
            disabled={!editable}
          />
        </div>

        <Checkbox
          isSelected={dialog.showAtStart}
          isDisabled={!editable}
          onChange={(showAtStart) => set({ showAtStart })}
        >
          Show to every player when the game starts
        </Checkbox>

        <section aria-labelledby="dialog-buttons" className="flex flex-col gap-2">
          <h3 id="dialog-buttons" className="font-semibold text-gold-bright">
            Buttons
          </h3>
          {dialog.buttons.length === 0 && (
            <p className="text-sm text-faint">No buttons yet: the dialog could not be closed.</p>
          )}
          <ol className="flex flex-col gap-2">
            {dialog.buttons.map((button, index) => (
              <ButtonRow
                key={button.id}
                button={button}
                index={index}
                count={dialog.buttons.length}
                others={others}
                editable={editable}
                onChange={(change) => setButton(index, change)}
                onMove={(by) => move(index, by)}
                onRemove={() => set({ buttons: dialog.buttons.filter((_, i) => i !== index) })}
              />
            ))}
          </ol>
          <div>
            <Button
              small
              disabled={!editable}
              onPress={() => set({ buttons: [...dialog.buttons, newButton()] })}
            >
              <span className="flex items-center gap-1">
                <Plus className="size-3.5" aria-hidden="true" />
                Add button
              </span>
            </Button>
          </div>
        </section>
      </div>

      <DialogPreview dialog={dialog} />
    </div>
  );
}

type ButtonRowProps = {
  button: DialogButton;
  index: number;
  count: number;
  others: Design[];
  editable: boolean;
  onChange: (change: Partial<DialogButton>) => void;
  onMove: (by: number) => void;
  onRemove: () => void;
};

function ButtonRow({
  button,
  index,
  count,
  others,
  editable,
  onChange,
  onMove,
  onRemove,
}: ButtonRowProps) {
  const label = `Button ${index + 1}`;
  const { action } = button;

  return (
    <li className="flex flex-col gap-1.5 rounded-[3px] border border-line bg-surface-2/40 p-2">
      <div className="flex items-center gap-2">
        <input
          value={button.text}
          disabled={!editable}
          onChange={(e) => onChange({ text: e.target.value })}
          aria-label={`${label} text`}
          className={`${designInputClass} min-w-0 flex-1`}
        />
        <input
          value={button.hotkey}
          maxLength={1}
          disabled={!editable}
          onChange={(e) => onChange({ hotkey: e.target.value.toUpperCase() })}
          aria-label={`${label} hotkey`}
          title="Hotkey: one letter or digit"
          placeholder="Key"
          className={`${designInputClass} w-14 text-center font-mono`}
        />
        <IconButton
          label={`Move ${label} up`}
          disabled={!editable || index === 0}
          onPress={() => onMove(-1)}
        >
          <ArrowUp className="size-4" aria-hidden="true" />
        </IconButton>
        <IconButton
          label={`Move ${label} down`}
          disabled={!editable || index === count - 1}
          onPress={() => onMove(1)}
        >
          <ArrowDown className="size-4" aria-hidden="true" />
        </IconButton>
        <IconButton label={`Remove ${label}`} disabled={!editable} onPress={onRemove}>
          <XClose className="size-4" aria-hidden="true" />
        </IconButton>
      </div>
      <div className="flex flex-wrap items-center gap-2 text-sm text-muted">
        {button.hotkey && (
          <Checkbox
            isSelected={button.highlightHotkey}
            isDisabled={!editable}
            onChange={(highlightHotkey) => onChange({ highlightHotkey })}
            className="mr-2 text-sm"
          >
            Gold hotkey letter
          </Checkbox>
        )}
        <span>When clicked:</span>
        <Select
          value={action.kind}
          options={Object.entries(ACTION_LABELS).map(([kind, text]) => ({
            value: kind,
            label: text,
            disabled: kind === "showDialog" && others.length === 0,
          }))}
          onChange={(kind) =>
            onChange({ action: actionOfKind(kind as ButtonAction["kind"], others) })
          }
          label={`${label}: when clicked`}
          isDisabled={!editable}
        />
        {action.kind === "showDialog" && (
          <Select
            value={action.dialog}
            options={others.map((other) => ({ value: other.id, label: other.name || "(unnamed)" }))}
            onChange={(dialog) => onChange({ action: { kind: "showDialog", dialog } })}
            label={`${label}: dialog to show`}
            isDisabled={!editable}
          />
        )}
        {action.kind === "runFunction" && (
          <input
            value={action.function}
            disabled={!editable}
            onChange={(e) =>
              onChange({ action: { kind: "runFunction", function: e.target.value } })
            }
            aria-label={`${label}: Lua function`}
            placeholder="OnHardMode"
            className={`${designInputClass} w-44 font-mono`}
          />
        )}
      </div>
      {action.kind === "runFunction" && action.function && (
        <p className="text-xs text-faint">
          Called with the player who clicked, if your code defines{" "}
          <code className="font-mono">function {action.function}(player)</code>.
        </p>
      )}
    </li>
  );
}

type IconButtonProps = {
  label: string;
  disabled: boolean;
  onPress: () => void;
  children: React.ReactNode;
};

function IconButton({ label, disabled, onPress, children }: IconButtonProps) {
  return (
    <button
      type="button"
      onClick={onPress}
      disabled={disabled}
      aria-label={label}
      title={label}
      className="rounded p-1.5 text-faint hover:bg-surface-2 hover:text-ink disabled:opacity-30"
    >
      {children}
    </button>
  );
}

function DialogPreview({ dialog }: { dialog: DialogDesign }) {
  return (
    <figure className="flex min-w-0 flex-col gap-1.5">
      <figcaption className="text-xs text-faint">Preview, as the game shows it</figcaption>
      <div className="@container relative flex aspect-video items-center justify-center overflow-hidden rounded-[3px] border border-line bg-[radial-gradient(ellipse_at_center,#2b3a2a_0%,#141c14_70%)]">
        <div className="flex w-[34cqw] flex-col items-stretch gap-[0.9cqw] rounded-[0.4cqw] border-[0.25cqw] border-frame bg-[#0b0d12]/95 px-[1.6cqw] py-[1.4cqw] shadow-[0_0_2cqw_rgba(0,0,0,0.8)]">
          <p className="text-center text-[1.5cqw] leading-snug font-semibold text-[#ffcc00]">
            <GameText text={titleText(dialog)} />
          </p>
          {dialog.buttons.map((button) => (
            <div
              key={button.id}
              className="rounded-[0.3cqw] border-[0.15cqw] border-[#6b5a3a] bg-[linear-gradient(#3a3024,#211a12)] px-[1cqw] py-[0.6cqw] text-center text-[1.3cqw] text-[#f0e6d0]"
            >
              <GameText text={buttonText(button)} singleLine />
            </div>
          ))}
        </div>
      </div>
    </figure>
  );
}
