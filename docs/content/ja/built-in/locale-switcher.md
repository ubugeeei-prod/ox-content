---
title: ロケールスイッチャー
description: 設定済みロケールをプルダウンし、兄弟ページへリンクするオプトインのヘッダー操作です。
---

# ロケールスイッチャー

`ssg.localeSwitcher` が有効で `i18n.locales` が空でないとき、
既定テーマのヘッダーはロケールのプルダウンを出します。現在のロケールが
トリガー表示になり、メニュー内でも印付けされます。
ロケールは兄弟ページがあるとき同じパスのその言語へリンクし、
なければロケールルート（`/{locale}/`、隠した既定ロケールなら `/`）へフォールバックします。

これはヘッダー操作だけです。MessageFormat
辞書や翻訳ランタイムは実装しません。

i18n が有効なとき、サイドバーとヘッダーナビのリンクも、兄弟ページがあれば
現在のロケールへ付け替えます。兄弟が無いページ（生成 API を含む）は
書いたままの href を保ちます。サイドバーとヘッダーナビの `text` は
`{ en: "Guide", ja: "ガイド" }` のロケールマップにできます。
フォールバックと折りたたみ状態の維持は[テーマ](../theming.md#ローカライズしたサイドバーラベル)を参照してください。

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
