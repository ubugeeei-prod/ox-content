/**
 * Publication dates from RSS / Atom items.
 */

import { parseDate } from "./feed-date";
import type { ParsedDate } from "./feed-format";

const MONTHS: Record<string, number> = {
  jan: 1,
  feb: 2,
  mar: 3,
  apr: 4,
  may: 5,
  jun: 6,
  jul: 7,
  aug: 8,
  sep: 9,
  oct: 10,
  nov: 11,
  dec: 12,
};

const RFC822 =
  /^(?:[A-Za-z]{3},\s+)?(\d{1,2})\s+([A-Za-z]{3})\s+(\d{4})\s+(\d{2}):(\d{2})(?::(\d{2}))?\s+(?:GMT|UTC|UT|([+-]\d{4}))$/;

export function parseFeedDate(value: string | undefined): ParsedDate | undefined {
  const trimmed = value?.trim();
  if (!trimmed) {
    return undefined;
  }
  return parseDate(trimmed) ?? parseRfc822(trimmed);
}

export function feedDateLabel(date: ParsedDate): string {
  return `${String(date.year).padStart(4, "0")}-${String(date.month).padStart(2, "0")}-${String(date.day).padStart(2, "0")}`;
}

export function feedDateIso(date: ParsedDate): string {
  return `${feedDateLabel(date)}T${String(date.hour).padStart(2, "0")}:${String(date.minute).padStart(2, "0")}:${String(date.second).padStart(2, "0")}Z`;
}

function parseRfc822(value: string): ParsedDate | undefined {
  const match = value.match(RFC822);
  if (!match) {
    return undefined;
  }
  const month = MONTHS[match[2]?.toLowerCase() ?? ""];
  if (!month) {
    return undefined;
  }
  const day = match[1]?.padStart(2, "0");
  const year = match[3];
  const hour = match[4];
  const minute = match[5];
  const second = (match[6] ?? "00").padStart(2, "0");
  const zone = match[7];
  const tz = zone ? `${zone.slice(0, 3)}:${zone.slice(3)}` : "Z";
  return parseDate(
    `${year}-${String(month).padStart(2, "0")}-${day}T${hour}:${minute}:${second}${tz}`,
  );
}
