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
})();
