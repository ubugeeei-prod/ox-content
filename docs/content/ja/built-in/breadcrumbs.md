---
title: パンくず
description: サイトルートからサイドバー祖先までの、オプトインのパンくず。
---

# パンくず

`ssg.breadcrumbs` または `theme.breadcrumbs` を有効にすると、各記事の上に、サイトルートからサイドバー祖先を経て現在ページまでの道筋が出ます。現在ページはリンクになりません。

機能は自分でオンにするまでオフです。

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

`false` または省略はパンくずをオフのままにします。`true` は既定でオンです。オブジェクトを渡しても機能はオンになります。

同じフラグはテーマ側にも置けます。

```ts
oxContent({
  ssg: {
    theme: {
      breadcrumbs: true,
    },
  },
});
```

画面上のパンくずは構造化データとは独立です。パンくずをオンにしても JSON-LD は出ません。構造化データは [JSON-LD](./json-ld.md) でオプトインします。エントリページはパンくずを出しません。

## Frontmatter

1 ページだけパンくずを隠します。

```md
---
title: Landing
breadcrumbs: false
---
```

| 値      | 結果                                     |
| ------- | ---------------------------------------- |
| 省略    | サイトの `breadcrumbs` オプションに従う  |
| `false` | このページではパンくずを隠す             |
| `true`  | サイトオプションがオンならパンくずを出す |

祖先 href が `javascript:`、`data:`、`vbscript:`、または `//` のときはリンクとして出しません。bare モードはパンくずの chrome を一切出しません。
