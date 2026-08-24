# Basic SSG with Vite

This example demonstrates how to use Ox Content with Vite for static site generation.

## Setup

Create a new Vite project and install dependencies:

```bash
npm create vite@latest my-docs -- --template vanilla-ts
cd my-docs
npm install @ox-content/vite-plugin
```

## Configuration

Create or update `vite.config.ts`:

```typescript
import { defineConfig } from "vite";
import { oxContent } from "@ox-content/vite-plugin";

export default defineConfig({
  plugins: [
    oxContent({
      srcDir: "docs",
      outDir: "dist",
      gfm: true,
      highlight: true,
      highlightTheme: "github-dark",
      // SSG is enabled by default
    }),
  ],
});
```

## Usage

Create markdown files in your `docs` directory:

```markdown
---
title: My First Page
description: A sample page using Ox Content
---

# Hello World

This is my first page with **Ox Content**.
```

### Using as Module Import

Import and use in your application:

```typescript
import content from "./docs/hello.md";

document.getElementById("app").innerHTML = content.html;
```

### Using SSG (Static HTML Generation)

By default, Ox Content generates static HTML files for each Markdown file during build:

```
docs/
  index.md      -> dist/index.html
  guide.md      -> dist/guide/index.html
  api/
    intro.md    -> dist/api/intro/index.html
```

## Building for Production

```bash
npm run build
```

The output will be in the `dist` directory, ready for deployment. Each Markdown file will have a corresponding HTML file.

## Disabling SSG

If you only want to use Ox Content as a module transformer (without generating static HTML files):

```typescript
oxContent({
  srcDir: "docs",
  ssg: false, // Disable SSG
});
```

## Bare Mode

For benchmarking or when using custom post-processing, use bare mode to output minimal HTML without navigation or styles:

```typescript
oxContent({
  srcDir: "docs",
  ssg: {
    bare: true,
  },
});
```

## Features

- Fast Rust-based Markdown parsing
- Static HTML generation (SSG) by default
- Sidebar navigation (auto-generated from files)
- Dark mode support
- Syntax highlighting with Shiki
- GitHub Flavored Markdown support
- Frontmatter parsing
- Table of contents generation
- Clean URLs (e.g., `/guide/` instead of `/guide.html`)
- Bare mode for benchmarking
