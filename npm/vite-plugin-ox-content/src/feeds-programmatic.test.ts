import { afterEach, describe, expect, it } from "vite-plus/test";
import * as fs from "node:fs/promises";
import * as os from "node:os";
import * as path from "node:path";
import { createDocsResolvedOptions } from "../test/fixtures/docs-fixture";
import { renderFeedFiles, resolveFeedsOptions } from "./feeds";
import { buildSsg } from "./ssg";
import type { FeedItemsResolveContext } from "./types";

type MediaItem = {
  title: string;
  link: string;
  pubDate: string;
  lang: string;
  kind?: "podcast" | "video";
  playlist?: boolean;
};

const mediaJson: MediaItem[] = [
  {
    title: "Guest appearance",
    link: "https://media.example.com/episode",
    pubDate: "2026-08-01",
    lang: "ja",
    kind: "podcast",
  },
  {
    title: "Playlist rollup",
    link: "https://media.example.com/playlist",
    pubDate: "2026-08-02",
    lang: "ja",
    kind: "video",
    playlist: true,
  },
];

const tempDirs: string[] = [];

afterEach(async () => {
  await Promise.all(tempDirs.splice(0).map((dir) => fs.rm(dir, { recursive: true, force: true })));
});

describe("programmatic feeds", () => {
  it("renders async media items without requiring an output directory", async () => {
    let context: FeedItemsResolveContext | undefined;

    const result = await renderFeedFiles({
      base: "/",
      siteUrl: "https://ryoppippi.example",
      siteName: "ryoppippi.com",
      options: resolveFeedsOptions({
        media: {
          formats: ["rss", "atom", "json"],
          path: "/works/media",
          title: "Media | ryoppippi.com",
          language: "ja",
          items: async (input) => {
            context = input;
            return mediaJson
              .filter((item) => !item.playlist)
              .map((item) => ({
                title: item.title,
                url: item.link,
                id: `media:${item.link}`,
                date: item.pubDate,
                description: `${item.kind === "podcast" ? "Podcast" : "YouTube"} | ${item.title}`,
                author: { name: "ryoppippi", url: "https://ryoppippi.com" },
                language: item.lang,
              }));
          },
        },
      }),
    });

    expect(context).toMatchObject({ name: "media", path: "/works/media", base: "/" });
    expect(context?.outDir).toBeUndefined();
    expect(result.warning).toBeUndefined();
    expect(result.files.map((file) => [file.path, file.contentType])).toEqual([
      ["works/media/feed.xml", "application/rss+xml; charset=utf-8"],
      ["works/media/atom.xml", "application/atom+xml; charset=utf-8"],
      ["works/media/feed.json", "application/feed+json; charset=utf-8"],
    ]);
    expect(result.files[0]?.content).toContain("<title>Guest appearance</title>");
    expect(result.files[0]?.content).not.toContain("Playlist rollup");
  });

  it("writes async JSON-backed media items during SSG", async () => {
    const root = await makeSite();
    let context: FeedItemsResolveContext | undefined;

    const result = await buildSsg(
      createDocsResolvedOptions({
        feeds: resolveFeedsOptions({
          media: {
            formats: ["rss", "atom", "json"],
            path: "/works/media",
            title: "Media | ryoppippi.com",
            language: "ja",
            items: async (input) => {
              context = input;
              return mediaJson
                .filter((item) => !item.playlist)
                .map((item) => ({
                  title: item.title,
                  url: item.link,
                  id: `media:${item.link}`,
                  date: item.pubDate,
                  description: `${item.kind === "podcast" ? "Podcast" : "YouTube"} | ${item.title}`,
                  content: `Curated external media entry for ${item.title}.`,
                  author: { name: "ryoppippi", url: "https://ryoppippi.com" },
                  image: "https://media.example.com/cover.png",
                  attachments: [
                    {
                      url: "https://media.example.com/episode.mp3",
                      mimeType: "audio/mpeg",
                      title: "Episode audio",
                      sizeInBytes: 12345,
                      durationInSeconds: 600,
                    },
                  ],
                  language: item.lang,
                }));
            },
          },
        }),
        ssg: {
          ...createDocsResolvedOptions().ssg,
          bare: true,
          siteUrl: "https://ryoppippi.example",
        },
      }),
      root,
    );

    expect(context).toMatchObject({ name: "media", path: "/works/media", base: "/" });
    expect(result.files).toEqual(
      expect.arrayContaining([
        path.join(root, "dist", "works", "media", "feed.xml"),
        path.join(root, "dist", "works", "media", "atom.xml"),
        path.join(root, "dist", "works", "media", "feed.json"),
      ]),
    );

    const rss = await fs.readFile(path.join(root, "dist", "works", "media", "feed.xml"), "utf8");
    expect(rss).toContain("<title>Guest appearance</title>");
    expect(rss).toContain(
      '<guid isPermaLink="false">media:https://media.example.com/episode</guid>',
    );
    expect(rss).toContain("<dc:creator>ryoppippi</dc:creator>");
    expect(rss).toContain('<enclosure url="https://media.example.com/episode.mp3"');
    expect(rss).not.toContain("Playlist rollup");

    const atom = await fs.readFile(path.join(root, "dist", "works", "media", "atom.xml"), "utf8");
    expect(atom).toContain('<entry xml:lang="ja">');
    expect(atom).toContain("<id>media:https://media.example.com/episode</id>");
    expect(atom).toContain(
      '<content type="text">Curated external media entry for Guest appearance.</content>',
    );
    expect(atom).toContain('<link rel="enclosure" href="https://media.example.com/episode.mp3"');

    const json = JSON.parse(
      await fs.readFile(path.join(root, "dist", "works", "media", "feed.json"), "utf8"),
    ) as {
      items: Array<{
        id: string;
        url: string;
        summary: string;
        content_text: string;
        authors: Array<{ name: string; url: string }>;
        image: string;
        attachments: Array<{ url: string; mime_type: string; duration_in_seconds: number }>;
        language: string;
      }>;
    };
    expect(json.items).toEqual([
      expect.objectContaining({
        id: "media:https://media.example.com/episode",
        url: "https://media.example.com/episode",
        summary: "Podcast | Guest appearance",
        content_text: "Curated external media entry for Guest appearance.",
        authors: [{ name: "ryoppippi", url: "https://ryoppippi.com" }],
        image: "https://media.example.com/cover.png",
        language: "ja",
      }),
    ]);
    expect(json.items[0]?.attachments[0]).toMatchObject({
      url: "https://media.example.com/episode.mp3",
      mime_type: "audio/mpeg",
      duration_in_seconds: 600,
    });
    expect(result.errors).toEqual([]);
  });
});

async function makeSite(): Promise<string> {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "ox-content-programmatic-feeds-"));
  tempDirs.push(root);
  const file = path.join(root, "content", "index.md");
  await fs.mkdir(path.dirname(file), { recursive: true });
  await fs.writeFile(file, "---\ntitle: Home\n---\n# Home\n", "utf8");
  return root;
}
