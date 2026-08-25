/// <reference types="vite-plus/client" />

type OxContentTocEntry = {
  depth: number;
  text: string;
  slug: string;
  children: OxContentTocEntry[];
};

type OxContentModule = {
  html: string;
  frontmatter: Record<string, unknown>;
  toc: OxContentTocEntry[];
};

declare module "*.md" {
  const content: OxContentModule;
  export default content;
  export const html: string;
  export const frontmatter: Record<string, unknown>;
  export const toc: OxContentModule["toc"];
}

declare module "*.mdx" {
  const content: OxContentModule;
  export default content;
  export const html: string;
  export const frontmatter: Record<string, unknown>;
  export const toc: OxContentModule["toc"];
}
