---
title: Locale Switcher
description: Opt-in header control that lists configured locales and links to the sibling page.
---

# Locale Switcher

When `ssg.localeSwitcher` is enabled and `i18n.locales` is non-empty, the
default theme header lists each locale. The current locale is marked. A
locale links to the same path in that language when the sibling page exists;
otherwise it falls back to the locale root (`/{locale}/`, or `/` for a hidden
default locale).

This is only the header control. It does not implement MessageFormat
dictionaries or a translation runtime.

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
