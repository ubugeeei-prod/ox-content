import { escapeXml, jsonString } from "./feed-format";

/** Channel fields that formats emit when the standard has a matching slot. */
export interface FeedChannelMeta {
  title?: string;
  language?: string;
  image?: string;
  favicon?: string;
  copyright?: string;
  home: string;
}

export function channelMeta(
  doc: { siteName: string; home: string },
  options: {
    language?: string;
    image?: string;
    favicon?: string;
    copyright?: string;
  },
): FeedChannelMeta {
  return {
    title: doc.siteName,
    home: doc.home,
    language: options.language,
    image: options.image,
    favicon: options.favicon,
    copyright: options.copyright,
  };
}

/** Inserts RSS 2.0 language, copyright, and image after the channel description. */
export function applyRssMeta(xml: string, meta: FeedChannelMeta): string {
  const extras: string[] = [];
  if (meta.language) extras.push(`    <language>${escapeXml(meta.language)}</language>`);
  if (meta.copyright) extras.push(`    <copyright>${escapeXml(meta.copyright)}</copyright>`);
  if (meta.image) {
    extras.push(
      `    <image>\n      <url>${escapeXml(meta.image)}</url>\n      <title>${escapeXml(meta.title ?? "")}</title>\n      <link>${escapeXml(meta.home)}</link>\n    </image>`,
    );
  }
  return extras.length === 0
    ? xml
    : xml.replace("</description>\n", `</description>\n${extras.join("\n")}\n`);
}

/** Inserts Atom xml:lang, icon, logo, and rights. */
export function applyAtomMeta(xml: string, meta: FeedChannelMeta): string {
  let out = meta.language
    ? xml.replace(
        '<feed xmlns="http://www.w3.org/2005/Atom">',
        `<feed xmlns="http://www.w3.org/2005/Atom" xml:lang="${escapeXml(meta.language)}">`,
      )
    : xml;
  const extras: string[] = [];
  if (meta.favicon) extras.push(`  <icon>${escapeXml(meta.favicon)}</icon>`);
  if (meta.image) extras.push(`  <logo>${escapeXml(meta.image)}</logo>`);
  if (meta.copyright) extras.push(`  <rights>${escapeXml(meta.copyright)}</rights>`);
  if (extras.length === 0) return out;
  const block = `${extras.join("\n")}\n`;
  return out.includes("  <entry>")
    ? out.replace("  <entry>", `${block}  <entry>`)
    : out.replace("</feed>", `${block}</feed>`);
}

/** Inserts JSON Feed 1.1 language, icon, and favicon. */
export function applyJsonMeta(json: string, meta: FeedChannelMeta): string {
  const extras: string[] = [];
  if (meta.language?.trim()) extras.push(`  "language": ${jsonString(meta.language)}`);
  if (meta.image?.trim()) extras.push(`  "icon": ${jsonString(meta.image)}`);
  if (meta.favicon?.trim()) extras.push(`  "favicon": ${jsonString(meta.favicon)}`);
  return extras.length === 0
    ? json
    : json.replace(',\n  "items":', `,\n${extras.join(",\n")},\n  "items":`);
}
