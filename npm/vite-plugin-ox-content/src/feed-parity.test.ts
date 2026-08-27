import { describe, expect, it } from "vite-plus/test";
import { generateAtom, generateJson, generateRss } from "./feed-format";
import { parseDate } from "./feed-date";
import { importNapiModuleSync } from "./napi";
import type { FeedDocument, FeedEntry } from "./feed-format";

/**
 * The TypeScript writer and `ox_content_ssg::generate_feeds` both produce feed
 * bodies, and `feed-format.ts` says it follows the Rust one. Nothing checked
 * that, and by the time #1074 was filed the two had drifted: six item fields
 * existed only in TypeScript.
 *
 * This corpus exercises every one of them, so the two cannot part ways again
 * without a red test.
 */
const DOC: FeedDocument = {
  siteName: "Docs & Co <x>",
  siteDescription: "Example docs",
  home: "https://example.com/",
  atomUrl: "https://example.com/atom.xml",
  jsonUrl: "https://example.com/feed.json",
};

const ITEMS = [
  {
    title: "Plain",
    loc: "https://example.com/a",
    description: "desc",
    date: "2024-01-02T03:04:05Z",
  },
  {
    // Every field the Rust side used to lack, plus markup that has to escape.
    title: "Rich & <escaped>",
    loc: "https://example.com/b",
    id: "urn:custom:b",
    description: "short summary",
    content: "the full body\nwith a newline",
    language: "ja",
    image: "https://example.com/img.png",
    date: "2023-06-07T08:09:10Z",
    authors: [{ name: "Jane <j>" }, { name: "Kim", url: "https://example.com/kim" }],
    attachments: [
      {
        url: "https://example.com/a.mp3",
        mimeType: "audio/mpeg",
        sizeInBytes: 1234,
        durationInSeconds: 60,
      },
      { url: "https://example.com/b.png", title: "Cover" },
    ],
  },
  // `content` with no `description`: the two are not interchangeable.
  {
    title: "ContentOnly",
    loc: "https://example.com/c",
    content: "body only",
    date: "2022-02-02T00:00:00Z",
  },
  { title: "NoDate", loc: "https://example.com/d", description: "no date" },
];

function nativeBodies() {
  const napi = importNapiModuleSync() as unknown as {
    generateFeedBodies(
      options: Record<string, unknown>,
      items: unknown[],
    ): { rssXml?: string; atomXml?: string; jsonFeed?: string };
  };
  return napi.generateFeedBodies(
    {
      enabled: true,
      siteUrl: "https://example.com",
      siteName: DOC.siteName,
      siteDescription: DOC.siteDescription,
      homePageUrl: DOC.home,
      rssUrl: "https://example.com/feed.xml",
      atomUrl: DOC.atomUrl,
      jsonUrl: DOC.jsonUrl,
      formats: ["rss", "atom", "json"],
      limit: 20,
    },
    ITEMS,
  );
}

function tsEntries(): FeedEntry[] {
  return ITEMS.map((item) => ({
    ...item,
    date: item.date ? parseDate(item.date) : undefined,
  })) as FeedEntry[];
}

describe("feed generation parity", () => {
  it("produces the same RSS body in both implementations", () => {
    expect(nativeBodies().rssXml).toBe(generateRss(DOC, tsEntries()));
  });

  it("produces the same Atom body in both implementations", () => {
    expect(nativeBodies().atomXml).toBe(generateAtom(DOC, tsEntries()));
  });

  it("produces the same JSON Feed body in both implementations", () => {
    expect(nativeBodies().jsonFeed).toBe(generateJson(DOC, tsEntries()));
  });

  it("carries the fields that used to exist only in TypeScript", () => {
    const { rssXml = "", atomXml = "", jsonFeed = "" } = nativeBodies();

    expect(rssXml).toContain("<dc:creator>Jane &lt;j&gt;</dc:creator>");
    expect(rssXml).toContain('<guid isPermaLink="false">urn:custom:b</guid>');
    expect(rssXml).toContain('<enclosure url="https://example.com/a.mp3"');
    expect(atomXml).toContain('<entry xml:lang="ja">');
    expect(atomXml).toContain('<content type="text">');
    expect(atomXml).toContain("<uri>https://example.com/kim</uri>");
    expect(jsonFeed).toContain('"image": "https://example.com/img.png"');
    expect(jsonFeed).toContain('"duration_in_seconds": 60');
  });
});
