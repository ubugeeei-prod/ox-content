# JSDoc `@throws` タグサポート計画

## 背景

現状の docs pipeline では、`@throws` は extractor で `DocTag` として抽出されるものの、`normalize_doc_metadata` で既知タグとして扱われない。そのため `tags` に残り、Markdown では generic `Tags` セクションに `@throws` の raw value として出力される。

`@param` / `@returns` / `@example` / `@since` と同様に、`@throws` はAPI利用者にとって呼び出し契約の一部なので、generic tagではなく専用の構造とセクションとして扱う。

## 現状の関係箇所

- 抽出: `crates/ox_content_docs/src/extractor.rs`
  - `DocTag` は `type_annotation`, `name`, `description`, `value` を持つ。
  - `ox_jsdoc` が `{Error}` や説明を解析できれば流用できる。
  - fallback parserでは raw value のみになる。
- 正規化: `crates/ox_content_docs/src/normalize.rs`
  - `normalize_doc_metadata` が `params`, `returns`, `examples`, `tags` に振り分ける。
  - `@throws` は現在 fallback の `normalized_tags` に入る。
- 公開IR: `crates/ox_content_docs/src/model.rs`
  - `ApiDocEntry` / `ApiDocMember` に `throws` 相当のフィールドがない。
- JSON: `crates/ox_content_docs/src/data.rs`
  - entry/member JSONは `tags` をそのまま出力する。
- Markdown:
  - pure: `crates/ox_content_docs/src/markdown/markdown_pure.rs`
  - html: `crates/ox_content_docs/src/markdown/markdown_html.rs`
  - structured tagは `is_structured_tag` で generic `Tags` から除外する。
- NAPI/TS:
  - `crates/ox_content_napi/src/docs_markdown_types.rs`
  - `crates/ox_content_napi/src/docs_bindings.rs`
  - `crates/ox_content_napi/index.d.ts`
  - `npm/vite-plugin-ox-content/src/types.ts`

## 提案するデータモデル

`@throws` / `@exception` を同じ構造へ正規化する。

Rust core:

```rust
pub struct NormalizedThrowsDoc {
    pub type_annotation: Option<String>,
    pub description: String,
}

pub struct ApiThrowsDoc {
    pub type_annotation: Option<String>,
    pub description: String,
}
```

追加先:

- `NormalizedDocEntry.throws: Vec<NormalizedThrowsDoc>`
- `NormalizedMember.throws: Vec<NormalizedThrowsDoc>`
- `ApiDocEntry.throws: Vec<ApiThrowsDoc>`
- `ApiDocMember.throws: Vec<ApiThrowsDoc>`
- NAPI: `JsDocThrows`
- Vite plugin types: `ThrowsDoc`

JSON / JS APIでは `throws?: ThrowsDoc[]` として出力する。既存 `tags` からは `throws` / `exception` を除外する。

## パース方針

タグ名:

- `@throws`
- `@exception`

優先順位:

1. `DocTag.type_annotation` があれば `type_annotation` に使う。
2. `DocTag.description` があれば `description` に使う。
3. fallback / raw tagでは `value` から簡易解析する。

raw value簡易解析:

- `{Type} description` -> `type_annotation = Some("Type")`, `description = "description"`
- `Type description` は曖昧なので、原則 `description = raw` とする。
- 空値は出力しない。ただし `{Type}` のみなら `type_annotation = Some("Type")`, `description = ""` として保持する。

## Markdown出力方針

pure markdown:

- `Parameters` と `Returns` の近くに `Throws` セクションを追加する。
- 推奨順序:
  1. Type Parameters
  2. Members
  3. Parameters
  4. Returns
  5. Throws
  6. Examples
  7. Tags
- 表示例:

```md
## Throws

- `ValidationError` — When input validation fails.
- When the operation is aborted.
```

html renderer:

- `ox-api-entry__section--throws` を追加する。
- 既存の params/returns 表示と同じく type link processing を通す。
- generic tags listには表示しない。

