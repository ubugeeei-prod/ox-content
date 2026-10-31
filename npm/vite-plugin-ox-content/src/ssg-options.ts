import type {
  A11yOptions,
  JsonLdOptions,
  JsonLdPublisherOptions,
  ReaderChromeOptions,
  ResolvedA11y,
  ResolvedJsonLd,
  ResolvedReaderChrome,
  ResolvedSsgOptions,
  SsgOptions,
} from "./types";
import { resolveBlogOptions } from "./blog";
import { resolveContributorsOption } from "./contributors";
import { resolvePageChromeOption } from "./header-chrome";
import { resolveLocaleSwitcherOption } from "./locale-switcher";
import { resolveMarkdownSourceOptions } from "./markdown-source";
import { resolveNotFoundOptions } from "./not-found";
import { resolveHeadValidation } from "./page-head";
import { resolveSectionIndexOptions } from "./section-index";
import { resolveTeamOptions } from "./team";
import { resolveTheme } from "./theme";

const DEFAULT_SSG_TRANSFORM_CONCURRENCY = 1;
const MAX_SSG_TRANSFORM_CONCURRENCY = 32;

/** Normalizes `blog` / `/blog` / `/blog/` to `blog`. Empty or unsafe values stay off. */
export function normalizeRoutePrefix(value?: string): string | undefined {
  if (!value) {
    return undefined;
  }
  const trimmed = value.trim();
  if (
    !trimmed ||
    trimmed.includes("\\") ||
    trimmed.includes("\0") ||
    trimmed.includes("://") ||
    trimmed.startsWith("//") ||
    /^[a-zA-Z]:/u.test(trimmed)
  ) {
    return undefined;
  }
  const segments = trimmed
    .replaceAll(/^\/+|\/+$/gu, "")
    .split("/")
    .filter((segment) => segment.length > 0);
  if (segments.length === 0 || segments.some((segment) => segment === "." || segment === "..")) {
    return undefined;
  }
  return segments.join("/");
}

function resolvedRoutePrefix(value?: string): { routePrefix: string } | Record<string, never> {
  const prefix = normalizeRoutePrefix(value);
  return prefix ? { routePrefix: prefix } : {};
}

export function resolveSsgTransformConcurrency(value?: number): number {
  if (value === undefined || !Number.isFinite(value)) {
    return DEFAULT_SSG_TRANSFORM_CONCURRENCY;
  }
  return Math.min(MAX_SSG_TRANSFORM_CONCURRENCY, Math.max(1, Math.trunc(value)));
}

/**
 * Resolves SSG options with defaults.
 */
