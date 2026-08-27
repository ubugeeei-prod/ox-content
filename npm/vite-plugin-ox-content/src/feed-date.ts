import type { ParsedDate } from "./feed-format";
import { importNapiModuleSync } from "./napi";

/**
 * Parse a feed date: epoch seconds, epoch milliseconds, or a civil date-time
 * with an optional zone offset.
 *
 * The rule lives in Rust (`ox_content_ssg::parse_date`) so the feeds the Vite
 * plugin writes and the ones the native SSG writes agree on what a date is.
 */
export function parseDate(value: string | undefined): ParsedDate | undefined {
  const parsed = importNapiModuleSync().parseFeedDate(value);
  if (!parsed) {
    return undefined;
  }
  return {
    unix: parsed.unix,
    year: parsed.year,
    month: parsed.month,
    day: parsed.day,
    hour: parsed.hour,
    minute: parsed.minute,
    second: parsed.second,
  };
}
