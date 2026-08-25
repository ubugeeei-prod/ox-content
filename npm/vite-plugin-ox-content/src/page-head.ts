import { importNapiModuleSync } from "./napi";

/** How invalid head descriptors are reported. */
export type HeadValidationMode = false | "off" | "warn" | "strict";

export interface SiteHead {
  name?: string;
  url?: string;
  locale?: string;
  titleTemplate?: string;
}

export interface HeadMeta {
  key?: string;
  name?: string;
  property?: string;
  httpEquiv?: string;
  content: string;
}

export interface HeadLink {
  key?: string;
  rel: string;
  href: string;
  hreflang?: string;
  type?: string;
  sizes?: string;
}

export interface HeadAlternate {
  lang: string;
  href: string;
}

export interface HeadJsonLd {
  key?: string;
  json: string;
}

/**
 * Build-time page-head input. Unhead-shaped, no client runtime.
 *
 * Unknown keys such as `twitter.imggg` are a TypeScript error here. Use
 * `metas` / `links` for extra tags.
 */
export interface HeadInput {
  site?: SiteHead;
  title?: string;
  titleTemplate?: string;
  titleSuffix?: boolean;
  description?: string;
  canonical?: string;
  robots?: string;
  ogImage?: string;
  ogType?: string;
  twitterCard?: "summary" | "summary_large_image" | (string & {});
  social?: boolean;
  emitSiteName?: boolean;
  trusted?: boolean;
  metas?: HeadMeta[];
  links?: HeadLink[];
  alternates?: HeadAlternate[];
  jsonLd?: HeadJsonLd[];
  validation?: "off" | "warn" | "strict";
}

export interface HeadDiagnostic {
  strict: boolean;
  message: string;
}

export interface RenderedHead {
  html: string;
  diagnostics: HeadDiagnostic[];
}

/** Resolve descriptors to escaped `<head>` markup. Build-time only. */
export function renderHead(input: HeadInput): RenderedHead {
  return importNapiModuleSync().renderHead(JSON.stringify(input));
}

export function resolveHeadValidation(
  value: HeadValidationMode | undefined,
): false | "warn" | "strict" {
  if (value === "warn" || value === "strict") {
    return value;
  }
  return false;
}

export function reportHeadDiagnostics(
  diagnostics: HeadDiagnostic[],
  validation: false | "warn" | "strict",
): void {
  if (!validation || diagnostics.length === 0) {
    return;
  }
  const fatal = diagnostics.filter((item) => item.strict);
  if (validation === "strict" && fatal.length > 0) {
    throw new Error(`[ox-content] ${fatal[0].message}`);
  }
  if (validation === "warn") {
    for (const item of diagnostics) {
      console.warn(`[ox-content] ${item.message}`);
    }
  }
}
