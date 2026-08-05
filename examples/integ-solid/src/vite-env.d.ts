/// <reference types="vite/client" />

declare module "*.md" {
  import type { Component } from "solid-js";

  export const frontmatter: Record<string, unknown>;
  const MarkdownContent: Component;
  export default MarkdownContent;
}

declare module "*.markdown" {
  import type { Component } from "solid-js";

  export const frontmatter: Record<string, unknown>;
  const MarkdownContent: Component;
  export default MarkdownContent;
}

declare module "*.mdx" {
  import type { Component } from "solid-js";

  export const frontmatter: Record<string, unknown>;
  const MarkdownContent: Component;
  export default MarkdownContent;
}
