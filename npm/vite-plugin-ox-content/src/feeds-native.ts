/**
 * Bridges the resolved feed inputs onto `generateFeedBodies`.
 *
 * The TypeScript renderer this replaces was a second implementation of
 * `ox_content_ssg::generate_feeds`, kept in step only by a parity test. The
 * two had already drifted once (#1074, six item fields missing on the Rust
 * side) and the sibling date parser drifted far enough to be wrong on live
 * input (#1068). One implementation cannot drift.
 */

import { importNapiModuleSync } from "./napi";
import type { FeedDocument, FeedEntry, ParsedDate } from "./feed-format";
import type { FeedFormat } from "./types";

interface NativeFeedBodies {
  rssXml?: string;
  atomXml?: string;
  jsonFeed?: string;
}

interface NativeFeedModule {
  generateFeedBodies(options: Record<string, unknown>, items: unknown[]): NativeFeedBodies;
}

/** Channel fields the formats emit when the standard has a matching slot. */
export interface FeedChannelExtras {
  language?: string;
  image?: string;
  favicon?: string;
  copyright?: string;
}

export function renderFeedBodies(
  doc: FeedDocument,
  items: readonly FeedEntry[],
  formats: readonly FeedFormat[],
  extras: FeedChannelExtras,
  siteUrl: string | undefined,
): NativeFeedBodies {
  const napi = importNapiModuleSync() as unknown as NativeFeedModule;
  return napi.generateFeedBodies(
    {
      enabled: true,
      siteUrl,
      siteName: doc.siteName,
      siteDescription: doc.siteDescription,
      homePageUrl: doc.home,
      // `feedDocument` derives the other two from this same prefix.
      rssUrl: doc.atomUrl.replace(/atom\.xml$/, "feed.xml"),
      atomUrl: doc.atomUrl,
      jsonUrl: doc.jsonUrl,
      formats: [...formats],
      // Selection already happened in TypeScript, which knows the publish-state
      // rules. Passing the count back keeps the native side from trimming twice.
      limit: items.length,
      ...extras,
    },
    items.map(nativeItem),
  );
}

/**
 * `FeedEntry` carries the date already parsed into UTC components; the native
 * side takes the string. Every component survives the round trip, so this is
 * the same instant the TypeScript renderer would have formatted.
 */
function nativeItem(item: FeedEntry): Record<string, unknown> {
  return {
    title: item.title,
    loc: item.loc,
    description: item.description,
    content: item.content,
    id: item.id,
    date: item.date ? isoFromParsed(item.date) : undefined,
    authors: item.authors,
    image: item.image,
    attachments: item.attachments,
    language: item.language,
  };
}

function isoFromParsed(date: ParsedDate): string {
  const pad = (value: number, width = 2) => String(value).padStart(width, "0");
  return `${pad(date.year, 4)}-${pad(date.month)}-${pad(date.day)}T${pad(date.hour)}:${pad(
    date.minute,
  )}:${pad(date.second)}Z`;
}
