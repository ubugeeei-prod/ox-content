---
title: パンくずリスト
description: サイトルートからサイドバー祖先をたどる、オプトインのパンくず軌跡です。
---

# パンくずリスト

`ssg.breadcrumbs` または `theme.breadcrumbs` が有効なとき、各記事は
サイトルートからサイドバー祖先を経て現在ページまでの軌跡を得ます。
軌跡は記事の上に置かれます。現在ページはリンクではありません。

機能は、オンにするまでオフです。

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      ssg: {
        breadcrumbs: true,
      },
    }),
  ],
};
```

`false` または省略すると軌跡はオフのままです。`true` は既定を有効にします。オブジェクトも
機能を有効にします。

同じフラグはテーマにも置けます。

```ts
oxContent({
  ssg: {
    theme: {
      breadcrumbs: true,
    },
  },
});
```

見える軌跡は構造化データとは独立です。パンくずを有効にしても
JSON-LD は出しません。

## Frontmatter

1 ページで軌跡を隠します。

```md
---
title: Landing
breadcrumbs: false
---
```

| 値      | 結果                                      |
| ------- | ----------------------------------------- |
| 省略    | サイトの `breadcrumbs` オプションに従う   |
| `false` | このページで軌跡を隠す                    |
| `true`  | サイトオプションがオンなら軌跡を残す      |

`javascript:`、`data:`、`vbscript:`、または `//` を使う祖先 href は
リンクとして出しません。bare モードはパンくずクロムを決して出しません。
