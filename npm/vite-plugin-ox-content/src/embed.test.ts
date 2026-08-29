import { Buffer } from "node:buffer";
import { afterEach, describe, expect, it } from "vite-plus/test";
import { createDocsResolvedOptions } from "../test/fixtures/docs-fixture";
import {
  clearGitHubResourceCache,
  collectGitHubResources,
  collectGitHubRepos,
  collectGitHubSources,
  isSafeGitHubRepo,
  parseGitHubResourceReference,
  parseGitHubPermalink,
  transformGitHub,
} from "./plugins/github";
import { summarizeCommitMessage } from "./plugins/github/source";
import { transformBuiltinEmbeds, transformMermaidStatic } from "./plugins";
import { collectOgpUrls, isSafeOgpUrl, transformOgp } from "./plugins/ogp";
import { renderMarkdown } from "./render-markdown";
import { transformMarkdown } from "./transform";

function mockGitHubSourceFetch(): typeof fetch {
  return (async (input: RequestInfo | URL) => {
    const url = requestUrl(input);
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

afterEach(() => {
  clearGitHubResourceCache();
});

describe("builtin embed input hardening", () => {
  it("leaves HTML without GitHub embeds byte-for-byte untouched", async () => {
    const html = "<p data-kind=plain>Just prose</p>";
    await expect(transformGitHub(html)).resolves.toBe(html);
  });

  it("leaves HTML without Open Graph embeds byte-for-byte untouched", async () => {
    const html = "<p data-kind=plain>Just prose</p>";
    await expect(transformOgp(html)).resolves.toBe(html);
  });

  it("leaves HTML without Mermaid blocks byte-for-byte untouched", async () => {
    const html = "<p data-kind=plain>Just prose</p>";
    await expect(transformMermaidStatic(html)).resolves.toBe(html);
  });

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

  it("accepts GitHub work item and gist URLs", async () => {
    expect(parseGitHubResourceReference("https://github.com/acme/project/issues/42")).toEqual({
      kind: "issue",
      repo: "acme/project",
      number: 42,
      permalink: "https://github.com/acme/project/issues/42",
      apiUrl: "https://api.github.com/repos/acme/project/issues/42",
    });
    expect(
      parseGitHubResourceReference("https://github.com/acme/project/pull/7/files"),
    ).toMatchObject({
      kind: "pull",
      repo: "acme/project",
      number: 7,
    });
    expect(
      parseGitHubResourceReference(
        "https://github.com/acme/project/commit/0123456789abcdef0123456789abcdef01234567",
      ),
    ).toMatchObject({
      kind: "commit",
      repo: "acme/project",
      sha: "0123456789abcdef0123456789abcdef01234567",
    });
    expect(
      parseGitHubResourceReference("https://gist.github.com/acme/0123456789abcdef0123456789abcdef"),
    ).toMatchObject({
      kind: "gist",
      gistOwner: "acme",
      gistId: "0123456789abcdef0123456789abcdef",
    });
    expect(
      await collectGitHubResources(
        [
          '<GitHub url="https://github.com/acme/project/issues/42"></GitHub>',
          '<GitHub repo="acme/project" discussion="8"></GitHub>',
        ].join(""),
      ),
    ).toHaveLength(2);

    expect(parseGitHubResourceReference("http://github.com/acme/project/issues/42")).toBeNull();
    expect(
      parseGitHubResourceReference("https://user:pass@github.com/acme/project/issues/42"),
    ).toBeNull();
    expect(
      parseGitHubResourceReference("https://github.com.evil/acme/project/issues/42"),
    ).toBeNull();
    expect(parseGitHubResourceReference("https://github.com/acme/project/issues/0")).toBeNull();
  });

  it("expands GitHub work items and gists into static resource cards", async () => {
    const originalFetch = globalThis.fetch;
    const requests: string[] = [];
    globalThis.fetch = (async (input) => {
      const url = requestUrl(input);
      requests.push(url);
      switch (url) {
        case "https://api.github.com/repos/acme/project/issues/42":
          return okJson({
            title: "Fix escaped cards",
            state: "open",
            html_url: "https://github.com/acme/project/issues/42",
            body: "Cards should remain **static**.",
            comments: 3,
            updated_at: "2026-08-26T12:00:00Z",
            user: { login: "octo", avatar_url: "https://avatars.githubusercontent.com/u/1?v=4" },
            labels: [{ name: "bug" }, { name: "docs" }],
          });
        case "https://api.github.com/repos/acme/project/issues/7":
          return okJson({
            title: "Add provider cards",
            state: "closed",
            html_url: "https://github.com/acme/project/pull/7",
            comments: 1,
            updated_at: "2026-08-25T12:00:00Z",
            user: { login: "maintainer" },
            labels: [{ name: "feature" }],
          });
        case "https://api.github.com/repos/acme/project/commits/0123456789abcdef0123456789abcdef01234567":
          return okJson({
            sha: "0123456789abcdef0123456789abcdef01234567",
            html_url:
              "https://github.com/acme/project/commit/0123456789abcdef0123456789abcdef01234567",
            commit: {
              message: "feat: add resources\n\nDetailed body",
              author: { name: "Jane Doe", date: "2026-08-24T01:02:03Z" },
            },
            author: { login: "jane", avatar_url: "https://avatars.githubusercontent.com/u/2?v=4" },
          });
        case "https://api.github.com/repos/acme/project/discussions/8":
          return okJson({
            title: "How should cards fall back?",
            state: "open",
            html_url: "https://github.com/acme/project/discussions/8",
            comments: 5,
            updated_at: "2026-08-23T00:00:00Z",
            user: { login: "reader" },
          });
        case "https://api.github.com/gists/0123456789abcdef0123456789abcdef":
          return okJson({
            id: "0123456789abcdef0123456789abcdef",
            html_url: "https://gist.github.com/acme/0123456789abcdef0123456789abcdef",
            description: "Minimal repro",
            comments: 2,
            updated_at: "2026-08-22T00:00:00Z",
            owner: { login: "acme" },
            files: {
              "repro.ts": { filename: "repro.ts", language: "TypeScript" },
              "README.md": { filename: "README.md", language: "Markdown" },
            },
          });
        default:
          throw new Error(`unexpected request ${url}`);
      }
    }) as typeof fetch;

    try {
      const html = await transformGitHub(
        [
          '<GitHub url="https://github.com/acme/project/issues/42"></GitHub>',
          '<GitHub url="https://github.com/acme/project/pull/7"></GitHub>',
          '<GitHub url="https://github.com/acme/project/commit/0123456789abcdef0123456789abcdef01234567"></GitHub>',
          '<GitHub url="https://github.com/acme/project/discussions/8"></GitHub>',
          '<GitHub url="https://gist.github.com/acme/0123456789abcdef0123456789abcdef"></GitHub>',
        ].join("\n"),
      );

      expect(html).toContain("ox-github-resource-card--issue");
      expect(html).toContain("Fix escaped cards");
      expect(html).toContain("bug, docs");
      expect(html).toContain("ox-github-resource-card--pull");
      expect(html).toContain("pull request #7");
      expect(html).toContain("ox-github-resource-card--commit");
      expect(html).toContain("feat: add resources");
      expect(html).toContain("ox-github-resource-card--discussion");
      expect(html).toContain("How should cards fall back?");
      expect(html).toContain("ox-github-resource-card--gist");
      expect(html).toContain("repro.ts, README.md");
      expect(html).toContain('aria-label="GitHub acme/project issue: Fix escaped cards"');

      await transformGitHub('<GitHub url="https://github.com/acme/project/issues/42"></GitHub>');
      expect(requests.filter((url) => url.endsWith("/issues/42"))).toHaveLength(1);
    } finally {
      globalThis.fetch = originalFetch;
    }
  });

  it("renders link-only GitHub resource cards when metadata is unavailable", async () => {
    const originalFetch = globalThis.fetch;
    const originalWarn = console.warn;
    const warnings: string[] = [];
    globalThis.fetch = (async () => new Response("{}", { status: 404 })) as typeof fetch;
    console.warn = (message?: unknown) => {
      warnings.push(String(message));
    };

    try {
      const html = await transformGitHub(
        '<GitHub url="https://github.com/acme/project/issues/404"></GitHub>',
        undefined,
        { cache: false },
      );

      expect(html).toContain("ox-github-resource-card--issue");
      expect(html).toContain("error");
      expect(html).toContain("acme/project#404");
      expect(html).toContain("Unavailable");
      expect(warnings[0]).toContain("404");
    } finally {
      globalThis.fetch = originalFetch;
      console.warn = originalWarn;
    }
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
            appleMusic: false,
            speakerDeck: false,
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
        '<AppleMusic url="https://music.apple.com/gb/album/1989-taylors-version/1708308989"></AppleMusic>',
        '<SpeakerDeck url="https://speakerdeck.com/player/abcdef1234567890" title="My Talk" author="Jane Doe"></SpeakerDeck>',
        '<StackBlitz url="https://stackblitz.com/edit/vitejs-vite"></StackBlitz>',
        '<Tweet id="123">static text</Tweet>',
      ].join(""),
      {
        github: false,
        openGraph: false,
        spotify: true,
        appleMusic: true,
        speakerDeck: true,
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
