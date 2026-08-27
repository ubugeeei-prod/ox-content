import { describe, expect, it } from "vite-plus/test";
import { importNapiModuleSync } from "./napi";
import { parseDate } from "./feed-date";

const ACCEPTED: [string, string, number][] = [
  ["a civil date", "2024-01-02", 1704153600],
  ["a UTC date-time", "2024-01-02T03:04:05Z", 1704164645],
  ["a positive offset", "2024-01-02T03:04:05+09:00", 1704132245],
  ["a negative offset", "2024-01-02T03:04:05-05:30", 1704184445],
  ["a space separator", "2024-01-02 03:04:05Z", 1704164645],
  ["fractional seconds", "2024-01-02T03:04:05.123Z", 1704164645],
  ["epoch seconds", "1700000000", 1700000000],
  ["epoch milliseconds", "1700000000000", 1700000000],
  ["a leap day", "2024-02-29T00:00:00Z", 1709164800],
  ["a pre-epoch instant", "1969-12-31T23:59:59Z", -1],
];

const REJECTED = [
  "",
  "not a date",
  "2024-13-02",
  "2024-01-32",
  "2024-01-02T24:00:00Z",
  "2024-01-02X03:04:05Z",
];

describe("parseDate", () => {
  for (const [label, input, unix] of ACCEPTED) {
    it(`accepts ${label}`, () => {
      expect(parseDate(input)?.unix).toBe(unix);
    });
  }

  it("rejects values that name no instant", () => {
    for (const input of REJECTED) {
      expect(parseDate(input), input).toBeUndefined();
    }
    expect(parseDate(undefined)).toBeUndefined();
  });

  // Frontmatter dates arrive with whatever padding the author left, and two of
  // the three call sites do not trim before asking. Dropping the date silently
  // is how a post loses its position in a feed.
  it("tolerates surrounding whitespace", () => {
    expect(parseDate(" 2024-01-02T03:04:05Z ")?.unix).toBe(1704164645);
    expect(parseDate("\t2024-01-02")?.unix).toBe(1704153600);
  });

  // A digit string past the safe-integer range is not a date anyone meant.
  // Rounding it into a year in the billions and writing that to a feed is
  // worse than refusing it.
  it("refuses an epoch value it cannot represent exactly", () => {
    expect(parseDate("99999999999999999999")).toBeUndefined();
  });

  it("agrees with the native parser it delegates to", () => {
    const napi = importNapiModuleSync();
    for (const input of [...ACCEPTED.map(([, value]) => value), ...REJECTED]) {
      expect(parseDate(input)?.unix, input).toBe(napi.parseFeedDate(input)?.unix);
    }
  });
});
