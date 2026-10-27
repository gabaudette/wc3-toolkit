import { describe, expect, test } from "bun:test";
import { clock } from "./TimerWindowEditor";

describe("clock", () => {
  test("shows minutes and two-digit seconds, as the game's timer window does", () => {
    expect(clock(90)).toBe("1:30");
    expect(clock(5)).toBe("0:05");
    expect(clock(3600)).toBe("60:00");
  });

  test("rounds partial seconds up and never goes below zero", () => {
    expect(clock(59.2)).toBe("1:00");
    expect(clock(-3)).toBe("0:00");
  });
});
