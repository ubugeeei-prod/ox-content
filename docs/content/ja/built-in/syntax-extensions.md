---
title: 構文拡張
description: オプトインの執筆構文 — 絵文字ショートコード、Wiki リンク、属性構文、CJK 強調。
---

# 構文拡張

非標準の Markdown 構文はオプトインです。サイトが拡張を明示的にオンにするまで、普通の文書はどこでも同じように描画されます。

| オプション        | 型                                  | 既定    |
| ----------------- | ----------------------------------- | ------- |
| `emojiShortcodes` | `boolean` / `EmojiShortcodeOptions` | `false` |
| `wikiLinks`       | `boolean` / `WikiLinkOptions`       | `false` |
| `attrs`           | `boolean` / `AttrsOptions`          | `false` |
| `cjkEmphasis`     | `boolean`                           | `false` |

## 絵文字ショートコード

GitHub 風の `:shortcode:` エイリアスを Unicode 絵文字へ展開します。

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      emojiShortcodes: true,
    }),
  ],
};
```

組み込み表はよく使うエイリアスを数百件カバーします。展開はフェンスとインラインコードの外で走り、未知のショートコードはそのままです。

```md
Ship it :rocket: :tada:

Status: :white_check_mark: passed, :warning: flaky, :x: failed

Unknown aliases like :no-such-emoji: stay untouched, and so does
inline code: `:rocket:`.
```

描画:

Ship it :rocket: :tada:

Status: :white_check_mark: passed, :warning: flaky, :x: failed

Unknown aliases like :no-such-emoji: stay untouched, and so does
inline code: `:rocket:`.

### 独自ショートコード

独自の値は組み込み表にマージされ、衝突時は上書きします。キーはコロンなしで書きます。

```ts
oxContent({
  emojiShortcodes: {
    custom: {
      shipit: "🚢",
      oxc: "🦀",
    },
  },
});
```

## Wiki リンク

Obsidian 風の `[[target]]` リンクを普通のサイトリンクへ解決します。

```ts
oxContent({
  wikiLinks: {
    // トップレベルの `base` オプションが既定。
    baseUrl: "/docs/",
  },
});
```

展開は Markdown パースの前に走り、フェンス付きコードブロックとインラインコードスパンは保護されます。次のソースがあるとき、

```md
See [[getting-started|Getting started]] and [[api/transform#options]].
```

変換は次を出します。

```html
<p>
  See <a href="/docs/getting-started">Getting started</a> and
  <a href="/docs/api/transform#options">api/transform#options</a>.
</p>
```

`[[target]]` はターゲットをラベルにし、`[[target|label]]` で上書きします。`#fragment` 部分は slug 化します。サイト相対のターゲットには `baseUrl` を付けます。

Wiki リンクは生 HTML のパースよりも前に走るので、埋め込み HTML のリテラル `<code>` タグの中の `[[...]]` も展開されます。リテラルの例は Markdown のコードスパンかフェンスに置いてください。

## 属性構文

`markdown-it-attrs` 構文で ID、クラス、属性を足します。

```ts
oxContent({
  attrs: true,
});
```

対応するトークンは `#id`、`.class`、`key=value` です。末尾の `{...}` ブロックは、その行から描画された要素に付きます。

```md
A lead paragraph. {.lead}

## Install {.section data-section=install}
```

結果は次です。

```html
<p class="lead">A lead paragraph.</p>

<h2 id="install" class="section" data-section="install">Install</h2>
```

変換は文書全体に対する描画後の HTML パスです。Markdown に埋め込んだ生 HTML も影響を受けるので、リテラルの `{...}` 例はコードスパンかフェンスに置いてください。

## CJK 強調

CJK 文字に隣接する強調に設定は不要です。CommonMark の区切り規則がすでに許しており、ASCII スペースも要りません。

```md
これは**重要**です。次の文でも*強調*できます。
```

描画:

これは**重要**です。次の文でも*強調*できます。

素の CommonMark が拒否するのは、区切りランが外側で句読点に直接接している場合です。隣接規則は Unicode 句読点を一括して読むので、東アジアの句読点は ASCII 句読点と同じようにランを止め、`A**強調。**B` はリテラルのままです。ラテン語の本文では間にスペースが入ることが多いので、あまり当たりません。CJK は句読点を直前の語に付けるので、日常的に起きます。

`cjkEmphasis` はその判定に限って、東アジアの句読点を普通の文字として分類します。

```ts
oxContent({
  cjkEmphasis: true,
});
```

```md
A**強調。**B
```

オプションがオンなら `A<strong>強調。</strong>B` になり、オフならリテラルのままです。半角 ASCII 句読点は意図して触らないので、ラテン文書のパースはどちらでも同じです。

これは仕様からの意図した逸脱なので、オプトインです。境界と再分類する文字範囲の正確な話は [CJK Emphasis](/examples/cjk-emphasis.md) を見てください。

## 関連

- [Markdown の土台](./markdown.md) — これらの拡張が乗る既定構文。
- [コードブロック](./code-blocks.md) — フェンス向けの注釈と取り込み構文。
