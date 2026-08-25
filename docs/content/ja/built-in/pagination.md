---
title: 前へ / 次へ
description: サイドバー順から生成する、オプトインの前後ページリンク。
---

# 前へ / 次へ

`ssg.pagination` を有効にすると、各記事の本文のあと・最終更新行の前に、前へ / 次へリンクが付きます。並びはサイドバーです。グループを深さ優先で平坦化し、サイト内 href のない項目は飛ばします。

機能は自分でオンにするまでオフです。

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      ssg: {
        pagination: true,
      },
    }),
  ],
};
```

`false` または省略はページャーをオフのままにします。`true` は既定でオンです。オブジェクトを渡しても機能はオンになります。

最初のページには前へのリンクはありません。最後のページには次へのリンクはありません。ページが 1 つだけのサイドバーでは何も出ません。エントリページはページャーを出しません。bare モードはページャーの chrome を一切出しません。

## Frontmatter

片側だけ差し替えるか、隠します。

```md
---
title: Guide
prev:
  text: Back to intro
  link: /intro/
next: false
---
```

| 値                                        | 結果                             |
| ----------------------------------------- | -------------------------------- |
| 省略                                      | サイドバーから隣ページを自動決定 |
| `false`                                   | その側を隠す                     |
| `{ text, link }` または `{ title, href }` | その側を差し替える               |

`javascript:` や `data:` を使った上書き href は捨てられます。
