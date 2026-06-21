import { describe, expect, it } from "vite-plus/test";
import { transformPm } from "./pm";
import { resetTabGroupCounter, transformTabs } from "./tabs";
import { transformYouTube } from "./youtube";

/**
 * Characterization tests for the pure (network-free) embed transforms.
 *
 * Before this file these transforms had no direct coverage: the SSG snapshot
 * tests only ever exercised their no-op path (pages without the element).
 * These tests pin the exact HTML output so it can be relied on as the
 * equivalence target when the transforms are ported to Rust.
 */

describe("transformYouTube output", () => {
  it("wraps a bare id in a privacy-enhanced responsive embed", async () => {
    const html = await transformYouTube(`<p><youtube id="dQw4w9WgXcQ"></youtube></p>`);
    expect(html).toBe(
      `<p><div class="ox-youtube" style="aspect-ratio: 16/9;">` +
        `<iframe src="https://www.youtube-nocookie.com/embed/dQw4w9WgXcQ" ` +
        `title="YouTube video dQw4w9WgXcQ" ` +
        `allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share" ` +
        `referrerpolicy="strict-origin-when-cross-origin" allowfullscreen loading="lazy">` +
        `</iframe></div></p>`,
    );
  });

  it("extracts the id from a url and honours title/start", async () => {
    const html = await transformYouTube(
      `<youtube url="https://youtu.be/dQw4w9WgXcQ" title="Demo" start="30"></youtube>`,
    );
    expect(html).toBe(
      `<div class="ox-youtube" style="aspect-ratio: 16/9;">` +
        `<iframe src="https://www.youtube-nocookie.com/embed/dQw4w9WgXcQ" ` +
        `title="Demo" ` +
        `allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share" ` +
        `referrerpolicy="strict-origin-when-cross-origin" allowfullscreen loading="lazy">` +
        `</iframe></div>`,
    );
  });

  it("returns input unchanged when there is no <youtube> element", async () => {
    const html = `<p>Plain prose with a <a href="/x">link</a> and no embeds.</p>`;
    expect(await transformYouTube(html)).toBe(html);
  });
});

describe("transformTabs output", () => {
  it("expands <tabs> into the no-JS radio/label structure plus noscript fallback", async () => {
    resetTabGroupCounter();
    const html = await transformTabs(
      `<tabs><tab title="A">alpha</tab><tab title="B">beta</tab></tabs>`,
    );
    expect(html).toBe(
      `<div class="ox-tabs-container"><div class="ox-tabs" data-group="0">` +
        `<div class="ox-tabs-header">` +
        `<input type="radio" name="ox-tabs-0" id="ox-tab-0-0" checked>` +
        `<label for="ox-tab-0-0">Tab 1</label>` +
        `<input type="radio" name="ox-tabs-0" id="ox-tab-0-1">` +
        `<label for="ox-tab-0-1">Tab 2</label></div>` +
        `<div class="ox-tab-panel" data-tab="0">alpha</div>` +
        `<div class="ox-tab-panel" data-tab="1">beta</div></div>` +
        `<noscript><div class="ox-tabs-fallback">` +
        `<details open><summary>Tab 1</summary>` +
        `<div class="ox-tabs-fallback-content">alpha</div></details>` +
        `<details><summary>Tab 2</summary>` +
        `<div class="ox-tabs-fallback-content">beta</div></details>` +
        `</div></noscript></div>`,
    );
  });

  it("returns input unchanged when there is no <tabs> element", async () => {
    const html = `<p>Plain prose with a <a href="/x">link</a> and no embeds.</p>`;
    expect(await transformTabs(html)).toBe(html);
  });
});

describe("transformPm output", () => {
  it("expands <pm> into npm/pnpm/yarn/bun install tabs", async () => {
    resetTabGroupCounter();
    const html = await transformPm(`<pm>npm install -D vite</pm>`);
    expect(html).toMatchSnapshot();
  });

  it("omits the sync group attribute by default", async () => {
    resetTabGroupCounter();
    const html = await transformPm(`<pm>npm i vite</pm>`);
    expect(html).toMatchSnapshot();
  });

  it("emits the sync group attribute when sync is enabled", async () => {
    resetTabGroupCounter();
    const html = await transformPm(`<pm>npm i vite</pm>`, { sync: true });
    expect(html).toMatchSnapshot();
  });

  it("returns input unchanged when there is no <pm> element", async () => {
    resetTabGroupCounter();
    const html = `<p>Plain prose with a <a href="/x">link</a> and no embeds.</p>`;
    expect(await transformPm(html)).toBe(html);
  });
});
