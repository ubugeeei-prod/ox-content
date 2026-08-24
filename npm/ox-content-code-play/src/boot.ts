import { hydrateCodePlay } from "./hydrate";

export interface BootDocument {
  readyState: string;
  addEventListener(type: string, listener: () => void, options?: { once?: boolean }): void;
}

/**
 * Mount every `[data-ox-code-play]` widget. The SSG client (`ox-code-play.js`)
 * calls this on load; apps that import `@ox-content/code-play/client` call it
 * themselves.
 */
export function bootCodePlay(
  hydrate: (root?: ParentNode) => void = hydrateCodePlay,
  doc: BootDocument | undefined = typeof document === "undefined" ? undefined : document,
): void {
  if (!doc) {
    return;
  }
  const run = () => hydrate();
  if (doc.readyState === "loading") {
    doc.addEventListener("DOMContentLoaded", run, { once: true });
    return;
  }
  run();
}
