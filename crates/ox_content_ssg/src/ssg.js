const toggle = document.querySelector(".menu-toggle"),
  mobileMenuBtn = document.querySelector("[data-mobile-menu]"),
  sidebar = document.querySelector(".sidebar"),
  overlay = document.querySelector(".overlay");

let lastMenuTrigger = null;

const setMenuOpen = (open, trigger = null, restoreFocus = false) => {
  if (!sidebar || !overlay) return;

  sidebar.classList.toggle("open", open);
  overlay.classList.toggle("open", open);
  document.body.classList.toggle("menu-open", open);
  [toggle, mobileMenuBtn].forEach((button) => button?.setAttribute("aria-expanded", String(open)));

  if (open && trigger instanceof HTMLElement) {
    lastMenuTrigger = trigger;
  } else if (!open) {
    const triggerToRestore = lastMenuTrigger;
    lastMenuTrigger = null;
    if (restoreFocus) {
      requestAnimationFrame(() => triggerToRestore?.focus());
    }
  }
};

const toggleMenu = (trigger) => setMenuOpen(!sidebar?.classList.contains("open"), trigger);

if (sidebar && overlay) {
  toggle?.addEventListener("click", () => toggleMenu(toggle));
  mobileMenuBtn?.addEventListener("click", () => toggleMenu(mobileMenuBtn));
  overlay.addEventListener("click", () => setMenuOpen(false));
  sidebar
    .querySelectorAll("a")
    .forEach((a) => a.addEventListener("click", () => setMenuOpen(false)));
  document.addEventListener("keydown", (e) => {
    if (e.key === "Escape" && sidebar.classList.contains("open")) {
      e.preventDefault();
      setMenuOpen(false, null, true);
    }
  });
}

if (sidebar) {
  const savedPos = sessionStorage.getItem("sidebarScroll");
  if (savedPos) sidebar.scrollTop = parseInt(savedPos, 10);
  sidebar.addEventListener("scroll", () =>
    sessionStorage.setItem("sidebarScroll", sidebar.scrollTop),
  );
}

const navStateStoragePrefix = "ox-content:nav:{{base}}:";
const getNavState = (key) => {
  try {
    return localStorage.getItem(navStateStoragePrefix + key);
  } catch {
    return null;
  }
};
const setNavState = (key, open) => {
  try {
    localStorage.setItem(navStateStoragePrefix + key, open ? "open" : "closed");
  } catch {
    // Ignore storage failures so navigation remains usable.
  }
};

document.querySelectorAll("details[data-ox-nav-state-key]").forEach((details) => {
  const key = details.getAttribute("data-ox-nav-state-key");
  if (!key) return;

  const savedState = getNavState(key);
  if (savedState === "open") {
    details.open = true;
  } else if (savedState === "closed") {
    details.open = false;
  }

  details.addEventListener("toggle", () => setNavState(key, details.open));
});

