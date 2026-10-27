export type Result<T> = { ok: true; value: T } | { ok: false; error: string };

export function ok<T>(value: T): Result<T> {
  return { ok: true, value };
}

export function failed(error: string): Result<never> {
  return { ok: false, error };
}

export function errorMessage(error: unknown): string {
  if (typeof error === "string") {
    return error;
  }

  if (error instanceof Error) {
    return error.message;
  }

  return "Something went wrong.";
}

export async function attempt<T>(work: () => Promise<T>): Promise<Result<T>> {
  try {
    return ok(await work());
  } catch (error) {
    return failed(errorMessage(error));
  }
}

export async function settle<T>(
  pending: Promise<Result<T>>,
  onValue: (value: T) => void,
  onError: (error: string) => void,
): Promise<void> {
  const result = await pending;

  if (result.ok) {
    onValue(result.value);
    return;
  }

  onError(result.error);
}

export async function whenOk<T>(
  pending: Promise<Result<T>>,
  onValue: (value: T) => void,
): Promise<void> {
  const result = await pending;

  if (result.ok) {
    onValue(result.value);
  }
}

export function attemptSync<T>(work: () => T): Result<T> {
  try {
    return ok(work());
  } catch (error) {
    return failed(errorMessage(error));
  }
}
