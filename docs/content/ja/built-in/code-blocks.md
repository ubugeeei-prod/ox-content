---
title: コードブロック
description: フェンス付きコードブロック向けのシンタックスハイライト、コード注釈、コードインポートです。
---

# コードブロック

3 つのオプトイン機能がフェンス付きコードブロックを拡張します。tree-sitter シンタックス
ハイライト、ハイライトと diff マーカー向けの注釈構文、
本物のソースファイルからのスニペットインポートです。このサイトは 3 つとも有効にしているので、
下の例はすべてライブ描画です。

サンプルのオンデマンド **Run** / **Typecheck** は別パッケージ
[`@ox-content/code-play`](../packages/code-play.md) です。
`@ox-content/vite-plugin` の一部ではありません。[Code Play の例](/examples/code-play.md) を見てください。

| オプション        | 型                                   | 既定    |
| ----------------- | ------------------------------------ | ------- |
| `highlight`       | `boolean`                            | `false` |
| `codeAnnotations` | `boolean` / `CodeAnnotationsOptions` | `false` |
| `codeImports`     | `boolean` / `CodeImportOptions`      | `false` |
| `codeGroups`      | `boolean` / `CodeGroupOptions`       | `false` |

## シンタックスハイライト

ハイライトはオプトインです。有効にすると、フェンス付きブロックと言語タグ付きインライン
コードはネイティブ tree-sitter エンジンを通ります。ネイティブ文法がない言語は
普通の `<pre><code>` のままです。ハイライトされません。
専用の文法を同梱していない近い形式は、既存文法でソース文字列を安全に保てる場合だけ
best-effort alias として扱います。`jsonc` / `json5` / `webmanifest` は JSON、
`vue` / `svelte` / `astro` / `angular` は HTML、`flow` / `javascriptreact` は
JavaScript、`typescriptreact` は TSX を使います。`dotenv`、`.env`、`gitignore`、
`npmrc`、`ini`、`conf` などの dotfile / config タグは、エスケープ済みの plain text
として描画します。

### 対応言語

下のフェンスタグはネイティブ文法でトークン化します。同じセルの alias は同じ文法です。
Vue / Svelte / Astro / Angular は HTML 文法のままです。crates.io に、この
tree-sitter 系列と合うメンテされた専用文法がまだありません。

| 言語       | フェンスタグ                                                              |
| ---------- | ------------------------------------------------------------------------- |
| TypeScript | `typescript`, `ts`, `cts`, `mts`                                          |
| TSX        | `tsx`, `typescriptreact`                                                  |
| JavaScript | `javascript`, `js`, `cjs`, `mjs`, `jsx`, `javascriptreact`, `flow`, `mdx` |
| Rust       | `rust`, `rs`                                                              |
| JSON       | `json`, `jsonc`, `json5`, `webmanifest`                                   |
| CSS        | `css`                                                                     |
| Less       | `less`                                                                    |
| HTML       | `html`, `vue`, `svelte`, `astro`, `angular`                               |
| XML        | `xml`, `svg`, `xsl`, `xslt`, `rss`, `atom`, `plist`, `xsd`                |
| Python     | `python`, `py`                                                            |
| Go         | `go`, `golang`                                                            |
| Java       | `java`                                                                    |
| C          | `c`, `h`                                                                  |
| C++        | `cpp`, `c++`, `cc`, `hpp`, `cxx`                                          |
| YAML       | `yaml`, `yml`                                                             |
| Markdown   | `markdown`, `md`                                                          |
| Bash       | `bash`, `sh`, `shell`, `zsh`, `shellscript`                               |
| TOML       | `toml`                                                                    |
| WGSL       | `wgsl`                                                                    |
| SQL        | `sql`                                                                     |
| GraphQL    | `graphql`, `gql`                                                          |
| Dockerfile | `dockerfile`, `docker`, `containerfile`                                   |
| Ruby       | `ruby`, `rb`                                                              |
| PHP        | `php`                                                                     |
| Nix        | `nix`                                                                     |
| C#         | `csharp`, `cs`                                                            |
| Swift      | `swift`                                                                   |
| Kotlin     | `kotlin`, `kt`                                                            |
| GLSL       | `glsl`                                                                    |
| Lua        | `lua`                                                                     |
| HCL        | `hcl`, `terraform`, `tf`, `tfvars`                                        |
| Make       | `make`, `makefile`, `mk`                                                  |
| Diff       | `diff`, `patch`, `udiff`                                                  |

未知のタグは普通の `<pre><code>` のままです。例: `haskell`、`scala`、
`elixir`、`perl`、`elm`、`assembly`、`asm`、`llvm`、`clojure`、
`brainfuck`。無関係な文法へ alias しません。`text`、`dotenv`、`ini` などの
plain タグはエスケープのみで、トークン化しません。

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      highlight: true,
    }),
  ],
};
```

トークン色は `<pre class="ox-highlight css-variables">` 上の
`--octc-syntax-*` CSS カスタムプロパティです。ハイライトは
tree-sitter のみで、`@ox-content/theme-color-*` パッケージが
これらの変数を解決します。カラースキームがなければプロパティは GitHub
Dark にフォールバックします。ハイライトのあと、コードブロックメタデータ（注釈、行番号）は
ネイティブ出力へ戻しマージされます。

## コード注釈

注釈はオプトインなので、サイトが注釈構文を選ばない限り、普通のフェンスはリテラルのままです。

```ts
oxContent({
  highlight: true,
  codeAnnotations: {
    // "attribute" (default) | "vitepress" | "both"
    notation: "both",
    // Attribute name used by the attribute syntax. Default: "annotate".
    metaKey: "annotate",
    // Render line numbers for every block. Default: false.
    defaultLineNumbers: false,
  },
});
```

対応する注釈の種類は `highlight`、`warning`、`error` です。

### 属性記法

既定の記法は、`;` で区切った `kind:lines` グループを持つ単一のフェンス属性です。
行セレクターは単一行（`5`）と範囲（`3-4`）を受け付けます。

````md
```ts annotate="highlight:1,6;warning:2;error:3"
export function loadUser(input: string) {
  if (!input) console.warn("missing payload");
  throw new Error("missing id");
}

