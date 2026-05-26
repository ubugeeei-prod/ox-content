export const i18nConfig = {
  enabled: true,
  defaultLocale: __OX_I18N_DEFAULT_LOCALE__,
  locales: __OX_I18N_LOCALES__,
  hideDefaultLocale: __OX_I18N_HIDE_DEFAULT_LOCALE__,
};

export const dictionaries = __OX_I18N_DICTIONARIES__;

export function t(key, params, locale) {
  const dict = dictionaries[locale || i18nConfig.defaultLocale] || {};
  let message = dict[key];
  if (!message) {
    const fallback = dictionaries[i18nConfig.defaultLocale] || {};
    message = fallback[key] || key;
  }
  if (params) {
    for (const [k, v] of Object.entries(params)) {
      message = message.split("{$" + k + "}").join(String(v));
    }
  }
  return message;
}

export function getLocaleFromPath(pathname) {
  const match = pathname.match(new RegExp("^/([A-Za-z]{2,3}(?:-[A-Za-z0-9]+)*)(/|$)"));
  if (match) {
    const code = match[1];
    if (i18nConfig.locales.some((l) => l.code === code)) {
      return code;
    }
  }
  return i18nConfig.defaultLocale;
}

export function localePath(pathname, locale) {
  const current = getLocaleFromPath(pathname);
  let clean = pathname;
  if (current !== i18nConfig.defaultLocale || !i18nConfig.hideDefaultLocale) {
    const prefix = "/" + current;
    if (clean === prefix) clean = "/";
    else if (clean.startsWith(prefix + "/")) clean = clean.slice(prefix.length);
  }
  if (locale === i18nConfig.defaultLocale && i18nConfig.hideDefaultLocale) {
    return clean || "/";
  }
  return "/" + locale + (clean.startsWith("/") ? clean : "/" + clean);
}

const formatterCache = new Map();

function getFormatter(kind, locale, options) {
  const key = kind + ":" + locale + ":" + JSON.stringify(options || {});
  if (!formatterCache.has(key)) {
    formatterCache.set(key, new Intl[kind](locale, options));
  }
  return formatterCache.get(key);
}

export function getLocaleMeta(locale) {
  const code = locale || i18nConfig.defaultLocale;
  return i18nConfig.locales.find((l) => l.code === code) || { code, name: code, dir: "ltr" };
}

export function formatDate(value, options, locale) {
  return getFormatter("DateTimeFormat", locale || i18nConfig.defaultLocale, options).format(
    value instanceof Date ? value : new Date(value),
  );
}

export function formatDateParts(value, options, locale) {
  return getFormatter("DateTimeFormat", locale || i18nConfig.defaultLocale, options).formatToParts(
    value instanceof Date ? value : new Date(value),
  );
}

export function formatNumber(value, options, locale) {
  return getFormatter("NumberFormat", locale || i18nConfig.defaultLocale, options).format(value);
}

export function formatNumberParts(value, options, locale) {
  return getFormatter("NumberFormat", locale || i18nConfig.defaultLocale, options).formatToParts(
    value,
  );
}

export function formatRelativeTime(value, unit, options, locale) {
  return getFormatter("RelativeTimeFormat", locale || i18nConfig.defaultLocale, options).format(
    value,
    unit,
  );
}

export function formatList(values, options, locale) {
  return getFormatter("ListFormat", locale || i18nConfig.defaultLocale, options).format(values);
}

export function formatListParts(values, options, locale) {
  return getFormatter("ListFormat", locale || i18nConfig.defaultLocale, options).formatToParts(
    values,
  );
}

export function formatDisplayName(value, type, options, locale) {
  if (!Intl.DisplayNames) return String(value);
  const displayType = type || "language";
  return (
    getFormatter("DisplayNames", locale || i18nConfig.defaultLocale, {
      type: displayType,
      ...options,
    }).of(value) || String(value)
  );
}

export function createIntl(locale, defaults = {}) {
  const meta = getLocaleMeta(locale);
  const code = meta.code;
  return {
    locale: code,
    meta,
    dir: meta.dir || "ltr",
    date: (value, options) => formatDate(value, { ...defaults.date, ...options }, code),
    dateParts: (value, options) => formatDateParts(value, { ...defaults.date, ...options }, code),
    number: (value, options) => formatNumber(value, { ...defaults.number, ...options }, code),
    numberParts: (value, options) =>
      formatNumberParts(value, { ...defaults.number, ...options }, code),
    relativeTime: (value, unit, options) =>
      formatRelativeTime(value, unit, { ...defaults.relativeTime, ...options }, code),
    list: (values, options) => formatList(values, { ...defaults.list, ...options }, code),
    listParts: (values, options) => formatListParts(values, { ...defaults.list, ...options }, code),
    displayName: (value, type, options) =>
      formatDisplayName(value, type, { ...defaults.displayName, ...options }, code),
  };
}

export default {
  i18nConfig,
  dictionaries,
  t,
  getLocaleFromPath,
  localePath,
  getLocaleMeta,
  createIntl,
  formatDate,
  formatDateParts,
  formatNumber,
  formatNumberParts,
  formatRelativeTime,
  formatList,
  formatListParts,
  formatDisplayName,
};
