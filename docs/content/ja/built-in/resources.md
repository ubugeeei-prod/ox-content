---
title: ページリソースと画像処理
description: オプトインのページバンドル資産と、ビルド時のリサイズ・クロップ・形式変換。
---

# ページリソースと画像処理

`resources` を有効にすると、各 Markdown ページのディレクトリが **バンドル**
になります。ページの隣（またはそのディレクトリ配下）にある画像は、相対 URL
で参照できます。クエリ文字列の変換を付けると、ビルド時にリサイズ、クロップ、
形式変換が行われます。

省略または `false` ではオフです。既存サイトの挙動は変わりません。
図・キャプション・遅延読み込み用の `images` とは別オプションで、併用できます。

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      resources: true,
    }),
  ],
};
```

`false` または省略はオフです。`true` または `{}` はデフォルトでオンです。
オブジェクトはオンにしたうえで、指定したフィールドだけ上書きします。

```ts
oxContent({
  resources: {
    formats: ["png", "jpeg", "webp"],
    widths: [400, 800],
    missing: "error",
    dedupe: false,
  },
});
```

| オプション  | 型                             | 既定                       |
| ----------- | ------------------------------ | -------------------------- |
| `resources` | `boolean` / `ResourcesOptions` | `false`                    |
| `formats`   | `string[]`                     | `["png", "jpeg", "webp"]`  |
| `widths`    | `number[]`                     | `[]`（正の幅ならどれでも） |
| `missing`   | `"error"` / `"warn"`           | `"error"`                  |
| `dedupe`    | `boolean`                      | `false`                    |

## ページバンドル

バンドルルートは、Markdown ファイルが置いてあるディレクトリです。

```
content/
  guide.md
  hero.png
  posts/
    hello.md
    cover.png
```

`guide.md` からは `![Hero](./hero.png)` と `![Hero](hero.png)` が
`content/hero.png` に解決されます。`posts/hello.md` の `./cover.png` は
`content/posts/cover.png` です。同じページディレクトリ配下の入れ子ファイルも
バンドル内です。

SSG は生成 HTML の隣にファイルをコピーするので、出力ツリーでも相対 URL が
そのまま使えます。

## 変換

クエリ文字列でビルド時の派生画像を要求します。

```md
![Wide](./hero.png?width=800)
![Fill](./hero.png?width=800&height=400&crop=center)
![Jpeg](./hero.png?width=400&format=jpeg)
```

クエリに `&` が含まれるときは `<destination>` を使います。そうしないと
Markdown が URL の残りを本文として扱います。

| パラメータ     | 意味                                                |
| -------------- | --------------------------------------------------- |
| `width` / `w`  | 目標幅（ピクセル）                                  |
| `height` / `h` | 目標高さ（ピクセル）                                |
| `crop=center`  | `width` × `height` を覆うまで拡大し、中央でクロップ |
| `crop=x,y,w,h` | ソースからその矩形を切り出す                        |
| `format`       | 出力コンテナ。`png`、`jpeg` / `jpg`、または `webp`  |

`width` か `height` の一方だけを付けた場合、もう一方はソースの縦横比に
従います。`crop=center` には `width` と `height` の両方が必要です。

ピクセル変換の再エンコード対象は **PNG** と **JPEG** です。`webp` はソースが
すでに WebP で、ピクセル変換がないときだけコピーします。`widths` が空でない
場合、`?width=` はその一覧の値でなければなりません。`?format=` は `formats`
に含まれている必要があります。

## キャッシュ

派生ファイルはプロジェクトルートの `.cache/ox-content-resources/` に
キャッシュされます。キャッシュキーは次の SHA-256 です。

- ソースの絶対パス
- ソースファイルの mtime
- 正規化した変換パラメータ（`width`、`height`、`crop`、`format`）

同じキーの再ビルドは再エンコードせず、キャッシュしたバイト列をコピーします。
ソースファイルか変換パラメータを変えるとキーも出力ファイル名も新しくなります。

## コンテンツの重複排除

`resources.dedupe` は **既定でオフ** です。`true` や `{}` では有効になりません。
同一の出力バイトを 1 回だけ書くには `dedupe: true` を指定します。

```ts
oxContent({
  resources: { dedupe: true },
});
```

ダイジェストは **最終出力バイト**、NUL、配信用拡張子（JPEG は `jpg`）の
SHA-256 です。正規ファイルは次のパスです。

`/assets/content/<sha256>.<ext>`

`base` が付きます（`/docs/` なら `/docs/assets/content/...`）。そのリソースを
指す HTML の `src`、`poster`、該当する `href` はこの URL に書き換わります。
消費した変換パラメータ以外のクエリとハッシュ断片は残します。リモート、
`data:`、`javascript:` は書き換えません。

あるダイジェストを最初に出したページだけが正規ファイルを書きます。後続
ページはそのパスとハッシュを再利用します。重複排除のために画像をデコード
しません。大きなファイルはストリームでハッシュします。

元のページ出力パスはエイリアスとして残します。ファイルシステムが許せば
ハードリンク、できなければコピーです。リンク失敗で共有 inode を上書き
しません。名前はビルド間で決定的です。

同じバイトでも拡張子が違えば別ファイルです。バイトが違えば別ファイルです。

## 欠けているソース

相対画像が存在しないとき、既定の `missing: "error"` はビルドを **失敗**
させます（`PageResourceError`）。`missing: "warn"` にするとページは出力し、
SSG 結果に問題を記録します。

```ts
oxContent({
  resources: { missing: "warn" },
});
```

## パストラバーサル

解決後のソースは、**ページバンドル** と `srcDir` の両方の内側に留まらなければ
なりません。ページディレクトリの外へ出る `../` は拒否され、ビルドは失敗します。
絶対ファイルシステムパス、`javascript:`、`data:`、`vbscript:`、
プロトコル相対の `//`、サイトルートの `/...` はページリソースとして処理しません。

フェンスコード、インデントコード、インラインコード内の画像構文は `<img>` に
ならないため、そのまま残ります。

書き換えた `src` は HTML エスケープします。出力名はリソースのベース名と
キャッシュキーの接尾辞から作り、著者入力をそのまま使いません。

## 関連

- [画像](./images.md)
- [SSG 出力プリミティブ](./ssg-output.md)
- [サイト生成](./site-generation.md)
- [組み込み機能の概要](../built-in-features.md)
