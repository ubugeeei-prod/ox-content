export const TWEET_COPY_RESET_MS = 6000;
const initializedRoots = new WeakMap<TweetCardsRoot, () => void>();
const resetTimers = new WeakMap<TweetCopyAction, ReturnType<typeof setTimeout>>();
const handledEvents = new WeakSet<TweetCopyClickEvent>();

export interface TweetCopyAction {
  getAttribute(name: string): string | null;
  setAttribute(name: string, value: string): void;
  removeAttribute(name: string): void;
  querySelector?(selector: string): { textContent: string } | null;
}

export interface TweetCopyClickEvent {
  readonly defaultPrevented?: boolean;
  readonly target?: unknown;
  preventDefault(): void;
}

export interface TweetCardsRoot {
  addEventListener(type: string, listener: (event: TweetCopyClickEvent) => unknown): void;
  removeEventListener(type: string, listener: (event: TweetCopyClickEvent) => unknown): void;
}

export interface InitTweetCardsOptions {
  clipboard?: Pick<Clipboard, "writeText">;
  copiedMs?: number;
}

export function initTweetCards(root?: TweetCardsRoot, options: InitTweetCardsOptions = {}): void {
  const resolvedRoot = root ?? defaultTweetCardsRoot();
  if (!resolvedRoot) return;
  ensureTweetCards(resolvedRoot, options);
}

export const initTwitterCards = initTweetCards;

export function enhanceTweetCopyActions(
  root?: TweetCardsRoot,
  options: InitTweetCardsOptions = {},
): () => void {
  const resolvedRoot = root ?? defaultTweetCardsRoot();
  return resolvedRoot ? ensureTweetCards(resolvedRoot, options) : () => {};
}

function ensureTweetCards(root: TweetCardsRoot, options: InitTweetCardsOptions): () => void {
  const existing = initializedRoots.get(root);
  if (existing) {
    return existing;
  }

  const copiedMs = Number.isFinite(options.copiedMs)
    ? Math.max(0, options.copiedMs ?? TWEET_COPY_RESET_MS)
    : TWEET_COPY_RESET_MS;
  const onClick = (event: TweetCopyClickEvent) => {
    if (handledEvents.has(event) || event.defaultPrevented) return;
    const action = closestTweetCopyAction(event.target);
    if (!action) return;
    const url = action.getAttribute("data-ox-tweet-copy-url") ?? action.getAttribute("href");
    const clipboard = options.clipboard ?? globalThis.navigator?.clipboard;
    if (!url || !clipboard?.writeText) return;

    handledEvents.add(event);
    event.preventDefault();
    return clipboard
      .writeText(url)
      .then(() => setTweetCopyState(action, "copied", copiedMs))
      .catch(() => setTweetCopyState(action, "failed", copiedMs));
  };
  root.addEventListener("click", onClick);
  const cleanup = () => {
    root.removeEventListener("click", onClick);
    initializedRoots.delete(root);
  };
  initializedRoots.set(root, cleanup);
  return cleanup;
}

function defaultTweetCardsRoot(): TweetCardsRoot | undefined {
  return (globalThis as typeof globalThis & { document?: TweetCardsRoot }).document;
}

function closestTweetCopyAction(target: unknown): TweetCopyAction | undefined {
  if (!target || typeof target !== "object" || !("closest" in target)) return undefined;
  const action = (target as { closest(selector: string): unknown }).closest("[data-ox-tweet-copy]");
  if (!action || typeof action !== "object") return undefined;
  if (
    !("getAttribute" in action) ||
    !("setAttribute" in action) ||
    !("removeAttribute" in action)
  ) {
    return undefined;
  }
  return action as TweetCopyAction;
}

function setTweetCopyState(
  action: TweetCopyAction,
  state: "copied" | "failed" | undefined,
  copiedMs: number,
): void {
  const previousTimer = resetTimers.get(action);
  if (previousTimer) {
    globalThis.clearTimeout(previousTimer);
  }

  const label = state === "copied" ? "Copied!" : state === "failed" ? "Copy failed" : "Copy link";
  const status = action.querySelector?.("[data-ox-tweet-copy-status]");
  if (state === "copied") {
    action.setAttribute("data-ox-tweet-copied", "");
  } else {
    action.removeAttribute("data-ox-tweet-copied");
  }
  action.setAttribute("aria-label", label);
  action.setAttribute("title", label);
  if (status) {
    status.textContent = state ? label : "";
  }

  if (state) {
    resetTimers.set(
      action,
      globalThis.setTimeout(() => setTweetCopyState(action, undefined, copiedMs), copiedMs),
    );
  } else {
    resetTimers.delete(action);
  }
}