const user = loadUser(payload);
console.log(user);
```
````

描画結果:

```ts annotate="highlight:1,6;warning:2;error:3"
export function loadUser(input: string) {
  if (!input) console.warn("missing payload");
  throw new Error("missing id");
}

const user = loadUser(payload);
console.log(user);
```

### VitePress 記法

`notation: "vitepress"`（または `"both"`）は、互換のフェンス
メタデータとインラインコメントディレクティブを有効にします。フェンス meta の部品は独立して合成されます。

- `{1,3}` — ハイライト行。
- `[config.ts]` — ブロックの上に描画されるファイル名ラベル。
- `:line-numbers` / `:line-numbers=7` / `:no-line-numbers` — ブロックごとの行番号。
  任意の開始付き。

````md
```ts:line-numbers=7 {1,3} [config.ts]
const token = readToken();
const expires = readExpiry(token);
refreshBefore(expires);
```
````

描画結果:

```ts:line-numbers=7 {1,3} [config.ts]
const token = readToken();
const expires = readExpiry(token);
refreshBefore(expires);
```

インラインコメントディレクティブは乗っている行に注釈を付け、出力から取り除かれます。
このブロックは 2 行目に `// [!code warning]`、3 行目に `// [!code error]` で書いています。

```ts
const token = readToken();
console.warn("Token expires soon"); // [!code warning]
throw new Error("Token is invalid"); // [!code error]
```

diff 記法は削除に `// [!code --]`、追加に `// [!code ++]` を使います。
このブロックは 2 つの `return` 行に付けています。

```ts
export function resolve(id: string) {
  return legacyResolve(id); // [!code --]
  return nativeResolve(id); // [!code ++]
}
```

`// [!code focus]`（範囲なら `// [!code focus:3]`）は、フォーカスした行以外を暗くします。

インラインディレクティブは、コードブロック内のどこに現れても消費されます。
外側のフェンスに入れ子になったフェンス例も含みます。注釈に見えるテキストを行に出す必要があるときは、
下のエスケープディレクティブを使ってください。

### エスケープ

単独の `// [!code escape]` コメントは出力から取り除かれ、
次の行をリテラルに描画します。このブロックは最初の `console.warn` 行の上にエスケープコメントを書いているので、
その `// [!code warning]` はテキストとして残り、2 つ目は注釈になります。

```ts
// [!code escape]
console.warn("literal"); // [!code warning]
console.warn("annotated"); // [!code warning]
```

### カスタム meta キー

`annotate` をよりドメイン固有の属性名へ差し替えます。

```ts
oxContent({
  codeAnnotations: {
    metaKey: "markers",
  },
});
```

````md
```ts markers="highlight:2;warning:3"
const token = readToken();
refreshToken(token);
console.warn("Token expires soon");
```
````

## コードインポート

コピー＆ペーストせず、検査済みソースファイルを Markdown へインポートします。

```ts
oxContent({
  codeImports: {
    // Root for `@/` imports. Defaults to the Vite project root.
    rootDir: process.cwd(),
  },
});
```

フェンス言語はファイル拡張子から推測され、インポートしたスニペットは
インラインフェンスと同じハイライトと注釈パイプラインを通ります。

単独行に `<<< @/snippets/greet.ts` と書くと、ファイル全体をインポートします。

<<< @/snippets/greet.ts

`{1-4}` 接尾辞 — `<<< @/snippets/greet.ts{1-4}` — は行範囲をインポートします。

<<< @/snippets/greet.ts{1-4}

名前付き接尾辞 — `<<< @/snippets/greet.ts{greet}` — は
`#region greet` / `#endregion greet` コメントで区切られた領域をインポートし、マーカー
自身は取り除きます。

<<< @/snippets/greet.ts{greet}

インポートは transform 時に解決されるので、ソースファイルを編集すると
インポートしているすべてのページが更新され、古い docs スニペットは起きにくくなります。

`<<<` 参照はフェンス付きコードブロックの中でも解決されるので、リテラルに見せたいときは
（このページのように）インラインコードで構文を引用してください。

## コードグループ

隣り合う JS / TS / shell の別例は、`codeGroups` をオンにしてフェンスを `::: code-group` で囲みます。手書きの `<tabs>` は不要です。タイトルは ` ```ts [label] ` かフェンス meta です。[コードグループ](./code-groups.md) を見てください。

## 関連

- [コードグループ](./code-groups.md) — VitePress 風のグループ化フェンス。
- [品質チェック](./quality-checks.md) — コードブロック自体を lint、型チェック、テストする。
- [型ホバー](./typed-hover.md) — `twoslash` フェンスのビルド時 TypeScript 型オーバーレイ。
- [コード注釈の例](/examples/code-annotations.md)
- [コードインポートの例](/examples/code-imports.md)
