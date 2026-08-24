export function sourcePathname(id: string): string {
  return id.split("?")[0].split("#")[0];
}

export function isConfiguredSourceFile(id: string, extensions: readonly string[]): boolean {
  const pathname = sourcePathname(id).toLowerCase();
  return extensions.some((extension) => pathname.endsWith(extension.toLowerCase()));
}

export function isMdxSourceFile(id: string): boolean {
  return sourcePathname(id).toLowerCase().endsWith(".mdx");
}

export function resolveMdxForSourceFile(id: string, configured?: boolean): boolean {
  return configured ?? isMdxSourceFile(id);
}
