---
title: Locale Switcher
description: Opt-in header dropdown that lists configured locales and links to the sibling page.
---

# Locale Switcher

When `ssg.localeSwitcher` is enabled and `i18n.locales` is non-empty, the
default theme header shows a locale dropdown. The current locale is the
trigger label and is marked in the menu. A locale links to the same path in
that language when the sibling page exists; otherwise it falls back to the
locale root (`/{locale}/`, or `/` for a hidden default locale).

This is only the header control. It does not implement MessageFormat
dictionaries or a translation runtime.

When i18n is on, sidebar and header nav links also follow the current locale
if that sibling page exists. Missing siblings — including generated API pages
— keep the authored English href. Sidebar `text` stays a string; put locale
maps on header `nav` items only.

The feature is off unless you turn it on. Omitted or `false` emits no
switcher, even when `available_locales` is set. Existing `html` `lang` and
`dir` attributes stay as they are.

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      i18n: {
        enabled: true,
        defaultLocale: "en",
        locales: [
          { code: "en", name: "English" },
          { code: "ja", name: "日本語" },
          { code: "ar", name: "العربية", dir: "rtl" },
        ],
      },
      ssg: {
        localeSwitcher: true,
      },
    }),
  ],
};
```

`false` or omitted keeps the switcher off. `true` enables the defaults. An
object also enables the feature.

Each locale link honors `dir` for RTL languages. Locale names and codes are
escaped. `javascript:`, `data:`, and `vbscript:` locale roots are rejected.
Bare mode never emits the switcher.

This documentation site enables the switcher with `en` and `ja`. Japanese
guides live under [`/ja/`](/ja/).
