// Port of react-tweet date-utils.ts (MIT, Copyright (c) 2023 Luis Alvarez).
// Notices live in social-tweet-full.css and docs/content/credits.md.

const DATE_OPTIONS: Intl.DateTimeFormatOptions = {
  hour: "numeric",
  minute: "2-digit",
  hour12: true,
  month: "short",
  day: "numeric",
  year: "numeric",
};

export function resolveTweetTimeZone(timeZone?: string): string {
  const candidate = timeZone?.trim();
  if (!candidate) return "UTC";
  try {
    new Intl.DateTimeFormat("en-US", { timeZone: candidate }).format();
    return candidate;
  } catch {
    return "UTC";
  }
}

export function formatFullDate(
  createdAt: string | undefined,
  timeZone = "UTC",
): { iso: string; label: string } | undefined {
  if (!createdAt) return undefined;
  const date = new Date(createdAt);
  if (Number.isNaN(date.valueOf())) return undefined;
  const zone = resolveTweetTimeZone(timeZone);
  const parts = new Intl.DateTimeFormat("en-US", { ...DATE_OPTIONS, timeZone: zone }).formatToParts(
    date,
  );
  const get = (type: Intl.DateTimeFormatPartTypes) =>
    parts.find((part) => part.type === type)?.value ?? "";
  return {
    iso: date.toISOString(),
    label: `${get("hour")}:${get("minute")} ${get("dayPeriod")} · ${get("month")} ${get("day")}, ${get("year")}`,
  };
}
