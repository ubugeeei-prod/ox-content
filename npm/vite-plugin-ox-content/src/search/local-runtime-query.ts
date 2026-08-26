export const LOCAL_SEARCH_QUERY_RUNTIME = String.raw`function normalizeQueryValue(value) {
  return String(value || "")
    .trim()
    .toLowerCase();
}

function normalizeFilterValue(value) {
  return normalizeQueryValue(value).replace(/^\/+|\/+$/g, "");
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
      } else if (char === "\\") {
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
    } else if (/\s/.test(char)) {
      push(false);
    } else {
      current += char;
    }
  }

  if (escaping) current += "\\";
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

  return {
    raw,
    text: textParts.join(" ").trim(),
    terms,
    phrases,
    prefixes,
    filters,
    scopes,
  };
}

function queryFilterValue(parsedQuery, names) {
  if (!parsedQuery) return "";
  const match = parsedQuery.filters.find((filter) => names.includes(filter.name));
  return match ? match.value : "";
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
}`;
