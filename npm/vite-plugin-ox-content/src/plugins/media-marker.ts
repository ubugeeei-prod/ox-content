import { importNapiModule } from "../napi";

/**
 * True when the document contains any tag the transform could rewrite.
 *
 * The list comes from the Rust registry rather than a copy kept here. The copy
 * had already drifted: `codesandbox` was missing, so a page whose only embed
 * was a `<CodeSandbox>` skipped the transform entirely and shipped the raw tag.
 */
export async function hasMediaMarker(html: string): Promise<boolean> {
  return (await markerPattern()).test(html);
}

interface NapiMediaModule {
  mediaEmbedTags(): { name: string; pascalOnly: boolean }[];
}

/**
 * Providers that live in this package rather than the Rust registry, so the
 * registry cannot name them. Reddit is fetched here, not rendered there.
 */
const TYPESCRIPT_ONLY_TAGS = ["reddit"] as const;

let cachedPattern: RegExp | undefined;

async function markerPattern(): Promise<RegExp> {
  if (cachedPattern) return cachedPattern;
  const mod = (await importNapiModule()) as unknown as NapiMediaModule;
  const tags = mod.mediaEmbedTags();
  const anyCase = [
    ...tags.filter((tag) => !tag.pascalOnly).map((tag) => tag.name),
    ...TYPESCRIPT_ONLY_TAGS,
  ];
  const pascalOnly = tags.filter((tag) => tag.pascalOnly).map((tag) => tag.name);
  // Two alternations because the Pascal-only tags must not match their
  // lowercase spelling — `<audio>` stays a plain audio element.
  const parts = [`(?:<(?:${anyCase.join("|")})[\\s/>])`];
  if (pascalOnly.length > 0) {
    parts.push(`(?:<(?:${pascalOnly.map(capitalize).join("|")})[\\s/>])`);
  }
  cachedPattern = new RegExp(parts.join("|"), "i");
  return cachedPattern;
}

function capitalize(value: string): string {
  return value.charAt(0).toUpperCase() + value.slice(1);
}
