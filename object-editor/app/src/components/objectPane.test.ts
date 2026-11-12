import { expect, test } from "bun:test";
import { failed, ok } from "../result";
import { fillLevels } from "./objectPane";

test("filling sets every level from the first and stops at a failure", async () => {
  const set: number[] = [];

  const filled = await fillLevels(
    (level) => {
      set.push(level);
      return Promise.resolve(ok(level));
    },
    2,
    4,
  );
  const locked = (level: number) => {
    if (level === 1) {
      return Promise.resolve(ok(level));
    }

    return Promise.resolve(failed(`Level ${level} is locked.`));
  };
  const stopped = await fillLevels(locked, 1, 3);

  expect(set).toEqual([2, 3, 4]);
  expect(filled).toEqual(ok(4));
  expect(stopped).toEqual(failed("Level 2 is locked."));
});
