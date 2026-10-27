import {
  ask,
  type ConfirmDialogOptions,
  type DialogFilter,
  open,
  type SaveDialogOptions,
  save,
} from "@tauri-apps/plugin-dialog";
import { report } from "./api";
import { attempt, type Result } from "./result";

type ChooseOptions = {
  title?: string;
  directory?: boolean;
  filters?: DialogFilter[];
  defaultPath?: string;
};

async function reported<T>(work: () => Promise<T>): Promise<Result<T>> {
  const result = await attempt(work);

  if (!result.ok) {
    report("error", "file dialog", result.error);
  }

  return result;
}

export function choosePath(options: ChooseOptions): Promise<Result<string | null>> {
  return reported(() => open({ ...options, multiple: false }));
}

export function chooseSavePath(options: SaveDialogOptions): Promise<Result<string | null>> {
  return reported(() => save(options));
}

export function confirm(message: string, options: ConfirmDialogOptions): Promise<Result<boolean>> {
  return reported(() => ask(message, options));
}
