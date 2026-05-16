import { defineConfig } from "vite-plus";
import { oxContent, defineTheme, defaultTheme } from "@ox-content/vite-plugin";
import { oxContentHighlightTheme } from "./ox-content-highlight-theme";

/**
 * Ox Content Documentation Site
 *
 * Dogfooding: Using ox-content to build ox-content's own documentation.
 * Uses SSG to generate static HTML from Markdown files.
 */
export default defineConfig(({ mode }) => {
  const isProd = mode === "production";
  const base = isProd ? "/ox-content/" : "/";

  return {
    // Site base path (for GitHub Pages in prod, root for dev)
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
          siteUrl: "https://ubugeeei.github.io",
          generateOgImage: true,
          ogImage: "https://ubugeeei.github.io/ox-content/og-image.png",
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
