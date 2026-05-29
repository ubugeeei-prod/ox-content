import { defineConfig } from "vite-plus";
import { oxContent, defineTheme, defaultTheme } from "@ox-content/vite-plugin";
import { oxContentHighlightTheme } from "./ox-content-highlight-theme";

/**
 * Ox Content Documentation Site
 *
 * Dogfooding: Using ox-content to build ox-content's own documentation.
 * Uses SSG to generate static HTML from Markdown files.
 */
export default defineConfig(() => {
  // Void serves the site at the domain root (not a subpath like GitHub Pages did).
  const base = "/";

  return {
    base,

    plugins: [
      oxContent({
        srcDir: "content",
        outDir: "dist/docs",
        base,

        // Enable per-page OG image generation (Chromium-based)
        ogImage: true,

        // SSG options with theme customization
        ssg: {
          siteName: "Ox Content",
          siteUrl: "https://ox-content.void.app",
          generateOgImage: true,
          ogImage: "https://ox-content.void.app/og-image.png",
          theme: defineTheme({
            extends: defaultTheme,
            header: {
              logo: "oxcontent-dark.svg",
              logoLight: "oxcontent-dark.svg",
              logoDark: "oxcontent-light.svg",
              showSiteNameText: false,
              logoWidth: 176,
              logoHeight: 37,
            },
            embed: {
              head: `
                <link rel="icon" href="${base}logo-icon.svg" type="image/svg+xml">
                <link rel="shortcut icon" href="${base}logo-icon.svg" type="image/svg+xml">
                <link rel="apple-touch-icon" href="${base}logo-icon.svg">
                <meta name="theme-color" content="#ffffff" media="(prefers-color-scheme: light)">
                <meta name="theme-color" content="#060816" media="(prefers-color-scheme: dark)">
                <link rel="preconnect" href="https://fonts.googleapis.com">
                <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
                <link href="https://fonts.googleapis.com/css2?family=IBM+Plex+Sans:wght@400;500;600;700;800&family=IBM+Plex+Mono:wght@400;500;600&display=swap" rel="stylesheet">
              `,
            },
            footer: {
              message:
                'Released under the <a href="https://opensource.org/licenses/MIT">MIT License</a>.',
              copyright: `Copyright © 2024-${new Date().getFullYear()} ubugeeei`,
            },
            sidebar: [
              {
                text: "Guide",
                items: [
                  { text: "Getting Started", link: "/getting-started.md" },
                  { text: "Theming", link: "/theming.md" },
                  { text: "Examples", link: "/examples/index.md" },
                ],
              },
              {
                text: "Advanced",
                items: [
                  { text: "Architecture", link: "/architecture.md" },
                  { text: "Performance", link: "/performance.md" },
                  { text: "Profiling Mode", link: "/profiling.md" },
                  { text: "mdast Bridge Example", link: "/examples/unplugin-mdast-bridge.md" },
                  {
                    text: "markdown-it Token Bridge",
                    link: "/examples/unplugin-markdown-it-token-bridge.md",
                  },
                  { text: "Development Setup", link: "/development-setup.md" },
                  {
                    text: "Editor Extension Roadmap",
                    link: "/editor-extension-roadmap.md",
                  },
                ],
              },
              {
                text: "Reference",
                collapsed: true,
                items: [
                  { text: "API Reference", link: "/api/index.md" },
                  { text: "Vite Plugin", link: "/packages/vite-plugin-ox-content.md" },
                  { text: "N-API", link: "/packages/napi.md" },
                  { text: "WebAssembly", link: "/packages/wasm.md" },
                  { text: "Vue Integration", link: "/packages/vite-plugin-ox-content-vue.md" },
                  { text: "React Integration", link: "/packages/vite-plugin-ox-content-react.md" },
                  {
                    text: "Svelte Integration",
                    link: "/packages/vite-plugin-ox-content-svelte.md",
                  },
                  { text: "i18n Package", link: "/packages/i18n.md" },
                ],
              },
            ],
            css: `
              .content h1,
              .hero-name {
                letter-spacing: -0.04em;
              }
            `,
          }),
        },

        // Enable syntax highlighting with Shiki
        highlight: true,
        highlightTheme: oxContentHighlightTheme,
        codeAnnotations: {
          notation: "both",
        },

        // Mermaid diagrams (native mmdc via NAPI)
        mermaid: true,

        // API documentation generation (like cargo doc)
        docs: {
          enabled: true,
          src: ["../npm/vite-plugin-ox-content/src"],
          out: "content/api",
          include: ["**/*.ts"],
          exclude: ["**/*.test.*"],
          toc: true,
          groupBy: "file",
          githubUrl: "https://github.com/ubugeeei/ox-content",
          generateNav: true,
        },
      }),
    ],

    server: {
      port: 4173,
    },

    preview: {
      port: 4173,
    },

    build: {
      outDir: "dist/docs",
    },
  };
});
