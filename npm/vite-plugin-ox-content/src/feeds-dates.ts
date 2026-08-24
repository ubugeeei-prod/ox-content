/** Date parsing and sort helpers for opt-in feeds. */

export const FEED_EPOCH = "1970-01-01T00:00:00Z";
const WEEKDAYS = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"] as const;
const MONTHS = [
  "Jan",
  "Feb",
  "Mar",
  "Apr",
  "May",
  "Jun",
  "Jul",
  "Aug",
  "Sep",
  "Oct",
  "Nov",
  "Dec",
] as const;

export type ParsedFeedDate = readonly [number, number, number, number, number, number];

export function parseFeedDate(value: string | undefined): ParsedFeedDate | undefined {
  const trimmed = value?.trim();
  if (!trimmed || trimmed.length < 10 || trimmed[4] !== "-" || trimmed[7] !== "-") {
    return undefined;
  }
  const year = Number(trimmed.slice(0, 4));
  const month = Number(trimmed.slice(5, 7));
  const day = Number(trimmed.slice(8, 10));
  if (!Number.isInteger(year) || month < 1 || month > 12 || day < 1 || day > 31) {
    return undefined;
  }
  const time = parseFeedTime(trimmed) ?? ([0, 0, 0] as const);
  return [year, month, day, time[0], time[1], time[2]];
}

function parseFeedTime(value: string): readonly [number, number, number] | undefined {
  if (value.length < 19 || (value[10] !== "T" && value[10] !== " ")) {
    return undefined;
  }
  const hour = Number(value.slice(11, 13));
  const minute = Number(value.slice(14, 16));
  const second = Number(value.slice(17, 19));
  return hour <= 23 && minute <= 59 && second <= 60 ? [hour, minute, second] : undefined;
}

function pad(value: number): string {
  return String(value).padStart(2, "0");
}

export function formatFeedRfc3339(date: ParsedFeedDate): string {
  return `${String(date[0]).padStart(4, "0")}-${pad(date[1])}-${pad(date[2])}T${pad(date[3])}:${pad(date[4])}:${pad(date[5])}Z`;
}

export function formatFeedRfc822(date: ParsedFeedDate): string {
  const table = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
  const year = date[1] < 3 ? date[0] - 1 : date[0];
  const weekday =
    (year +
      Math.trunc(year / 4) -
      Math.trunc(year / 100) +
      Math.trunc(year / 400) +
      table[date[1] - 1]! +
      date[2]) %
    7;
  return `${WEEKDAYS[weekday]}, ${pad(date[2])} ${MONTHS[date[1] - 1]} ${String(date[0]).padStart(4, "0")} ${pad(date[3])}:${pad(date[4])}:${pad(date[5])} +0000`;
}

export function compareFeedDates(left: ParsedFeedDate, right: ParsedFeedDate): number {
  for (let index = 0; index < left.length; index += 1) {
    if (left[index] !== right[index]) {
      return left[index]! < right[index]! ? -1 : 1;
    }
  }
  return 0;
}

export function newestFeedUpdated(dates: readonly (string | undefined)[]): string {
  for (const value of dates) {
    const date = parseFeedDate(value);
    if (date) return formatFeedRfc3339(date);
  }
  return FEED_EPOCH;
}
