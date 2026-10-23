import { getVersion } from "@tauri-apps/api/app";
import { useEffect, useState } from "react";
import { Button as AriaButton } from "react-aria-components";
import { WhatsNewDialog } from "./WhatsNewDialog";

/**
 * The editor's version ("v0.1.0"), from the app itself. Pressing it opens
 * "What's new". Nothing shows until the version is known.
 */
export function AppVersion() {
  const [version, setVersion] = useState<string | null>(null);
  const [open, setOpen] = useState(false);

  useEffect(() => {
    // Showing the version is a courtesy; without it the editor works the same.
    getVersion()
      .then(setVersion)
      .catch(() => setVersion(null));
  }, []);

  if (!version) {
    return null;
  }

  return (
    <>
      <AriaButton
        onPress={() => setOpen(true)}
        aria-label={`Version ${version}: what's new`}
        className="-my-1 rounded-[3px] px-1 py-1 font-sans text-xs font-normal text-faint hover:text-gold"
      >
        v{version}
      </AriaButton>
      <WhatsNewDialog open={open} onClose={() => setOpen(false)} />
    </>
  );
}
