---
title: リーダー chrome
description: コピーボタン、外部リンクアイコン、先頭へ戻る。
---

# リーダー chrome

`ssg.readerChrome` を有効にすると、テーマ付きページに次の操作が付きます。

- フェンスへの **Copy**
- 外部 `http(s)` リンクへのアイコンと `rel="noopener noreferrer"`
- スクロール後に出る **先頭へ戻る**

省略または `false` では追加のマークアップも JS も出しません。

```ts
oxContent({
  ssg: {
    readerChrome: true,
  },
});
```

オブジェクトで個別に切れます。

```ts
oxContent({
  ssg: {
    readerChrome: { copy: false },
  },
});
```

## 関連

- [英語版ガイド](/built-in/reader-chrome.md)
- [アクセシビリティ](./a11y.md)
