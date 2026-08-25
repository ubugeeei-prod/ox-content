---
title: JSDoc から API ドキュメント
description: JSDoc と TypeScript の型から API ドキュメントを生成します。
---

# JSDoc から API ドキュメント

Ox Content は、JSDoc コメントと TypeScript の型から直接 API ドキュメントを生成できます。
ソースファイルは [OXC](https://oxc.rs) パーサーで解析するので、抽出は速く、
別の型チェックパスなしで本物の TypeScript（ジェネリクス、オーバーロード、インターフェイス、列挙型）を理解します。

一度の実行で次を作ります。

- 各モジュールの **Markdown ページ**。シンタックスハイライトされたシグネチャ、
  パラメーター表、戻り値の型、例、（任意で）ソースリンク付きです。
- **`docs.json`** — 同じデータの機械可読ペイロード。ランタイムツール向けです。
- **`nav.ts`** — サイドバーに渡せる型付きナビゲーションツリーです。

## 有効化

ドキュメント生成は Vite プラグインの一部で、**既定でオン**（オプトアウト）です。
ソースを指して、出力ディレクトリを選んでください。

```ts
// vite.config.ts
import { defineConfig } from "vite";
import { oxContent } from "@ox-content/vite-plugin";

export default defineConfig({
  plugins: [
    oxContent({
      docs: {
        src: ["./src"],
        out: "docs/api",
        githubUrl: "https://github.com/your/repo",
      },
    }),
  ],
});
```

完全に切るには `docs: { enabled: false }` を設定します。

## オプション

| オプション                  | 既定                                             | 説明                                                                                            |
| --------------------------- | ------------------------------------------------ | ----------------------------------------------------------------------------------------------- |
| `enabled`                   | `true`                                           | ドキュメント生成の有効 / 無効。                                                                 |
| `src`                       | `['./src']`                                      | 走査するソースディレクトリ。                                                                    |
| `out`                       | `'docs/api'`                                     | 生成ドキュメントの出力ディレクトリ。                                                            |
| `include`                   | `['**/*.ts', '**/*.tsx', …]`                     | 含めるファイルの glob。                                                                         |
| `exclude`                   | `['**/*.test.*', '**/*.spec.*', 'node_modules']` | 除外する glob。                                                                                 |
| `entryPoints`               | —                                                | 再エクスポートされたドキュメントをまとめる公開 API 入口（後述）。                               |
| `format`                    | `'markdown'`                                     | `'markdown'`、`'json'`、または `'html'`。                                                       |
| `private`                   | `false`                                          | `@private` メンバーを含める。                                                                   |
| `internal`                  | `false`                                          | `@internal` メンバーを含める。                                                                  |
| `toc`                       | `true`                                           | ファイルごとに目次を出す。                                                                      |
| `groupBy`                   | `'file'`                                         | 出力を `'file'` または `'category'` でまとめる。                                                |
| `githubUrl`                 | —                                                | リポジトリ URL。設定するとシグネチャがソース行へリンクします。                                  |
| `linkStyle`                 | `'markdown'`                                     | 内部リンクのスタイル: `'markdown'`（`.md` リンク）または `'clean'`（拡張子なし）。              |
| `basePath`                  | `'/api'`                                         | 生成リンクとナビメタデータのルート接頭辞。                                                      |
| `pathStrategy`              | `'flat'`                                         | 出力レイアウト: `'flat'` または `'typedoc'`（後述）。                                           |
| `renderStyle`               | `'html'`                                         | 出力レンダラー: テーマ付き HTML-in-Markdown またはプレーン Markdown。                           |
| `indexFormat`               | `'none'`                                         | 索引項目の表示形式。                                                                            |
| `parametersFormat`          | `'none'`                                         | 値パラメーターと型パラメーターの表示形式。                                                      |
| `interfacePropertiesFormat` | `'none'`                                         | インターフェイスプロパティグループの表示形式。                                                  |
| `classPropertiesFormat`     | `'none'`                                         | クラスプロパティグループの表示形式。                                                            |
| `typeAliasPropertiesFormat` | `'none'`                                         | 型エイリアスプロパティグループの表示形式。                                                      |
| `enumMembersFormat`         | `'none'`                                         | 列挙型メンバーグループの表示形式。                                                              |
| `propertyMembersFormat`     | `'none'`                                         | プロパティが所有するネストしたオブジェクトリテラルメンバーを `'list'` または `'table'` で表示。 |
| `typeDeclarationFormat`     | `'none'`                                         | 戻り値の型宣言メンバーを `'list'` または `'table'` で表示。                                     |
| `typeParameters`            | `false`                                          | 宣言の型パラメーターと `@typeParam` / `@template` タグを抽出する。                              |
| `renderStats`               | `true`                                           | 生成した索引ページに統計サマリーを出す。                                                        |
| `renderGeneratedBy`         | `true`                                           | 生成ルート索引ページに generated-by 帰属を出す。                                                |
| `groupOrder`                | —                                                | モジュール索引セクションとナビグループ向けの TypeDoc 風グループ順。                             |
| `sort`                      | —                                                | エントリとメンバー向けの TypeDoc 風ソート戦略。                                                 |
| `sortEntryPoints`           | `true`                                           | 入口をアルファベット順にソート。`false` でソース順を保つ。                                      |
| `kindSortOrder`             | —                                                | モジュールセクションとナビグループ向けの TypeDoc 風宣言 kind 順位。                             |
| `generateNav`               | `true`                                           | `nav.ts` ナビゲーションファイルを出す。                                                         |

## 何が抽出されるか

文書化された各宣言は、`kind`（`function`、`class`、`interface`、`type`、`enum`、
`variable`、または `module`）、説明、シグネチャ、メンバーを持つエントリになります。
次の JSDoc タグを認識します。

| タグ                   | 効果                                                   |
| ---------------------- | ------------------------------------------------------ |
| `@param name desc`     | パラメーター表に行を追加します（型は TS から来ます）。 |
| `@returns desc`        | 戻り値を文書化します。                                 |
| `@example`             | フェンス付きコードブロックとして描画します。           |
| `@default value`       | パラメーター / プロパティの横に表示します。            |
| `@deprecated [reason]` | エントリを非推奨として印付けします。                   |
| `@private`             | `private: true` でない限り隠します。                   |
| `@internal`            | `internal: true` でない限り隠します。                  |

型は TypeScript の注釈そのものから読むので、`@param {Type}` の JSDoc 型構文は不要です。

## 入口と再エクスポート

既定では、ドキュメントはソースファイルごとにまとまります。パッケージが公開 API を
バレル（`index.ts`）経由で再エクスポートしているなら、それを `entryPoint` として渡し、
再エクスポートグラフをたどって実際にエクスポートされているものでまとめます。

```ts
docs: {
  entryPoints: [
    "./src/index.ts",
    { path: "./src/cli.ts", name: "CLI" },
  ],
}
```

別の名前で再エクスポートされたシンボルは、エクスポートされた名前の下で文書化されます。

## 出力レイアウト

`pathStrategy` は、生成 Markdown のディレクトリ形を制御します。

- **`flat`**（既定） — `out` 配下にモジュールごとに 1 ファイル（例:
  `docs/api/index.md`、`docs/api/parser.md`）。
- **`typedoc`** — モジュールを kind ごとのサブディレクトリ（`functions/`、
  `classes/`、`interfaces/`、…）に分ける入れ子の TypeDoc 風ツリー。大きな API に向きます。

Markdown と並んで、`writeDocs` は `docs.json`（構造化ペイロード）を出し、
`generateNav` がオンなら `nav.ts` も出します。マニフェストが生成ファイルを追跡するので、
古いページは次の実行で掃除されます。

## 表示形式

`renderStyle` は生成ページ本体を制御します。

- **`html`**（既定） — Markdown ファイル内に ox-content テーマ付き HTML を出します。
- **`markdown`** — raw HTML の足場なしのプレーン Markdown を出します。

表示形式オプションは `'none'`、`'list'`、`'table'` を受け付けます。索引、パラメーター、
メンバーグループ、ネストしたプロパティオブジェクトリテラル、戻り値の型宣言メンバーが
既定レイアウト、リスト、表のどれを使うかを制御します。

## ナビをサイドバーへつなぐ

`nav.ts` は型付きの `NavItem[]` をエクスポートするので、テーマのサイドバーへ差し込めます。

```ts
import { apiNav } from "./docs/api/nav";

// in your ssg theme config
sidebar: [
  ...apiNav,
  // …your hand-written sections
];
```

## 関連

- [`examples/gen-source-docs`](/examples/gen-source-docs.md) — 実行可能なセットアップ。
- このプロジェクトの生成 [API リファレンス](/api/index.md) 自体も、この機能で作られています。ページはドキュメントビルド時に書かれ、リポジトリには入れません。
