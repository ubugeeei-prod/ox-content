---
title: Ox Content SSG Example
description: Static site generation with Vite Environment API
author: Ox Content Team
date: 2024-01-01
---

# Ox Content SSG Example

Welcome to the **Ox Content** SSG example! This demonstrates how to use Ox Content with Vite's Environment API for static site generation.

## Features

This example showcases:

- **Markdown Transformation** - Automatic conversion to JavaScript modules
- **Frontmatter Parsing** - YAML frontmatter support
- **Table of Contents** - Auto-generated navigation
- **Hot Module Replacement** - Live updates during development
- **GFM Support** - GitHub Flavored Markdown features
- **Code Annotations** - Opt-in line markers with custom attributes and VitePress-compatible notation

## Code Example

Here's how you import Markdown files:

```typescript
import content from "./content/index.md";

// Access the rendered HTML
console.log(content.html);

// Access frontmatter data
console.log(content.frontmatter.title);

// Access table of contents
content.toc.forEach((entry) => {
  console.log(entry.text, entry.slug);
});
```

## More Pages

- [Code Annotations](./code-annotations.md)

## Task Lists

- [x] Setup Vite project
- [x] Install @ox-content/vite-plugin
- [x] Create Markdown content
- [ ] Deploy to production

## Links and Resources

- [Ox Content Documentation](https://github.com/ubugeeei/ox-content)
- [Vite Environment API](https://vitejs.dev/guide/api-environment.html)

## Conclusion

The Vite Environment API provides a powerful foundation for building SSG tools. Combined with Ox Content's high-performance Markdown parsing, you can build lightning-fast documentation sites.
