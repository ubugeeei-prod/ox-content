/**
 * Conservative TypeScript-to-JavaScript stripper for documentation samples.
 * Full checking goes through tsgo; this path only needs to run the snippet.
 */
export function stripTypeScript(code: string): string {
  return code
    .replace(/^\s*import\s+type\s+.*$/gm, "")
    .replace(/^\s*export\s+type\s+\w[\s\S]*?;\s*$/gm, "")
    .replace(/^\s*type\s+\w[\s\S]*?;\s*$/gm, "")
    .replace(/^\s*(?:export\s+)?interface\s+\w[\s\S]*?\{[\s\S]*?\n\}\s*$/gm, "")
    .replace(/\s+as\s+const\b/g, "")
    .replace(/\s+as\s+[^=,;)\n]+/g, "")
    .replace(/\s+satisfies\s+[^=,;)\n]+/g, "")
    .replace(/\)\s*:\s*[^{;=\n]+/g, ")")
    .replace(/([?]?)\s*:\s*[^,)=;{\n]+/g, "$1");
}
