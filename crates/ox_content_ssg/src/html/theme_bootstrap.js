(() => {
  const root = document.documentElement;
  let stored = null;

  try {
    stored = localStorage.getItem("theme");
  } catch {
    // Storage may be unavailable in privacy modes. The system scheme remains usable.
  }

  if (stored === "light" || stored === "dark") {
    root.setAttribute("data-theme", stored);
  } else {
    root.removeAttribute("data-theme");
  }
})();
