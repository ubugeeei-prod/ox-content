---
title: Git コントリビューター
description: 各記事の下に一意の git 作者を描画するオプトイン機能。
---

# Git コントリビューター

`ssg.contributors` を有効にすると、各記事はそのソースファイルの一意な git
作者を一覧します。名前はエスケープされます。明示的にオンにするまでオフです。
既存サイトの見た目は変わりません。

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      ssg: {
        contributors: true,
      },
    }),
  ],
};
```

`false` または省略では一覧は出ません。`true` は名前のみでオンです。オブジェクト
にすると機能がオンになり、`ignore` と `avatars` を設定できます。

```ts
oxContent({
  ssg: {
    contributors: {
      ignore: ["dependabot[bot]", "ci@example.com"],
      avatars: true,
    },
  },
});
```

| オプション         | 型                                  | 既定    |
| ------------------ | ----------------------------------- | ------- |
| `ssg.contributors` | `boolean` / `{ ignore?, avatars? }` | `false` |
| `ignore`           | `string[]`                          | `[]`    |
| `avatars`          | `boolean`                           | `false` |

作者はソースファイルに対する `git log --format=%an%x09%ae` から取ります。重複は
メールがあればメール、なければ名前でまとめます。比較は大文字小文字を無視します。
そのキーで最初に現れた名前が描画されます。コミット数はネイティブヘルパーから
取れますが、既定テーマには出しません。

無視リストは作者名の全文、またはメールの全文に一致します。大文字小文字は無視
します。部分一致はしません。無視された作者は HTML を作る前に落とします。

## アバター

既定は名前のみです。`avatars: true` にすると、git 作者メールがある場合に
Gravatar 画像を読みます。画像 URL は
`https://www.gravatar.com/avatar/{md5(email)}` です。生のメールは HTML、
`mailto:` リンク、`href` のいずれにも書きません。

アバターをオンにすると git 作者メールをハッシュし、`gravatar.com` に要求します。
作者メールのハッシュを公開してはいけないサイトは `avatars` をオフのままにしてください。

メールから GitHub のプロフィール URL は作りません。
`https://github.com/{login}` リンクは、後続のオプションが明示的な login を
渡さない限り出しません。この版はその対応付けを受け付けません。

安全でないアバター URL（`javascript:`、`data:`、プロトコル相対の `//`、
`http:`）は省略します。マークアップに届くのは `https:` のアバター URL だけです。

## `.git` が無いとき

公開された npm tarball、履歴の無い CI チェックアウト、git 作業ツリーの外の
パスでは空の一覧になります。ビルドは失敗しません。ページはコントリビューター
用のマークアップを出さず、実行時の警告も必須ではありません。このページがその
文書化された挙動です。

ネイティブヘルパー `getGitContributors(filePath, root?)` は、`root` が省略された
とき、git が無いとき、`git log` が失敗したとき、ファイルにコミットが無いとき
`[]` を返します。結果はファイルパスと `HEAD` コミットをキーに、プロセスの間
キャッシュされます。

## マークアップ

一覧は記事の下、`lastUpdated` もオンならその後に描画されます。コンテナは
`.contributors` です。各作者は `.contributor` で、任意の `.contributor-avatar`
画像と `.contributor-name` の span を持ちます。敵対的な名前はエスケープされます
（`<`、`"`、その他の HTML 特殊文字）。

bare モードでも同じ `contributors` 配列がページデータに載ります。既定の bare
テンプレートは一覧を出しません。独自の `ssg.render` テーマは
`page.contributors` を読めます。

## 関連

- [サイト生成](./site-generation.md)
- [最終更新](./site-generation.md)
- [組み込み機能の概要](../built-in-features.md)
