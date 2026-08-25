---
title: ドキュメントサイト機能のロードマップ
description: 焦点を絞った issue として追跡している、オプトインの組み込みドキュメントサイト機能です。
---

# ドキュメントサイト機能のロードマップ

トラッキング issue: [#650](https://github.com/ubugeeei-prod/ox-content/issues/650)。
この一覧は 3.0 のワークストリームです。リリーストラッカーは
[#699](https://github.com/ubugeeei-prod/ox-content/issues/699) です。

Ox Content は、非標準の Markdown と追加のサイト振る舞いは **オプトイン** にしています。下の各項目は、サイトが有効にするまでオフのままのファーストパーティ組み込みです。
作業は、まず落ちるテストを置いた小さな conventional PR で入ります。

ほかで追跡しているもの:

- 対話的なサンプル実行: [#648](https://github.com/ubugeeei-prod/ox-content/issues/648)
- i18n / MF2 コア: [#451](https://github.com/ubugeeei-prod/ox-content/issues/451)

## Phase A — 執筆

| 機能                                   | Issue                                                          | 状態    |
| -------------------------------------- | -------------------------------------------------------------- | ------- |
| カスタムコンテナ（`::: tip`）          | [#665](https://github.com/ubugeeei-prod/ox-content/issues/665) | planned |
| 数式（インライン `$` / ブロック `$$`） | [#666](https://github.com/ubugeeei-prod/ox-content/issues/666) | shipped |
| Markdown ファイルインクルード          | [#667](https://github.com/ubugeeei-prod/ox-content/issues/667) | shipped |
| 図、キャプション、遅延読み込み画像     | [#668](https://github.com/ubugeeei-prod/ox-content/issues/668) | planned |
| インラインバッジ                       | [#669](https://github.com/ubugeeei-prod/ox-content/issues/669) | planned |
| ファイルツリーブロック                 | [#670](https://github.com/ubugeeei-prod/ox-content/issues/670) | shipped |
| ステップリスト                         | [#671](https://github.com/ubugeeei-prod/ox-content/issues/671) | planned |
| card / link-card / card-grid ブロック  | [#672](https://github.com/ubugeeei-prod/ox-content/issues/672) | shipped |

## Phase B — サイト出力

| 機能                                    | Issue                                                          | 状態    |
| --------------------------------------- | -------------------------------------------------------------- | ------- |
| `sitemap.xml`、`robots.txt`、`llms.txt` | [#673](https://github.com/ubugeeei-prod/ox-content/issues/673) | shipped |
| RSS / Atom / JSON フィード              | [#674](https://github.com/ubugeeei-prod/ox-content/issues/674) | shipped |
| リダイレクト、エイリアス、パス書き換え  | [#675](https://github.com/ubugeeei-prod/ox-content/issues/675) | shipped |
| 下書き、非公開、予約公開ページ          | [#676](https://github.com/ubugeeei-prod/ox-content/issues/676) | shipped |
| カスタム 404 ページ                     | [#677](https://github.com/ubugeeei-prod/ox-content/issues/677) | shipped |
| パーマリンクと frontmatter カスケード   | [#678](https://github.com/ubugeeei-prod/ox-content/issues/678) | planned |

## Phase C — テーマクロム

| 機能                                         | Issue                                                          | 状態    |
| -------------------------------------------- | -------------------------------------------------------------- | ------- |
| 前へ / 次へページリンク                      | [#679](https://github.com/ubugeeei-prod/ox-content/issues/679) | planned |
| コードコピー、外部リンクアイコン、先頭へ戻る | [#680](https://github.com/ubugeeei-prod/ox-content/issues/680) | planned |
| ヘッダーナビ、告知バー、ページ単位のクロム   | [#681](https://github.com/ubugeeei-prod/ox-content/issues/681) | shipped |
| パンくずリスト                               | [#682](https://github.com/ubugeeei-prod/ox-content/issues/682) | shipped |
| チーム / メンバーページ                      | [#683](https://github.com/ubugeeei-prod/ox-content/issues/683) | shipped |
| ロケールスイッチャー                         | [#684](https://github.com/ubugeeei-prod/ox-content/issues/684) | shipped |
| スキップリンクと印刷スタイル                 | [#685](https://github.com/ubugeeei-prod/ox-content/issues/685) | planned |

## Phase D — コンテンツモデル

| 機能                                             | Issue                                                          | 状態    |
| ------------------------------------------------ | -------------------------------------------------------------- | ------- |
| タクソノミーと関連ページ                         | [#687](https://github.com/ubugeeei-prod/ox-content/issues/687) | planned |
| ブログ（索引、著者、タグ、読了時間、アーカイブ） | [#688](https://github.com/ubugeeei-prod/ox-content/issues/688) | planned |
| ドキュメントのバージョニング                     | [#689](https://github.com/ubugeeei-prod/ox-content/issues/689) | planned |
| 生成されるセクション索引ページ                   | [#690](https://github.com/ubugeeei-prod/ox-content/issues/690) | planned |
| ページリソースと画像処理                         | [#691](https://github.com/ubugeeei-prod/ox-content/issues/691) | planned |

## Phase E — 連携

| 機能                                      | Issue                                                          | 状態    |
| ----------------------------------------- | -------------------------------------------------------------- | ------- |
| Git コントリビューター                    | [#692](https://github.com/ubugeeei-prod/ox-content/issues/692) | planned |
| TypeScript フェンスの型ホバーオーバーレイ | [#693](https://github.com/ubugeeei-prod/ox-content/issues/693) | shipped |
| ホスト型検索プロバイダーアダプター        | [#694](https://github.com/ubugeeei-prod/ox-content/issues/694) | planned |
| PWA マニフェストとサービスワーカー        | [#695](https://github.com/ubugeeei-prod/ox-content/issues/695) | shipped |
| 構造化データ（JSON-LD）                   | [#696](https://github.com/ubugeeei-prod/ox-content/issues/696) | planned |