member rendering:

- method / constructor / getter / setter など callable member の詳細表示に `Throws` を出す。
- propertyなど非callableに `@throws` が付いた場合は、正規化で保持しつつMarkdown詳細に出す方針にする。誤記の可能性はあるが、ユーザーが書いた情報は落とさない。

## 実装手順

1. `normalize.rs`
   - `NormalizedThrowsDoc` を追加。
   - `NormalizedDocMetadata` に `throws: Vec<NormalizedThrowsDoc>` を追加。
   - `normalize_doc_metadata` で `throws` / `exception` を専用処理へ分岐。
   - `normalized_throws_from_tag` と raw fallback parserを追加。
   - generic tagsから `throws` / `exception` を除外。
2. `model.rs`
   - `ApiThrowsDoc` を追加。
   - `ApiDocEntry` / `ApiDocMember` に `#[serde(default)] throws` を追加。
3. graph / conversion
   - normalized entry/memberから API IRへ `throws` を渡す。
   - `docs_bindings.rs` の map / convert 関数にも追加。
4. JSON
   - `data.rs` の entry/member JSONに `throws` 配列を追加。
5. Markdown
   - pure rendererに `push_throws` を追加。
   - html rendererに `push_throws_html` を追加。
   - `is_structured_tag` に `throws` / `exception` を追加するか、正規化済みIRでは残らない前提にする。ただし手書き `ApiDocEntry.tags` 入力の後方互換のため追加しておく。
6. TypeScript型
   - `crates/ox_content_napi/index.d.ts`
   - `npm/vite-plugin-ox-content/src/types.ts`
   - `JsDocEntry`, `JsDocMember`, `JsDocsMarkdownEntry` に `throws?: ThrowsDoc[]` を追加。

## テスト計画

Rust core:

- `normalize.rs`
  - `@throws {ValidationError} When invalid.` が `entry.throws` に入る。
  - `@exception {AbortError} When aborted.` が同じ構造に入る。
  - `tags` に `throws` / `exception` が残らない。
  - `{Type}` なしの `@throws When invalid.` も description として保持される。
- `data.rs`
  - `docs.json` に `throws` が出力され、`tags.throws` は出ない。
- Markdown pure
  - `## Throws` が出る。
  - `## Tags` に `@throws` が出ない。
  - inline link記法が処理される。
- Markdown html
  - `ox-api-entry__section--throws` が出る。
  - generic tags sectionに重複しない。
- member docs
  - class method / interface method の `throws` が出る。

NAPI / plugin:

- `generateDocsMarkdown` 入力に `throws` を渡してMarkdownに反映される。
- `extractDocsFromEntryPoints` の戻り型に `throws` が含まれる。
- `writeGeneratedDocs` 経由のsnapshot/fixtureで `throws` がgeneric tagsに落ちない。

## 互換性

- 追加フィールドは `#[serde(default)]` / optional TS property として扱うため、既存JSON入力は壊さない。
- ただし `tags.throws` を利用していたユーザーには出力場所が変わる。これは今回の目的通り、既知タグ正規化として扱う。
- 手書きで `tags: [{ tag: "throws", ... }]` を渡す古い呼び出しに対しては、`is_structured_tag` でgeneric tagsから除外するだけでは情報が消える。移行互換を重視するなら、Markdown renderer側で tags fallback から throws を拾う補助を入れる。

## 推奨方針

まずは正規化済みIRに `throws` を追加する。Markdown rendererだけで `tags` を特別扱いする実装は避ける。理由は、JSON / NAPI / Vite plugin型でも `@throws` を構造化して扱えるようにしないと、Markdown以外の利用者にはraw tag問題が残るため。

手書きMarkdown入力互換として、rendererでは `entry.throws` が空の場合に `entry.tags` の `throws` / `exception` を一時的に表示する fallback を検討する。ただし同時に `is_structured_tag` でgeneric tagsからは除外し、二重出力を避ける。
