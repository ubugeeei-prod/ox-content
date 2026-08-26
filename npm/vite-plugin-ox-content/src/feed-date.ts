import type { ParsedDate } from "./feed-format";

export function parseDate(value: string | undefined): ParsedDate | undefined {
  if (!value) {
    return undefined;
  }
  if (/^\d+$/.test(value)) {
    const n = Number(value);
    return unixToDate(value.length >= 13 ? Math.trunc(n / 1000) : n);
  }
  return parseCivilDate(value);
}

function parseCivilDate(value: string): ParsedDate | undefined {
  if (value.length < 10 || value[4] !== "-" || value[7] !== "-") {
    return undefined;
  }
  const year = Number(value.slice(0, 4));
  const month = Number(value.slice(5, 7));
  const day = Number(value.slice(8, 10));
  let hour = 0;
  let minute = 0;
  let second = 0;
  let offset = 0;
  if (value.length > 10) {
    const rest = value.slice(10);
    const time = rest.startsWith("T") || rest.startsWith(" ") ? rest.slice(1) : "";
    if (time.length < 8 || time[2] !== ":" || time[5] !== ":") {
      return undefined;
    }
    hour = Number(time.slice(0, 2));
    minute = Number(time.slice(3, 5));
    second = Number(time.slice(6, 8));
    const parsedOffset = parseOffset(timezoneSuffix(time));
    if (parsedOffset == null) {
      return undefined;
    }
    offset = parsedOffset;
  }
  const unix = civilToUnix(year, month, day, hour, minute, second);
  return unix == null ? undefined : unixToDate(unix - offset);
}

function timezoneSuffix(rest: string): string {
  const afterTime = rest.slice(8);
  if (afterTime.startsWith(".")) {
    const index = afterTime.search(/[Z+-]/);
    return index === -1 ? "" : afterTime.slice(index);
  }
  return afterTime;
}

function parseOffset(tz: string): number | undefined {
  if (!tz || tz === "Z") {
    return 0;
  }
  if (tz.length < 6) {
    return undefined;
  }
  const sign = tz[0] === "+" ? 1 : tz[0] === "-" ? -1 : 0;
  if (!sign) {
    return undefined;
  }
  return sign * (Number(tz.slice(1, 3)) * 3600 + Number(tz.slice(4, 6)) * 60);
}

function civilToUnix(
  year: number,
  month: number,
  day: number,
  hour: number,
  minute: number,
  second: number,
): number | undefined {
  if (month < 1 || month > 12 || day < 1 || day > 31 || hour > 23 || minute > 59 || second > 60) {
    return undefined;
  }
  let y = year;
  if (month <= 2) {
    y -= 1;
  }
  const era = Math.trunc((y >= 0 ? y : y - 399) / 400);
  const yoe = y - era * 400;
  const shifted = month + (month > 2 ? -3 : 9);
  const doy = Math.trunc((153 * shifted + 2) / 5) + day - 1;
  const doe = yoe * 365 + Math.trunc(yoe / 4) - Math.trunc(yoe / 100) + doy;
  const days = era * 146097 + doe - 719468;
  return days * 86400 + hour * 3600 + minute * 60 + second;
}

function unixToDate(unix: number): ParsedDate | undefined {
  const days = Math.floor(unix / 86400);
  const tod = ((unix % 86400) + 86400) % 86400;
  const z = days + 719468;
  const era = Math.trunc((z >= 0 ? z : z - 146096) / 146097);
  const doe = z - era * 146097;
  const yoe = Math.trunc(
    (doe - Math.trunc(doe / 1460) + Math.trunc(doe / 36524) - Math.trunc(doe / 146096)) / 365,
  );
  const year = yoe + era * 400;
  const doy = doe - (365 * yoe + Math.trunc(yoe / 4) - Math.trunc(yoe / 100));
  const mp = Math.trunc((5 * doy + 2) / 153);
  const day = doy - Math.trunc((153 * mp + 2) / 5) + 1;
  const month = mp < 10 ? mp + 3 : mp - 9;
  return {
    unix,
    year: year + (month <= 2 ? 1 : 0),
    month,
    day,
    hour: Math.trunc(tod / 3600),
    minute: Math.trunc((tod % 3600) / 60),
    second: tod % 60,
  };
}
