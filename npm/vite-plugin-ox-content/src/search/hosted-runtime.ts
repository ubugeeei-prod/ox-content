export const HOSTED_SEARCH_RUNTIME = `function normalizeQueryValue(value) {
  return String(value || "").trim().toLowerCase();
}
function normalizeFilterValue(value) {
  return normalizeQueryValue(value).replace(/^\\/+|\\/+$/g, "");
}
function pushUnique(values, value) {
  if (value && !values.includes(value)) values.push(value);
}
function parseSearchParts(query) {
  const parts = [];
  let current = "";
  let quoted = false;
  let escaping = false;
  const push = (isQuoted) => {
    const value = current.trim();
    if (value) parts.push({ value, quoted: isQuoted });
    current = "";
  };
  for (const char of String(query || "")) {
    if (quoted) {
      if (escaping) {
        current += char;
        escaping = false;
      } else if (char === "\\\\") {
        escaping = true;
      } else if (char === '"') {
        push(true);
        quoted = false;
      } else {
        current += char;
      }
      continue;
    }
    if (char === '"') {
      push(false);
      quoted = true;
    } else if (/\\s/.test(char)) {
      push(false);
    } else {
      current += char;
    }
  }
  if (escaping) current += "\\\\";
  push(quoted);
  return parts;
}
function normalizeFilterName(name) {
  const normalized = normalizeQueryValue(name).replace(/_/g, "-");
  if (!/^[a-z0-9-]+$/.test(normalized)) return "";
  switch (normalized) {
    case "lang":
    case "language":
    case "locale":
      return "locale";
    case "v":
    case "version":
      return "version";
    case "section":
    case "scope":
      return "scope";
    default:
      return normalized;
  }
}
function parseQueryFilter(value) {
  const colon = value.indexOf(":");
  if (colon <= 0) return null;
  const name = normalizeFilterName(value.slice(0, colon));
  const filterValue = normalizeFilterValue(value.slice(colon + 1));
  return name && filterValue ? { name, value: filterValue } : null;
}
export function parseSearchQuery(query) {
  const scopes = [];
  const terms = [];
  const phrases = [];
  const prefixes = [];
  const filters = [];
  const textParts = [];
  const raw = String(query || "");
  for (const part of parseSearchParts(raw)) {
    if (part.quoted) {
      const phrase = normalizeQueryValue(part.value);
      if (phrase) {
        pushUnique(phrases, phrase);
        textParts.push(part.value);
      }
      continue;
    }
    if (part.value.startsWith("@") && part.value.length > 1) {
      pushUnique(scopes, normalizeFilterValue(part.value.slice(1)));
      continue;
    }
    const filter = parseQueryFilter(part.value);
    if (filter) {
      if (filter.name === "scope") pushUnique(scopes, filter.value);
      if (!filters.some((item) => item.name === filter.name && item.value === filter.value)) {
        filters.push(filter);
      }
      continue;
    }
    if (part.value.endsWith("*")) {
      const prefix = normalizeQueryValue(part.value.slice(0, -1));
      if (prefix) {
        pushUnique(prefixes, prefix);
        textParts.push(part.value.slice(0, -1));
      }
      continue;
    }
    const term = normalizeQueryValue(part.value);
    if (term) {
      pushUnique(terms, term);
      textParts.push(part.value);
    }
  }
  return { raw, text: textParts.join(" ").trim(), terms, phrases, prefixes, filters, scopes };
}
function queryRefinements(parsedQuery) {
  const refinements = [];
  for (const scope of parsedQuery.scopes) {
    refinements.push({ kind: "scope", label: "@" + scope, value: scope });
  }
  for (const filter of parsedQuery.filters) {
    refinements.push({
      kind: "filter",
      name: filter.name,
      label: filter.name + ":" + filter.value,
      value: filter.value,
    });
  }
  for (const phrase of parsedQuery.phrases) {
    refinements.push({ kind: "phrase", label: '"' + phrase + '"', value: phrase });
  }
  for (const prefix of parsedQuery.prefixes) {
    refinements.push({ kind: "prefix", label: prefix + "*", value: prefix });
  }
  return refinements;
}
function stableResultId(result, index) {
  const seed = String(result.id || result.url || index || "result")
    .toLowerCase()
    .replace(/[^a-z0-9_-]+/g, "-")
    .replace(/^-+|-+$/g, "");
  return "ox-search-result-" + (seed || index);
}
function resultAriaLabel(result) {
  const parts = [result.title || "Untitled search result"];
  if (result.metadata?.section) parts.push("section " + result.metadata.section);
  if (result.metadata?.locale) parts.push("language " + result.metadata.locale);
  if (result.metadata?.version) parts.push("version " + result.metadata.version);
  if (result.ranking?.reasons?.length) {
    parts.push(result.ranking.reasons.length + " ranking reasons");
  }
  return parts.join(", ");
}
export function formatSearchResultForCard(result, index = 0) {
  const metadata = result.metadata || { scopes: result.scopes || [], section: result.section || "" };
  const ranking = result.ranking || {
    score: Number(result.score || 0),
    fields: [],
    reasons: [],
  };
  const card = {
    id: stableResultId(result, index),
    role: "option",
    title: String(result.title || ""),
    url: String(result.url || ""),
    snippet: String(result.snippet || ""),
    metadata,
    ranking,
    badges: [
      metadata.section && { kind: "section", label: metadata.section },
      metadata.locale && { kind: "locale", label: metadata.locale },
      metadata.version && { kind: "version", label: metadata.version },
      ...(metadata.scopes || []).slice(0, 1).map((scope) => ({ kind: "scope", label: "@" + scope })),
    ].filter(Boolean),
  };
  card.ariaLabel = result.ariaLabel || result.aria_label || resultAriaLabel({ ...result, metadata, ranking });
  return card;
}
export function createSearchUiState(query, results = [], options = {}) {
  const parsedQuery = parseSearchQuery(query);
  const cards = results.map((result, index) => formatSearchResultForCard(result, index));
  const isComposing = Boolean(options.isComposing || options.composing);
  const isLoading = Boolean(options.loading);
  const active = Boolean(
    parsedQuery.text ||
      parsedQuery.scopes.length ||
      parsedQuery.filters.length ||
      parsedQuery.prefixes.length ||
      parsedQuery.phrases.length,
  );
  const status = isComposing
    ? "composing"
    : !active
      ? "empty"
      : isLoading
        ? "loading"
        : cards.length
          ? "results"
          : "no-results";
  const activeIndex = Math.min(Math.max(Number(options.activeIndex || 0), 0), Math.max(cards.length - 1, 0));
  return {
    status,
    parsedQuery,
    refinements: queryRefinements(parsedQuery),
    resultCount: cards.length,
    activeDescendant: cards[activeIndex]?.id || "",
    ariaLiveMessage:
      status === "empty"
        ? "Type to search"
        : status === "loading" || status === "composing"
          ? "Loading search results"
          : status === "no-results"
            ? "No results"
            : cards.length + " results",
    cards,
  };
}
export async function search(query, options = {}) {
  const hosted = searchOptions.hosted;
  const parsedQuery = parseSearchQuery(query);
  if (
    !hosted ||
    (!parsedQuery.text &&
      parsedQuery.scopes.length === 0 &&
      parsedQuery.filters.length === 0 &&
      parsedQuery.prefixes.length === 0 &&
      parsedQuery.phrases.length === 0)
  ) return [];
  const limit = options.limit ?? searchOptions.limit;
  try {
    const response = await fetch(hosted.endpoint, {
      method: "POST",
      headers: {
        "content-type": "application/json",
        accept: "application/json",
        "x-app-id": hosted.appId,
        "x-index-name": hosted.indexName,
        "x-search-key": hosted.searchKey,
      },
      body: JSON.stringify({
        query: parsedQuery.text || parsedQuery.raw,
        rawQuery: parsedQuery.raw,
        parsedQuery,
        limit,
        indexName: hosted.indexName,
      }),
    });
    if (!response.ok) return [];
    const data = await response.json();
    const hits = Array.isArray(data) ? data : (data && (data.hits || data.results)) || [];
    return hits.slice(0, limit).map(normalizeHostedHit);
  } catch {
    return [];
  }
}
function normalizeHostedHit(hit) {
  if (!hit || typeof hit !== "object") {
    return {
      id: "",
      title: "",
      url: "",
      score: 0,
      matches: [],
      snippet: "",
      scopes: [],
      metadata: { scopes: [], section: "", filters: [] },
      ranking: { score: 0, fields: [], reasons: [] },
      ariaLabel: "Untitled search result",
    };
  }
  const score = Number(hit.score ?? 0) || 0;
  const metadata =
    hit.metadata && typeof hit.metadata === "object"
      ? {
          scopes: Array.isArray(hit.metadata.scopes) ? hit.metadata.scopes.map(String) : [],
          section: String(hit.metadata.section ?? ""),
          locale: hit.metadata.locale ? String(hit.metadata.locale) : undefined,
          version: hit.metadata.version ? String(hit.metadata.version) : undefined,
          filters: Array.isArray(hit.metadata.filters) ? hit.metadata.filters : [],
        }
      : {
          scopes: Array.isArray(hit.scopes) ? hit.scopes.map(String) : [],
          section: String(hit.section ?? ""),
          filters: [],
        };
  const ranking =
    hit.ranking && typeof hit.ranking === "object"
      ? {
          score: Number(hit.ranking.score ?? score) || score,
          fields: Array.isArray(hit.ranking.fields) ? hit.ranking.fields.map(String) : [],
          reasons: Array.isArray(hit.ranking.reasons) ? hit.ranking.reasons.map(String) : [],
        }
      : { score, fields: [], reasons: [] };
  const result = {
    id: String(hit.id ?? hit.objectID ?? ""),
    title: String(hit.title ?? ""),
    url: String(hit.url ?? ""),
    score,
    matches: Array.isArray(hit.matches) ? hit.matches.map(String) : [],
    snippet: String(hit.snippet ?? hit.content ?? ""),
    scopes: metadata.scopes,
    metadata,
    ranking,
  };
  result.ariaLabel = String(hit.ariaLabel ?? hit.aria_label ?? "") || resultAriaLabel(result);
  return result;
}
export { searchOptions };
export default {
  search,
  searchOptions,
  parseSearchQuery,
  createSearchUiState,
  formatSearchResultForCard,
};
`;
