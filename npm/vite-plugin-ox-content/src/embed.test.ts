import { Buffer } from "node:buffer";
import { describe, expect, it } from "vite-plus/test";
import { createDocsResolvedOptions } from "../test/fixtures/docs-fixture";
import {
  collectGitHubRepos,
  collectGitHubSources,
  isSafeGitHubRepo,
  parseGitHubPermalink,
  transformGitHub,
} from "./plugins/github";
import { summarizeCommitMessage } from "./plugins/github/source";
import { transformBuiltinEmbeds } from "./plugins";
import { collectOgpUrls, isSafeOgpUrl, transformOgp } from "./plugins/ogp";
import { renderMarkdown } from "./render-markdown";
import { transformMarkdown } from "./transform";

function mockGitHubSourceFetch(): typeof fetch {
  return (async (input: RequestInfo | URL) => {
    const url = String(input);
    if (url.includes("/commits")) {
      return {
        ok: true,
        status: 200,
        json: async () => [
          {
            sha: "abc123def456",
            html_url: "https://github.com/acme/project/commit/abc123def456",
            commit: { message: "feat: add source cards\n\nMore detail." },
          },
        ],
      } as Response;
    }

    return {
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
    } as Response;
  }) as typeof fetch;
}

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

  it("keeps only the first line of a commit message", () => {
    expect(summarizeCommitMessage("feat: add cards\n\nbody")).toBe("feat: add cards");
    expect(summarizeCommitMessage("x".repeat(130))).toBe(`${"x".repeat(119)}…`);
  });

  it("expands GitHub source permalinks into code cards", async () => {
    const originalFetch = globalThis.fetch;
    globalThis.fetch = mockGitHubSourceFetch();

    try {
      const html = await transformGitHub(
        '<GitHub permalink="https://github.com/acme/project/blob/abc123/src/index.ts#L2-L3"></GitHub>',
        undefined,
        { cache: false },
      );

      expect(html).toContain("ox-code-block");
      expect(html).toContain('data-line-number="2"');
      expect(html).toContain("feat: add source cards");
      expect(html).toContain("abc123d");
      expect(html).toMatchSnapshot();
    } finally {
      globalThis.fetch = originalFetch;
    }
  });

  it("highlights GitHub source cards after embed expansion", async () => {
    const originalFetch = globalThis.fetch;
    globalThis.fetch = mockGitHubSourceFetch();

    try {
      const result = await transformMarkdown(
        '<GitHub permalink="https://github.com/acme/project/blob/abc123/src/index.ts#L2-L3" />',
        "docs/embed.md",
        createDocsResolvedOptions({
          embeds: {
            github: { cache: false },
            openGraph: false,
            pm: false,
            spotify: false,
            stackBlitz: false,
            twitter: false,
            bluesky: false,
            webContainer: false,
          },
        }),
      );

      expect(result.html).toContain("ox-highlight");
      expect(result.html).toContain("ox-github-code-block");
      expect(result.html).toContain('data-line-number="2"');
      expect(result.html).toMatch(/--octc-syntax-/);
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

  it("keeps block cards out of generated paragraphs", async () => {
    const html = await transformBuiltinEmbeds(
      '<p>Before <OgCard url="javascript:alert(1)"></OgCard> after.</p>',
      {
        github: false,
        openGraph: {},
      },
    );

    expect(html).toContain("<p>Before </p>");
    expect(html).toContain('<a class="ox-ogp-simple"');
    expect(html).toContain("<p> after.</p>");
    expect(html).not.toContain('<p><a class="ox-ogp-simple"');
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
        '<StackBlitz url="https://stackblitz.com/edit/vitejs-vite"></StackBlitz>',
        '<Tweet id="123">static text</Tweet>',
      ].join(""),
      {
        github: false,
        openGraph: false,
        spotify: true,
        stackBlitz: true,
        twitter: true,
      },
    );

    expect(html).toMatchSnapshot();
  });
});

const issue879Options = {
  ssg: false as const,
  frontmatter: false,
  highlight: false,
  embeds: {
    github: false,
    openGraph: false,
    twitter: true,
    bluesky: false,
  },
  ogViewer: false,
  search: false,
  toc: false,
};

describe("built-in embeds vs MDX island lowering (#879)", () => {
  it("renders Tweet in .md without a dangling closer", async () => {
    const result = await renderMarkdown('<Tweet id="1234567890" />', "/virtual/article.md", {
      ...issue879Options,
      mdx: false,
    });

    expect(result.html).toContain("ox-tweet");
    expect(result.html).not.toMatch(/<\/Tweet>/i);
    expect(result.html).not.toContain('data-ox-island="Tweet"');
  });

  it("renders Tweet in .mdx as a built-in embed, not an island", async () => {
    const result = await renderMarkdown('<Tweet id="1234567890" />', "/virtual/article.mdx", {
      ...issue879Options,
      mdx: true,
    });

    expect(result.html).toContain("ox-tweet");
    expect(result.html).not.toContain('data-ox-island="Tweet"');
    expect(result.html).not.toMatch(/<\/Tweet>/i);
  });

  it("sends PascalCase OgCard to the OGP path in both .md and .mdx", async () => {
    const originalFetch = globalThis.fetch;
    globalThis.fetch = (async () =>
      ({
        ok: true,
        text: async () =>
          `<html><head><title>Example Domain</title><meta property="og:title" content="Example Domain"></head></html>`,
      }) as Response) as typeof fetch;

    try {
      const ogOptions = {
        ssg: false as const,
        frontmatter: false,
        highlight: false,
        embeds: {
          github: false,
          openGraph: { cache: false },
          twitter: false,
          bluesky: false,
        },
        ogViewer: false,
        search: false,
        toc: false,
      };
      const source = '<OgCard url="https://example.com" />';

      const markdown = await renderMarkdown(source, "/virtual/article.md", {
        ...ogOptions,
        mdx: false,
      });
      const mdx = await renderMarkdown(source, "/virtual/article.mdx", {
        ...ogOptions,
        mdx: true,
      });

      for (const result of [markdown, mdx]) {
        expect(result.html).toMatch(/ox-ogp-(?:card|simple)/);
        expect(result.html).not.toContain('data-ox-island="OgCard"');
        expect(result.html).not.toMatch(/<\/OgCard>/i);
      }
    } finally {
      globalThis.fetch = originalFetch;
    }
  });

  it("lets a document-local import override a built-in name", async () => {
    const result = await renderMarkdown(
      'import Tweet from "./Tweet"\n\n<Tweet id="1234567890" />\n',
      "/virtual/article.mdx",
      { ...issue879Options, mdx: true },
    );

    expect(result.html).toContain('data-ox-island="Tweet"');
    expect(result.html).not.toContain("ox-tweet");
    expect(result.components).toContain("Tweet");
  });

  it("keeps a Tweet inside a blockquote as valid block HTML", async () => {
    const result = await renderMarkdown(
      '> before\n>\n> <Tweet id="1234567890" />\n>\n> after\n',
      "/virtual/article.md",
      { ...issue879Options, mdx: false },
    );

    expect(result.html).toContain("<blockquote>");
    expect(result.html).toContain("ox-tweet");
    expect(result.html).not.toMatch(/<\/Tweet>/i);
    expect(result.html).toMatch(/<blockquote>[\s\S]*<\/blockquote>/);
    expect(result.html).not.toMatch(/<p>[\s\S]*<article class="ox-tweet"/);
  });
});
