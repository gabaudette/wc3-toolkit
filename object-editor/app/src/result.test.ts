import { describe, expect, test } from "bun:test";
import { attempt, attemptSync, errorMessage, failed, ok, settle, whenOk } from "./result";

describe("attempt", () => {
  test("turns a rejection or a throw into a failed result", async () => {
    expect(await attempt(() => Promise.resolve(3))).toEqual(ok(3));
    expect(await attempt(() => Promise.reject("No map is open."))).toEqual(
      failed("No map is open."),
    );
    expect(
      attemptSync(() => {
        throw new Error("Storage is off.");
      }),
    ).toEqual(failed("Storage is off."));
  });
});

describe("settle", () => {
  test("hands the value or the error to the matching callback", async () => {
    const seen: string[] = [];

    await settle(
      Promise.resolve(ok("a")),
      (value) => seen.push(value),
      (error) => seen.push(`!${error}`),
    );
    await settle(
      Promise.resolve(failed("b")),
      (value) => seen.push(value),
      (error) => seen.push(`!${error}`),
    );
    await whenOk(Promise.resolve(failed("c")), (value: string) => seen.push(value));

    expect(seen).toEqual(["a", "!b"]);
  });
});

test("errors from commands arrive as plain messages", () => {
  expect(errorMessage("Bad value.")).toBe("Bad value.");
  expect(errorMessage(new Error("Gone."))).toBe("Gone.");
  expect(errorMessage(42)).toBe("Something went wrong.");
});
