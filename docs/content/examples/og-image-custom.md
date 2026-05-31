# Custom OG Image Templates

Generate unique Open Graph images for each page using a custom template and frontmatter data.

## Overview

Ox Content can generate per-page OG images at build time using Chromium (via Playwright). You provide a template function that takes page metadata as props and returns an HTML string — Ox Content renders it to a 1200x630 PNG.

By default a built-in dark gradient template is used. With the custom template feature, you can write your own template and pass arbitrary frontmatter fields as props.

Supported template formats:

| Extension     | Framework              | Rendering                     |
| ------------- | ---------------------- | ----------------------------- |
| `.ts`         | TypeScript             | Direct HTML string return     |
| `.vue`        | Vue SFC                | SSR via `vue/server-renderer` |
| `.svelte`     | Svelte SFC             | SSR via `svelte/server`       |
| `.tsx`/`.jsx` | React Server Component | SSR via `react-dom/server`    |

## Quick Start

### 1. Enable OG Image Generation

```ts
// vite.config.ts
import { defineConfig } from "vite";
import { oxContent } from "@ox-content/vite-plugin";

export default defineConfig({
  plugins: [
    oxContent({
      srcDir: "content",

      ogImage: true,
      ogImageOptions: {
        template: "./og.tsx", // path relative to project root
      },

      ssg: {
        siteName: "My Site",
      },
    }),
  ],
});
```

### 2. Create a Template

The template file must **default-export a function** that receives props and returns JSX (or an HTML string for `.ts` templates).

```tsx
// og.tsx
export default function OgTemplate(props: {
  title: string;
  description?: string;
  siteName?: string;
  category?: string;
  coverColor?: string;
}) {
  const { title, description, siteName, category, coverColor = "#6366f1" } = props;

  return (
    <>
      <style>{`
        .og {
          width: 100%;
          height: 100%;
          display: flex;
          flex-direction: column;
          justify-content: center;
          padding: 60px 80px;
          background: #fff;
          font-family: system-ui, sans-serif;
        }
        .category {
          color: ${coverColor};
          font-size: 16px;
          font-weight: 600;
          text-transform: uppercase;
        }
        .title {
          font-size: 52px;
          font-weight: 800;
          color: #0f172a;
          margin: 16px 0;
        }
        .description {
          font-size: 22px;
          color: #475569;
        }
        .site-name {
          margin-top: auto;
          font-size: 16px;
          color: #94a3b8;
        }
      `}</style>
      <div className="og">
        {category && <span className="category">{category}</span>}
        <h1 className="title">{title}</h1>
        {description && <p className="description">{description}</p>}
        {siteName && <span className="site-name">{siteName}</span>}
      </div>
    </>
  );
}
```

### 3. Use Frontmatter for Custom Data

Any frontmatter field is passed to your template as a prop:

```markdown
---
title: Getting Started
description: Learn how to set up Ox Content
author: Jane Doe
category: Guide
coverColor: "#059669"
tags:
  - setup
  - tutorial
---

# Getting Started

...
```

In the template, access these as `props.category`, `props.coverColor`, `props.tags`, etc.

## Vue SFC Template

Write your OG image template as a Vue Single File Component:

```vue
<!-- og.vue -->
<script setup lang="ts">
const props = defineProps<{
  title: string;
  description?: string;
  siteName?: string;
  category?: string;
  coverColor?: string;
}>();
const color = props.coverColor ?? "#6366f1";
</script>

<template>
  <div class="og">
    <div
      class="accent-bar"
      :style="{ background: `linear-gradient(90deg, ${color}, ${color}cc)` }"
    />
    <div class="body">
      <span v-if="category" class="category" :style="{ background: color }">{{ category }}</span>
      <h1 class="title">{{ title }}</h1>
      <p v-if="description" class="description">{{ description }}</p>
    </div>
    <span v-if="siteName" class="site-name">{{ siteName }}</span>
  </div>
</template>

<style scoped>
.og {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  padding: 60px 80px;
  background: #fff;
  font-family: system-ui, sans-serif;
}
.accent-bar {
  height: 8px;
}
.body {
  flex: 1;
  display: flex;
  flex-direction: column;
  justify-content: center;
  gap: 16px;
}
.category {
  display: inline-block;
  color: #fff;
  padding: 6px 16px;
  border-radius: 20px;
  font-size: 14px;
  font-weight: 600;
  text-transform: uppercase;
  align-self: flex-start;
}
.title {
  font-size: 52px;
  font-weight: 800;
  color: #0f172a;
  margin: 0;
}
.description {
  font-size: 22px;
  color: #475569;
  margin: 0;
}
.site-name {
  margin-top: auto;
  font-size: 16px;
  color: #94a3b8;
}
</style>
```

