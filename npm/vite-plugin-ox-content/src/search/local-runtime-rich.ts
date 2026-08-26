export const LOCAL_SEARCH_RICH_RUNTIME = String.raw`function getScopesForDoc(doc) {
  const source = (doc.id || doc.url || "").replace(/^\/+/, "").toLowerCase();
  const segments = source.split("/").filter(Boolean);

  if (segments.length <= 1) {
    return [];
  }

  const scopes = [];
  let current = "";
  for (const segment of segments.slice(0, -1)) {
    current = current ? current + "/" + segment : segment;
    scopes.push(current);
  }

  return scopes;
}

function matchesScopes(doc, scopes) {
  if (!scopes.length) {
    return true;
  }

  const docScopes = doc.__oxScopes || (doc.__oxScopes = new Set(getScopesForDoc(doc)));
  return scopes.some((scope) => docScopes.has(scope));
}

function firstPathSegment(path) {
  let start = 0;
  while (path.charCodeAt(start) === 47) start++;
  if (start >= path.length) return "";
  const slash = path.indexOf("/", start);
  return (slash === -1 ? path.slice(start) : path.slice(start, slash)).toLowerCase();
}

function normalizeLocaleFilter(options, defaults, parsedQuery) {
  const locale = (queryFilterValue(parsedQuery, ["locale"]) || options.locale || "").toLowerCase();
  if (!locale) return null;

  const localeCodes = new Set();
  const codes = options.localeCodes || defaults.localeCodes || [];
  for (let i = 0; i < codes.length; i++) {
    if (codes[i]) localeCodes.add(codes[i].toLowerCase());
  }
  if (localeCodes.size === 0) localeCodes.add(locale);

  const rawPrefixes = options.versionPrefixes || defaults.versionPrefixes || [];
  const prefixes = [];
  for (let i = 0; i < rawPrefixes.length; i++) {
    const prefix = (rawPrefixes[i] || "").replace(/^\/+|\/+$/g, "").toLowerCase();
    if (prefix) prefixes.push(prefix);
  }
  const activeVersion = normalizeFilterValue(
    queryFilterValue(parsedQuery, ["version"]) || options.version || "",
  );
  if (activeVersion && !prefixes.includes(activeVersion)) prefixes.push(activeVersion);
  if (prefixes.length > 1) prefixes.sort((left, right) => right.length - left.length);

  return {
    locale,
    localeCodes,
    defaultLocale: (options.defaultLocale || defaults.defaultLocale || "en").toLowerCase(),
    prefixes,
    cacheKey: locale + "\0" + prefixes.join("/"),
  };
}

function localeForDoc(doc, filter) {
  if (doc.__oxLocaleKey === filter.cacheKey) return doc.__oxLocale;

  let source = (doc.id || doc.url || "").replace(/^\/+/, "").toLowerCase();
  for (let i = 0; i < filter.prefixes.length; i++) {
    const prefix = filter.prefixes[i];
    if (source === prefix) {
      source = "";
      break;
    }
    if (source.startsWith(prefix) && source.charCodeAt(prefix.length) === 47) {
      source = source.slice(prefix.length + 1);
      break;
    }
  }

  const first = firstPathSegment(source);
  const locale = first && filter.localeCodes.has(first) ? first : filter.defaultLocale;
  doc.__oxLocaleKey = filter.cacheKey;
  doc.__oxLocale = locale;
  return locale;
}

function matchesLocale(doc, filter) {
  return !filter || localeForDoc(doc, filter) === filter.locale;
}

function normalizeVersionFilter(options, defaults, parsedQuery) {
  const version = normalizeFilterValue(
    queryFilterValue(parsedQuery, ["version"]) || options.version || "",
  );
  if (!version) return null;

  const rawPrefixes = options.versionPrefixes || defaults.versionPrefixes || [];
  const prefixes = [];
  for (let i = 0; i < rawPrefixes.length; i++) {
    const prefix = normalizeFilterValue(rawPrefixes[i]);
    if (prefix) prefixes.push(prefix);
  }
  if (prefixes.length > 1) prefixes.sort((left, right) => right.length - left.length);

  return { version, prefixes, cacheKey: version + "\0" + prefixes.join("/") };
}

function versionForDoc(doc, filter) {
  if (doc.__oxVersionKey === filter.cacheKey) return doc.__oxVersion;

  const source = (doc.id || doc.url || "").replace(/^\/+/, "").toLowerCase();
  for (let i = 0; i < filter.prefixes.length; i++) {
    const prefix = filter.prefixes[i];
    if (
      source === prefix ||
      (source.startsWith(prefix) && source.charCodeAt(prefix.length) === 47)
    ) {
      const parts = prefix.split("/");
      const version = parts[parts.length - 1] || prefix;
      doc.__oxVersionKey = filter.cacheKey;
      doc.__oxVersion = version;
      return version;
    }
  }

  const first = firstPathSegment(source);
  const version = first === filter.version ? first : "";
  doc.__oxVersionKey = filter.cacheKey;
  doc.__oxVersion = version;
  return version;
}

function matchesVersion(doc, filter) {
  return !filter || versionForDoc(doc, filter) === filter.version;
}

function matchesSearchConstraints(doc, parsedQuery, localeFilter, versionFilter) {
  return (
    matchesLocale(doc, localeFilter) &&
    matchesVersion(doc, versionFilter) &&
    matchesScopes(doc, parsedQuery.scopes)
  );
}

function getFieldLabel(field) {
  switch (field) {
    case "Title":
      return "title";
    case "Heading":
      return "heading";
    case "Body":
      return "body";
    case "Code":
      return "code";
    default:
      return "body";
  }
}

function fieldOrder(field) {
  switch (field) {
    case "title":
      return 0;
    case "heading":
      return 1;
    case "body":
      return 2;
    case "code":
      return 3;
    default:
      return 4;
  }
}

function containsPhrase(value, phrase) {
  return String(value || "")
    .toLowerCase()
    .includes(phrase);
}

function phraseFieldsForDoc(doc, phrase) {
  const fields = [];
  if (containsPhrase(doc.title, phrase)) fields.push("Title");
  if ((doc.headings || []).some((heading) => containsPhrase(heading, phrase))) {
    fields.push("Heading");
  }
  if (containsPhrase(doc.body, phrase)) fields.push("Body");
  if ((doc.code || []).some((code) => containsPhrase(code, phrase))) fields.push("Code");
  return fields;
}

function addCandidateMatch(entry, term, field, kind) {
  entry.matches.add(term);
  entry.fields.add(getFieldLabel(field));
  entry.reasons.add(getFieldLabel(field) + " " + kind + " match: " + term);
}

function addPhraseScore(index, docScores, parsedQuery, localeFilter, versionFilter, phrase) {
  for (let docIdx = 0; docIdx < index.documents.length; docIdx++) {
    const doc = index.documents[docIdx];
    if (!doc || !matchesSearchConstraints(doc, parsedQuery, localeFilter, versionFilter)) continue;

    const fields = phraseFieldsForDoc(doc, phrase);
    if (!fields.length) continue;

    let entry = docScores.get(docIdx);
    if (entry === undefined) {
      entry = { score: 0, matches: new Set(), fields: new Set(), reasons: new Set() };
      docScores.set(docIdx, entry);
    }

    for (const field of fields) {
      entry.score += getFieldBoost(field) * 3.0;
      entry.matches.add(phrase);
      entry.fields.add(getFieldLabel(field));
      entry.reasons.add(getFieldLabel(field) + " phrase match: " + phrase);
    }
  }
}

function matchingSection(doc, matches) {
  const headings = doc.headings || [];
  for (const heading of headings) {
    if (matches.some((term) => containsPhrase(heading, term))) return heading;
  }
  return headings[0] || "";
}

function resultMetadata(doc, parsedQuery, localeFilter, versionFilter, matches) {
  const metadata = {
    scopes: getScopesForDoc(doc),
    section: matchingSection(doc, matches),
    filters: parsedQuery.filters,
  };
  if (localeFilter) metadata.locale = localeForDoc(doc, localeFilter);
  if (versionFilter) metadata.version = versionForDoc(doc, versionFilter);
  return metadata;
}

function resultRanking(score, data, parsedQuery) {
  const fields = Array.from(data.fields).sort(
    (left, right) => fieldOrder(left) - fieldOrder(right),
  );
  const reasons = Array.from(data.reasons);
  for (const scope of parsedQuery.scopes) reasons.push("scope filter: " + scope);
  for (const filter of parsedQuery.filters) reasons.push(filter.name + " filter: " + filter.value);
  reasons.sort();
  return {
    score,
    fields: [...new Set(fields)],
    reasons: [...new Set(reasons)],
  };
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

function stableResultId(result, index) {
  const seed = String(result.id || result.url || index || "result")
    .toLowerCase()
    .replace(/[^a-z0-9_-]+/g, "-")
    .replace(/^-+|-+$/g, "");
  return "ox-search-result-" + (seed || index);
}

export function formatSearchResultForCard(result, index = 0) {
  const metadata = result.metadata || { scopes: result.scopes || [], section: "" };
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
      ...(metadata.scopes || [])
        .slice(0, 1)
        .map((scope) => ({ kind: "scope", label: "@" + scope })),
    ].filter(Boolean),
  };
  card.ariaLabel =
    result.ariaLabel || result.aria_label || resultAriaLabel({ ...result, metadata, ranking });
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
  const activeIndex = Math.min(
    Math.max(Number(options.activeIndex || 0), 0),
    Math.max(cards.length - 1, 0),
  );

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
}`;
