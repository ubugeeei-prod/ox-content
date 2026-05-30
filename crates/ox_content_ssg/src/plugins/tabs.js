// Opt-in synced tab groups.
//
// Tab groups rendered with a `data-ox-tab-group="<key>"` attribute (emitted only
// when syncing is enabled) keep their active tab in sync across the page and
// persist the choice in localStorage. Groups without the attribute are left
// alone, so the default no-JavaScript CSS widget is unaffected.
(() => {
  const GROUP_SELECTOR = ".ox-tabs[data-ox-tab-group]";
  const STORAGE_PREFIX = "ox-tab-group:";

  function storageKey(group) {
    return STORAGE_PREFIX + group;
  }

  function readStored(group) {
    try {
      return window.localStorage.getItem(storageKey(group));
    } catch {
      return null;
    }
  }

  function writeStored(group, label) {
    try {
      window.localStorage.setItem(storageKey(group), label);
    } catch {
      // Ignore storage failures (private mode, quota, etc.).
    }
  }

  // Map a group element's inputs to their tab labels, in order.
  function labelFor(tabs, input) {
    const id = input.id;
    const label = tabs.querySelector('label[for="' + id + '"]');
    return label ? label.textContent.trim() : null;
  }

  function selectByLabel(tabs, label) {
    const inputs = tabs.querySelectorAll('input[type="radio"]');
    for (const input of inputs) {
      if (labelFor(tabs, input) === label) {
        if (!input.checked) input.checked = true;
        return true;
      }
    }
    return false;
  }

  function init() {
    const groups = Array.from(document.querySelectorAll(GROUP_SELECTOR));
    if (groups.length === 0) return;

    // Restore persisted selections first.
    for (const tabs of groups) {
      const group = tabs.getAttribute("data-ox-tab-group");
      const stored = readStored(group);
      if (stored) selectByLabel(tabs, stored);
    }

    // Broadcast a selection to every group sharing the same key.
    function broadcast(group, label, source) {
      writeStored(group, label);
      for (const tabs of groups) {
        if (tabs === source) continue;
        if (tabs.getAttribute("data-ox-tab-group") === group) {
          selectByLabel(tabs, label);
        }
      }
    }

    for (const tabs of groups) {
      const group = tabs.getAttribute("data-ox-tab-group");
      tabs.addEventListener("change", (event) => {
        const input = event.target;
        if (!input || input.type !== "radio") return;
        const label = labelFor(tabs, input);
        if (label) broadcast(group, label, tabs);
      });
    }
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init);
  } else {
    init();
  }
})();