const themeToggle = document.querySelector(".theme-toggle"),
  readThemePreference = () => {
    try {
      const stored = localStorage.getItem("theme");
      return stored === "light" || stored === "dark" ? stored : "system";
    } catch {
      return "system";
    }
  },
  // `theme-color` tracks the system scheme through its media query, so a forced
  // theme leaves the browser painting its own window — and the gap it shows
  // between two documents mid-navigation — in the opposite color. Pinning the
  // matching meta and muting the other keeps the two in step.
  syncThemeColor = (theme) => {
    for (const meta of document.querySelectorAll('meta[name="theme-color"][media]')) {
      const base = (meta.dataset.octcMedia ??= meta.media),
        scheme = base.includes("dark") ? "dark" : "light";
      meta.media =
        theme === "light" || theme === "dark" ? (theme === scheme ? "all" : "not all") : base;
    }
  },
  applyThemePreference = (theme) => {
    if (theme === "light" || theme === "dark") {
      document.documentElement.setAttribute("data-theme", theme);
    } else {
      document.documentElement.removeAttribute("data-theme");
    }
    syncThemeColor(theme);
  },
  setTheme = (theme) => {
    if (theme !== "light" && theme !== "dark") return;
    applyThemePreference(theme);
    try {
      localStorage.setItem("theme", theme);
    } catch {
      // The visual preference still applies when storage is unavailable.
    }
  },
  getTheme = () =>
    document.documentElement.getAttribute("data-theme") ||
    (matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light"),
  syncThemePreference = () => applyThemePreference(readThemePreference());

window.addEventListener("pageshow", syncThemePreference);
window.addEventListener("storage", (event) => {
  if (event.key === "theme") syncThemePreference();
});

themeToggle?.addEventListener("click", () => setTheme(getTheme() === "dark" ? "light" : "dark"));

const markdownTableScrollLabel = () =>
  (document.documentElement.lang || "").toLowerCase().startsWith("ja")
    ? "横スクロールできる表"
    : "Scrollable table";

let tableScrollResizePending = false;
const enhanceMarkdownTables = () => {
  const label = markdownTableScrollLabel();
  document.querySelectorAll(".content table").forEach((table) => {
    if (!(table instanceof HTMLElement)) return;
    const scrollable = table.scrollWidth > table.clientWidth + 1;
    table.toggleAttribute("data-ox-table-scrollable", scrollable);
    if (scrollable) {
      if (!table.hasAttribute("tabindex")) {
        table.tabIndex = 0;
        table.dataset.oxTableScrollTabindex = "true";
      }
      if (!table.hasAttribute("aria-label") && !table.hasAttribute("aria-labelledby")) {
        table.setAttribute("aria-label", label);
        table.dataset.oxTableScrollLabel = "true";
      }
    } else {
      if (table.dataset.oxTableScrollTabindex === "true") {
        table.removeAttribute("tabindex");
        delete table.dataset.oxTableScrollTabindex;
      }
      if (table.dataset.oxTableScrollLabel === "true") {
        table.removeAttribute("aria-label");
        delete table.dataset.oxTableScrollLabel;
      }
    }
  });
};

enhanceMarkdownTables();
window.addEventListener("resize", () => {
  if (tableScrollResizePending) return;
  tableScrollResizePending = true;
  requestAnimationFrame(() => {
    tableScrollResizePending = false;
    enhanceMarkdownTables();
  });
});

const searchBtn = document.querySelector(".search-button");
let searchApiPromise = null;

const loadSearchApi = async () => {
  if (searchApiPromise) {
    return searchApiPromise;
  }

  searchApiPromise = new Promise((resolve) => {
    if (typeof window.__oxContentInitSearch === "function") {
      resolve(window.__oxContentInitSearch());
      return;
    }

    const script = document.createElement("script");
    script.src = "__OX_CONTENT_SEARCH_CHUNK__";
    script.defer = true;
    script.onload = () =>
      resolve(
        typeof window.__oxContentInitSearch === "function" ? window.__oxContentInitSearch() : null,
      );
    script.onerror = () => {
      console.warn("[ox-content] Search chunk failed to load");
      searchApiPromise = null;
      resolve(null);
    };
    document.head.appendChild(script);
  });

  return searchApiPromise;
};

const openSearch = async () => {
  const api = await loadSearchApi();
  api?.openSearch();
};

const isTypingTarget = (target) =>
  target instanceof HTMLInputElement ||
  target instanceof HTMLTextAreaElement ||
  target instanceof HTMLSelectElement ||
  (target instanceof HTMLElement && target.isContentEditable);

searchBtn?.addEventListener("click", () => {
  void openSearch();
});

document.addEventListener("keydown", (e) => {
  if (
    (e.key === "/" && !isTypingTarget(e.target)) ||
    ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "k")
  ) {
    e.preventDefault();
    void openSearch();
  }
});

