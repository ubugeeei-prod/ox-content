import type { LocaleConfig } from "./types";

/**
 * Sibling page or locale-root href for one locale in the default-theme switcher.
 */
export interface SsgLocalePath {
  code: string;
  href?: string;
  root?: string;
}

/**
 * Resolves `ssg.localeSwitcher`. Omitted / `false` stay off. `true` or an
 * object enables the control.
 */
export function resolveLocaleSwitcherOption(
  value: boolean | Record<string, unknown> | undefined,
): boolean {
  return value === true || (typeof value === "object" && value !== null);
}

export function normalizeLocalePath(path: string): string {
  return path.replaceAll("\\", "/").replace(/^\/+|\/+$/g, "");
}

export function remainderPath(urlPath: string, locale: string): string {
  const normalized = normalizeLocalePath(urlPath);
  if (normalized === locale) {
    return "";
  }
  const prefix = `${locale}/`;
  if (normalized.startsWith(prefix)) {
    return normalized.slice(prefix.length);
  }
  return normalized;
}

export function pathForLocale(
  remainder: string,
  locale: string,
  defaultLocale: string,
  hideDefaultLocale: boolean,
): string {
  if (hideDefaultLocale && locale === defaultLocale) {
    return remainder;
  }
  return remainder ? `${locale}/${remainder}` : locale;
}

export function defaultLocaleRoot(base: string, locale: string): string {
  const prefix = base.endsWith("/") ? base : `${base}/`;
  return `${prefix}${locale}/`;
}

export function buildLocalePaths(options: {
  currentPath: string;
  locales: LocaleConfig[];
  defaultLocale: string;
  hideDefaultLocale: boolean;
  pages: Array<{ path: string; href: string }>;
  base: string;
  roots?: Record<string, string>;
}): SsgLocalePath[] {
  const currentLocale =
    options.locales.find((locale) => {
      const normalized = normalizeLocalePath(options.currentPath);
      return normalized === locale.code || normalized.startsWith(`${locale.code}/`);
    })?.code ?? options.defaultLocale;
  const remainder = remainderPath(options.currentPath, currentLocale);
  const existing = new Map(
    options.pages.map((page) => [normalizeLocalePath(page.path), page.href]),
  );

  return options.locales.map((locale) => {
    const sibling = pathForLocale(
      remainder,
      locale.code,
      options.defaultLocale,
      options.hideDefaultLocale,
    );
    const href = existing.get(normalizeLocalePath(sibling));
    const configuredRoot = options.roots?.[locale.code];
    const root =
      configuredRoot ??
      (options.hideDefaultLocale && locale.code === options.defaultLocale
        ? options.base.endsWith("/")
          ? options.base
          : `${options.base}/`
        : defaultLocaleRoot(options.base, locale.code));
    return { code: locale.code, href, root };
  });
}
