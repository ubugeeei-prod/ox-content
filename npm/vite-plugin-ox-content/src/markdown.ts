import * as path from "path";

export const DEFAULT_MARKDOWN_EXTENSIONS = [".md", ".markdown", ".mdx"] as const;

export function normalizeMarkdownExtensions(extensions?: readonly string[]): string[] {
  const values = extensions?.length ? extensions : DEFAULT_MARKDOWN_EXTENSIONS;
  const seen = new Set<string>();
  const normalized: string[] = [];

  for (const extension of values) {
    const value = extension.startsWith(".") ? extension : `.${extension}`;
    const key = value.toLowerCase();
    if (!seen.has(key)) {
      seen.add(key);
      normalized.push(value);
    }
  }

  return normalized;
}

export function isMarkdownFilePath(
  filePath: string,
  extensions: readonly string[] = DEFAULT_MARKDOWN_EXTENSIONS,
): boolean {
  const pathname = filePath.split("?")[0].split("#")[0].toLowerCase();
  return extensions.some((extension) => pathname.endsWith(extension.toLowerCase()));
}

export function stripMarkdownExtension(
  filePath: string,
  extensions: readonly string[] = DEFAULT_MARKDOWN_EXTENSIONS,
): string {
  const match = [...extensions]
    .sort((left, right) => right.length - left.length)
    .find((extension) => filePath.toLowerCase().endsWith(extension.toLowerCase()));

  return match ? filePath.slice(0, -match.length) : filePath;
}

export function markdownGlobPattern(srcDir: string, extensions: readonly string[]): string {
  const suffixes = extensions.map((extension) => extension.replace(/^\./, ""));
  if (suffixes.length === 1) {
    return path.join(srcDir, `**/*.${suffixes[0]}`);
  }
  return path.join(srcDir, `**/*.{${suffixes.join(",")}}`);
}
