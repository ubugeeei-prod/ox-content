---
title: Slides
description: Build source-first slide decks from Markdown with a browser editor, presenter mode, PDF export, and typed custom renderers.
---

# Slides

Ox Content Slides is a source-first presentation pipeline: one Markdown file per
slide by default, browser editing during development, static HTML for every
slide, presenter mode, per-slide SEO metadata, and optional PDF export.

The built slide pages are ordinary multi-page HTML. They do not need a client
runtime for navigation, which keeps the public bundle small while preserving a
fast authoring loop in the dev server.

## Install

```bash
vp install @ox-content/vite-plugin-slides
```

```ts
// vite.config.ts
import { defineConfig } from "vite";
import { oxContentSlides } from "@ox-content/vite-plugin-slides";

export default defineConfig({
  plugins: [
    oxContentSlides({
      srcDir: "slides",
      routeBase: "slides",
      pdf: true,
      ssg: {
        generateOgImage: true,
        siteUrl: "https://example.com",
      },
    }),
  ],
});
```

## Authoring Model

The first-choice layout is `./slides` with one file per slide:

```txt
slides/
  0001.md
  0002.md
  0003.md
```

Each numeric file becomes a slide in the root deck. Nested folders create
additional decks:

```txt
slides/
  product-demo/
    0001.md
    0002.md
  roadmap/
    0001.md
    0002.md
```

For quick imports or small decks, a single Markdown file can still contain
multiple slides separated by `---`, but separate files remain the most editable,
reviewable, and automation-friendly shape.

## Speaker Notes

Put speaker notes in HTML comments. They render in presenter mode and stay out
of the public slide body.

```md
# Launch Plan

- Ship the static deck
- Share the PDF artifact

<!-- notes:
Open with the desired audience outcome.
Call out the fallback PDF before the demo.
-->
```

## Browser Editor

During `vite dev`, the plugin mounts a GUI editor at `/slides/editor/` by
default. The editor reads and writes the files under `srcDir`, shows the deck
outline, previews the selected slide, and can create the next numbered Markdown
slide in the current deck. The inspector controls write ordinary frontmatter, so
layout edits made in the GUI remain easy to review and edit by hand.

```md
---
layout: "split"
align: "center"
density: "spacious"
accent: "#111111"
---

# Source-first editing

- GUI controls update this frontmatter
- Text edits and automation use the same file
```

Supported layout controls:

| Control | Values                                                   |
| ------- | -------------------------------------------------------- |
| Layout  | `stack`, `statement`, `split`, `quote`, `code`, `canvas` |
| Align   | `start`, `center`, `end`                                 |
| Density | `compact`, `balanced`, `spacious`                        |
| Accent  | CSS color token used by links, strong text, and quote    |

The `canvas` layout turns each top-level Markdown block into a movable layer in
the preview. Drag the layer body to place it and drag the corner handle to
resize it. The canvas shows a subtle grid, center guides, and light snap
correction while dragging, but the editor still writes the result back to
frontmatter as `placements`, so direct manipulation, text editing, code review,
and automation all share one source file.

```md
---
layout: "canvas"
placements: '[{"x":6,"y":8,"w":48,"h":22},{"x":54,"y":18,"w":36,"h":64}]'
---

# Direct placement

- This list can be moved and resized independently.
```

Disable or move it when needed:

```ts
oxContentSlides({
  editor: false,
});

oxContentSlides({
  editor: {
    route: "deckhouse",
  },
});
```

The editor is dev-server only. It is not emitted into the production deck, so it
does not add JavaScript to built slide pages.

## SEO Output

Each slide is a separate URL, so crawlers can index individual ideas rather than
a single opaque app shell. When `ssg.siteUrl` is set, slide pages receive a
canonical URL. Slide title, deck title, description, social preview tags, and
per-slide OG images are emitted when enabled.

```ts
oxContentSlides({
  ssg: {
    siteUrl: "https://example.com",
    generateOgImage: true,
  },
});
```

## PDF Export

