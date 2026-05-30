const toggle = document.querySelector(".menu-toggle"),
  sidebar = document.querySelector(".sidebar"),
  overlay = document.querySelector(".overlay");

if (toggle && sidebar && overlay) {
  const close = () => {
    sidebar.classList.remove("open");
    overlay.classList.remove("open");
  };

  toggle.addEventListener("click", () => {
    sidebar.classList.toggle("open");
    overlay.classList.toggle("open");
  });
  overlay.addEventListener("click", close);
  sidebar.querySelectorAll("a").forEach((a) => a.addEventListener("click", close));
}

if (sidebar) {
  const savedPos = sessionStorage.getItem("sidebarScroll");
  if (savedPos) sidebar.scrollTop = parseInt(savedPos, 10);
  sidebar.addEventListener("scroll", () =>
    sessionStorage.setItem("sidebarScroll", sidebar.scrollTop),
  );
}

const themeToggle = document.querySelector(".theme-toggle"),
  setTheme = (theme) => {
    document.documentElement.setAttribute("data-theme", theme);
    localStorage.setItem("theme", theme);
  },
  getTheme = () => document.documentElement.getAttribute("data-theme") || "light";

themeToggle?.addEventListener("click", () => setTheme(getTheme() === "dark" ? "light" : "dark"));

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

const mobileMenuBtn = document.querySelector("[data-mobile-menu]"),
  mobileSearchBtn = document.querySelector("[data-mobile-search]"),
  mobileThemeBtn = document.querySelector("[data-mobile-theme]");

mobileMenuBtn?.addEventListener("click", () => {
  if (sidebar && overlay) {
    sidebar.classList.toggle("open");
    overlay.classList.toggle("open");
  }
});

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

  return { searchOverlay, searchInput, searchResults, searchClose };
};

const createOxContentSearchState = () => ({
  searchIndex: null,
  selectedIdx: 0,
  results: [],
  searchTimeout: null,
});

const loadOxContentSearchIndex = async (state) => {
  if (state.searchIndex) return;
  try {
    state.searchIndex = await (await fetch("{{base}}search-index.json")).json();
  } catch (e) {
    console.warn("Search index load failed:", e);
  }
};

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
  // A doc's scopes derive only from its (immutable) id/url, but this runs once
  // per posting per term — the same doc is revisited many times in a query.
  // Cache the Set on the doc so it's built once for the index's lifetime.
  const docScopes = doc.__oxScopes || (doc.__oxScopes = new Set(getOxContentScopesForDoc(doc)));
  return scopes.some((scope) => docScopes.has(scope));
};

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

const scoreOxContentSearchTerms = (searchIndex, parsedQuery, tokens) => {
  const k1 = 1.2,
    b = 0.75,
    scores = new Map();

  if (!tokens.length) {
    searchIndex.documents.forEach((doc, idx) => {
      if (matchesOxContentScopes(doc, parsedQuery.scopes)) {
        scores.set(idx, { score: 0, matches: new Set() });
      }
    });
  }

  for (let i = 0; i < tokens.length; i++) {
    const token = tokens[i],
      isLast = i === tokens.length - 1;
    const terms =
      isLast && token.length >= 2
        ? // Prefix expansion scans the whole vocabulary. Materialize the term
          // list once and reuse it across keystrokes instead of rebuilding the
          // `Object.keys` array on every query.
          (
            searchIndex.__oxIndexKeys ||
            (searchIndex.__oxIndexKeys = Object.keys(searchIndex.index))
          ).filter((term) => term.startsWith(token))
        : searchIndex.index[token]
          ? [token]
          : [];

    addOxContentTermScores(searchIndex, scores, parsedQuery.scopes, terms, k1, b);
  }

  return scores;
};

const addOxContentTermScores = (searchIndex, scores, scopes, terms, k1, b) => {
  for (const term of terms) {
    const postings = searchIndex.index[term] || [],
      df = searchIndex.df[term] || 1,
      idf = Math.log((searchIndex.doc_count - df + 0.5) / (df + 0.5) + 1);

    for (const posting of postings) {
      const doc = searchIndex.documents[posting.doc_idx];
      if (!doc || !matchesOxContentScopes(doc, scopes)) continue;

      const boost = posting.field === "Title" ? 10 : posting.field === "Heading" ? 5 : 1,
        score =
          idf *
          ((posting.tf * (k1 + 1)) /
            (posting.tf + k1 * (1 - b + (b * doc.body.length) / searchIndex.avg_dl))) *
          boost;

      if (!scores.has(posting.doc_idx)) {
        scores.set(posting.doc_idx, { score: 0, matches: new Set() });
      }

      const entry = scores.get(posting.doc_idx);
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

const buildOxContentSearchResults = (searchIndex, parsedQuery) => {
  const tokens = tokenizeOxContentSearchText(parsedQuery.text);
  const scores = scoreOxContentSearchTerms(searchIndex, parsedQuery, tokens);

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
  await loadOxContentSearchIndex(state);
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

  state.results = buildOxContentSearchResults(state.searchIndex, parsedQuery);
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
};

const handleOxContentSearchKeydown = (e, elements, state, closeSearch) => {
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
