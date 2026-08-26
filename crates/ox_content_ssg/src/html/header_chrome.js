(() => {
  const dropdownTriggers = ".header-nav-dropdown > button, .ox-locale-switcher > button";
  const openDropdowns =
    ".header-nav-dropdown > button[aria-expanded='true'], .ox-locale-switcher > button[aria-expanded='true'], .ox-version-switcher > button[aria-expanded='true']";
  const closeDropdowns = (except) => {
    document.querySelectorAll(openDropdowns).forEach((button) => {
      if (button !== except) button.setAttribute("aria-expanded", "false");
    });
  };

  document.querySelectorAll(dropdownTriggers).forEach((button) => {
    button.addEventListener("click", (event) => {
      event.stopPropagation();
      const open = button.getAttribute("aria-expanded") === "true";
      closeDropdowns(open ? null : button);
      button.setAttribute("aria-expanded", open ? "false" : "true");
    });
  });

  document.addEventListener("click", () => closeDropdowns(null));
  document.addEventListener("keydown", (event) => {
    if (event.key !== "Escape") return;
    const open = document.querySelector(openDropdowns);
    closeDropdowns(null);
    if (open instanceof HTMLElement) open.focus();
  });

  document.querySelectorAll("[data-ox-announce]").forEach((bar) => {
    const key = bar.getAttribute("data-ox-announce");
    if (!key) return;
    const storageKey = "ox-content:announce:" + key;
    try {
      if (localStorage.getItem(storageKey) === "1") {
        bar.hidden = true;
        document.body.classList.remove("ox-has-announce");
        return;
      }
    } catch {
      // Ignore storage failures so the bar stays usable.
    }
    bar.querySelector(".ox-announce-dismiss")?.addEventListener("click", () => {
      try {
        localStorage.setItem(storageKey, "1");
      } catch {
        // Ignore storage failures so dismiss still hides the bar.
      }
      bar.hidden = true;
      document.body.classList.remove("ox-has-announce");
    });
  });

  const resetTimers = new WeakMap();
  const setCopyMarkdownState = (button, status, state) => {
    const previousTimer = resetTimers.get(button);
    if (previousTimer) clearTimeout(previousTimer);
    const label =
      state === "copied" ? "Copied" : state === "failed" ? "Copy failed" : "Copy as Markdown";
    if (state) button.dataset.copyState = state;
    else delete button.dataset.copyState;
    button.setAttribute("aria-label", label);
    button.setAttribute("title", label);
    if (status) status.textContent = state ? label : "";
    if (state) {
      resetTimers.set(
        button,
        setTimeout(() => setCopyMarkdownState(button, status, null), 1500),
      );
    } else {
      resetTimers.delete(button);
    }
  };

  document.addEventListener("click", (event) => {
    const button =
      event.target instanceof Element ? event.target.closest("[data-ox-copy-markdown]") : null;
    if (!(button instanceof HTMLButtonElement)) return;
    const root = button.closest(".ox-markdown-source");
    const link = root?.querySelector(".ox-view-markdown");
    const href = link instanceof HTMLAnchorElement ? link.getAttribute("href") : "";
    const status = root?.querySelector("[data-ox-copy-markdown-status]");
    if (!href || !navigator.clipboard?.writeText) {
      setCopyMarkdownState(button, status, "failed");
      return;
    }
    fetch(href)
      .then((response) => {
        if (!response.ok) throw new Error("missing companion");
        return response.text();
      })
      .then((text) => navigator.clipboard.writeText(text))
      .then(() => setCopyMarkdownState(button, status, "copied"))
      .catch(() => setCopyMarkdownState(button, status, "failed"));
  });
})();
