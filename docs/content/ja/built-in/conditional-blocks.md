---
title: 条件付きブロック
description: 環境ごとの Markdown を静的に切り替える、オプトインの ::: if / ::: else。
---

# 条件付きブロック

条件付きブロックはオプトインです。オフのとき、`:::` 形式はリテラルのままです。オンにすると、Ox Content は Markdown をパースする前にページ frontmatter と `conditionalBlocks.values` から条件を評価します。

| オプション          | 型                                    | 既定    |
| ------------------- | ------------------------------------- | ------- |
| `conditionalBlocks` | `boolean` / `ConditionalBlockOptions` | `false` |

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      conditionalBlocks: {
        values: {
          runtime: "node",
          channels: ["stable", "alpha"],
        },
      },
    }),
  ],
};
```

## 書き方

`::: if`、`::: else if`、`::: elif`、`::: else` を使います。選ばれなかった分岐は、HTML、TOC、モジュールコード、検索抽出を作る前に取り除かれます。

```md
::: if runtime == "node"
Node-only setup.
::: else if runtime in ["deno", "bun"]
Alternative runtime setup.
::: else
Browser setup.
:::
```

このドキュメントビルドは `conditionalBlocks` を `runtime: "node"` でオンにしているので、下では選ばれた分岐だけが描画されます。

::: if runtime == "node"
Node-only setup.
::: else
Browser setup.
:::

## 式

式は意図的に小さく、静的です。使える構文は次の通りです。

| 構文                 | 例                                          |
| -------------------- | ------------------------------------------- |
| 真偽値、数値、文字列 | `release == "stable"`                       |
| `null`               | `frontmatter.variant != null`               |
| 配列                 | `runtime in ["node", "deno"]`               |
| 等価                 | `audience == "library"` / `tier != "draft"` |
| 真偽演算子           | `runtime == "node" and channel == "stable"` |
| 丸かっこ             | `(runtime == "node") or experimental`       |
| ページ frontmatter   | `frontmatter.runtime == "browser"`          |
| 共有のビルド時設定   | `config.runtime == "node"`                  |

裸の識別子は、まずページ frontmatter を読み、無ければ `conditionalBlocks.values` にフォールバックします。同じキーが両方にあるときは `frontmatter.name` または `config.name` を使って明示します。条件としてそのまま使う値は真偽値である必要があります。JavaScript のような truthy / falsy はありません。

```md
---
runtime: browser
---

::: if runtime == "browser"
The page frontmatter branch wins.
::: else if config.runtime == "node"
The shared config branch is skipped.
:::
```

ユーザー JavaScript は実行しません。値はパース済みの JSON 風データだけから来ます。フェンス、インライン、インデントコードの中のマーカーはリテラルのままです。閉じていない条件付きブロックもリテラルのまま残り、transform の警告を出します。

## 検索

検索インデックスはページ変換と同じ前処理オプションを使うので、隠れた分岐の見出しや本文は静的インデックスに入りません。公開している検索ヘルパーを直接呼ぶ場合は、同じ `conditionalBlocks` を渡してください。

## 関連

- [カスタムコンテナ](./containers.md)
- [検索](./search.md)
- [組み込み機能の一覧](../built-in-features.md)
