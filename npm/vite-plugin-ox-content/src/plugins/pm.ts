/**
 * Package Manager Tabs Plugin
 *
 * Transforms <pm>npm install …</pm> blocks into a tab group with one tab per
 * package manager (npm/pnpm/yarn/bun). The single npm-style command is converted
 * to each package manager's equivalent natively in Rust (`transformPmEmbeds` in
 * @ox-content/napi), and the result reuses the same `ox-tabs` widget markup as
 * the generic `<tabs>` plugin so styling and keyboard navigation are consistent.
 *
 * Syncing is opt-in (off by default): when enabled, the rendered group carries a
 * `data-ox-tab-group="pkg-manager"` attribute so the client runtime can keep
 * every package-manager group on the page in sync via localStorage.
 *
 * Package-manager groups share the tab-group counter with the `<tabs>` plugin so
 * `data-group` ids (and the CSS produced by `generateTabsCSS`) stay unique.
 */

import { importNapiModule } from "../napi";
import { getTabGroupCounter, setTabGroupCounter } from "./tabs";

/** Options for {@link transformPm}. */
export interface PmOptions {
  /**
   * Enable opt-in synced package-manager tab groups. When `true`, a
   * `data-ox-tab-group="pkg-manager"` attribute is emitted so the client runtime
   * syncs the active package manager across every pm group on the page and
   * persists the choice in localStorage.
   * @default false
   */
  sync?: boolean;
}

/**
 * Transform `<pm>` package-manager blocks in HTML into install tabs.
 *
 * @param html - Rendered HTML potentially containing `<pm>` blocks.
 * @param options - Package-manager tab options (syncing is opt-in).
 * @returns The rewritten HTML.
 */
export async function transformPm(html: string, options?: PmOptions): Promise<string> {
  // Cheap marker check: skip the NAPI call entirely when there's no `<pm>`
  // element. The Rust side guards the same way, but short-circuiting here avoids
  // marshalling the whole document across the boundary.
  if (!/<pm[\s/>]/i.test(html)) {
    return html;
  }

  const mod = await importNapiModule();
  const startGroup = getTabGroupCounter();
  const result = mod.transformPmEmbeds(html, startGroup, {
    sync: options?.sync ?? false,
  });
  setTabGroupCounter(startGroup + result.groupCount);
  return result.html;
}
