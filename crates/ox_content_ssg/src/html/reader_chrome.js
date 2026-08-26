(() => {
  const root = document.documentElement;
  if (!root.hasAttribute("data-ox-reader-chrome")) return;

  const reduced = matchMedia("(prefers-reduced-motion: reduce)").matches;
  if (reduced) root.classList.add("ox-reader-chrome--reduced-motion");

  if (root.hasAttribute("data-ox-copy")) {
    const resetTimers = new WeakMap();
    const setCopyState = (button, status, state) => {
      const previousTimer = resetTimers.get(button);
      if (previousTimer) clearTimeout(previousTimer);

      const label =
        state === "copied" ? "Copied" : state === "failed" ? "Copy failed" : "Copy code";
      if (state) button.dataset.copyState = state;
      else delete button.dataset.copyState;
      button.setAttribute("aria-label", label);
      button.setAttribute("title", label);
      if (status) status.textContent = state ? label : "";

      if (state) {
        resetTimers.set(
          button,
          setTimeout(() => setCopyState(button, status, null), 1500),
        );
      } else {
        resetTimers.delete(button);
      }
    };

    document.addEventListener("click", (event) => {
      const button =
        event.target instanceof Element ? event.target.closest("[data-ox-copy]") : null;
      if (!(button instanceof HTMLButtonElement)) return;
      const container = button.closest(".ox-code");
      const pre = container?.querySelector("pre");
      const status = container?.querySelector("[data-ox-copy-status]");
      const text = pre ? (pre.getAttribute("data-ox-code-source") ?? pre.textContent ?? "") : "";
      if (!navigator.clipboard?.writeText) {
        setCopyState(button, status, "failed");
        return;
      }
      navigator.clipboard
        .writeText(text)
        .then(() => setCopyState(button, status, "copied"))
        .catch(() => setCopyState(button, status, "failed"));
    });
  }

  const top = document.querySelector(".ox-back-to-top[data-ox-back-to-top]");
  if (top instanceof HTMLElement && root.hasAttribute("data-ox-back-to-top")) {
    const sync = () => {
      top.hidden = window.scrollY < 320;
    };
    sync();
    window.addEventListener("scroll", sync, { passive: true });
    top.addEventListener("click", () => {
      window.scrollTo({ top: 0, behavior: reduced ? "auto" : "smooth" });
    });
  }
})();
