import { expect, test } from "@playwright/test";
import { generateHtmlPage } from "../../src/ssg";
import { transformGitHub } from "../../src/plugins/github";

/**
 * `.ox-code-block` makes every `.line` a block, and the producers of those
 * lines write a literal newline between them. Inside a `pre` that newline is
 * preserved, so each line broke twice and the block rendered at roughly double
 * height — reported for a titled fence and again for the GitHub source card.
 *
 * Measuring the pitch between two lines catches it for any producer; a
 * screenshot would only catch it on the page it captures.
 */
function mockGitHubSourceFetch(): typeof fetch {
  return (async (input: RequestInfo | URL) => {
    const url = typeof input === "string" ? input : input instanceof URL ? input.href : input.url;
    if (url.includes("/commits")) {
      return {
        ok: true,
        status: 200,
        json: async () => [
          {
            sha: "abc123def456",
            html_url: "https://github.com/acme/project/commit/abc123def456",
            commit: { message: "feat: add source cards" },
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
        html_url: "https://github.com/acme/project/blob/abc123/src/index.ts#L1-L3",
      }),
    } as Response;
  }) as typeof fetch;
}

test("a GitHub source card sets one line break per line, not two", async ({ page }) => {
  const originalFetch = globalThis.fetch;
  globalThis.fetch = mockGitHubSourceFetch();
  let content: string;
  try {
    content = await transformGitHub(
      '<GitHub permalink="https://github.com/acme/project/blob/abc123/src/index.ts#L1-L3"></GitHub>',
      undefined,
      { cache: false },
    );
  } finally {
    globalThis.fetch = originalFetch;
  }

  expect(content).toContain("ox-code-block");

  const html = await generateHtmlPage(
    {
      title: "line pitch",
      content,
      toc: [],
      frontmatter: {},
      path: "/vrt-line-pitch",
      href: "/vrt-line-pitch/index.html",
    },
    [],
    "Ox Content",
    "/",
  );

  await page.setViewportSize({ width: 800, height: 600 });
  await page.setContent(html, { waitUntil: "domcontentloaded" });

  const measured = await page.evaluate(() => {
    const lines = document.querySelectorAll("pre.ox-code-block .line");
    if (lines.length < 2) return null;
    const first = lines[0].getBoundingClientRect();
    const second = lines[1].getBoundingClientRect();
    return { pitch: second.top - first.top, height: first.height };
  });

  expect(measured).not.toBeNull();
  expect(measured!.height).toBeGreaterThan(0);
  // Two breaks per line would put the pitch at roughly twice the line height.
  expect(measured!.pitch).toBeCloseTo(measured!.height, 0);
});
