import { Buffer } from "node:buffer";
import { describe, expect, it } from "vite-plus/test";
import {
  collectGitHubRepos,
  collectGitHubSources,
  isSafeGitHubRepo,
  parseGitHubPermalink,
  transformGitHub,
} from "./plugins/github";
import { transformBuiltinEmbeds } from "./plugins";
import { collectOgpUrls, isSafeOgpUrl, transformOgp } from "./plugins/ogp";

describe("builtin embed input hardening", () => {
  it("accepts only safe GitHub repo references", async () => {
    expect(isSafeGitHubRepo("ubugeeei-prod/ox-content")).toBe(true);
    expect(isSafeGitHubRepo("../secret")).toBe(false);
    expect(isSafeGitHubRepo("owner/repo?tab=readme")).toBe(false);

    await expect(
      collectGitHubRepos(
        '<GitHub repo="ubugeeei-prod/ox-content"></GitHub><GitHub repo="../secret"></GitHub>',
      ),
    ).resolves.toEqual(["ubugeeei-prod/ox-content"]);

    const html = await transformGitHub(
      '<GitHub repo="../secret"></GitHub>',
      new Map([["../secret", null]]),
    );
    expect(html).toMatchSnapshot();
  });

  it("accepts GitHub source permalinks and loc ranges", async () => {
    const permalink =
      "https://github.com/ubugeeei-prod/ox-content/blob/278098b/npm/vite-plugin-ox-content/src/plugins/github.ts#L10-L12";
    expect(parseGitHubPermalink(permalink)).toMatchObject({
      repo: "ubugeeei-prod/ox-content",
      ref: "278098b",
      path: "npm/vite-plugin-ox-content/src/plugins/github.ts",
      lines: { start: 10, end: 12 },
    });
    expect(
      parseGitHubPermalink("https://example.com/ubugeeei-prod/ox-content/blob/main/a.ts"),
    ).toBe(null);

    await expect(
      collectGitHubSources(
        `<GitHub permalink="${permalink}"></GitHub><GitHub repo="ubugeeei-prod/ox-content" path="README.md" ref="main" loc="1-2"></GitHub>`,
      ),
    ).resolves.toMatchObject([
      { repo: "ubugeeei-prod/ox-content", lines: { start: 10, end: 12 } },
      { repo: "ubugeeei-prod/ox-content", path: "README.md", lines: { start: 1, end: 2 } },
    ]);
  });

  it("expands GitHub source permalinks into code cards", async () => {
    const originalFetch = globalThis.fetch;
    globalThis.fetch = async () =>
      ({
        ok: true,
        status: 200,
        json: async () => ({
          type: "file",
          encoding: "base64",
          content: Buffer.from("const first = 1;\nconst second = 2;\nconst third = 3;\n").toString(
            "base64",
          ),
          size: 48,
          html_url: "https://github.com/acme/project/blob/abc123/src/index.ts#L2-L3",
        }),
      }) as Response;

    try {
      const html = await transformGitHub(
        '<GitHub permalink="https://github.com/acme/project/blob/abc123/src/index.ts#L2-L3"></GitHub>',
        undefined,
        { cache: false },
      );

      expect(html).toMatchSnapshot();
    } finally {
      globalThis.fetch = originalFetch;
    }
  });

  it("accepts only public http OGP URLs", async () => {
    expect(isSafeOgpUrl("https://example.com/post")).toBe(true);
    expect(isSafeOgpUrl("http://127.0.0.1/admin")).toBe(false);
    expect(isSafeOgpUrl("http://[::1]/admin")).toBe(false);
    expect(isSafeOgpUrl("https://fcdomain.example/post")).toBe(true);
    expect(isSafeOgpUrl("javascript:alert(1)")).toBe(false);

    await expect(
      collectOgpUrls(
        '<OgCard url="https://example.com/post"></OgCard><OgCard url="http://127.0.0.1/admin"></OgCard>',
      ),
    ).resolves.toEqual(["https://example.com/post"]);

    const html = await transformOgp(
      '<OgCard url="javascript:alert(1)"></OgCard>',
      new Map([["javascript:alert(1)", null]]),
    );
    expect(html).toMatchSnapshot();
  });

  it("runs GitHub and Open Graph embeds through the shared builtin transform", async () => {
    const html = await transformBuiltinEmbeds(
      '<GitHub repo="../secret"></GitHub><OgCard url="javascript:alert(1)"></OgCard>',
      {
        github: {},
        openGraph: {},
      },
    );

    expect(html).toMatchSnapshot();
  });

  it("does not let self-closing embed tags swallow trailing content", async () => {
    const html = await transformBuiltinEmbeds(
      '<GitHub repo="../secret" />\n<p>after github</p>\n<OgCard url="javascript:alert(1)" />\n<p>after ogp</p>',
      {
        github: {},
        openGraph: {},
      },
    );

    expect(html).toContain("after github");
    expect(html).toContain("after ogp");
    expect(html).toMatchSnapshot();
  });

  it("can disable builtin embeds", async () => {
    const input = '<GitHub repo="../secret"></GitHub><OgCard url="javascript:alert(1)"></OgCard>';
    await expect(
      transformBuiltinEmbeds(input, {
        github: false,
        openGraph: false,
      }),
    ).resolves.toBe(input);
  });

  it("keeps package-manager tabs opt-in", async () => {
    const input = "<pm>npm install -D vite</pm>";

    await expect(
      transformBuiltinEmbeds(input, {
        github: false,
        openGraph: false,
      }),
    ).resolves.toBe(input);

    const html = await transformBuiltinEmbeds(input, {
      github: false,
      openGraph: false,
      pm: {},
    });
    expect(html).toMatchSnapshot();
  });

  it("renders opt-in static media embeds through the shared builtin transform", async () => {
    const html = await transformBuiltinEmbeds(
      [
        '<Spotify url="https://open.spotify.com/track/abc123"></Spotify>',
        '<Tweet id="123">static text</Tweet>',
      ].join(""),
      {
        github: false,
        openGraph: false,
        spotify: true,
        twitter: true,
      },
    );

    expect(html).toMatchSnapshot();
  });
});
