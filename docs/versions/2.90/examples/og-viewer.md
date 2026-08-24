# OG Viewer

Built-in dev tool for previewing Open Graph metadata of all your pages.

## Overview

OG Viewer is a development tool that runs at `/__og-viewer` during `vite dev`. It scans all Markdown files in your content directory and displays their OG metadata with social card previews and validation warnings.

## Usage

OG Viewer is **enabled by default**. Start your dev server and navigate to:

```
http://localhost:5173/__og-viewer
```

To disable it:

```ts
oxContent({
  ogViewer: false,
});
```

## Features

### Metadata Display

Each page card shows:

- File path and URL path
- Title, description, author, tags
- OG image URL (computed from your config)

### Validation

OG Viewer checks each page for common issues:

| Check                | Level   | Condition                                 |
| -------------------- | ------- | ----------------------------------------- |
| Missing title        | Error   | No title in frontmatter or `# heading`    |
| Missing description  | Warning | No `description` in frontmatter           |
| Title too long       | Warning | More than 70 characters                   |
| Description too long | Warning | More than 200 characters                  |
| No siteUrl           | Warning | ogImage enabled but `ssg.siteUrl` not set |

### Social Card Previews

Two preview modes are rendered for each page:

- **Twitter** - `summary_large_image` card format
- **Facebook** - Open Graph card format

Both show the computed OG image, title, and description as they would appear when shared.

### Filtering & Search

- **All** - Show all pages
- **Warnings** - Show only pages with warnings
- **Errors** - Show only pages with errors
- **Search** - Filter by path, title, or description

### Refresh

Click the **Refresh** button to re-scan files and update metadata without reloading the page.

## API Endpoint

OG Viewer also exposes a JSON API:

```
GET /__og-viewer/api/pages
```

Returns an array of page objects with all metadata and validation warnings. Useful for CI integration or custom tooling.
