(() => {
  const root = document.documentElement;
  if (!root.hasAttribute("data-ox-reader-chrome")) return;

  const reduced = matchMedia("(prefers-reduced-motion: reduce)").matches;
  if (reduced) root.classList.add("ox-reader-chrome--reduced-motion");

  if (root.hasAttribute("data-ox-copy")) {
    document.addEventListener("click", (event) => {
      const button =
        event.target instanceof Element ? event.target.closest("[data-ox-copy]") : null;
      if (!(button instanceof HTMLButtonElement)) return;
      const pre = button.closest(".ox-code")?.querySelector("pre");
      const text = pre ? pre.textContent || "" : "";
      if (!navigator.clipboard?.writeText) return;
      navigator.clipboard
        .writeText(text)
        .then(() => {
          button.dataset.copied = "true";
          button.setAttribute("aria-label", "Copied");
          setTimeout(() => {
            delete button.dataset.copied;
            button.setAttribute("aria-label", "Copy code");
          }, 1500);
        })
        .catch(() => {});
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
