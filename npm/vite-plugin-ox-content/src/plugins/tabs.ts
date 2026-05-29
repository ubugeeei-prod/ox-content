/**
 * Tabs Plugin - Pure CSS implementation
 *
 * Transforms <Tabs>/<Tab> components into accessible HTML with CSS :has()
 * based tab switching (no JavaScript required).
 *
 * The HTML rewrite is performed in Rust (`transformTabsEmbeds` in
 * @ox-content/napi), replacing the previous rehype parse/stringify round-trip.
 * The per-build group counter (consumed by `generateTabsCSS`) stays here and is
 * advanced by the number of groups the Rust transform reports, so CSS
 * generation still covers exactly the groups that were emitted.
 */

import { importNapiModule } from "../napi";

let tabGroupCounter = 0;

/**
 * Reset tab group counter (for testing and per dev-server request).
 */
export function resetTabGroupCounter(): void {
  tabGroupCounter = 0;
}

/**
 * Transform Tabs components in HTML.
 */
export async function transformTabs(html: string): Promise<string> {
  // Cheap marker check: skip the NAPI call entirely when there's no `<tabs>`
  // element. The Rust side guards the same way, but short-circuiting here
  // avoids marshalling the whole document across the boundary.
  if (!/<tabs/i.test(html)) {
    return html;
  }

  const mod = await importNapiModule();
  const result = mod.transformTabsEmbeds(html, tabGroupCounter);
  tabGroupCounter += result.groupCount;
  return result.html;
}

/**
 * Generate dynamic CSS for :has() based tab switching.
 * This is needed because :has() selectors need unique IDs.
 */
export function generateTabsCSS(groupCount: number): string {
  if (groupCount === 0) return "";

  let css = "/* Dynamic Tabs CSS */\n";

  for (let g = 0; g < groupCount; g++) {
    for (let t = 0; t < 8; t++) {
      css += `.ox-tabs[data-group="${g}"]:has(#ox-tab-${g}-${t}:checked) .ox-tab-panel[data-tab="${t}"] { display: block; }\n`;
    }
  }

  return css;
}