Set `pdf: true` to emit a deck-wide print shell and `deck.pdf` during build.
The PDF renderer uses the same generated HTML as the deck, which keeps browser
preview and exported output aligned.

```ts
oxContentSlides({
  pdf: {
    fileName: "launch-plan.pdf",
    pageWidth: "10in",
    pageHeight: "5.625in",
  },
});
```

## Framework And MDX Sources

Markdown and HTML work without custom setup. Other source formats are supported
through typed renderers, so teams can bring their own MDX, React, Vue SFC, or
Svelte SFC compiler without coupling the slide package to one UI framework.

```ts
import { defineConfig } from "vite";
import { oxContentSlides, type SlideSourceRenderer } from "@ox-content/vite-plugin-slides";

const mdxRenderer: SlideSourceRenderer = async (source, context) => {
  const html = await renderMdxToHtml(source, {
    filePath: context.filePath,
  });

  return {
    html,
    title: "MDX slide",
    frontmatter: { format: "mdx" },
  };
};

export default defineConfig({
  plugins: [
    oxContentSlides({
      extensions: [".md", ".mdx"],
      renderers: {
        ".mdx": mdxRenderer,
      },
    }),
  ],
});
```

The same renderer contract works for `.tsx`, `.vue`, and `.svelte` files:
compile the file to static HTML, return the HTML plus optional title,
description, frontmatter, and notes, then let the slide shell handle routing,
presenter mode, SEO, and PDF output.

## Themes

Themes are pluggable token modules. Export a normal TypeScript object and pass
it to the plugin.

```ts
// slide-theme.ts
import type { SlideThemeConfig } from "@ox-content/vite-plugin-slides";

export const editorialTheme = {
  fontSans: '"IBM Plex Sans", system-ui, sans-serif',
  fontMono: '"IBM Plex Mono", ui-monospace, monospace',
  colorPrimary: "#111111",
  surfaceRadius: "4px",
  padding: "4.8rem",
} satisfies SlideThemeConfig;
```

```ts
import { editorialTheme } from "./slide-theme";

oxContentSlides({
  theme: editorialTheme,
});
```

## AI-Ready Source

The recommended `./slides/0001.md` layout is intentionally plain. Agents,
scripts, and review tools can create, reorder, diff, summarize, and translate
slides without reverse-engineering a binary deck format.

Keep slide files small, use frontmatter for structured metadata, and keep
speaker notes beside the slide source. That gives both the text editor and the
browser editor the same source of truth.

## Reference

| Option           | Type                                  | Default                         | Purpose                                  |
| ---------------- | ------------------------------------- | ------------------------------- | ---------------------------------------- |
| `srcDir`         | `string`                              | `"slides"`                      | Source directory                         |
| `outDir`         | `string`                              | `"dist"`                        | Build output directory                   |
| `routeBase`      | `string`                              | `"slides"`                      | Public route segment                     |
| `editor`         | `boolean \| SlideEditorOptions`       | `true`                          | Dev-server GUI editor                    |
| `presenter`      | `boolean`                             | `true`                          | Presenter-mode routes                    |
| `separator`      | `string`                              | `"---"`                         | Single-file deck separator               |
| `pdf`            | `boolean \| SlidePdfOptions`          | `false`                         | PDF export                               |
| `theme`          | `SlideThemeConfig`                    | `{}`                            | Shell theme tokens                       |
| `extensions`     | `string[]`                            | `[".md", ".markdown", ".html"]` | Source extensions                        |
| `renderers`      | `Record<string, SlideSourceRenderer>` | `{}`                            | Custom source renderers                  |
| `ssg`            | `boolean \| SsgOptions`               | `true`                          | Static output, SEO, and OG configuration |
| `ogImageOptions` | `OgImageOptions`                      | inherited defaults              | Per-slide OG image rendering             |
| `animations`     | `boolean`                             | `true`                          | Built-in shell transitions               |

## Build

```bash
vp build
```

The build emits slide HTML, presenter pages, OG images when configured, a print
shell, and the PDF when `pdf` is enabled.
