---
title: "@ox-content/vite-plugin-solid"
description: Markdown に Solid component island を埋め込む Solid 連携。
---

# @ox-content/vite-plugin-solid

Ox Content の Solid 連携です。Markdown 内の Solid component を island として
埋め込み、Solid 2 と `@solidjs/vite-plugin` の native compiler で処理します。

## インストール

```bash
vp install @ox-content/vite-plugin-solid solid-js@next @solidjs/web@next @solidjs/vite-plugin
```

この 3.x beta adapter は Solid 2 と `@solidjs/vite-plugin` を対象にしています。
Solid 1 と `vite-plugin-solid` の peer dependency path は維持しません。Solid 1 の
app は古い adapter release を使ってください。

## 使い方

```ts
// vite.config.ts
import { defineConfig } from "vite";
import solid from "@solidjs/vite-plugin";
import { oxContentSolid } from "@ox-content/vite-plugin-solid";

export default defineConfig({
  plugins: [
    oxContentSolid({
      srcDir: "docs",
      components: "./src/components/*.tsx",
    }),
    solid({ extensions: [".md", ".markdown", ".mdx"], compiler: "native" }),
  ],
});
```

`oxContentSolid()` は `solid()` より前に置きます。どちらも `enforce: "pre"` なので、
配列順が重要です。また `solid()` には Markdown extension を渡してください。
Solid の JSX は compile-time only で、Markdown はこの plugin で Solid JSX へ変換し、
`@solidjs/vite-plugin` が DOM / SSR 命令へ compile します。

## component 登録

`components` には glob または明示 map を渡せます。

```ts
components: "./src/components/*.tsx";

components: {
  Counter: "./src/components/Counter.tsx",
  Alert: "./src/components/Alert.tsx",
}
```

`.mdx` では、その document 内の relative import が global map より優先されます。

```md
import GtvChart from './gtv-chart/GtvChart.tsx'

<GtvChart title="ok" />
```

## island

登録済み、または document-local import された component を使う Markdown は
`@ox-content/islands` の marker と runtime で hydrate されます。各 island は
`@solidjs/web` の `render` で mount され、Markdown component の unmount 時に
dispose されます。

component を使わない Markdown は island runtime を使わず、単一の `innerHTML`
binding として compile されます。

## HTML string の独自ホスト

`renderMarkdown()` で HTML string を得て、それを独自の Solid page shell に入れる
host は、Markdown document を Vite module として import しなくても Solid adapter
を使えます。`renderSolidHtmlHost()` は document-local MDX import を解決し、host が
渡した server module loader で component を読み、island body を Solid SSR HTML に
置き換えます。

```ts
import { renderSolidHtmlHost, type MdxImport } from "@ox-content/vite-plugin-solid";

const imports: MdxImport[] = [
  { source: "./Chart.tsx", specifiers: [{ imported: "default", local: "Chart", kind: "default" }] },
];

const rendered = await renderSolidHtmlHost({
  html: markdown.html,
  documentPath: "/repo/docs/report.mdx",
  root: "/repo",
  srcDir: "docs",
  imports,
  components: { Badge: "./src/components/Badge.tsx" },
  loadModule: (moduleId) => viteDevServer.ssrLoadModule(moduleId),
  resolveClientModule: (module) => `/assets/islands/${module.name}.js`,
});
```

module cache は 1 回の render call に閉じます。development edit 後は host 側で page
state を invalidation し、改めて呼び直してください。`modules` はその render の
server-side metadata です。browser に渡す JSON には `clientModules` か host の
resolver が返した公開 identity だけを serialize し、absolute filesystem path を
混ぜないでください。

diagnostics は missing component、module load failure、missing export、SSR error、
unsupported document-local import form を document/component context 付きで返します。
対応する document-local form は Markdown-module adapter と同じで、default import と
local binding 付き named import です。

client helper は Solid hydration ではなく fresh mount の bridge です。既存の
Ox Content island payload と slot HTML を読み、target を空にしてから caller の
`@solidjs/web` renderer に渡します。load strategy、dispose、cancellation を共有
runtime に任せるため、`initIslands()` と組み合わせます。

```tsx
import { initIslands } from "@ox-content/islands";
import { render } from "@solidjs/web";
import { createSolidHtmlHostHydrate } from "@ox-content/vite-plugin-solid";
import * as components from "./generated-island-client-modules";

const hydrate = createSolidHtmlHostHydrate({
  components,
  render(Component, props, element, slotHtml) {
    const dispose = render(
      () =>
        slotHtml ? (
          <Component {...props}>
            <div innerHTML={slotHtml} />
          </Component>
        ) : (
          <Component {...props} />
        ),
      element,
    );
    return dispose;
  },
});

initIslands(hydrate, { selector: ".ox-content [data-ox-island]" });
```

## 独自ホストの island stylesheet

server-rendered island は、client module が mount する前から CSS を必要とすることが
あります。`resolveSolidIslandStylesheets()` は Vite build manifest または
development module graph から、island module identity に必要な stylesheet URL を
解決します。

```ts
import { resolveSolidIslandStylesheets } from "@ox-content/vite-plugin-solid";

const styles = resolveSolidIslandStylesheets({
  modules: rendered.modules.map((module) => module.serverModuleId),
  manifest: viteManifest,
  base: "/docs/",
});

for (const stylesheet of styles.stylesheets) {
  head.push(`<link rel="stylesheet" href="${escapeHtml(stylesheet.href)}">`);
}
```

build 解決は static `imports` を辿り、CSS を重複排除し、imported chunk の CSS を
importing island の CSS より先に並べ、`base` と emitted hashed filename を尊重します。
development 解決は Vite 風の module graph（`getModuleById()` と任意の
`getModulesByFile()`）を受け取り、CSS query parameter を残すため HMR URL をそのまま
stylesheet として読めます。CSS を持たない有効な island は stylesheet も diagnostic も
返しません。manifest / module graph に entry が無ければ `missing-module`、resolver を
何も渡さなければ `missing-resolver` diagnostic を返します。

## HMR

component を編集すると hot reload されます。変更された component を使う Markdown
module も同時に invalidation されます。

## Rust と N-API codegen

Rust renderer は Vite pipeline なしで、rendered Markdown HTML から Solid code を
直接出せます。

```ts
import { renderFrameworkComponentCode } from "@ox-content/napi";

renderFrameworkComponentCode("<p>Hello</p>", "solid", [], "component");
```

この path は JSX compiler なしで動く必要があるため、`solid-js/h` の hyperscript
entrypoint を対象にします。Vite plugin は JSX を出力し、Solid compiler がより速く
細かい出力へ compile します。