const scrollToHash = () => {
  const hash = location.hash;
  if (!hash) return;

  const target = document.querySelector(hash);
  if (!target) return;

  setTimeout(() => target.scrollIntoView({ behavior: "smooth", block: "start" }), 100);
};

scrollToHash();
window.addEventListener("hashchange", scrollToHash);
document.querySelectorAll('a[href^="#"]').forEach((a) =>
  a.addEventListener("click", (e) => {
    const hash = a.getAttribute("href");
    const target = hash ? document.querySelector(hash) : null;
    if (target) {
      e.preventDefault();
      target.scrollIntoView({ behavior: "smooth", block: "start" });
      history.pushState(null, null, hash);
    }
  }),
);

const mobileSearchBtn = document.querySelector("[data-mobile-search]"),
  mobileThemeBtn = document.querySelector("[data-mobile-theme]");

mobileSearchBtn?.addEventListener("click", () => {
  void openSearch();
});

mobileThemeBtn?.addEventListener("click", () => setTheme(getTheme() === "dark" ? "light" : "dark"));

document.querySelectorAll(".ox-api-controls").forEach((controls) => {
  const targetSelector = controls.getAttribute("data-ox-api-target");
  if (!targetSelector) return;

  controls.querySelectorAll("[data-ox-api-toggle]").forEach((button) => {
    button.addEventListener("click", () => {
      const shouldOpen = button.getAttribute("data-ox-api-toggle") === "expand";
      document.querySelectorAll(targetSelector).forEach((entry) => {
        if (entry instanceof HTMLDetailsElement) {
          entry.open = shouldOpen;
        }
      });
    });
  });
});

// ox-content:search:start
const getOxContentSearchElements = () => {
  const searchOverlay = document.querySelector(".search-modal-overlay"),
    searchInput = document.querySelector(".search-input"),
    searchResults = document.querySelector(".search-results"),
    searchClose = document.querySelector(".search-close");

  if (!searchOverlay || !searchInput || !searchResults) {
    return null;
  }

  return {
    searchOverlay,
    searchInput,
    searchResults,
    searchClose,
    localeSelect: searchOverlay.querySelector("[data-search-filter=locale]"),
    versionSelect: searchOverlay.querySelector("[data-search-filter=version]"),
  };
};

const createOxContentSearchState = () => ({
  searchIndex: null,
  indexes: new Map(),
  filter: null,
  selectedIdx: 0,
  results: [],
  searchTimeout: null,
});

const defaultOxContentSearchIndexUrl = () =>
  document.documentElement.getAttribute("data-ox-search-index") || "{{base}}search-index.json";

const readOxContentSearchFilter = (elements) => {
  const localeSelect = elements.localeSelect;
  const versionSelect = elements.versionSelect;
  const selectedVersion =
    versionSelect instanceof HTMLSelectElement ? versionSelect.selectedOptions[0] : null;
  const localeCodes = new Set();
  if (localeSelect instanceof HTMLSelectElement) {
    for (const option of localeSelect.options) {
      if (option.value) localeCodes.add(option.value.toLowerCase());
    }
  }
  return {
    locale: localeSelect instanceof HTMLSelectElement ? localeSelect.value.toLowerCase() : "",
    defaultLocale: (localeSelect?.getAttribute("data-default-locale") || "en").toLowerCase(),
    localeCodes,
    versionPrefix: (selectedVersion?.getAttribute("data-prefix") || "")
      .replace(/^\/+|\/+$/g, "")
      .toLowerCase(),
    indexUrl: selectedVersion?.getAttribute("data-index") || defaultOxContentSearchIndexUrl(),
  };
};

const loadOxContentSearchIndex = async (state, indexUrl) => {
  const cached = state.indexes.get(indexUrl);
  if (cached) {
    state.searchIndex = cached;
    return;
  }
  try {
    const index = await (await fetch(indexUrl)).json();
    state.indexes.set(indexUrl, index);
    state.searchIndex = index;
  } catch (e) {
    console.warn("Search index load failed:", e);
    state.searchIndex = null;
  }
};

