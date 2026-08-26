import { afterEach, describe, expect, it } from "vite-plus/test";
import { resolveBuiltinEmbedOptions } from "../resolve-options";
import { renderMarkdown } from "../render-markdown";
import { normalizeSelfClosingEmbeds, transformAllPlugins, transformBuiltinEmbeds } from ".";
import { clearProviderArticleCache } from "./provider-articles";
import { clearProviderPackageCache } from "./provider-packages";
import { clearProviderPlaygroundCache } from "./provider-playgrounds";

const originalFetch = globalThis.fetch;

afterEach(() => {
  globalThis.fetch = originalFetch;
  clearProviderArticleCache();
  clearProviderPackageCache();
  clearProviderPlaygroundCache();
});

describe("provider-grade static embed cards", () => {
  it("renders provider cards through transformAllPlugins", async () => {
    const html = await transformAllPlugins(
      [
        '<GoogleMaps url="https://www.google.com/maps/place/Tokyo+Station/" place="Tokyo Station" address="1 Chome Marunouchi" />',
        '<Qiita url="https://qiita.com/ubugeeei/items/abcdef123456" title="Rust docs pipeline" author="ubugeeei" tags="Rust, Markdown">Static cards.</Qiita>',
        '<Zenn url="https://zenn.dev/ubugeeei/articles/ox-content" title="Ox Content notes" />',
        '<NpmPackage url="https://www.npmjs.com/package/vite" version="7.0.0" license="MIT" />',
        '<CratesIo url="https://crates.io/crates/serde" version="1.0.0" />',
        '<PyPI url="https://pypi.org/project/requests" version="2.32.0" />',
        '<DockerHub url="https://hub.docker.com/_/nginx" downloads="123456" />',
        '<CodePen url="https://codepen.io/ubugeeei/pen/abc123" title="Card demo" />',
        '<JSFiddle url="https://jsfiddle.net/ubugeeei/abc123/2/" title="Fiddle demo" />',
        '<Observable url="https://observablehq.com/@d3/bar-chart" title="Bar chart" />',
        '<Discord url="https://discord.gg/abc123" server="Ox Content" channel="announcements" />',
        '<Mastodon url="https://mastodon.social/@docs/111" author="@docs@mastodon.social">Fediverse release note.</Mastodon>',
        '<Facebook url="https://www.facebook.com/example/posts/123" title="Launch note" />',
        '<Threads url="https://www.threads.net/@example/post/abc" author="@example" />',
        '<Instagram url="https://www.instagram.com/p/abc123/" author="@example" image="https://cdn.example.com/photo.jpg" />',
      ].join("\n"),
      {
        github: false,
        openGraph: false,
        mermaid: false,
        googleMaps: true,
        qiita: { fetch: false },
        zenn: { fetch: false },
        packageRegistry: { fetch: false },
        playgrounds: { fetch: false },
        discord: true,
        fediverse: true,
        facebook: true,
        threads: true,
        instagram: true,
      },
    );

    expect(html).toContain("ox-provider-card--google-maps");
    expect(html).toContain("ox-provider-card--qiita");
    expect(html).toContain("ox-provider-card--zenn");
    expect(html).toContain("ox-provider-card--npm");
    expect(html).toContain("ox-provider-card--crates-io");
    expect(html).toContain("ox-provider-card--pypi");
    expect(html).toContain("ox-provider-card--docker-hub");
    expect(html).toContain("ox-provider-card--codepen");
    expect(html).toContain("ox-provider-card--jsfiddle");
    expect(html).toContain("ox-provider-card--observable");
    expect(html).toContain("ox-provider-card--discord");
    expect(html).toContain("ox-provider-card--mastodon");
    expect(html).toContain("ox-provider-card--facebook");
    expect(html).toContain("ox-provider-card--threads");
    expect(html).toContain("ox-provider-card--instagram");
    expect(html).not.toMatch(
      /<\/(?:GoogleMaps|Qiita|Zenn|NpmPackage|CratesIo|PyPI|DockerHub|CodePen|JSFiddle|Observable|Discord|Mastodon|Facebook|Threads|Instagram)>/,
    );
  });

  it("defaults provider cards to build-time metadata fetch", () => {
    expect(resolveBuiltinEmbedOptions({ qiita: true }).qiita).toEqual({});
    expect(resolveBuiltinEmbedOptions({ zenn: { fetch: false } }).zenn).toEqual({
      fetch: false,
    });
    expect(resolveBuiltinEmbedOptions({ packageRegistry: true }).packageRegistry).toEqual({});
    expect(resolveBuiltinEmbedOptions({ playgrounds: true }).playgrounds).toEqual({});
  });

  it("enriches Qiita and Zenn article metadata before static rendering", async () => {
    let requests = 0;
    globalThis.fetch = async (input) => {
      const url = requestUrl(input);
      requests += 1;
      if (url === "https://qiita.com/api/v2/items/abcdef123456") {
        return okJson({
          title: "Qiita fetched title",
          body: "# Intro\nFetched Qiita excerpt with `code`.",
          created_at: "2026-08-25T03:04:05+09:00",
          likes_count: 42,
          comments_count: 7,
          tags: [{ name: "Rust" }, { name: "Markdown" }],
          user: {
            id: "ubugeeei",
            profile_image_url: "https://cdn.qiita.com/avatar.png",
          },
        });
      }
      if (url === "https://zenn.dev/api/articles/ox-content") {
        return okJson({
          article: {
            title: "Zenn fetched title",
            published_at: "2026-08-26T12:00:00.000+09:00",
            liked_count: 12,
            comments_count: 3,
            og_image_url: "https://res.cloudinary.com/zenn/image/upload/card.png",
            topics: [{ name: "Vite" }],
            user: {
              username: "docs",
              avatar_small_url: "https://static.zenn.studio/avatar.png",
            },
          },
        });
      }
      return new Response("{}", { status: 404 });
    };

    const input = [
      '<Qiita url="https://qiita.com/ubugeeei/items/abcdef123456"></Qiita>',
      '<Zenn url="https://zenn.dev/docs/articles/ox-content"></Zenn>',
    ].join("\n");
    const html = await transformBuiltinEmbeds(input, {
      github: false,
      openGraph: false,
      qiita: true,
      zenn: true,
    });

    expect(html).toContain("Qiita fetched title");
    expect(html).toContain("Fetched Qiita excerpt with code.");
    expect(html).toContain("Zenn fetched title");
    expect(html).toContain("ox-provider-card__image");
    await transformBuiltinEmbeds(input, {
      github: false,
      openGraph: false,
      qiita: true,
      zenn: true,
    });
    expect(requests).toBe(2);
  });

  it("keeps provider cards literal when disabled or rejected", async () => {
    const input = '<Qiita url="https://qiita.com/ubugeeei/items/abcdef123456"></Qiita>';
    await expect(
      transformBuiltinEmbeds(input, {
        github: false,
        openGraph: false,
        qiita: false,
      }),
    ).resolves.toBe(input);

    await expect(
      transformBuiltinEmbeds(
        '<Instagram url="https://user:pass@instagram.com/p/abc123/"></Instagram>',
        {
          github: false,
          openGraph: false,
          instagram: true,
        },
      ),
    ).resolves.toBe('<Instagram url="https://user:pass@instagram.com/p/abc123/"></Instagram>');
  });

  it("renders link-only package cards when metadata fetch fails", async () => {
    const originalWarn = console.warn;
    console.warn = () => {};
    globalThis.fetch = async () => new Response("{}", { status: 404 });
    try {
      const html = await transformBuiltinEmbeds(
        '<NpmPackage url="https://www.npmjs.com/package/private"></NpmPackage>',
        {
          github: false,
          openGraph: false,
          packageRegistry: true,
        },
      );

      expect(html).toContain("ox-provider-card--npm");
      expect(html).toContain("private");
      expect(html).toContain("Open package");
    } finally {
      console.warn = originalWarn;
    }
  });

  it("normalizes self-closing provider authoring forms", () => {
    expect(
      normalizeSelfClosingEmbeds('<GoogleMaps url="https://www.google.com/maps/place/Tokyo" />'),
    ).toBe('<GoogleMaps url="https://www.google.com/maps/place/Tokyo"></GoogleMaps>');
  });

  it("renders provider cards in MDX as built-ins instead of islands", async () => {
    const result = await renderMarkdown(
      '<Qiita url="https://qiita.com/ubugeeei/items/abcdef123456" title="Article" />',
      "/virtual/article.mdx",
      {
        ssg: false,
        frontmatter: false,
        highlight: false,
        embeds: {
          github: false,
          openGraph: false,
          qiita: { fetch: false },
        },
        ogViewer: false,
        search: false,
        toc: false,
        mdx: true,
      },
    );

    expect(result.html).toContain("ox-provider-card--qiita");
    expect(result.html).not.toContain('data-ox-island="Qiita"');
  });
});

function requestUrl(input: Parameters<typeof fetch>[0]): string {
  if (typeof input === "string") return input;
  if (input instanceof URL) return input.href;
  return input.url;
}

function okJson(value: unknown): Response {
  return new Response(JSON.stringify(value), {
    status: 200,
    headers: { "content-type": "application/json" },
  });
}
