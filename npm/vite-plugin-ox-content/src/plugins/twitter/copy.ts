// Progressive Copy link enhancement. Static cards stay useful without JS;
// integrations that call this get the react-tweet Copy link / Copied! swap.

export const TWEET_COPY_RESET_MS = 6000;

export interface TweetCopyAction {
  addEventListener(type: string, listener: (event: TweetCopyClickEvent) => unknown): void;
  removeEventListener(type: string, listener: (event: TweetCopyClickEvent) => unknown): void;
  getAttribute(name: string): string | null;
  setAttribute(name: string, value: string): void;
  removeAttribute(name: string): void;
}

export interface TweetCopyClickEvent {
  preventDefault(): void;
}

export interface TweetCopyRoot {
  querySelectorAll(selector: string): Iterable<TweetCopyAction>;
}

export interface EnhanceTweetCopyOptions {
  clipboard?: Pick<Clipboard, "writeText">;
  copiedMs?: number;
}

export function enhanceTweetCopyActions(
  root: TweetCopyRoot = globalThis.document as TweetCopyRoot,
  options: EnhanceTweetCopyOptions = {},
): () => void {
  const copiedMs = options.copiedMs ?? TWEET_COPY_RESET_MS;
  const clipboard = options.clipboard ?? globalThis.navigator?.clipboard;
  const cleanups: Array<() => void> = [];

  for (const action of root.querySelectorAll("[data-ox-tweet-copy]")) {
    let timer: ReturnType<typeof setTimeout> | undefined;
    const reset = () => {
      action.removeAttribute("data-ox-tweet-copied");
      action.setAttribute("aria-label", "Copy link to post");
    };
    const onClick = (event: TweetCopyClickEvent) => {
      const url = action.getAttribute("data-ox-tweet-copy-url") ?? action.getAttribute("href");
      if (!url || !clipboard?.writeText) return;
      event.preventDefault();
      return clipboard.writeText(url).then(() => {
        action.setAttribute("data-ox-tweet-copied", "");
        action.setAttribute("aria-label", "Copied!");
        if (timer !== undefined) globalThis.clearTimeout(timer);
        timer = globalThis.setTimeout(reset, copiedMs);
      });
    };
    action.addEventListener("click", onClick);
    cleanups.push(() => {
      if (timer !== undefined) globalThis.clearTimeout(timer);
      action.removeEventListener("click", onClick);
    });
  }

  return () => {
    for (const stop of cleanups) stop();
  };
}
