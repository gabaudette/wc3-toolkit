import { memo, type ReactNode, useEffect, useRef, useState } from "react";
import { classes } from "../ui";

type TabPanelProps = { shown: boolean; children: ReactNode };

const HIDE_FROM_FOCUS_MS = 1000;

function afterPaint(run: () => void): () => void {
  let timer = 0;
  const frame = window.requestAnimationFrame(() => {
    timer = window.setTimeout(run);
  });

  return () => {
    window.cancelAnimationFrame(frame);
    window.clearTimeout(timer);
  };
}

function blur(target: EventTarget | null): void {
  if (target instanceof HTMLElement) {
    target.blur();
  }
}

export function TabPanel({ shown, children }: TabPanelProps) {
  const [inert, setInert] = useState(!shown);
  const panelRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (shown) {
      return afterPaint(() => setInert(false));
    }

    const panel = panelRef.current;
    const stayOut = (e: FocusEvent) => blur(e.target);

    if (panel?.contains(document.activeElement)) {
      blur(document.activeElement);
    }

    panel?.addEventListener("focusin", stayOut);
    const timer = window.setTimeout(() => setInert(true), HIDE_FROM_FOCUS_MS);

    return () => {
      panel?.removeEventListener("focusin", stayOut);
      window.clearTimeout(timer);
    };
  }, [shown]);

  return (
    <div
      ref={panelRef}
      inert={inert}
      className={classes(
        "col-start-1 row-start-1 flex min-h-0 flex-col",
        !shown && "-translate-x-[200vw]",
      )}
    >
      <TabContent shown={shown}>{children}</TabContent>
    </div>
  );
}

function hiddenOrSame(before: TabPanelProps, after: TabPanelProps): boolean {
  return !after.shown || before.children === after.children;
}

const TabContent = memo(function TabContent({ children }: TabPanelProps) {
  return children;
}, hiddenOrSame);
