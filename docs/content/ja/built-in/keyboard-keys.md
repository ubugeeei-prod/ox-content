---
title: キーボードキー
description: オプトインの `{kbd:...}` ショートカットを意味的なキー要素として描画します。
---

# キーボードキー

プロダクトドキュメントやエディタガイドでは、Ctrl K や Command Shift P のような
ショートカットを出すことがよくあります。`{kbd:...}` 記法はオプトインで、既定は
オフです。

| オプション     | 型                                | 既定    |
| -------------- | --------------------------------- | ------- |
| `keyboardKeys` | `boolean` / `KeyboardKeysOptions` | `false` |

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      keyboardKeys: true,
    }),
  ],
};
```

`false` または省略はソースを変えません。`true` またはオブジェクトは変換をオンにします。
クライアント JavaScript はなく、実行時のプラットフォーム判定もしません。

## 書き方

`{kbd:Ctrl+K}` または `{kbd:Cmd Shift P}` と書きます。`+` と空白のどちらでも
キーを分けます。レンダラは安定した class 付きの入れ子 `<kbd>` を出します。

{kbd:Ctrl+K} {kbd:Cmd Shift P} {kbd:Esc}

```md
Press {kbd:Ctrl+K} or {kbd:Cmd Shift P}.
```

Press {kbd:Ctrl+K} or {kbd:Cmd Shift P}.

単一キー、句読点、組み合わせはすべて有効です。`cmd`、`ctrl`、`shift`、`esc`
などの組み込みエイリアスは、機能がオンのときだけ正規化します。未知のトークンは
書いたまま残します。

```md
`{kbd:Ctrl+K}`
```

`{kbd:Ctrl+K}`

リテラルにするにはバックスラッシュでエスケープします。`\{kbd:Ctrl+K}` は
`{kbd:Ctrl+K}` のままです。空、閉じていない、改行をまたぐ記法は見えるまま残します。
フェンス、インデントコード、インラインコード、HTML コメント、生の `code` /
`pre` / `script` / `style` は書き換えません。

## オプション

```ts
oxContent({
  keyboardKeys: {
    style: "symbols",
    aliases: {
      cmd: "Cmd",
    },
  },
});
```

| フィールド | 型                       | 既定      |
| ---------- | ------------------------ | --------- |
| `enabled`  | `boolean`                | `true`    |
| `style`    | `"words"` / `"symbols"`  | `"words"` |
| `aliases`  | `Record<string, string>` | `{}`      |

`style: "words"` は `cmd` を `Command` にします。`style: "symbols"` は `⌘`
にします。カスタム `aliases` は大文字小文字を区別せず、組み込み表を上書きします。
ラベルはビルド時に決まります。

## 関連

- [インラインバッジ](./badges.md)
- [構文拡張](./syntax-extensions.md)
- [組み込み機能の一覧](../built-in-features.md)
