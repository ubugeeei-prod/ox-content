const TWEET_COPY_RESET_MS = 6000;
const oxTweetInitializedRoots = new WeakSet();
const oxTweetCopyResetTimers = new WeakMap();
const oxTweetHandledEvents = new WeakSet();

// eslint-disable-next-line no-unused-vars
function initTweetCards(rootInput, options) {
  const fallbackDocument = typeof document === "undefined" ? undefined : document;
  const root = rootInput ?? fallbackDocument;
  if (!root || oxTweetInitializedRoots.has(root) || typeof root.addEventListener !== "function") {
    return;
  }

  oxTweetInitializedRoots.add(root);
  const copiedMs =
    options && Number.isFinite(options.copiedMs)
      ? Math.max(0, options.copiedMs)
      : TWEET_COPY_RESET_MS;
  const getClipboard = () =>
    options && options.clipboard
      ? options.clipboard
      : typeof navigator === "undefined"
        ? undefined
        : navigator.clipboard;

  root.addEventListener("click", (event) => {
    if (oxTweetHandledEvents.has(event) || event.defaultPrevented) return;
    const action = closestTweetCopyAction(event.target);
    if (!action) return;

    const url = action.getAttribute("data-ox-tweet-copy-url") || action.getAttribute("href");
    const clipboard = getClipboard();
    if (!url || !clipboard || typeof clipboard.writeText !== "function") return;

    oxTweetHandledEvents.add(event);
    event.preventDefault();
    clipboard
      .writeText(url)
      .then(() => setTweetCopyState(action, "copied", copiedMs))
      .catch(() => setTweetCopyState(action, "failed", copiedMs));
  });
}

// eslint-disable-next-line no-unused-vars
const initTwitterCards = initTweetCards;

function closestTweetCopyAction(target) {
  if (!target || typeof target.closest !== "function") return undefined;
  const action = target.closest("[data-ox-tweet-copy]");
  if (!action || typeof action.getAttribute !== "function") return undefined;
  return action;
}

function setTweetCopyState(action, state, copiedMs) {
  const previousTimer = oxTweetCopyResetTimers.get(action);
  if (previousTimer) clearTimeout(previousTimer);

  const label = state === "copied" ? "Copied!" : state === "failed" ? "Copy failed" : "Copy link";
  const status =
    typeof action.querySelector === "function"
      ? action.querySelector("[data-ox-tweet-copy-status]")
      : undefined;

  if (state === "copied") action.setAttribute("data-ox-tweet-copied", "");
  else action.removeAttribute("data-ox-tweet-copied");
  action.setAttribute("aria-label", label);
  action.setAttribute("title", label);
  if (status) status.textContent = state ? label : "";

  if (state) {
    oxTweetCopyResetTimers.set(
      action,
      setTimeout(() => setTweetCopyState(action, undefined, copiedMs), copiedMs),
    );
  } else {
    oxTweetCopyResetTimers.delete(action);
  }
}
