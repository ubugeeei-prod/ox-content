---
title: 前へ / 次へリンク
description: サイドバー順から生成する、オプトインの前後ページリンクです。
---

# 前へ / 次へリンク

`ssg.pagination` が有効なとき、各記事は本文のあと、最終更新行の前に前後リンクを得ます。
順序はサイドバーから来ます。グループは深さ優先で平坦化され、サイト内 href のない項目は
スキップされます。

機能は、オンにするまでオフです。

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

`false` または省略するとページャーはオフのままです。`true` は既定を有効にします。オブジェクトも
機能を有効にします。

最初のページには前へリンクがありません。最後のページには次へリンクがありません。
1 ページだけのサイドバーは何も出しません。入口ページはページャーをスキップします。bare モードは
ページャークロムを決して出しません。

## Frontmatter

片方を上書きするか、隠します。

```md
---
title: Guide
prev:
  text: Back to intro
  link: /intro/
next: false
---
```

| 値                                    | 結果                           |
| ------------------------------------- | ------------------------------ |
| 省略                                  | サイドバーからの自動隣接       |
| `false`                               | その側を隠す                   |
| `{ text, link }` または `{ title, href }` | その側を置き換える         |

`javascript:` または `data:` を使う上書き href は捨てられます。
