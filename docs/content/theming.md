---
title: Theming
description: Customize the appearance of your documentation site with ox-content's Theme API.
---

# Theming

ox-content provides a flexible Theme API that allows you to customize the appearance of your documentation site. You can use CSS variables for simple customization or write full JSX themes for complete control.

## Quick Start

### CSS Variable Customization

```ts
// vite.config.ts
import { defineConfig } from "vite";
import { oxContent, defineTheme, defaultTheme } from "@ox-content/vite-plugin";

export default defineConfig({
  plugins: [
    oxContent({
      ssg: {
        siteName: "My Docs",
        theme: defineTheme({
          extends: defaultTheme,
          colors: {
            primary: "#3498db",
          },
          socialLinks: {
            github: "https://github.com/your/repo",
          },
          footer: {
            message: "Released under the MIT License.",
            copyright: "Copyright © 2024 My Company",
          },
        }),
      },
    }),
  ],
});
```

### JSX Theme (Full Control)

ox-content supports JSX/TSX themes that render to static HTML with **zero client-side JavaScript** by default.

```tsx
// theme/Layout.tsx
import { usePageProps, useSiteConfig, useNav, raw, each } from "@ox-content/vite-plugin";

export function Layout({ children }) {
  const page = usePageProps();
  const site = useSiteConfig();
  const nav = useNav();

  return (
    <html lang="en">
      <head>
        <meta charset="UTF-8" />
        <title>
          {page.title} - {site.name}
        </title>
      </head>
      <body>
        <nav>
          {each(nav, (group) => (
            <div>
              <h3>{group.title}</h3>
              <ul>
                {each(group.items, (item) => (
                  <li>
                    <a href={item.href}>{item.title}</a>
                  </li>
                ))}
              </ul>
            </div>
          ))}
        </nav>
        <main>{children}</main>
      </body>
    </html>
  );
}
```

Configure your `tsconfig.json` for JSX:

```json
{
  "compilerOptions": {
    "jsx": "react-jsx",
    "jsxImportSource": "@ox-content/vite-plugin"
  }
}
```

## CSS Variables Reference

All CSS variables use the `--octc-` prefix for namespacing.

## Entry Page Modes

The default theme supports two landing page modes:

- `default` - the more branded, marketing-style entry page
- `subtle` - a quieter docs.rs-like presentation with tighter spacing and a more restrained hero

```ts
defineTheme({
  extends: defaultTheme,
  entryPage: {
    mode: "subtle",
  },
});
```

### Colors

| Option                  | CSS Variable                 | Description                                   |
| ----------------------- | ---------------------------- | --------------------------------------------- |
| `colors.primary`        | `--octc-color-primary`       | Primary accent color for links, active states |
| `colors.primaryHover`   | `--octc-color-primary-hover` | Primary color on hover                        |
| `colors.background`     | `--octc-color-bg`            | Main background color                         |
| `colors.backgroundAlt`  | `--octc-color-bg-alt`        | Alternative background (sidebar, code blocks) |
| `colors.text`           | `--octc-color-text`          | Main text color                               |
| `colors.textMuted`      | `--octc-color-text-muted`    | Muted/secondary text color                    |
| `colors.border`         | `--octc-color-border`        | Border color                                  |
| `colors.codeBackground` | `--octc-color-code-bg`       | Code block background                         |
| `colors.codeText`       | `--octc-color-code-text`     | Code block text color                         |

### Layout

| Option                   | CSS Variable               | Description                              |
| ------------------------ | -------------------------- | ---------------------------------------- |
| `layout.sidebarWidth`    | `--octc-sidebar-width`     | Sidebar width (default: `260px`)         |
| `layout.headerHeight`    | `--octc-header-height`     | Header height (default: `60px`)          |
| `layout.maxContentWidth` | `--octc-max-content-width` | Maximum content width (default: `960px`) |

### Fonts

| Option       | CSS Variable       | Description           |
| ------------ | ------------------ | --------------------- |
| `fonts.sans` | `--octc-font-sans` | Sans-serif font stack |
| `fonts.mono` | `--octc-font-mono` | Monospace font stack  |

## Page Props & Hooks

Access page data in your theme components using hooks:

### `usePageProps()`

Returns the current page's data:

```tsx
function PageHeader() {
  const page = usePageProps();

  return (
    <header>
      <h1>{page.title}</h1>
      {page.description && <p>{page.description}</p>}
    </header>
  );
}
```

**Available properties:**

- `title` - Page title
- `description` - Page description
- `html` - Rendered HTML content
- `toc` - Table of contents
- `path` - Source file path
- `url` - Output URL
- `frontmatter` - Raw frontmatter object
- `layout` - Layout name

### `useSiteConfig()`

Returns site-wide configuration:

```tsx
function SiteHeader() {
  const site = useSiteConfig();

  return <header>{site.name}</header>;
}
```

### `useNav()`

Returns navigation groups:

