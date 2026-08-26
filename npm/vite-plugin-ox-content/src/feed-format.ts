/** RSS / Atom / JSON Feed string bodies used by `feeds.ts`. */

import type { FeedItemAttachment, FeedItemAuthor } from "./types";

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
  content?: string;
  loc: string;
  id?: string;
  date?: ParsedDate;
  authors?: FeedItemAuthor[];
  image?: string;
  attachments?: FeedItemAttachment[];
  language?: string;
}

export function generateRss(doc: FeedDocument, items: readonly FeedEntry[]): string {
  const dc = items.some((item) => item.authors?.length || item.language)
    ? ' xmlns:dc="http://purl.org/dc/elements/1.1/"'
    : "";
  let xml = `<?xml version="1.0" encoding="UTF-8"?>\n<rss version="2.0"${dc}>\n  <channel>\n    <title>`;
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
    xml += "</link>\n";
    xml += rssGuid(item);
    const description = item.content ?? item.description;
    if (description) {
      xml += "      <description>";
      xml += escapeXml(description);
      xml += "</description>\n";
    }
    if (item.date) {
      xml += `      <pubDate>${formatRfc822(item.date)}</pubDate>\n`;
    }
    xml += rssItemMeta(item);
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
    xml += `  <entry${item.language ? ` xml:lang="${escapeXml(item.language)}"` : ""}>\n    <title>`;
    xml += escapeXml(item.title);
    xml += '</title>\n    <link href="';
    xml += escapeXml(item.loc);
    xml += '"/>\n    <id>';
    xml += escapeXml(entryId(item));
    xml += `</id>\n    <updated>${item.date ? formatRfc3339(item.date) : updated}</updated>\n`;
    if (item.description) {
      xml += "    <summary>";
      xml += escapeXml(item.description);
      xml += "</summary>\n";
    }
    if (item.content) {
      xml += `    <content type="text">${escapeXml(item.content)}</content>\n`;
    }
    for (const author of item.authors ?? []) {
      xml += atomAuthor(author);
    }
    for (const attachment of item.attachments ?? []) {
      xml += atomAttachment(attachment);
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
    json += jsonString(entryId(item));
    json += ',\n      "url": ';
    json += jsonString(item.loc);
    json += ',\n      "title": ';
    json += jsonString(item.title);
    if (item.description && item.content) {
      json += ',\n      "summary": ';
      json += jsonString(item.description);
    }
    if (item.description || item.content) {
      json += ',\n      "content_text": ';
      json += jsonString(item.content ?? item.description);
    }
    if (item.date) {
      json += ',\n      "date_published": ';
      json += jsonString(formatRfc3339(item.date));
    }
    json += jsonItemMeta(item);
    json += "\n    }";
  });
  json += "\n  ]\n}\n";
  return json;
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

function entryId(item: FeedEntry): string {
  return item.id ?? item.loc;
}

function rssGuid(item: FeedEntry): string {
  const id = entryId(item);
  const attr = id === item.loc ? "" : ' isPermaLink="false"';
  return `      <guid${attr}>${escapeXml(id)}</guid>\n`;
}

function rssItemMeta(item: FeedEntry): string {
  let xml = "";
  for (const author of item.authors ?? []) {
    xml += `      <dc:creator>${escapeXml(author.name)}</dc:creator>\n`;
  }
  if (item.language) {
    xml += `      <dc:language>${escapeXml(item.language)}</dc:language>\n`;
  }
  for (const attachment of item.attachments ?? []) {
    xml += rssEnclosure(attachment);
  }
  return xml;
}

function rssEnclosure(attachment: FeedItemAttachment): string {
  return `      <enclosure${xmlAttrs([
    ["url", attachment.url],
    ["type", attachment.mimeType],
    ["length", attachment.sizeInBytes],
  ])}/>\n`;
}

function atomAuthor(author: FeedItemAuthor): string {
  const uri = author.url ? `\n      <uri>${escapeXml(author.url)}</uri>` : "";
  return `    <author>\n      <name>${escapeXml(author.name)}</name>${uri}\n    </author>\n`;
}

function atomAttachment(attachment: FeedItemAttachment): string {
  return `    <link${xmlAttrs([
    ["rel", "enclosure"],
    ["href", attachment.url],
    ["type", attachment.mimeType],
    ["length", attachment.sizeInBytes],
    ["title", attachment.title],
  ])}/>\n`;
}

function xmlAttrs(attrs: Array<[string, string | number | undefined]>): string {
  return attrs
    .flatMap(([key, value]) => (value == null ? [] : [` ${key}="${escapeXml(String(value))}"`]))
    .join("");
}

function jsonItemMeta(item: FeedEntry): string {
  let json = "";
  if (item.authors?.length) {
    json += ',\n      "authors": [';
    json += item.authors.map(jsonAuthor).join(", ");
    json += "]";
  }
  if (item.image) {
    json += ',\n      "image": ';
    json += jsonString(item.image);
  }
  if (item.language) {
    json += ',\n      "language": ';
    json += jsonString(item.language);
  }
  if (item.attachments?.length) {
    json += ',\n      "attachments": [';
    json += item.attachments.map(jsonAttachment).join(", ");
    json += "]";
  }
  return json;
}

function jsonAuthor(author: FeedItemAuthor): string {
  const url = author.url ? `, "url": ${jsonString(author.url)}` : "";
  return `{\n        "name": ${jsonString(author.name)}${url}\n      }`;
}

function jsonAttachment(attachment: FeedItemAttachment): string {
  const fields = [
    `"url": ${jsonString(attachment.url)}`,
    attachment.mimeType ? `"mime_type": ${jsonString(attachment.mimeType)}` : "",
    attachment.title ? `"title": ${jsonString(attachment.title)}` : "",
    numberJsonField("size_in_bytes", attachment.sizeInBytes),
    numberJsonField("duration_in_seconds", attachment.durationInSeconds),
  ].filter(Boolean);
  return `{\n        ${fields.join(",\n        ")}\n      }`;
}

function numberJsonField(name: string, value: number | undefined): string {
  return value == null ? "" : `"${name}": ${String(value)}`;
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
