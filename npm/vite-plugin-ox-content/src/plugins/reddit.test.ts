import { afterEach, describe, expect, it } from "vite-plus/test";
import { resolveBuiltinEmbedOptions } from "../resolve-options";
import { normalizeSelfClosingEmbeds, transformBuiltinEmbeds } from ".";
import { isReservedBuiltinComponent } from "./embed-transform";
import {
  clearRedditCache,
  parseRedditPostReference,
  resolveRedditEmbedOptions,
  transformRedditEmbeds,
} from "./reddit";
import { redditCardSnapshots } from "./reddit.snapshots";

const originalFetch = globalThis.fetch;

afterEach(() => {
  globalThis.fetch = originalFetch;
  clearRedditCache();
});

describe("Reddit embeds", () => {
  it("normalizes canonical reddit.com and share URLs safely", () => {
    expect(
      parseRedditPostReference(
        "https://old.reddit.com/r/webdev/comments/ABC123/release_notes/?utm_source=share",
      ),
    ).toMatchObject({
      id: "abc123",
      subreddit: "webdev",
      slug: "release_notes",
      url: "https://www.reddit.com/r/webdev/comments/abc123/release_notes/",
      apiUrl: "https://www.reddit.com/comments/abc123.json?raw_json=1",
    });
    expect(parseRedditPostReference("https://redd.it/ABC123?share_id=ignored")).toMatchObject({
      id: "abc123",
      url: "https://www.reddit.com/comments/abc123/",
    });
    expect(parseRedditPostReference("https://www.reddit.com/gallery/ABC123")).toMatchObject({
      id: "abc123",
      url: "https://www.reddit.com/comments/abc123/",
      apiUrl: "https://www.reddit.com/comments/abc123.json?raw_json=1",
    });
    expect(parseRedditPostReference("https://www.reddit.com/r/webdev/s/AbC_123")).toEqual({
      subreddit: "webdev",
      shareId: "AbC_123",
      url: "https://www.reddit.com/r/webdev/s/AbC_123/",
    });
    expect(parseRedditPostReference("http://www.reddit.com/r/webdev/comments/abc123/x")).toBeNull();
    expect(parseRedditPostReference("https://example.com/r/webdev/comments/abc123/x")).toBeNull();
    expect(
      parseRedditPostReference("https://user:pass@reddit.com/r/webdev/comments/abc123/x"),
    ).toBeNull();
  });

  it("defaults enabled cards to build-time metadata fetch", () => {
    expect(resolveRedditEmbedOptions({}).fetch).toBe(true);
    expect(resolveRedditEmbedOptions({ fetch: false }).fetch).toBe(false);
    expect(resolveBuiltinEmbedOptions(undefined).reddit).toBe(false);
    expect(resolveBuiltinEmbedOptions(false).reddit).toBe(false);
    expect(resolveBuiltinEmbedOptions({ reddit: true }).reddit).toEqual({});
    expect(resolveBuiltinEmbedOptions({ reddit: { fetch: false } }).reddit).toEqual({
      fetch: false,
    });
  });

  it("registers Reddit as a reserved self-closing embed", () => {
    expect(isReservedBuiltinComponent("Reddit")).toBe(true);
    expect(normalizeSelfClosingEmbeds('<Reddit url="https://redd.it/abc123" />')).toBe(
      '<Reddit url="https://redd.it/abc123"></Reddit>',
    );
  });

  it("renders a fetched text post and reuses the in-memory cache", async () => {
    let requests = 0;
    globalThis.fetch = async (input) => {
      requests += 1;
      expect(String(input)).toBe("https://www.reddit.com/comments/abc123.json?raw_json=1");
      return okJson(
        listing({
          title: "Ship static Reddit cards <without scripts>",
          selftext: "The release notes include static cards.\nNo widget script.",
          subreddit: "webdev",
          author: "octo_user",
          score: 15420,
          num_comments: 87,
          created_utc: 1754092800,
          permalink: "/r/webdev/comments/abc123/release_notes/",
          url: "https://www.reddit.com/r/webdev/comments/abc123/release_notes/",
        }),
      );
    };

    const input =
      '<Reddit url="https://old.reddit.com/r/webdev/comments/abc123/release_notes/?utm_source=share"></Reddit>';
    expect(await transformRedditEmbeds(input)).toBe(redditCardSnapshots.textPost);
    await expect(transformRedditEmbeds(input)).resolves.toContain("ox-reddit-card");
    expect(requests).toBe(1);
  });

  it("renders image posts with an original link", async () => {
    globalThis.fetch = async () =>
      okJson(
        listing({
          title: "The new renderer output",
          subreddit: "oxcontent",
          author: "image_author",
          score: 999,
          num_comments: 12,
          created_utc: 1754179200,
          permalink: "/r/oxcontent/comments/img999/the_new_renderer_output/",
          post_hint: "image",
          url_overridden_by_dest: "https://i.redd.it/render-output.png",
          preview: {
            images: [
              {
                source: {
                  url: "https://preview.redd.it/render-output.png?width=960&format=png",
                  width: 960,
                  height: 540,
                },
              },
            ],
          },
        }),
      );

    const html = await transformRedditEmbeds('<Reddit url="https://redd.it/img999"></Reddit>', {
      cache: false,
    });
    expect(html).toBe(redditCardSnapshots.imagePost);
  });

  it("falls back for unavailable posts and share links without fetchable ids", async () => {
    let requests = 0;
    globalThis.fetch = async () => {
      requests += 1;
      return { ok: false, status: 404, json: async () => null } as Response;
    };

    const html = await transformRedditEmbeds(
      [
        '<Reddit url="https://www.reddit.com/r/webdev/comments/missing/gone"></Reddit>',
        '<Reddit url="https://www.reddit.com/r/webdev/s/AbC_123"></Reddit>',
      ].join(""),
      { cache: false },
    );
    expect(html).toBe(redditCardSnapshots.unavailablePost);
    expect(requests).toBe(1);
  });

  it("hardens unsupported or unsafe inputs without fetching", async () => {
    let requests = 0;
    globalThis.fetch = async () => {
      requests += 1;
      throw new Error("should not fetch");
    };

    const html = await transformRedditEmbeds(
      [
        '<Reddit url="javascript:alert(1)"></Reddit>',
        '<Reddit url="https://example.com/r/webdev/comments/abc123/x"></Reddit>',
      ].join(""),
    );
    expect(html).toBe(redditCardSnapshots.unsafeInputs);
    expect(requests).toBe(0);
  });

  it("runs through the shared builtin transform only when Reddit is enabled", async () => {
    const input =
      '<Reddit url="https://www.reddit.com/r/webdev/comments/abc123/release_notes/"></Reddit>';
    await expect(
      transformBuiltinEmbeds(input, { github: false, openGraph: false, reddit: false }),
    ).resolves.toBe(input);

    globalThis.fetch = async () =>
      okJson(
        listing({
          title: "Enabled from builtin options",
          subreddit: "webdev",
          author: "enabled_user",
          score: 1,
          num_comments: 1,
          created_utc: 1754092800,
          permalink: "/r/webdev/comments/abc123/release_notes/",
        }),
      );

    await expect(
      transformBuiltinEmbeds(input, { github: false, openGraph: false, reddit: true }),
    ).resolves.toContain("Enabled from builtin options");
  });

  it("can render a link-only fallback without network access", async () => {
    let requests = 0;
    globalThis.fetch = async () => {
      requests += 1;
      throw new Error("should not fetch");
    };

    const html = await transformRedditEmbeds('<Reddit id="abc123"></Reddit>', { fetch: false });
    expect(html).toBe(redditCardSnapshots.fetchDisabled);
    expect(requests).toBe(0);
  });
});

function okJson(value: unknown): Response {
  return { ok: true, status: 200, json: async () => value } as Response;
}

function listing(data: Record<string, unknown>): unknown {
  return [
    {
      kind: "Listing",
      data: {
        children: [{ kind: "t3", data }],
      },
    },
  ];
}
