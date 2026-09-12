import { stringify as stringifyYaml } from "yaml";
import { importNapiModuleSync } from "./napi";

/** Markdown source split into its frontmatter and the body that follows it. */
export interface ParsedFrontmatter {
  /** Parsed frontmatter keys. Empty when the source declares no frontmatter. */
  frontmatter: Record<string, unknown>;
  /** Markdown content with the frontmatter block removed. */
  content: string;
}

/**
 * Splits Markdown source into its frontmatter and body.
 *
 * This is the same parse the collection pipeline performs, so a document read
 * this way agrees with how Ox Content will read it during a build.
 */
export function parseFrontmatter(source: string): ParsedFrontmatter {
  const prepared = importNapiModuleSync().prepareSource(source, { frontmatter: true });
  return { frontmatter: prepared.frontmatter, content: prepared.content };
}

/**
 * Serialises frontmatter and body back into Markdown source.
 *
 * Round-trips with {@link parseFrontmatter}: the emitted document parses back to
 * the values passed in. Frontmatter key order is preserved so generated files
 * stay stable across runs. Passing no keys returns the body unchanged, since a
 * document without frontmatter should not gain an empty block.
 */
export function stringifyFrontmatter(frontmatter: Record<string, unknown>, content = ""): string {
  if (Object.keys(frontmatter).length === 0) {
    return content;
  }
  return `---\n${stringifyYaml(frontmatter)}---\n${content}`;
}
