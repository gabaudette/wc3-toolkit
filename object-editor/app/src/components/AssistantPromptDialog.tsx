import { useEffect, useState } from "react";
import { api } from "../api";
import { assistantPrompt } from "../assistantPrompt";
import { attempt, settle } from "../result";
import type { AssistantContext } from "../types";
import { Button } from "./Button";
import { DialogShell } from "./DialogShell";

type AssistantPromptDialogProps = {
  open: boolean;
  onClose: () => void;
};

const GOAL_ROWS = 3;
const PROMPT_ROWS = 14;
const GOAL_EXAMPLE =
  "A Dota 2 style 5v5 map: three lanes with creep waves, towers, heroes with four abilities, items and shops.";

export function AssistantPromptDialog({ open, onClose }: AssistantPromptDialogProps) {
  const [goal, setGoal] = useState("");
  const [context, setContext] = useState<AssistantContext | null>(null);
  const [status, setStatus] = useState("");

  useEffect(() => {
    if (!open) {
      return;
    }

    setStatus("");
    void settle(api.assistantContext(), setContext, () => setContext(null));
  }, [open]);

  const prompt = assistantPrompt(goal, context);

  async function copy() {
    const copied = await attempt(() => navigator.clipboard.writeText(prompt));

    if (copied.ok) {
      setStatus("Copied. Paste it into your AI assistant's chat.");
      return;
    }

    setStatus("The clipboard is not available: select the prompt below and copy it by hand.");
  }

  return (
    <DialogShell
      open={open}
      title="Prompt for an AI assistant"
      wide
      onClose={onClose}
      footer={
        <>
          <Button onPress={onClose}>Close</Button>
          <Button primary onPress={() => void copy()}>
            Copy prompt
          </Button>
        </>
      }
    >
      <p className="text-sm text-muted">
        Paste this into an AI assistant such as Claude or ChatGPT. It explains Tinkerbench, the
        World Editor's part and the Lua harness, so the plan and code you get back fit these tools.
        {context?.map && ` It includes a summary of ${context.map.name}.`}
      </p>
      <label className="flex flex-col gap-1 text-sm text-muted">
        The map you want to make
        <textarea
          value={goal}
          rows={GOAL_ROWS}
          placeholder={GOAL_EXAMPLE}
          onChange={(e) => setGoal(e.target.value)}
          className="rounded-[3px] border border-field bg-well px-3 py-1.5 text-ink outline-none focus:border-focus"
        />
      </label>
      <label className="flex flex-col gap-1 text-sm text-muted">
        Prompt
        <textarea
          value={prompt}
          readOnly
          rows={PROMPT_ROWS}
          className="rounded-[3px] border border-field bg-well px-3 py-1.5 font-mono text-xs text-ink outline-none focus:border-focus"
        />
      </label>
      <p role="status" className="text-sm text-gold-bright">
        {status}
      </p>
    </DialogShell>
  );
}
