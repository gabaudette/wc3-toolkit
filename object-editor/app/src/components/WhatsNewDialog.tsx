import { useMemo } from "react";
import changelog from "../../../../CHANGELOG.md?raw";
import { parseChangelog, parseInline } from "../changelog";
import { Button } from "./Button";
import { DialogShell } from "./DialogShell";

type WhatsNewDialogProps = {
  open: boolean;
  onClose: () => void;
};

function ItemText({ text }: { text: string }) {
  return (
    <>
      {parseInline(text).map((run, i) => {
        const key = `${i}-${run.text}`;

        if (run.code) {
          return (
            <code key={key} className="font-mono text-[0.9em] text-gold-bright">
              {run.text}
            </code>
          );
        }

        return <span key={key}>{run.text}</span>;
      })}
    </>
  );
}

export function WhatsNewDialog({ open, onClose }: WhatsNewDialogProps) {
  const releases = useMemo(() => parseChangelog(changelog), []);

  return (
    <DialogShell
      open={open}
      title="What's new"
      onClose={onClose}
      footer={
        <Button primary onPress={onClose}>
          Close
        </Button>
      }
    >
      {releases.map((release) => (
        <section key={release.title} className="flex flex-col gap-2">
          <h3 className="font-display text-xl font-bold text-gold">{release.title}</h3>
          {release.groups.map((group) => (
            <div key={group.title}>
              <h4 className="font-display text-sm font-bold text-muted">{group.title}</h4>
              <ul className="mt-1 flex list-disc flex-col gap-1 pl-5 text-sm leading-relaxed">
                {group.items.map((item) => (
                  <li key={item}>
                    <ItemText text={item} />
                  </li>
                ))}
              </ul>
            </div>
          ))}
        </section>
      ))}
    </DialogShell>
  );
}
