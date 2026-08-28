const oxReaderChromeInitializedRoots = new WeakSet();
const oxReaderChromeResetTimers = new WeakMap();

// eslint-disable-next-line no-unused-vars
function initReaderChrome(rootInput) {
  const fallbackDocument = typeof document === "undefined" ? undefined : document;
  const root = rootInput ?? fallbackDocument;
  if (!root || oxReaderChromeInitializedRoots.has(root)) return;

  const scope = root.nodeType === 9 ? root : root;
  const rootElement = root.nodeType === 9 ? root.documentElement : root;
  if (!rootElement || typeof rootElement.querySelector !== "function") return;

  const hasRootAttribute =
    typeof rootElement.hasAttribute === "function"
      ? (name) => rootElement.hasAttribute(name)
      : () => false;
  const hasCopy =
    hasRootAttribute("data-ox-copy") || Boolean(rootElement.querySelector("[data-ox-copy]"));
  const hasBackToTop =
    hasRootAttribute("data-ox-back-to-top") ||
    Boolean(rootElement.querySelector(".ox-back-to-top[data-ox-back-to-top]"));
  const hasReaderChrome = hasRootAttribute("data-ox-reader-chrome");
  if (!hasReaderChrome && !hasCopy && !hasBackToTop) return;

  oxReaderChromeInitializedRoots.add(root);

  const reduced =
    typeof matchMedia === "function" && matchMedia("(prefers-reduced-motion: reduce)").matches;
  if (reduced && rootElement.classList)
    rootElement.classList.add("ox-reader-chrome--reduced-motion");

  if (hasCopy && typeof scope.addEventListener === "function") {
    scope.addEventListener("click", (event) => {
      const button =
        event.target instanceof Element ? event.target.closest("[data-ox-copy]") : null;
      if (!(button instanceof HTMLButtonElement)) return;
      if (event.defaultPrevented) return;
      event.preventDefault();
      const container = button.closest(".ox-code");
      const pre = container?.querySelector("pre");
      const status = container?.querySelector("[data-ox-copy-status]");
      const text = pre ? (pre.getAttribute("data-ox-code-source") ?? pre.textContent ?? "") : "";
      if (!navigator.clipboard?.writeText) {
        setReaderChromeCopyState(button, status, "failed");
        return;
      }
      navigator.clipboard
        .writeText(text)
        .then(() => setReaderChromeCopyState(button, status, "copied"))
        .catch(() => setReaderChromeCopyState(button, status, "failed"));
    });
  }

  if (hasBackToTop && fallbackDocument) {
    const findBackToTop = () => rootElement.querySelector(".ox-back-to-top[data-ox-back-to-top]");
    const sync = () => {
      const top = findBackToTop();
      if (top instanceof HTMLElement) top.hidden = window.scrollY < 320;
    };
    sync();
    window.addEventListener("scroll", sync, { passive: true });
    if (typeof scope.addEventListener === "function") {
      scope.addEventListener("click", (event) => {
        const button =
          event.target instanceof Element
            ? event.target.closest(".ox-back-to-top[data-ox-back-to-top]")
            : null;
        if (!(button instanceof HTMLElement)) return;
        if (event.defaultPrevented) return;
        event.preventDefault();
        window.scrollTo({ top: 0, behavior: reduced ? "auto" : "smooth" });
      });
    }
  }
}

function setReaderChromeCopyState(button, status, state) {
  const previousTimer = oxReaderChromeResetTimers.get(button);
  if (previousTimer) clearTimeout(previousTimer);

  const label = state === "copied" ? "Copied" : state === "failed" ? "Copy failed" : "Copy code";
  if (state) button.dataset.copyState = state;
  else delete button.dataset.copyState;
  button.setAttribute("aria-label", label);
  button.setAttribute("title", label);
  if (status) status.textContent = state ? label : "";

  if (state) {
    oxReaderChromeResetTimers.set(
      button,
      setTimeout(() => setReaderChromeCopyState(button, status, null), 1500),
    );
  } else {
    oxReaderChromeResetTimers.delete(button);
  }
}
