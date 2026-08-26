---
title: Markdown パーシャル
description: 再利用利用スニペット向けの、オプトインのパラメータ付き Markdown パーシャル。
---

# Markdown パーシャル

パラメータ付きパーシャルはオプトインで、既定はオフです。有効にすると、HTML コメントのディレクティブが別の Markdown ファイルをインライン展開し、名前付きの `{{ values }}` をホスト文書のパース前に置換します。既存の `<!-- @include: -->` の挙動は変わりません。

| オプション | 型                            | 既定    |
| ---------- | ----------------------------- | ------- |
| `partials` | `boolean` / `PartialsOptions` | `false` |

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      partials: true,
    }),
  ],
};
```

`false` または省略はディレクティブをリテラルのままにします。`true` またはオブジェクトで変換を有効にします。展開はビルド時のみです。

## ディレクティブ

```md
<!-- @partial: ./_partials/install.md package="ox-content" manager="pnpm" -->
```

このサイトは `partials` をオンにしているので、次の文はライブのパーシャルです。

<!-- @partial: ./_partials/install.md package="ox-content" manager="pnpm" -->

パスは引用符で囲んでも構いません。`install.md` のような裸の名前は `_partials` の下で解決します（`root` で変更できます）。`{{ name }}` の置換は HTML エスケープされるので、`<script>` を生のマークアップとして注入できません。

## 欠けているパラメータ

欠けている `{{ name }}` はリテラルのまま残します。空文字には置き換えません。`missing: "error"` にすると、プレースホルダは残したまま変換診断を出します。

```ts
oxContent({
  partials: {
    root: "_partials",
    missing: "literal",
  },
});
```

| フィールド | 型                      | 既定          |
| ---------- | ----------------------- | ------------- |
| `enabled`  | `boolean`               | `true`        |
| `rootDir`  | `string`                | プロジェクト  |
| `root`     | `string`                | `"_partials"` |
| `missing`  | `"literal"` / `"error"` | `"literal"`   |

## パスの安全

- 相対の `./` と `../` は現在のファイルから解決します。
- `@/` と先頭の `/` は `rootDir` から解決します。
- canonicalize のあと、`rootDir` の外に出るパスは拒否します。ディレクティブはソースに残り、変換エラーを報告します。
- 循環と 16 段より深い入れ子は変換エラーです。診断にはホストファイルと行番号が入ります。

## 展開されないもの

フェンスコード、インデントコード、インラインコードの中ではディレクティブは展開しません。`@partial:` ではない HTML コメントはそのままです。`<!-- @include: PATH -->` も同様です。

## 関連

- [ファイル取り込み](./includes.md)
- [組み込み機能の一覧](../built-in-features.md)
