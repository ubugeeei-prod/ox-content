import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { defineConfig } from "vite-plus";
import { oxContent, defineTheme, defaultTheme } from "@ox-content/vite-plugin";
import { codePlay } from "@ox-content/code-play";

/**
 * Ox Content Documentation Site
 *
 * Dogfooding: Using ox-content to build ox-content's own documentation.
 * Uses SSG to generate static HTML from Markdown files.
 */
export default defineConfig(({ mode }) => {
  const isProd = mode === "production";
  const base = process.env.OX_CONTENT_DOCS_BASE ?? (isProd ? "/ox-content/" : "/");
  const siteUrl = process.env.OX_CONTENT_DOCS_SITE_URL ?? "https://ubugeeei-prod.github.io";
  const ogImage = new URL("og-image.png", siteUrl.replace(/\/?$/, base)).href;

  return {
    base, // Site base path: GitHub Pages uses /ox-content/, Void uses /.

    plugins: [
      // The gallery lives in public/ but is generated and gitignored, so a
      // fresh checkout (CI included) doesn't have it — without this step the
      // deployed site 404s /theme-gallery.html.
      {
        name: "ox-content-docs:theme-gallery",
        buildStart() {
          const script = fileURLToPath(new URL("../scripts/theme-gallery.mjs", import.meta.url));
          const result = spawnSync("node", [script], { stdio: "inherit" });
          if (result.status !== 0) {
            throw new Error("scripts/theme-gallery.mjs failed");
          }
        },
      },
      oxContent({
        srcDir: "content",
        outDir: "dist/docs",
        base,
        semanticFootnotes: true,
        // Enable per-page OG image generation (Chromium-based)
        ogImage: true,

        // Crawl manifests and collection feeds. siteUrl is set above, so
        // these files are safe to write for the deployed docs site.
        siteMaps: true,
        publishState: true,
        feeds: true,
        versions: {
          current: "3.0.0-alpha",
          entries: [
            {
              id: "3.0.0-alpha",
              label: "3.0.0-alpha",
              prefix: "",
            },
            {
              id: "2.90.0",
              label: "2.90.0",
              prefix: "2.90",
              dir: "versions/2.90",
            },
          ],
        },
        i18n: {
          enabled: true,
          defaultLocale: "en",
          locales: [
            { code: "en", name: "English" },
            { code: "ja", name: "日本語" },
          ],
          hideDefaultLocale: true,
          check: false,
        },

        // Static HTML redirects. Safe: only aliases/map entries emit files,
        // and destinations stay same-origin unless allowExternal is set.
        redirects: true,

        // SSG options with theme customization
        ssg: {
          siteName: "Ox Content",
          siteUrl,
          pagination: true,
          breadcrumbs: true,
          readerChrome: { backToTop: false },
          markdownSource: { copy: true },
          a11y: true,
          pageChrome: true,
          localeSwitcher: true,
          notFound: true,
          generateOgImage: true,
          ogImage,
          theme: defineTheme({
            extends: defaultTheme,
            aside: true,
            nav: [
              { text: { en: "Guide", ja: "ガイド" }, link: `${base}getting-started/` },
              { text: "API", link: `${base}api/` },
            ],
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
                <script type="module" src="${base}ox-code-play.js"></script>
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
                  {
                    text: "Built-in Features",
                    link: "/built-in-features.md",
                    items: [
                      { text: "Markdown Baseline", link: "/built-in/markdown.md" },
                      { text: "Component Matrix", link: "/built-in/component-matrix.md" },
                      { text: "Heading Permalinks", link: "/built-in/heading-permalinks.md" },
                      { text: "Syntax Extensions", link: "/built-in/syntax-extensions.md" },
                      { text: "Cross References", link: "/built-in/cross-references.md" },
                      { text: "Citations", link: "/built-in/citations.md" },
                      { text: "Custom Containers", link: "/built-in/containers.md" },
                      { text: "Cards", link: "/built-in/cards.md" },
                      { text: "Step Lists", link: "/built-in/steps.md" },
                      { text: "File Includes", link: "/built-in/includes.md" },
                      { text: "Markdown Partials", link: "/built-in/partials.md" },
                      { text: "File Tree", link: "/built-in/file-tree.md" },
                      { text: "Data Tables", link: "/built-in/data-tables.md" },
                      { text: "Inline Badges", link: "/built-in/badges.md" },
                      { text: "NotByAI Badge", link: "/built-in/not-by-ai.md" },
                      { text: "Keyboard Keys", link: "/built-in/keyboard-keys.md" },
                      { text: "Abbreviations", link: "/built-in/abbreviations.md" },
                      { text: "Definition Lists", link: "/built-in/definition-lists.md" },
                      { text: "Magic Links", link: "/built-in/magic-links.md" },
                      { text: "Images", link: "/built-in/images.md" },
                      { text: "Page resources", link: "/built-in/resources.md" },
                      { text: "Code Blocks", link: "/built-in/code-blocks.md" },
                      { text: "Code Groups", link: "/built-in/code-groups.md" },
                      { text: "Embeds", link: "/built-in/embeds.md" },
                      { text: "Mermaid Diagrams", link: "/built-in/mermaid.md" },
                      { text: "Math", link: "/built-in/math.md" },
                      { text: "Search", link: "/built-in/search.md" },
                      { text: "Collections", link: "/built-in/collections.md" },
                      { text: "Quality Checks", link: "/built-in/quality-checks.md" },
                      { text: "Typed Hover", link: "/built-in/typed-hover.md" },
                      { text: "Site Generation", link: "/built-in/site-generation.md" },
                      { text: "SSG output primitives", link: "/built-in/ssg-output.md" },
                      { text: "Component styles", link: "/built-in/component-styles.md" },
                      { text: "Page head", link: "/built-in/page-head.md" },
                      { text: "SEO", link: "/built-in/seo.md" },
                      { text: "Previous / Next", link: "/built-in/pagination.md" },
                      { text: "Breadcrumbs", link: "/built-in/breadcrumbs.md" },
                      { text: "Reader Chrome", link: "/built-in/reader-chrome.md" },
                      { text: "Locale Switcher", link: "/built-in/locale-switcher.md" },
                      { text: "Accessibility", link: "/built-in/a11y.md" },
                      { text: "Header chrome", link: "/built-in/header-chrome.md" },
                      { text: "Sitemap / robots / llms.txt", link: "/built-in/site-maps.md" },
                      { text: "Markdown source companions", link: "/built-in/markdown-source.md" },
                      { text: "Draft / unlisted / scheduled", link: "/built-in/drafts.md" },
                      { text: "Permalinks and Cascade", link: "/built-in/permalinks.md" },
                      { text: "Redirects and aliases", link: "/built-in/redirects.md" },
                      { text: "Custom 404", link: "/built-in/not-found.md" },
                      { text: "RSS / Atom / JSON feeds", link: "/built-in/feeds.md" },
                      { text: "Blog", link: "/built-in/blog.md" },
                      { text: "PWA manifest and service worker", link: "/built-in/pwa.md" },
                      { text: "Self-hosted Iconify CSS", link: "/built-in/icons.md" },
                      { text: "Taxonomies", link: "/built-in/taxonomies.md" },
                      { text: "Documentation versioning", link: "/built-in/versioning.md" },
                      { text: "Team / members page", link: "/built-in/team.md" },
                      { text: "Section index pages", link: "/built-in/section-index.md" },
                    ],
                  },
                  { text: "Theming", link: "/theming.md" },
                  { text: "Theme Presets", link: "/theme-presets.md" },
                  { text: "MDX & Components", link: "/mdx.md" },
                  { text: "API Docs from JSDoc", link: "/jsdoc.md" },
                  { text: "Internationalization", link: "/i18n.md" },
                  { text: "Examples", link: "/examples/index.md" },
                  { text: "Credits", link: "/credits.md" },
                ],
              },
              {
                text: "Advanced",
                items: [
                  { text: "Architecture", link: "/architecture.md" },
                  { text: "Panic Prevention", link: "/panic-prevention.md" },
                  { text: "Performance", link: "/performance.md" },
                  { text: "Profiling Mode", link: "/profiling.md" },
                  { text: "mdast Bridge Example", link: "/examples/unplugin-mdast-bridge.md" },
                  {
                    text: "markdown-it Token Bridge",
                    link: "/examples/unplugin-markdown-it-token-bridge.md",
                  },
                  { text: "Development Setup", link: "/development-setup.md" },
                  { text: "Release Operations", link: "/release.md" },
                  { text: "Docs Deployment", link: "/deployment.md" },
                  {
                    text: "Editor Extension Roadmap",
                    link: "/editor-extension-roadmap.md",
                  },
                  { text: "Code Play Roadmap", link: "/code-play-roadmap.md" },
                  { text: "Ox Content 3.0 Roadmap", link: "/v3-roadmap.md" },
                  {
                    text: "Docs Site Feature Roadmap",
                    link: "/docs-site-feature-roadmap.md",
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
                  {
                    text: "Solid Integration",
                    link: "/packages/vite-plugin-ox-content-solid.md",
                  },
                  { text: "i18n Package", link: "/packages/i18n.md" },
                  { text: "Checker and Language Server", link: "/packages/lsp.md" },
                  { text: "Code Play", link: "/packages/code-play.md" },
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

        // Enable native tree-sitter syntax highlighting
        highlight: true,
        codeAnnotations: {
          notation: "both",
        },

        // Opt-in authoring features used by the Built-in Features guides,
        // so those pages can render live examples inline.
        headingPermalinks: true,
        emojiShortcodes: true,
        math: true,
        cjkEmphasis: true,
        badges: true,
        notByAi: true,
        keyboardKeys: true,
        abbreviations: true,
        definitionLists: true,
        magicLinks: {
          aliases: {
            Oxc: {
              href: "https://oxc.rs",
              image: "https://github.com/oxc-project.png",
            },
          },
        },
        citations: {
          bibliography: "references.json",
          rootDir: fileURLToPath(new URL("content", import.meta.url)),
        },
        images: true,
        codeImports: true,
        includes: true,
        partials: true,
        conditionalBlocks: { values: { runtime: "node" } },
        cards: true,
        steps: true,
        codeGroups: true,
        fileTree: true,
        dataTables: true,
        typedHover: true,
        embeds: {
          pm: true,
          twitter: {
            fetch: true,
            mediaOutputDir: "public/twitter",
            mediaPublicPath: `${base}twitter`,
          },
          bluesky: true,
          webContainer: true,
          spotify: true,
          appleMusic: true,
          speakerDeck: true,
          audio: true,
          video: true,
        },
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
          githubUrl: "https://github.com/ubugeeei-prod/ox-content",
          generateNav: true,
        },
      }),
      codePlay({
        languages: {
          javascript: true,
          typescript: { execute: true, typecheck: true },
        },
        srcDir: "content",
        outDir: "dist/docs",
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
