/** RSS / Atom / JSON Feed string bodies used by `feeds.ts`. */

export interface ParsedDate {
  unix: number;
  year: number;
  month: number;
  day: number;
  hour: number;
  minute: number;
  second: number;
}

export interface FeedDocument {
  siteName: string;
  siteDescription?: string;
  home: string;
  atomUrl: string;
  jsonUrl: string;
}

export interface FeedEntry {
  title: string;
  description?: string;
  loc: string;
  date?: ParsedDate;
}

export function generateRss(doc: FeedDocument, items: readonly FeedEntry[]): string {
  let xml = '<?xml version="1.0" encoding="UTF-8"?>\n<rss version="2.0">\n  <channel>\n    <title>';
  xml += escapeXml(doc.siteName);
  xml += "</title>\n    <link>";
  xml += escapeXml(doc.home);
  xml += "</link>\n    <description>";
  xml += escapeXml(channelDescription(doc));
  xml += "</description>\n";
  for (const item of items) {
    xml += "    <item>\n      <title>";
    xml += escapeXml(item.title);
    xml += "</title>\n      <link>";
    xml += escapeXml(item.loc);
    xml += "</link>\n      <guid>";
    xml += escapeXml(item.loc);
    xml += "</guid>\n";
    if (item.description) {
      xml += "      <description>";
      xml += escapeXml(item.description);
      xml += "</description>\n";
    }
    if (item.date) {
      xml += `      <pubDate>${formatRfc822(item.date)}</pubDate>\n`;
    }
    xml += "    </item>\n";
  }
  xml += "  </channel>\n</rss>\n";
  return xml;
}

export function generateAtom(doc: FeedDocument, items: readonly FeedEntry[]): string {
  const updated = items[0]?.date ? formatRfc3339(items[0].date) : "1970-01-01T00:00:00Z";
  let xml =
    '<?xml version="1.0" encoding="UTF-8"?>\n<feed xmlns="http://www.w3.org/2005/Atom">\n  <title>';
  xml += escapeXml(doc.siteName);
  xml += '</title>\n  <link href="';
  xml += escapeXml(doc.atomUrl);
  xml += '" rel="self"/>\n  <link href="';
  xml += escapeXml(doc.home);
  xml += '" rel="alternate"/>\n  <id>';
  xml += escapeXml(doc.home);
  xml += `</id>\n  <updated>${updated}</updated>\n`;
  if (doc.siteDescription?.trim()) {
    xml += "  <subtitle>";
    xml += escapeXml(doc.siteDescription);
    xml += "</subtitle>\n";
  }
  for (const item of items) {
    xml += "  <entry>\n    <title>";
    xml += escapeXml(item.title);
    xml += '</title>\n    <link href="';
    xml += escapeXml(item.loc);
    xml += '"/>\n    <id>';
    xml += escapeXml(item.loc);
    xml += `</id>\n    <updated>${item.date ? formatRfc3339(item.date) : updated}</updated>\n`;
    if (item.description) {
      xml += "    <summary>";
      xml += escapeXml(item.description);
      xml += "</summary>\n";
    }
    xml += "  </entry>\n";
  }
  xml += "</feed>\n";
  return xml;
}

export function generateJson(doc: FeedDocument, items: readonly FeedEntry[]): string {
  let json = '{\n  "version": "https://jsonfeed.org/version/1.1",\n  "title": ';
  json += jsonString(doc.siteName);
  json += ',\n  "home_page_url": ';
  json += jsonString(doc.home);
  json += ',\n  "feed_url": ';
  json += jsonString(doc.jsonUrl);
  if (doc.siteDescription?.trim()) {
    json += ',\n  "description": ';
    json += jsonString(doc.siteDescription);
  }
  json += ',\n  "items": [';
  items.forEach((item, index) => {
    if (index > 0) {
      json += ",";
    }
    json += '\n    {\n      "id": ';
    json += jsonString(item.loc);
    json += ',\n      "url": ';
    json += jsonString(item.loc);
    json += ',\n      "title": ';
    json += jsonString(item.title);
    if (item.description) {
      json += ',\n      "content_text": ';
      json += jsonString(item.description);
    }
    if (item.date) {
      json += ',\n      "date_published": ';
      json += jsonString(formatRfc3339(item.date));
    }
    json += "\n    }";
  });
  json += "\n  ]\n}\n";
  return json;
}

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

function channelDescription(doc: FeedDocument): string {
  const description = doc.siteDescription?.trim();
  return description ? description : doc.siteName;
}

export function escapeXml(value: string): string {
  return value.replace(/[&<>"']/g, (ch) => {
    switch (ch) {
      case "&":
        return "&amp;";
      case "<":
        return "&lt;";
      case ">":
        return "&gt;";
      case '"':
        return "&quot;";
      default:
        return "&#39;";
    }
  });
}

export function jsonString(value: string): string {
  let escaped = '"';
  for (const ch of value) {
    const code = ch.codePointAt(0) ?? 0;
    if (ch === '"') {
      escaped += '\\"';
    } else if (ch === "\\") {
      escaped += "\\\\";
    } else if (ch === "\n") {
      escaped += "\\n";
    } else if (ch === "\r") {
      escaped += "\\r";
    } else if (ch === "\t") {
      escaped += "\\t";
    } else if (ch === "<") {
      escaped += "\\u003c";
    } else if (ch === ">") {
      escaped += "\\u003e";
    } else if (ch === "&") {
      escaped += "\\u0026";
    } else if (code < 0x20) {
      escaped += `\\u${code.toString(16).padStart(4, "0")}`;
    } else {
      escaped += ch;
    }
  }
  escaped += '"';
  return escaped;
}

function formatRfc3339(date: ParsedDate): string {
  return `${pad(date.year, 4)}-${pad(date.month, 2)}-${pad(date.day, 2)}T${pad(date.hour, 2)}:${pad(date.minute, 2)}:${pad(date.second, 2)}Z`;
}

function formatRfc822(date: ParsedDate): string {
  const weekdays = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
  const months = [
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
  ];
  return `${weekdays[weekdayUtc(date.year, date.month, date.day)]}, ${pad(date.day, 2)} ${months[date.month - 1]} ${pad(date.year, 4)} ${pad(date.hour, 2)}:${pad(date.minute, 2)}:${pad(date.second, 2)} +0000`;
}

function pad(value: number, width: number): string {
  return String(value).padStart(width, "0");
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

function weekdayUtc(year: number, month: number, day: number): number {
  const table = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
  const y = month < 3 ? year - 1 : year;
  return (
    (((y + Math.trunc(y / 4) - Math.trunc(y / 100) + Math.trunc(y / 400) + table[month - 1] + day) %
      7) +
      7) %
    7
  );
}
