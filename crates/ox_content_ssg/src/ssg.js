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
window.__oxContentInitSearch = (() => {
  let api = null;

  return () => {
    if (api) {
      return api;
    }

    const searchOverlay = document.querySelector(".search-modal-overlay"),
      searchInput = document.querySelector(".search-input"),
      searchResults = document.querySelector(".search-results"),
      searchClose = document.querySelector(".search-close");

    if (!searchOverlay || !searchInput || !searchResults) {
      return null;
    }

    let searchIndex = null,
      selectedIdx = 0,
      results = [],
      searchTimeout = null;

    const openSearch = () => {
      searchOverlay.classList.add("open");
      searchInput.focus();
    };

    const closeSearch = () => {
      searchOverlay.classList.remove("open");
      searchInput.value = "";
      searchResults.innerHTML = "";
      selectedIdx = 0;
      results = [];
    };

    const loadIndex = async () => {
      if (searchIndex) return;
      try {
        searchIndex = await (await fetch("{{base}}search-index.json")).json();
      } catch (e) {
        console.warn("Search index load failed:", e);
      }
    };

    const parseScopedQuery = (query) => {
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

    const getScopesForDoc = (doc) => {
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

    const matchesScopes = (doc, scopes) => {
      if (!scopes.length) return true;
      const docScopes = new Set(getScopesForDoc(doc));
      return scopes.some((scope) => docScopes.has(scope));
    };

    const tokenize = (text) => {
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

    const render = () => {
      if (!results.length) {
        searchResults.innerHTML = '<div class="search-empty">No results</div>';
        return;
      }

      searchResults.innerHTML = results
        .map(
          (result, index) =>
            '<a href="' +
            result.url +
            '" class="search-result' +
            (index === selectedIdx ? " selected" : "") +
            '"><div class="search-result-title">' +
            result.title +
            (result.scopes?.length
              ? '<span class="search-result-scope">@' + result.scopes[0] + "</span>"
              : "") +
            "</div>" +
            (result.snippet
              ? '<div class="search-result-snippet">' + result.snippet + "</div>"
              : "") +
            "</a>",
        )
        .join("");
    };

    const search = async (query) => {
      await loadIndex();
      if (!searchIndex) {
        searchResults.innerHTML = '<div class="search-empty">Index unavailable</div>';
        return;
      }

      const parsedQuery = parseScopedQuery(query);
      if (!parsedQuery.text && parsedQuery.scopes.length === 0) {
        searchResults.innerHTML = "";
        results = [];
        return;
      }

      const tokens = tokenize(parsedQuery.text);
      const k1 = 1.2,
        b = 0.75,
        scores = new Map();

      if (!tokens.length) {
        searchIndex.documents.forEach((doc, idx) => {
          if (matchesScopes(doc, parsedQuery.scopes)) {
            scores.set(idx, { score: 0, matches: new Set() });
          }
        });
      }

      for (let i = 0; i < tokens.length; i++) {
        const token = tokens[i],
          isLast = i === tokens.length - 1;
        let terms =
          isLast && token.length >= 2
            ? Object.keys(searchIndex.index).filter((term) => term.startsWith(token))
            : searchIndex.index[token]
              ? [token]
              : [];

        for (const term of terms) {
          const postings = searchIndex.index[term] || [],
            df = searchIndex.df[term] || 1,
            idf = Math.log((searchIndex.doc_count - df + 0.5) / (df + 0.5) + 1);

          for (const posting of postings) {
            const doc = searchIndex.documents[posting.doc_idx];
            if (!doc) continue;
            if (!matchesScopes(doc, parsedQuery.scopes)) continue;

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
      }

      results = Array.from(scores.entries())
        .map(([idx, data]) => {
          const doc = searchIndex.documents[idx];
          const scopes = getScopesForDoc(doc);
          let snippet = "";

          if (doc.body) {
            const bodyLower = doc.body.toLowerCase();
            let firstPos = -1;
            for (const match of data.matches) {
              const pos = bodyLower.indexOf(match);
              if (pos !== -1 && (firstPos === -1 || pos < firstPos)) {
                firstPos = pos;
              }
            }
            const start = firstPos === -1 ? 0 : Math.max(0, firstPos - 50),
              end = Math.min(doc.body.length, start + 150);
            snippet = doc.body.slice(start, end);
            if (start > 0) snippet = "..." + snippet;
            if (end < doc.body.length) snippet += "...";
          }

          return { ...doc, score: data.score, scopes, snippet };
        })
        .sort((a, b) => b.score - a.score || a.title.localeCompare(b.title))
        .slice(0, 10);

      selectedIdx = 0;
      render();
    };

    searchClose?.addEventListener("click", closeSearch);
    searchOverlay.addEventListener("click", (e) => {
      if (e.target === searchOverlay) closeSearch();
    });
    searchInput.addEventListener("input", () => {
      if (searchTimeout) clearTimeout(searchTimeout);
      searchTimeout = setTimeout(() => search(searchInput.value), 150);
    });
    searchInput.addEventListener("keydown", (e) => {
      if (e.key === "Escape") closeSearch();
      else if (e.key === "ArrowDown") {
        e.preventDefault();
        if (selectedIdx < results.length - 1) {
          selectedIdx++;
          render();
        }
      } else if (e.key === "ArrowUp") {
        e.preventDefault();
        if (selectedIdx > 0) {
          selectedIdx--;
          render();
        }
      } else if (e.key === "Enter" && results[selectedIdx]) {
        e.preventDefault();
        location.href = results[selectedIdx].url;
      }
    });

    api = {
      openSearch,
      closeSearch,
    };

    return api;
  };
})();
// ox-content:search:end