```ts
// vite.config.ts
ogImageOptions: {
  template: './og.vue',
  // Optional: use Rust-based compiler instead of @vue/compiler-sfc
  // vuePlugin: 'vizejs',
}
```

Requires `vue` and `@vue/compiler-sfc` as dev dependencies (or `@vizejs/vite-plugin` if using `vuePlugin: 'vizejs'`).

## Svelte SFC Template

Write your OG image template as a Svelte component with runes:

```svelte
<!-- og.svelte -->
<script lang="ts">
  let { title, description, siteName, category, coverColor = '#6366f1' }: {
    title: string;
    description?: string;
    siteName?: string;
    category?: string;
    coverColor?: string;
  } = $props();
</script>

<div class="og">
  <div class="accent-bar" style:background="linear-gradient(90deg, {coverColor}, {coverColor}cc)"></div>
  <div class="body">
    {#if category}
      <span class="category" style:background={coverColor}>{category}</span>
    {/if}
    <h1 class="title">{title}</h1>
    {#if description}
      <p class="description">{description}</p>
    {/if}
  </div>
  {#if siteName}
    <span class="site-name">{siteName}</span>
  {/if}
</div>

<style>
  .og {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    padding: 60px 80px;
    background: #fff;
    font-family: system-ui, sans-serif;
  }
  .accent-bar {
    height: 8px;
  }
  .body {
    flex: 1;
    display: flex;
    flex-direction: column;
    justify-content: center;
    gap: 16px;
  }
  .category {
    display: inline-block;
    color: #fff;
    padding: 6px 16px;
    border-radius: 20px;
    font-size: 14px;
    font-weight: 600;
    text-transform: uppercase;
    align-self: flex-start;
  }
  .title {
    font-size: 52px;
    font-weight: 800;
    color: #0f172a;
    margin: 0;
  }
  .description {
    font-size: 22px;
    color: #475569;
    margin: 0;
  }
  .site-name {
    margin-top: auto;
    font-size: 16px;
    color: #94a3b8;
  }
</style>
```

```ts
// vite.config.ts
ogImageOptions: {
  template: './og.svelte',
}
```

Requires `svelte` (v5+) as a dev dependency.

## React Server Component Template

Write your OG image template as a React component (supports async Server Components with React 19+):

```tsx
// og.tsx
export default function OgTemplate(props: {
  title: string;
  description?: string;
  siteName?: string;
  category?: string;
  coverColor?: string;
}) {
  const { title, description, siteName, category, coverColor = "#6366f1" } = props;

  return (
    <>
      <style>{`
        .og {
          width: 100%;
          height: 100%;
          display: flex;
          flex-direction: column;
          padding: 60px 80px;
          background: #fff;
          font-family: system-ui, sans-serif;
        }
        .accent-bar {
          height: 8px;
        }
        .body {
          flex: 1;
          display: flex;
          flex-direction: column;
          justify-content: center;
          gap: 16px;
        }
        .category {
          display: inline-block;
          color: #fff;
          padding: 6px 16px;
          border-radius: 20px;
          font-size: 14px;
          font-weight: 600;
          text-transform: uppercase;
          align-self: flex-start;
        }
        .title {
          font-size: 52px;
          font-weight: 800;
          color: #0f172a;
          margin: 0;
        }
        .description {
          font-size: 22px;
          color: #475569;
          margin: 0;
        }
        .site-name {
          margin-top: auto;
          font-size: 16px;
          color: #94a3b8;
        }
      `}</style>
      <div className="og">
        <div
          className="accent-bar"
          style={{ background: `linear-gradient(90deg, ${coverColor}, ${coverColor}cc)` }}
        />
        <div className="body">
          {category && (
            <span className="category" style={{ background: coverColor }}>
              {category}
            </span>
          )}
          <h1 className="title">{title}</h1>
          {description && <p className="description">{description}</p>}
        </div>
        {siteName && <span className="site-name">{siteName}</span>}
      </div>
    </>
  );
}
```