export function resolveSsgOptions(ssg: SsgOptions | boolean | undefined): ResolvedSsgOptions {
  if (ssg === false) {
    return {
      enabled: false,
      extension: ".html",
      transformConcurrency: resolveSsgTransformConcurrency(undefined),
      externalizeAssets: true,
      clean: false,
      minifyHtml: false,
      bare: false,
      generateOgImage: false,
      lastUpdated: false,
      contributors: resolveContributorsOption(undefined),
      pagination: false,
      breadcrumbs: false,
      jsonLd: false,
      headValidation: false,
      readerChrome: false,
      localeSwitcher: false,
      a11y: false,
      pageChrome: false,
      markdownSource: resolveMarkdownSourceOptions(undefined),
      notFound: resolveNotFoundOptions(undefined),
      team: resolveTeamOptions(undefined),
      blog: resolveBlogOptions(undefined),
      sectionIndex: resolveSectionIndexOptions(undefined),
    };
  }

  if (ssg === true || ssg === undefined) {
    return {
      enabled: true,
      extension: ".html",
      transformConcurrency: resolveSsgTransformConcurrency(undefined),
      externalizeAssets: true,
      clean: false,
      minifyHtml: false,
      bare: false,
      generateOgImage: false,
      lastUpdated: false,
      contributors: resolveContributorsOption(undefined),
      pagination: false,
      breadcrumbs: false,
      jsonLd: false,
      headValidation: false,
      readerChrome: false,
      localeSwitcher: false,
      a11y: false,
      pageChrome: false,
      markdownSource: resolveMarkdownSourceOptions(undefined),
      notFound: resolveNotFoundOptions(undefined),
      team: resolveTeamOptions(undefined),
      blog: resolveBlogOptions(undefined),
      sectionIndex: resolveSectionIndexOptions(undefined),
      theme: resolveTheme(undefined),
    };
  }

  return {
    enabled: ssg.enabled ?? true,
    extension: ssg.extension ?? ".html",
    ...resolvedRoutePrefix(ssg.routePrefix),
    transformConcurrency: resolveSsgTransformConcurrency(ssg.transformConcurrency),
    externalizeAssets: ssg.externalizeAssets ?? true,
    clean: ssg.clean ?? false,
    minifyHtml: ssg.minifyHtml ?? false,
    bare: ssg.bare ?? false,
    render: ssg.render,
    lang: ssg.lang,
    head: ssg.head,
    bodyStart: ssg.bodyStart,
    bodyEnd: ssg.bodyEnd,
    siteName: ssg.siteName,
    ogImage: ssg.ogImage,
    generateOgImage: ssg.generateOgImage ?? false,
    lastUpdated: ssg.lastUpdated ?? false,
    contributors: resolveContributorsOption(ssg.contributors),
    pagination: resolvePaginationOption(ssg.pagination),
    breadcrumbs: resolvePaginationOption(ssg.breadcrumbs),
    jsonLd: resolveJsonLdOption(ssg.jsonLd),
    headValidation: resolveHeadValidation(ssg.headValidation),
    readerChrome: resolveReaderChromeOption(ssg.readerChrome),
    localeSwitcher: resolveLocaleSwitcherOption(ssg.localeSwitcher),
    a11y: resolveA11yOption(ssg.a11y),
    pageChrome: resolvePageChromeOption(ssg.pageChrome),
    markdownSource: resolveMarkdownSourceOptions(ssg.markdownSource),
    notFound: resolveNotFoundOptions(ssg.notFound),
    team: resolveTeamOptions(ssg.team),
    blog: resolveBlogOptions(ssg.blog),
    sectionIndex: resolveSectionIndexOptions(ssg.sectionIndex),
    siteUrl: ssg.siteUrl,
    theme: resolveTheme(ssg.theme),
    navigation: ssg.navigation,
  };
}

function resolvePaginationOption(value: boolean | Record<string, unknown> | undefined): boolean {
  return value === true || (typeof value === "object" && value !== null);
}

function resolveJsonLdOption(value: boolean | JsonLdOptions | undefined): ResolvedJsonLd {
  if (value === true) {
    return { breadcrumbs: true };
  }
  if (value && typeof value === "object") {
    const publisher = resolveJsonLdPublisher(value.publisher);
    return {
      breadcrumbs: value.breadcrumbs !== false,
      ...(publisher ? { publisher } : {}),
      ...(value.type ? { type: value.type } : {}),
      ...(value.graph ? { graph: value.graph } : {}),
    };
  }
  return false;
}

function resolveJsonLdPublisher(
  publisher: JsonLdPublisherOptions | undefined,
): { name?: string; url?: string } | undefined {
  if (!publisher || typeof publisher !== "object") {
    return undefined;
  }
  const name = publisher.name?.trim();
  const url = publisher.url?.trim();
  if (!name && !url) {
    return undefined;
  }
  return {
    ...(name ? { name } : {}),
    ...(url ? { url } : {}),
  };
}

function resolveReaderChromeOption(
  value: boolean | ReaderChromeOptions | undefined,
): ResolvedReaderChrome {
  if (value === true) {
    return { copy: true, externalLinks: true, backToTop: true };
  }
  if (value && typeof value === "object") {
    return {
      copy: value.copy !== false,
      externalLinks: value.externalLinks !== false,
      backToTop: value.backToTop !== false,
    };
  }
  return false;
}

const DEFAULT_SKIP_LINK_LABEL = "Skip to content";

function resolveA11yOption(value: boolean | A11yOptions | undefined): ResolvedA11y {
  if (value === true) {
    return { skipLinkLabel: DEFAULT_SKIP_LINK_LABEL };
  }
  if (value && typeof value === "object") {
    const label = value.skipLinkLabel?.trim();
    return { skipLinkLabel: label || DEFAULT_SKIP_LINK_LABEL };
  }
  return false;
}
