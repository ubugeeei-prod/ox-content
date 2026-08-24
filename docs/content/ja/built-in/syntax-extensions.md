---
title: 構文拡張
description: 絵文字ショートコード、Wiki リンク、属性構文、CJK 強調。いずれもオプトイン。
---

# 構文拡張

非標準の Markdown 構文はオプトインです。有効にするまで、ふつうの文書はどこでも同じように描画されます。

| オプション        | 型                                  | 既定    |
| ----------------- | ----------------------------------- | ------- |
| `emojiShortcodes` | `boolean` / `EmojiShortcodeOptions` | `false` |
| `wikiLinks`       | `boolean` / `WikiLinkOptions`       | `false` |
| `attrs`           | `boolean` / `AttrsOptions`          | `false` |
| `cjkEmphasis`     | `boolean`                           | `false` |

## 絵文字ショートコード

GitHub 風の `:shortcode:` を Unicode 絵文字に展開します。フェンスとインラインコードの外だけが対象で、未知の別名はそのまま残ります。

```ts
oxContent({
  emojiShortcodes: true,
});
```

```md
Ship it :rocket: :tada:
```

## Wiki リンク

`[[page]]` をサイト内リンクにします。省略または `false` ではリテラルのままです。

```ts
oxContent({
  wikiLinks: true,
});
```

## 属性構文

見出しや画像などに `{#id .class}` を付けます。

```ts
oxContent({
  attrs: true,
});
```

## CJK 強調

日本語・中国語・韓国語の強調記号を正しく扱うためのオプトインです。

```ts
oxContent({
  cjkEmphasis: true,
});
```

## 関連

- [英語版ガイド](/built-in/syntax-extensions.md)
- [カスタムコンテナ](./containers.md)
