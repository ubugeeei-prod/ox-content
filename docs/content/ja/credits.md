---
title: クレジット
description: Ox Content のコミュニティクレジットと貢献の要約です。
---

# クレジット

Ox Content は [ubugeeei](https://github.com/ubugeeei) がメンテナンスしています。

このページは、プロジェクトを形作ったコミュニティ貢献を記録します。

## コミュニティクレジット

### kazupon

JSDoc 対応とドキュメント品質まわりで、大きなコミュニティ貢献をしてくれた
[kazupon](https://github.com/kazupon) に特に感謝します。

貢献の要約:

- JSDoc 対応を第一級の API ドキュメントワークフローとして形作るのに協力しました。
- Ox Content 自身のドキュメントで使っている API ドキュメント生成パイプラインに貢献しました。
- 生成 API ドキュメントとユーザー向けドキュメントの品質を改善しました。

### ryoppippi

Markdown 属性とリッチなソーシャル埋め込みの見た目の同等性について、本番移行からの
フィードバックをくれた [ryoppippi](https://github.com/ryoppippi) に感謝します。

貢献の要約:

- ryoppippi.com の Ox Content 移行中に見つかった、inline link と変換済み画像の
  属性ターゲットの regression を報告しました。
- sveltweet を通して、Twitter / X full card の見た目の契約の検証に協力しました。
- ryoppippi.com の移行中に、組み込みテーマ向けのセルフホスト Web フォント取得を
  要望しました。
- ryoppippi.com の移行中に、使った Iconify アイコン向けのセルフホスト CSS を
  要望しました。
- ryoppippi.com の移行中に、余分な HTML フォールバックページを出さない
  ホスト用リダイレクト出力を要望しました。
- ryoppippi.com の移行中に、Vite middleware server を閉じると本番 SSG 出力が
  繰り返されることを報告しました。
- オプトインの `<NotByAI />` 執筆開示バッジは、ryoppippi.com の Ox Content
  移行で要望され、本番実装として先に入った機能です。
- ページ出力と Markdown companion が `ssg.routePrefix` 配下に配置される一方、
  生成 feed の item URL から prefix が抜けていた問題を報告しました。
- ryoppippi.com の JSON-backed media feed 移行のために、programmatic feed item
  source を要望しました。

## 第三者の帰属

### react-tweet と sveltweet

Twitter / X の任意オプション `appearance: "full"` カードは静的 HTML / CSS です。見た目の契約（レイアウト、色トークン、操作アイコン）は
[react-tweet](https://github.com/vercel/react-tweet)（MIT、Copyright (c) 2023
Luis Alvarez）と [sveltweet](https://github.com/ryoppippi/sveltweet)（MIT、
Copyright (c) 2024 ryoppippi）に従います。実行時にこれらのパッケージへ依存しません。

両方のプロジェクトの MIT 著作権表示と許諾文は
`crates/ox_content_ssg/src/plugins/social-tweet-full.css` に再掲しています。

X、Twitter、および関連する標章は、それぞれの権利者の商標です。

### Not By AI バッジのアートワーク

オプトインの `<NotByAI />` バッジは、[Not By AI](https://notbyai.fyi) の公式
ライト / ダーク「Written by Human, Not By AI」SVG をベンダーしています。
コピーはベンダー時にサニタイズし、静的 HTML としてインラインします。実行時に
notbyai.fyi からスクリプトやアセットは読みません。

Not By AI および関連する標章は、それぞれの権利者の商標です。利用資格と商用
利用の条件は [Not By AI のガイドライン](https://notbyai.fyi) を見てください。
