---
title: ロケールスイッチャー
description: 設定済みロケールを列挙し、兄弟ページへリンクするオプトインのヘッダー操作です。
---

# ロケールスイッチャー

`ssg.localeSwitcher` が有効で `i18n.locales` が空でないとき、
既定テーマのヘッダーは各ロケールを列挙します。現在のロケールは印付けされます。
ロケールは兄弟ページがあるとき同じパスのその言語へリンクし、
なければロケールルート（`/{locale}/`、隠した既定ロケールなら `/`）へフォールバックします。

これはヘッダー操作だけです。MessageFormat
辞書や翻訳ランタイムは実装しません。

機能は、オンにするまでオフです。省略または `false` は、
`available_locales` が設定されていてもスイッチャーを出しません。既存の `html` の `lang` と
`dir` 属性はそのままです。

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

`false` または省略するとスイッチャーはオフのままです。`true` は既定を有効にします。
オブジェクトも機能を有効にします。

各ロケールリンクは RTL 言語向けに `dir` を尊重します。ロケール名とコードは
エスケープされます。`javascript:`、`data:`、`vbscript:` のロケールルートは拒否されます。
bare モードはスイッチャーを決して出しません。
