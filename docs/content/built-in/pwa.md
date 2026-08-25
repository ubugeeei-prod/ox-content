---
title: PWA manifest and service worker
description: Opt-in web app manifest and conservative offline cache written next to generated HTML.
---

# PWA manifest and service worker

When `pwa` is enabled and `ssg.siteUrl` is set, the SSG build writes a web app
manifest next to the generated HTML and, by default, a conservative service
worker:

- `manifest.webmanifest` — name, start URL, theme colors, standalone display
- `sw.js` — caches hashed `assets/` files and HTML pages

Themed pages also get `<link rel="manifest">`. When offline caching is on, they
get a tiny script that registers `sw.js`.

**This adds client JavaScript.** The register script runs in the browser. The
service worker intercepts same-origin `GET` requests. Existing sites stay
unchanged until you turn the feature on.

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      pwa: true,
      ssg: {
        siteUrl: "https://example.com",
      },
    }),
  ],
};
```

`false` or omitted keeps the files and the client script off. `true` enables
the defaults: a manifest plus offline caching. An object enables the feature
and overrides only the fields you set:

```ts
oxContent({
  pwa: {
    name: "Docs",
    shortName: "Docs",
    themeColor: "#0f172a",
    backgroundColor: "#ffffff",
    startUrl: "/docs/",
  },
  ssg: {
    siteUrl: "https://example.com",
    siteName: "Docs",
  },
});
```

## Manifest only, no offline cache

Set `offline: false` to keep installability metadata without a service worker
or register script. `manifest.webmanifest` and `<link rel="manifest">` are
still written. `sw.js` is not.

```ts
oxContent({
  pwa: {
    offline: false,
    name: "Docs",
  },
  ssg: {
    siteUrl: "https://example.com",
  },
});
```

| Option            | Type                     | Default                           |
| ----------------- | ------------------------ | --------------------------------- |
| `pwa`             | `boolean` / `PwaOptions` | `false`                           |
| `offline`         | `boolean`                | `true`                            |
| `name`            | `string`                 | `ssg.siteName`                    |
| `shortName`       | `string`                 | `name`                            |
| `themeColor`      | `string`                 | `#000000`                         |
| `backgroundColor` | `string`                 | `#ffffff`                         |
| `startUrl`        | `string`                 | the Vite `base` (`/` or `/docs/`) |

`start_url` and `scope` are base-relative site paths, not absolute origins.
`javascript:`, protocol-relative `//`, and other non-path values fall back to
`base`. Theme and background colors accept `#rgb` / `#rrggbb` / `#rrggbbaa` or
a CSS color name. Hostile values fall back to the defaults.

## Offline caching behavior

The service worker is conservative:

- **HTML pages** use **network-first**. A cached copy is used only when the
  network fails.
- **Hashed assets** under `{base}assets/` (`ox-content-*-{hash}.css` / `.js`)
  use **cache-first**.
- Cross-origin requests and non-`GET` methods are ignored.

The worker does not precache the whole site at install time. Pages enter the
cache after a successful navigation.

## `ssg.siteUrl`

If `pwa` is enabled without `ssg.siteUrl`, no files are written and themed
pages do not get the manifest link or register script. The build continues and
emits a warning.

## Client JavaScript

Enabling `offline` (the default when `pwa` is on) injects a register script
into themed pages:

```html
<script>
  if ("serviceWorker" in navigator) navigator.serviceWorker.register("/sw.js");
</script>
```

Bare SSG output (`ssg.bare`) does not receive the link or the script. The
files are still written when `siteUrl` is set, so a custom shell can register
the worker itself.

Names, colors, and URLs are escaped so they cannot break out of the manifest
JSON or injected HTML attributes.

## Related

- [Site Generation](./site-generation.md)
- [Sitemap / robots / llms.txt](./site-maps.md)
- [Built-in Features overview](../built-in-features.md)
