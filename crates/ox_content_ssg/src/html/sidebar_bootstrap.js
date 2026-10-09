(() => {
  const sidebar = document.getElementById("ox-sidebar");
  if (!sidebar) return;

  const read = (storage, key) => {
    try {
      return window[storage].getItem(key);
    } catch {
      return null;
    }
  };

  sidebar.querySelectorAll("details[data-ox-nav-state-key]").forEach((details) => {
    const key = details.getAttribute("data-ox-nav-state-key");
    const state = read("localStorage", "ox-content:nav:{{base}}:" + key);
    if (state === "open") {
      details.open = true;
    } else if (state === "closed") {
      details.open = false;
    }
  });

  const saved = parseInt(read("sessionStorage", "sidebarScroll") ?? "", 10);
  if (saved > 0) sidebar.scrollTop = saved;

  const active = sidebar.querySelector(".nav-link.active");
  const view = sidebar.clientHeight;
  if (!active || !view || !active.getClientRects().length) return;
  const top =
    active.getBoundingClientRect().top - sidebar.getBoundingClientRect().top + sidebar.scrollTop;
  if (top < sidebar.scrollTop || top + active.offsetHeight > sidebar.scrollTop + view) {
    sidebar.scrollTop = Math.max(0, top - view / 3);
  }
})();
