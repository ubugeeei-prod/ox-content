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
    expect(isSafeGitHubRepo("ubugeeei/ox-content")).toBe(true);
    expect(isSafeGitHubRepo("../secret")).toBe(false);
    expect(isSafeGitHubRepo("owner/repo?tab=readme")).toBe(false);

    await expect(
      collectGitHubRepos(
        '<GitHub repo="ubugeeei/ox-content"></GitHub><GitHub repo="../secret"></GitHub>',
      ),
    ).resolves.toEqual(["ubugeeei/ox-content"]);

    const html = await transformGitHub(
      '<GitHub repo="../secret"></GitHub>',
      new Map([["../secret", null]]),
    );
    expect(html).toContain('href="#"');
  });

  it("accepts GitHub source permalinks and loc ranges", async () => {
    const permalink =
      "https://github.com/ubugeeei/ox-content/blob/278098b/npm/vite-plugin-ox-content/src/plugins/github.ts#L10-L12";
    expect(parseGitHubPermalink(permalink)).toMatchObject({
      repo: "ubugeeei/ox-content",
      ref: "278098b",
      path: "npm/vite-plugin-ox-content/src/plugins/github.ts",
      lines: { start: 10, end: 12 },
    });
    expect(parseGitHubPermalink("https://example.com/ubugeeei/ox-content/blob/main/a.ts")).toBe(
      null,
    );

    await expect(
      collectGitHubSources(
        `<GitHub permalink="${permalink}"></GitHub><GitHub repo="ubugeeei/ox-content" path="README.md" ref="main" loc="1-2"></GitHub>`,
      ),
    ).resolves.toMatchObject([
      { repo: "ubugeeei/ox-content", lines: { start: 10, end: 12 } },
      { repo: "ubugeeei/ox-content", path: "README.md", lines: { start: 1, end: 2 } },
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

      expect(html).toContain("ox-github-code");
      expect(html).toContain("src/index.ts");
      expect(html).toContain("L2-L3 - 2 LOC");
      expect(html).toContain('data-line="2"');
      expect(html).toContain("const second = 2;");
      expect(html).not.toContain("const first = 1;");
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
    expect(html).toContain('href="#"');
  });

  it("runs GitHub and Open Graph embeds through the shared builtin transform", async () => {
    const html = await transformBuiltinEmbeds(
      '<GitHub repo="../secret"></GitHub><OgCard url="javascript:alert(1)"></OgCard>',
      {
        github: {},
        openGraph: {},
      },
    );

    expect(html).toContain("ox-github-card");
    expect(html).toContain("ox-ogp-simple");
    expect(html).toContain('href="#"');
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
});