```ts
// vite.config.ts
ogImageOptions: {
  template: './og.tsx',
}
```

Requires `react` and `react-dom` (v19+) as dev dependencies.

## Options

All options are set in `ogImageOptions`:

```ts
ogImageOptions: {
  // Path to custom template (relative to project root)
  // Supports: .ts, .vue, .svelte, .tsx, .jsx
  template: './og.tsx',

  // Vue plugin selection (only for .vue templates)
  // 'vitejs' (default) uses @vue/compiler-sfc
  // 'vizejs' uses @vizejs/vite-plugin (Rust-based)
  vuePlugin: 'vitejs',

  // Image dimensions (pixels)
  width: 1200,   // default: 1200
  height: 630,   // default: 630

  // Content-hash based caching (skips re-render when nothing changed)
  cache: true,    // default: true

  // Number of pages rendered in parallel
  concurrency: 4, // default: 1
}
```

## Props Reference

Your template function receives an `OgImageTemplateProps` object:

| Prop          | Type        | Source                                                  |
| ------------- | ----------- | ------------------------------------------------------- |
| `title`       | `string`    | Extracted from first `#` heading or `title` frontmatter |
| `description` | `string?`   | `description` frontmatter                               |
| `siteName`    | `string?`   | `ssg.siteName` from plugin config                       |
| `author`      | `string?`   | `author` frontmatter                                    |
| `tags`        | `string[]?` | `tags` frontmatter                                      |
| `[key]`       | `unknown`   | Any other frontmatter field                             |

The `layout` field is excluded since it controls page rendering, not OG images.

## How It Works

1. **Template bundling** — Your template file is bundled with [rolldown](https://rolldown.rs/) at build time. For SFC formats (`.vue`, `.svelte`, `.tsx`), a framework-specific compiler plugin is applied during bundling, then the component is rendered to HTML via SSR.
2. **Cache key** — A SHA256 hash of the template source + all props + dimensions. When any of these change, the image is re-rendered.
3. **Rendering** — Ox Content opens a Chromium browser (via Playwright), loads your HTML, and takes a screenshot. The browser session is automatically cleaned up via `using` (Explicit Resource Management).
4. **Output** — PNG files are written alongside your HTML output, and `<meta property="og:image">` tags are injected.

## Caching

Images are cached in `.cache/og-images/` based on content hash. The cache invalidates when:

- Template file contents change (SHA256 hash)
- Frontmatter data changes
- Image dimensions change

To force a full rebuild, delete `.cache/og-images/` or set `cache: false`.

## Example

See the full working example at [`examples/og-image-custom/`](https://github.com/ubugeeei-prod/ox-content/tree/main/examples/og-image-custom).

```
examples/og-image-custom/
├── og.ts          # TypeScript template
├── og.vue         # Vue SFC template
├── og.svelte      # Svelte SFC template
├── og.tsx         # React Server Component template
├── vite.config.ts          # Plugin configuration
└── src/content/
    ├── index.md            # coverColor: "#6366f1"
    ├── getting-started.md  # coverColor: "#059669"
    └── advanced-usage.md   # coverColor: "#dc2626"
```

Each page generates an OG image with different accent colors, categories, and tags — all driven by frontmatter. Choose whichever template format matches your stack.
