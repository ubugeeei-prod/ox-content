import type { BuiltinEmbedOptions, ResolvedSolidOptions, SolidIntegrationOptions } from "./types";

const DEFAULT_MARKDOWN_EXTENSIONS = [".md", ".markdown", ".mdx"] as const;

export function resolveSolidOptions(
  options: SolidIntegrationOptions,
): Omit<ResolvedSolidOptions, "components"> {
  return {
    srcDir: options.srcDir ?? "docs",
    outDir: options.outDir ?? "dist",
    base: options.base ?? "/",
    extensions: normalizeMarkdownExtensions(options.extensions),
    gfm: options.gfm ?? true,
    autolinks: options.autolinks ?? options.gfm ?? true,
    frontmatter: options.frontmatter ?? true,
    toc: options.toc ?? true,
    tocMaxDepth: options.tocMaxDepth ?? 3,
    codeAnnotations: resolveCodeAnnotationsOptions(options.codeAnnotations),
    verifySolidPlugin: options.verifySolidPlugin ?? true,
    embeds: resolveBuiltinEmbedOptions(options.embeds),
  };
}

export function normalizeMarkdownExtensions(extensions?: readonly string[]): string[] {
  const values = extensions?.length ? extensions : DEFAULT_MARKDOWN_EXTENSIONS;
  return Array.from(
    new Map(
      values.map((extension) => {
        const value = extension.startsWith(".") ? extension : `.${extension}`;
        return [value.toLowerCase(), value] as const;
      }),
    ).values(),
  );
}

export function isMarkdownFilePath(filePath: string, extensions: readonly string[]): boolean {
  const pathname = filePath.split("?")[0].split("#")[0].toLowerCase();
  return extensions.some((extension) => pathname.endsWith(extension.toLowerCase()));
}

function resolveCodeAnnotationsOptions(
  options: SolidIntegrationOptions["codeAnnotations"],
): ResolvedSolidOptions["codeAnnotations"] {
  if (!options) {
    return { enabled: false, metaKey: "annotate" };
  }

  if (options === true) {
    return { enabled: true, metaKey: "annotate" };
  }

  return { enabled: true, metaKey: options.metaKey ?? "annotate" };
}

function resolveBuiltinEmbedOptions(
  options: BuiltinEmbedOptions | false | undefined,
): ResolvedSolidOptions["embeds"] {
  if (options === false) return { github: false, openGraph: false };
  return {
    github: resolveSingleEmbedOptions(options?.github),
    openGraph: resolveSingleEmbedOptions(options?.openGraph),
  };
}

function resolveSingleEmbedOptions<T extends object>(options: boolean | T | undefined): T | false {
  if (options === false) return false;
  if (options === true || options === undefined) return {} as T;
  return options;
}