const firstOxContentPathSegment = (path) => {
  let start = 0;
  while (path.charCodeAt(start) === 47) start++;
  if (start >= path.length) return "";
  const slash = path.indexOf("/", start);
  return (slash === -1 ? path.slice(start) : path.slice(start, slash)).toLowerCase();
};

const oxContentDocLocale = (doc, filter) => {
  if (doc.__oxLocalePref === filter.versionPrefix) return doc.__oxLocale;

  let path = (doc.url || doc.id || "").replace(/^\/+/, "").toLowerCase();
  const prefix = filter.versionPrefix;
  if (prefix && (path === prefix || path.startsWith(prefix + "/"))) {
    path = path.slice(prefix.length + (path === prefix ? 0 : 1));
  }
  const first = firstOxContentPathSegment(path);
  const locale = first && filter.localeCodes.has(first) ? first : filter.defaultLocale;
  doc.__oxLocalePref = filter.versionPrefix;
  doc.__oxLocale = locale;
  return locale;
};

const matchesOxContentLocale = (doc, filter) =>
  !filter.locale || oxContentDocLocale(doc, filter) === filter.locale;

const parseOxContentScopedQuery = (query) => {
  const scopes = [];
  const terms = [];
  for (const part of query.trim().split(/\s+/).filter(Boolean)) {
    if (part.startsWith("@") && part.length > 1) {
      scopes.push(part.slice(1).toLowerCase());
    } else {
      terms.push(part);
    }
  }
  return { text: terms.join(" ").trim(), scopes: [...new Set(scopes)] };
};

const getOxContentScopesForDoc = (doc) => {
  const source = (doc.id || doc.url || "").replace(/^\/+/, "").toLowerCase();
  const segments = source.split("/").filter(Boolean);
  if (segments.length <= 1) return [];

  const scopes = [];
  let current = "";
  for (const segment of segments.slice(0, -1)) {
    current = current ? current + "/" + segment : segment;
    scopes.push(current);
  }
  return scopes;
};

const matchesOxContentScopes = (doc, scopes) => {
  if (!scopes.length) return true;
  // A doc's scopes derive only from its immutable id/url, but this predicate
  // runs once per posting per term. The same doc is therefore revisited many
  // times during a single query and across subsequent keystrokes. Cache the
  // Set on the document object so scope membership becomes one property read
  // plus `Set.has` for the rest of the index lifetime.
  const docScopes = doc.__oxScopes || (doc.__oxScopes = new Set(getOxContentScopesForDoc(doc)));
  return scopes.some((scope) => docScopes.has(scope));
};

const matchesOxContentDoc = (doc, scopes, filter) =>
  matchesOxContentLocale(doc, filter) && matchesOxContentScopes(doc, scopes);

const tokenizeOxContentSearchText = (text) => {
  const tokens = [];
  let current = "";

  for (const ch of text) {
    if (/[\u4E00-\u9FFF\u3400-\u4DBF\u3040-\u309F\u30A0-\u30FF\uAC00-\uD7AF]/.test(ch)) {
      if (current) {
        tokens.push(current.toLowerCase());
        current = "";
      }
      tokens.push(ch);
    } else if (/[a-zA-Z0-9_]/.test(ch)) {
      current += ch;
    } else if (current) {
      tokens.push(current.toLowerCase());
      current = "";
    }
  }

  if (current) tokens.push(current.toLowerCase());
  return tokens;
};

const renderOxContentSearchResults = (elements, state) => {
  if (!state.results.length) {
    elements.searchResults.innerHTML = '<div class="search-empty">No results</div>';
    return;
  }

  elements.searchResults.innerHTML = state.results
    .map(
      (result, index) =>
        '<a href="' +
        result.url +
        '" class="search-result' +
        (index === state.selectedIdx ? " selected" : "") +
        '"><div class="search-result-title">' +
        result.title +
        (result.scopes?.length
          ? '<span class="search-result-scope">@' + result.scopes[0] + "</span>"
          : "") +
        "</div>" +
        (result.snippet ? '<div class="search-result-snippet">' + result.snippet + "</div>" : "") +
        "</a>",
    )
    .join("");
};