```tsx
function Sidebar() {
  const nav = useNav();

  return (
    <nav>
      {each(nav, (group) => (
        <section>
          <h3>{group.title}</h3>
          {each(group.items, (item) => (
            <a href={item.href}>{item.title}</a>
          ))}
        </section>
      ))}
    </nav>
  );
}
```

### `useIsActive(path)`

Checks if a path is the current page:

```tsx
function NavLink({ href, children }) {
  const isActive = useIsActive(href);

  return (
    <a href={href} class={isActive ? "active" : ""}>
      {children}
    </a>
  );
}
```

## JSX Utilities

### `raw(html)`

Renders raw HTML without escaping:

```tsx
<div>{raw(page.html)}</div>
```

### `each(items, render)`

Maps over arrays:

```tsx
{
  each(items, (item, index) => <li key={index}>{item.name}</li>);
}
```

### `when(condition, content)`

Conditional rendering:

```tsx
{
  when(page.toc.length > 0, <aside class="toc">...</aside>);
}
```

## Type Generation

ox-content auto-generates TypeScript types based on your pages' frontmatter. The generated types are saved to your output directory.

```ts
// Generated: page-props.d.ts
export interface PageFrontmatter {
  title: string;
  description?: string;
  layout?: string;
  // ... other fields from your frontmatter
}

export type PageProps = import("@ox-content/vite-plugin").PageProps<PageFrontmatter>;
```

Use the generated types:

```tsx
import type { PageProps } from "./page-props";

function Layout() {
  const page = usePageProps<PageProps["frontmatter"]>();
  // page.frontmatter is now fully typed
}
```

## Layout Switching

Support multiple layouts based on frontmatter:

```tsx
// theme/index.tsx
import { createTheme } from "@ox-content/vite-plugin";
import { DefaultLayout } from "./layouts/Default";
import { EntryLayout } from "./layouts/Entry";
import { BlogLayout } from "./layouts/Blog";

export default createTheme({
  layouts: {
    default: DefaultLayout,
    entry: EntryLayout,
    blog: BlogLayout,
  },
});
```

In your markdown:

```md
---
layout: entry
title: Welcome
---

# Welcome to My Docs
```

## Social Links

Add social links to the header:

```ts
defineTheme({
  extends: defaultTheme,
  socialLinks: {
    github: "https://github.com/your/repo",
    twitter: "https://twitter.com/yourhandle",
    discord: "https://discord.gg/yourserver",
  },
});
```

## Slots

Inject custom HTML at specific locations:

```ts
defineTheme({
  extends: defaultTheme,
  slots: {
    head: '<link rel="preconnect" href="https://fonts.googleapis.com">',
    headerBefore: '<div class="announcement">New version!</div>',
    headerAfter: "",
    sidebarBefore: "",
    sidebarAfter: "",
    contentBefore: "",
    contentAfter: '<div class="feedback">Was this helpful?</div>',
    footerBefore: "",
    footer: '<footer class="custom">...</footer>',
  },
});
```

## Custom CSS and JavaScript

```ts
defineTheme({
  extends: defaultTheme,
  css: `
    .content h1 {
      color: #3b82f6;
      letter-spacing: -0.04em;
    }
  `,
  js: `
    console.log('Page loaded');
  `,
});
```

## Default Theme Values

```ts
const defaultTheme = {
  name: "default",
  colors: {
    primary: "#3b82f6",
    primaryHover: "#2563eb",
    background: "#ffffff",
    backgroundAlt: "#f5f7fb",
    text: "#131a30",
    textMuted: "#4f607b",
    border: "#d2dbea",
    codeBackground: "#0b1328",
    codeText: "#eaf2ff",
  },
  darkColors: {
    primary: "#60a5fa",
    primaryHover: "#93c5fd",
    background: "#060816",
    backgroundAlt: "#0d1528",
    text: "#ebf2ff",
    textMuted: "#8ea0bf",
    border: "#223252",
    codeBackground: "#0a1020",
    codeText: "#e7f0ff",
  },
  fonts: {
    sans: '"IBM Plex Sans", "Avenir Next", "Segoe UI Variable", "Segoe UI", sans-serif',
    mono: '"IBM Plex Mono", "SFMono-Regular", Consolas, monospace',
  },
  layout: {
    sidebarWidth: "260px",
    headerHeight: "60px",
    maxContentWidth: "960px",
  },
  socialLinks: {},
};
```

## TypeScript Support

All types are exported:

```ts
import type {
  ThemeConfig,
  ThemeColors,
  ThemeLayout,
  ThemeFonts,
  ThemeHeader,
  ThemeFooter,
  SocialLinks,
  ThemeSlots,
  ResolvedThemeConfig,
  PageProps,
  BasePageProps,
  SiteConfig,
  NavGroup,
  NavItem,
  ThemeComponent,
  ThemeProps,
} from "@ox-content/vite-plugin";
```
