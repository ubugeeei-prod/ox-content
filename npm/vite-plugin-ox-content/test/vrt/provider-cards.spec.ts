import { expect, test, type Page } from "@playwright/test";
import { generateHtmlPage } from "../../src/ssg";
import { transformMarkdown } from "../../src/transform";
import { createDocsResolvedOptions } from "../fixtures/docs-fixture";

/**
 * Every first-party provider that renders a static card, with the metadata
 * supplied as attributes so nothing here touches the network at test time.
 *
 * Shared card markup is shared: a change to `render_card` reshapes all of
 * these at once, and until now none of them had a visual baseline.
 */
const RESOLVED = [
  '<Qiita url="https://qiita.com/ubugeeei/items/abcdef123456" title="Static cards" author="ubugeeei" likes="42" />',
  '<Zenn url="https://zenn.dev/ubugeeei/articles/abcdef" title="Zenn article" author="ubugeeei" />',
  '<NpmPackage url="https://www.npmjs.com/package/vite" name="vite" description="Next generation frontend tooling" version="7.0.0" />',
  '<CratesIo url="https://crates.io/crates/serde" name="serde" description="Serialization framework" version="1.0.219" />',
  '<PyPI url="https://pypi.org/project/requests" name="requests" description="HTTP for Humans" version="2.32.3" />',
  '<DockerHub url="https://hub.docker.com/_/nginx" name="nginx" description="Official build of Nginx" />',
  '<CodePen url="https://codepen.io/ubugeeei/pen/abc123" title="Pen" author="ubugeeei" />',
  '<JSFiddle url="https://jsfiddle.net/ubugeeei/abc123/2/" title="Fiddle" author="ubugeeei" />',
  '<Observable url="https://observablehq.com/@d3/bar-chart" title="Bar chart" author="d3" />',
  '<Replit url="https://replit.com/@ubugeeei/markdown-playground" title="Repl" author="ubugeeei" />',
  '<Note url="https://note.com/ubugeeei/n/nabcdef123456" title="note article" author="ubugeeei" />',
  '<Figma url="https://www.figma.com/design/AbC123xyz/Design-System" title="Design system" />',
  '<GoogleSlides url="https://docs.google.com/presentation/d/1AbC_defGHI/edit" title="Deck" />',
  '<Vimeo url="https://vimeo.com/123456789" title="Vimeo clip" author="Studio" />',
  '<Loom url="https://www.loom.com/share/abcdef1234567890" title="Loom demo" />',
  '<Asciinema url="https://asciinema.org/a/569727" title="Terminal cast" />',
  '<Twitch url="https://www.twitch.tv/videos/40464143" title="Stream" author="twitchdev" />',
  '<Discord url="https://discord.com/channels/1/2" title="Discord message" server="Guild" channel="general" />',
  '<Mastodon url="https://mastodon.social/@docs/111" author="@docs@mastodon.social" likes="8">Fediverse note.</Mastodon>',
  '<Facebook url="https://www.facebook.com/post/1" title="Facebook post" />',
  '<Threads url="https://www.threads.net/@example/post/abc" title="Threads post" />',
  '<Instagram url="https://www.instagram.com/p/abc123/" title="Instagram post" />',
  '<GoogleMaps url="https://www.google.com/maps/place/Tokyo" place="Tokyo" address="Japan" />',
].join("\n\n");

/**
 * `fetch: false` on every provider that has a fetcher. These fixtures must not
 * depend on crates.io, CodePen, or Vimeo being reachable and unchanged — the
 * first run of this file without it made three live requests and got two 403s
 * and a 404, which would have baked a network failure into the baseline.
 */
const NO_FETCH = { fetch: false } as const;

/**
 * The same providers given URLs they will not embed — a real host with a shape
 * they do not serve. Each degrades to the neutral link fallback rather than
 * shipping its tag to the browser as an unknown element.
 */
const FALLBACK = [
  '<Qiita url="https://qiita.com/ubugeeei">Qiita profile</Qiita>',
  '<Zenn url="https://zenn.dev/ubugeeei">Zenn profile</Zenn>',
  '<Vimeo url="https://vimeo.com/">Vimeo home</Vimeo>',
  '<Observable url="https://observablehq.com/docs">Observable docs</Observable>',
].join("\n\n");

