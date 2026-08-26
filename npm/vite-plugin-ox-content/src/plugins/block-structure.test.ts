import { describe, expect, it } from "vite-plus/test";
import { normalizeBlockEmbedParagraphs } from "./block-structure";

describe("block embed paragraph normalization", () => {
  it("returns no-marker input byte-identical", async () => {
    const html = `<p>Plain prose with <a href="/docs">a link</a>.</p>`;
    expect(await normalizeBlockEmbedParagraphs(html)).toBe(html);
  });

  it("unwraps a standalone block embed from its paragraph", async () => {
    const html = `<p><div class="ox-youtube">video</div></p>`;
    expect(await normalizeBlockEmbedParagraphs(html)).toBe(`<div class="ox-youtube">video</div>`);
  });

  it("splits mixed prose around an Open Graph card", async () => {
    const html = `<p data-note="keep">before <a class="ox-ogp-card" href="/">card</a> after</p>`;
    expect(await normalizeBlockEmbedParagraphs(html)).toBe(
      `<p data-note="keep">before </p><a class="ox-ogp-card" href="/">card</a><p data-note="keep"> after</p>`,
    );
  });

  it("keeps adjacent embeds as sibling blocks without empty paragraphs", async () => {
    const html = `<p>
<figure class="ox-tweet">tweet</figure>
<a class="ox-ogp-simple" href="/">card</a>
</p>`;
    const normalized = await normalizeBlockEmbedParagraphs(html);
    expect(normalized).toContain(`<figure class="ox-tweet">tweet</figure>`);
    expect(normalized).toContain(`<a class="ox-ogp-simple" href="/">card</a>`);
    expect(normalized).not.toContain("<p>");
  });

  it("preserves the surrounding container while fixing nested paragraphs", async () => {
    const html = `<blockquote><p>read <a class="ox-ogp-card" href="/">card</a> now</p></blockquote>`;
    expect(await normalizeBlockEmbedParagraphs(html)).toBe(
      `<blockquote><p>read </p><a class="ox-ogp-card" href="/">card</a><p> now</p></blockquote>`,
    );
  });

  it("is idempotent", async () => {
    const html = `<p>before <a class="ox-ogp-simple" href="/">card</a> after</p>`;
    const once = await normalizeBlockEmbedParagraphs(html);
    expect(await normalizeBlockEmbedParagraphs(once)).toBe(once);
  });
});
