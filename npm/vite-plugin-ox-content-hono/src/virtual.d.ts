declare module "virtual:ox-content-hono/modules" {
  const modules: Readonly<Record<string, () => Promise<unknown>>>;
  export default modules;
}

declare module "virtual:ox-content-hono/client" {}
declare module "*.md" {
  import type { JSXNode } from "hono/jsx";
  export const frontmatter: Readonly<Record<string, unknown>>;
  export interface MarkdownHeading {
    depth: number;
    text: string;
    slug: string;
    children: readonly MarkdownHeading[];
  }
  export const toc: readonly MarkdownHeading[];
  const MarkdownContent: (props: { class?: string }) => Promise<JSXNode>;
  export default MarkdownContent;
}
declare module "*.markdown" {
  export { default, frontmatter, toc } from "*.md";
}
declare module "*.mdx" {
  export { default, frontmatter, toc } from "*.md";
}
