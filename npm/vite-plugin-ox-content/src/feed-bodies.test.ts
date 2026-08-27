import { describe, expect, it } from "vite-plus/test";
import { importNapiModuleSync } from "./napi";
import type { FeedDocument } from "./feed-format";

/**
 * The feed corpus, now asserted against the native renderer alone.
 *
 * This began as a parity check between `feed-format.ts` and
 * `ox_content_ssg::generate_feeds`. The TypeScript renderer is gone — it was a
 * second implementation kept in step by this file, and it had already drifted
 * once (#1074: six item fields existed only in TypeScript) while its sibling
 * date parser drifted far enough to be wrong on live input (#1068).
 *
 * The corpus outlives the comparison: every field that drift touched is still
 * exercised here, now against recorded bodies, so a change to the Rust side
 * shows up as a reviewable diff.
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

interface Bodies {
  rssXml?: string;
  atomXml?: string;
  jsonFeed?: string;
}

function generate(options: Record<string, unknown>, items: unknown[]): Bodies {
  const napi = importNapiModuleSync() as unknown as {
    generateFeedBodies(options: Record<string, unknown>, items: unknown[]): Bodies;
  };
  return napi.generateFeedBodies(options, items);
}

function nativeBodies(): Bodies {
  return generate(
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

describe("feed bodies", () => {
  it("renders the RSS body", () => {
    expect(nativeBodies().rssXml).toMatchSnapshot();
  });

  it("renders the Atom body", () => {
    expect(nativeBodies().atomXml).toMatchSnapshot();
  });

  it("renders the JSON Feed body", () => {
    expect(nativeBodies().jsonFeed).toMatchSnapshot();
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

const CHANNEL = {
  language: "en",
  image: "https://example.com/logo.png",
  favicon: "https://example.com/icon.png",
  copyright: "© 2026 Docs & Co",
};

function nativeWithChannel(items: unknown[]): Bodies {
  return generate(
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
      ...CHANNEL,
    },
    items,
  );
}

const PLAIN = [
  { title: "A", loc: "https://example.com/a", description: "d", date: "2024-01-02T03:04:05Z" },
];
const LOCALISED = [
  {
    title: "B",
    loc: "https://example.com/b",
    description: "d",
    language: "ja",
    date: "2024-01-02T03:04:05Z",
  },
];

describe("feed channel metadata", () => {
  it("carries title, language, image, favicon, and copyright into every format", () => {
    const bodies = nativeWithChannel(PLAIN);
    expect(bodies.rssXml).toMatchSnapshot();
    expect(bodies.atomXml).toMatchSnapshot();
    expect(bodies.jsonFeed).toMatchSnapshot();
  });

  it("keeps channel metadata when an item carries its own language", () => {
    const bodies = nativeWithChannel(LOCALISED);
    expect(bodies.rssXml).toContain("<copyright>");
    expect(bodies.jsonFeed).toContain('"favicon"');
  });

  // The renderer this replaced patched the finished string and anchored on the
  // literal `  <entry>`, which a localised entry does not match — so its icon,
  // logo, and rights landed *after* every entry whenever some item carried an
  // `xml:lang`. Generating the metadata in place puts it ahead either way.
  it("keeps Atom channel metadata ahead of the entries regardless of item language", () => {
    for (const items of [PLAIN, LOCALISED]) {
      const atom = nativeWithChannel(items).atomXml ?? "";
      expect(atom.indexOf("<icon>")).toBeGreaterThan(-1);
      expect(atom.indexOf("<icon>")).toBeLessThan(atom.indexOf("<entry"));
      expect(atom.indexOf("<rights>")).toBeLessThan(atom.indexOf("<entry"));
    }
  });
});