const scoreOxContentSearchTerms = (searchIndex, parsedQuery, tokens, filter) => {
  const k1 = 1.2,
    b = 0.75,
    scores = new Map();

  if (!tokens.length) {
    searchIndex.documents.forEach((doc, idx) => {
      if (matchesOxContentDoc(doc, parsedQuery.scopes, filter)) {
        scores.set(idx, { score: 0, matches: new Set() });
      }
    });
  }

  for (let i = 0; i < tokens.length; i++) {
    const token = tokens[i],
      isLast = i === tokens.length - 1;
    const terms =
      isLast && token.length >= 2
        ? // Prefix expansion scans the whole vocabulary. Materialize the
          // vocabulary key list once and reuse it across keystrokes instead of
          // rebuilding `Object.keys(searchIndex.index)` on every query. The
          // final `filter` still runs per query because the prefix changes.
          (
            searchIndex.__oxIndexKeys ||
            (searchIndex.__oxIndexKeys = Object.keys(searchIndex.index))
          ).filter((term) => term.startsWith(token))
        : searchIndex.index[token]
          ? [token]
          : [];

    addOxContentTermScores(searchIndex, scores, parsedQuery.scopes, terms, k1, b, filter);
  }

  return scores;
};

const addOxContentTermScores = (searchIndex, scores, scopes, terms, k1, b, filter) => {
  for (const term of terms) {
    const postings = searchIndex.index[term] || [],
      df = searchIndex.df[term] || 1,
      idf = Math.log((searchIndex.doc_count - df + 0.5) / (df + 0.5) + 1);

    for (const posting of postings) {
      const doc = searchIndex.documents[posting.doc_idx];
      if (!doc || !matchesOxContentDoc(doc, scopes, filter)) continue;

      const boost = posting.field === "Title" ? 10 : posting.field === "Heading" ? 5 : 1,
        score =
          idf *
          ((posting.tf * (k1 + 1)) /
            (posting.tf + k1 * (1 - b + (b * doc.body.length) / searchIndex.avg_dl))) *
          boost;

      // One Map lookup in the steady state instead of `has` followed by `get`.
      // This runs once per posting per term on every keystroke, so removing
      // the duplicate lookup matters more than the few cold inserts.
      let entry = scores.get(posting.doc_idx);
      if (entry === undefined) {
        entry = { score: 0, matches: new Set() };
        scores.set(posting.doc_idx, entry);
      }
      entry.score += score;
      entry.matches.add(term);
    }
  }
};

const createOxContentSnippet = (doc, matches) => {
  if (!doc.body) return "";

  const bodyLower = doc.body.toLowerCase();
  let firstPos = -1;
  for (const match of matches) {
    const pos = bodyLower.indexOf(match);
    if (pos !== -1 && (firstPos === -1 || pos < firstPos)) {
      firstPos = pos;
    }
  }

  const start = firstPos === -1 ? 0 : Math.max(0, firstPos - 50),
    end = Math.min(doc.body.length, start + 150);
  let snippet = doc.body.slice(start, end);
  if (start > 0) snippet = "..." + snippet;
  if (end < doc.body.length) snippet += "...";
  return snippet;
};

const buildOxContentSearchResults = (searchIndex, parsedQuery, filter) => {
  const tokens = tokenizeOxContentSearchText(parsedQuery.text);
  const scores = scoreOxContentSearchTerms(searchIndex, parsedQuery, tokens, filter);

  return Array.from(scores.entries())
    .map(([idx, data]) => {
      const doc = searchIndex.documents[idx];
      return {
        ...doc,
        score: data.score,
        scopes: getOxContentScopesForDoc(doc),
        snippet: createOxContentSnippet(doc, data.matches),
      };
    })
    .sort((a, b) => b.score - a.score || a.title.localeCompare(b.title))
    .slice(0, 10);
};

