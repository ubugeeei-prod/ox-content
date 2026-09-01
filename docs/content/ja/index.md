---
layout: entry
title: Ox Content
description: JavaScript 向けの Rust 製ドキュメント生成器と、高速な Markdown ツールキット。フレームワーク非依存のパイプライン、OG 画像、ゼロ JS 優先の MPA 出力。
hero:
  text: 高速な Markdown ツールキット
  tagline: Vite 時代のフレームワーク非依存ドキュメント生成器。OG 画像、テーマ、検索、Rust の速度、ゼロ JS 優先の MPA 出力を備えます。
  notice:
    title: 非公式プロジェクトです
    body:
      - このプロジェクトは VoidZero の公式プロダクトではありません。
      - ubugeeei が非公式の提案として ox-content を開発しています。将来 vp doc として採用されることを願っています。
      - 現在のブランディングとビジュアルは非公式のファンワークです。権利者から要請があれば改訂または取り下げます。
  image:
    src: oxcontent-dark.svg
    lightSrc: oxcontent-dark.svg
    darkSrc: oxcontent-light.svg
    alt: Ox Content ワードマーク
    width: 302
    height: 64
  actions:
    - theme: brand
      text: はじめる
      link: /ja/getting-started.md
    - theme: alt
      text: GitHub
      link: https://github.com/ubugeeei-prod/ox-content
    - theme: alt
      text: Sponsor
      link: https://github.com/sponsors/ubugeeei
features:
  - icon: "mdi:file-document-outline"
    title: JSDoc から API ドキュメント
    details: JavaScript / TypeScript プロジェクト向けに、docs.rs に近い偏りと第一級の Markdown ページでドキュメントを生成します。
    link: /ja/getting-started.md
  - icon: "mdi:layers-triple"
    title: フレームワーク非依存、Vite ネイティブ
    details: OG 画像、検索、テーマ、API ドキュメント、コンテンツ処理を内蔵したパイプラインです。
    link: /ja/theming.md
  - icon: "mdi:lightning-bolt"
    title: Rust と VoidZero の系譜
    details: 速度のために Rust で実装し、Vite / Oxc / Rolldown / Vitest の生態系に馴染むよう設計しています。
    link: /ja/architecture.md
  - icon: "mdi:web"
    title: ゼロ JS 優先の MPA
    details: 既定は高速なマルチページアプリです。島や対話 UI が必要なところにだけ JavaScript を足します。
  - icon: "mdi:puzzle-outline"
    title: 高速 Markdown エンジン
    details: パーサ、レンダラ、プラグインはデフォルトテーマの内部実装ではなく、ライブラリとして再利用できます。
    link: /ja/performance.md
  - icon: "mdi:connection"
    title: Vue / Svelte / React
    details: 第一級の統合で、コアパイプラインを捨てずにフレームワークコンポーネントを Markdown に埋め込めます。
---

## Ox Content とは

Ox Content は、JavaScript / TypeScript 向けの Rust 製ドキュメント生成器であり、高速な Markdown 処理ツールキットです。

いちばん短い説明は、「JavaScript のための `cargo doc` を、Vite ネイティブなワークフローで」です。

OG 画像、全文検索、テーマフック、API ドキュメント生成、再利用可能なコンテンツエンジンを含む、フレームワーク非依存のドキュメントパイプラインとしても使えます。

サイト出力は既定でゼロ JS 優先の MPA です。対話が必要なら island を hydrate し、Vue / Svelte / React と統合できます。

Ox Content はテーマだけではありません。Markdown パーサ、レンダラ、変換、プラグインを再利用できるので、サイト生成器の外でも Markdown ライブラリとして使えます。

## ユーザーガイド

- [はじめに](./getting-started.md)
- [組み込み機能](./built-in-features.md)
- [テーマ](./theming.md)
- [MDX とコンポーネント](./mdx.md)
- [ドキュメントのバージョン管理](./built-in/versioning.md)
- [国際化](./i18n.md)
- [事例](./examples/index.md)

英語のリファレンス（API、パッケージ、高度な解説）は [English home](/) から辿れます。ヘッダーの locale switcher で言語を切り替えられます。