const EMBEDS = {
  github: false,
  openGraph: false,
  pm: false,
  qiita: NO_FETCH,
  zenn: NO_FETCH,
  packageRegistry: NO_FETCH,
  playgrounds: NO_FETCH,
  vimeo: NO_FETCH,
  twitch: NO_FETCH,
  discord: true,
  fediverse: true,
  facebook: true,
  threads: true,
  instagram: true,
  googleMaps: true,
  loom: true,
  asciinema: true,
  figma: true,
  note: true,
  googleSlides: true,
} as const;

async function renderPage(markdown: string, name: string): Promise<string> {
  const result = await transformMarkdown(
    markdown,
    `docs/vrt-${name}.md`,
    createDocsResolvedOptions({ embeds: { ...EMBEDS } } as never),
  );
  return generateHtmlPage(
    {
      title: name,
      content: result.html,
      toc: result.toc,
      frontmatter: {},
      path: `/vrt-${name}`,
      href: `/vrt-${name}/index.html`,
    },
    [],
    "Ox Content",
    "/",
  );
}

test("static provider cards render from attributes alone", async ({ page }) => {
  const html = await renderPage(RESOLVED, "provider-cards");
  await page.setViewportSize({ width: 720, height: 1400 });
  await page.setContent(html, { waitUntil: "domcontentloaded" });

  // Every provider produced a card, not a leftover tag.
  await expect(page.locator(".ox-provider-card")).toHaveCount(23);
  await expect(page.locator("body")).not.toContainText("<Qiita");

  // The bug this guards is the stylesheet never shipping on a provider-only
  // page, which a pixel snapshot would catch only indirectly — and only on the
  // platform that generated it. Assert the rules actually landed instead.
  await expectCardStylesApplied(page);

  const firstCardHeight = await page
    .locator(".ox-provider-card")
    .first()
    .evaluate((node) => node.getBoundingClientRect().height);
  expect(firstCardHeight).toBeLessThan(170);
});

test("cards keep their column on a narrow viewport", async ({ page }) => {
  const html = await renderPage(RESOLVED, "provider-cards-mobile");
  await page.setViewportSize({ width: 320, height: 1400 });
  await page.setContent(html, { waitUntil: "domcontentloaded" });

  const overflow = await page.evaluate(() => {
    const content = document.querySelector(".content");
    return content ? content.scrollWidth - content.clientWidth : -1;
  });
  expect(overflow).toBeLessThanOrEqual(0);
});

test("unresolvable provider tags degrade to neutral links", async ({ page }) => {
  const html = await renderPage(FALLBACK, "provider-fallbacks");
  await page.setViewportSize({ width: 720, height: 600 });
  await page.setContent(html, { waitUntil: "domcontentloaded" });

  await expect(page.locator("a.ox-embed-fallback")).toHaveCount(4);
  // The fallback names no provider, so a look-alike host cannot borrow the
  // styling of the provider it is imitating.
  await expect(page.locator('[class*="ox-embed-fallback--"]')).toHaveCount(0);
  await expect(page.locator("body")).not.toContainText("<Qiita");

  // Neutral means it carries no card styling at all — it is a prose link, and
  // it has to stay a safe one.
  const fallback = page.locator("a.ox-embed-fallback").first();
  await expect(fallback).toHaveAttribute("href", /^https:\/\//);
  await expect(fallback).toHaveAttribute("rel", "noopener noreferrer");
  await expect(fallback).toHaveAttribute("target", "_blank");
});

/**
 * A provider card is styled when its own rules are in the cascade, not merely
 * when the element exists. `display` and `border-radius` both come from
 * provider-cards.css, so an unshipped stylesheet leaves them at their initial
 * values and this fails.
 */
async function expectCardStylesApplied(page: Page): Promise<void> {
  const card = page.locator(".ox-provider-card").first();
  await expect(card).toHaveCSS("display", /block|flex|grid/);
  const radius = await card.evaluate((node) => getComputedStyle(node).borderRadius);
  expect(radius).not.toBe("0px");
}