const runOxContentSearch = async (query, elements, state) => {
  const filter = state.filter || (state.filter = readOxContentSearchFilter(elements));
  await loadOxContentSearchIndex(state, filter.indexUrl);
  if (!state.searchIndex) {
    elements.searchResults.innerHTML = '<div class="search-empty">Index unavailable</div>';
    return;
  }

  const parsedQuery = parseOxContentScopedQuery(query);
  if (!parsedQuery.text && parsedQuery.scopes.length === 0) {
    elements.searchResults.innerHTML = "";
    state.results = [];
    return;
  }

  state.results = buildOxContentSearchResults(state.searchIndex, parsedQuery, filter);
  state.selectedIdx = 0;
  renderOxContentSearchResults(elements, state);
};

const registerOxContentSearchEvents = (elements, state, closeSearch) => {
  elements.searchClose?.addEventListener("click", closeSearch);
  elements.searchOverlay.addEventListener("click", (e) => {
    if (e.target === elements.searchOverlay) closeSearch();
  });
  elements.searchResults.addEventListener("click", (e) => {
    if (e.target instanceof Element && e.target.closest("a.search-result")) closeSearch();
  });
  elements.searchInput.addEventListener("input", () => {
    if (state.searchTimeout) clearTimeout(state.searchTimeout);
    state.searchTimeout = setTimeout(
      () => runOxContentSearch(elements.searchInput.value, elements, state),
      150,
    );
  });
  elements.searchInput.addEventListener("keydown", (e) => {
    handleOxContentSearchKeydown(e, elements, state, closeSearch);
  });
  elements.searchOverlay.addEventListener("keydown", (e) => {
    if (e.key === "Escape") closeSearch();
  });
  elements.searchOverlay.addEventListener("change", (e) => {
    if (!(e.target instanceof HTMLSelectElement) || !e.target.closest(".search-filters")) {
      return;
    }
    if (state.searchTimeout) clearTimeout(state.searchTimeout);
    state.filter = readOxContentSearchFilter(elements);
    void runOxContentSearch(elements.searchInput.value, elements, state);
  });
};

const handleOxContentSearchKeydown = (e, elements, state, closeSearch) => {
  if (e.isComposing || e.keyCode === 229) return;

  if (e.key === "Escape") closeSearch();
  else if (e.key === "ArrowDown") {
    e.preventDefault();
    if (state.selectedIdx < state.results.length - 1) {
      state.selectedIdx++;
      renderOxContentSearchResults(elements, state);
    }
  } else if (e.key === "ArrowUp") {
    e.preventDefault();
    if (state.selectedIdx > 0) {
      state.selectedIdx--;
      renderOxContentSearchResults(elements, state);
    }
  } else if (e.key === "Enter" && state.results[state.selectedIdx]) {
    e.preventDefault();
    location.href = state.results[state.selectedIdx].url;
  }
};

const createOxContentSearchApi = (elements) => {
  const state = createOxContentSearchState();
  state.filter = readOxContentSearchFilter(elements);
  const openSearch = () => {
    elements.searchOverlay.classList.add("open");
    document.body.classList.add("search-open");
    elements.searchInput.focus();
  };
  const closeSearch = () => {
    elements.searchOverlay.classList.remove("open");
    document.body.classList.remove("search-open");
    elements.searchInput.value = "";
    elements.searchResults.innerHTML = "";
    state.selectedIdx = 0;
    state.results = [];
  };

  registerOxContentSearchEvents(elements, state, closeSearch);
  return { openSearch, closeSearch };
};

window.__oxContentInitSearch = (() => {
  let api = null;

  return () => {
    if (api) {
      return api;
    }

    const elements = getOxContentSearchElements();
    if (!elements) {
      return null;
    }

    api = createOxContentSearchApi(elements);
    return api;
  };
})();
// ox-content:search:end
