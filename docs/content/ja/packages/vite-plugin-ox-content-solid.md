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
  resolveClientModule: (module) =>
    module.source === "document" ? `./docs/${module.name}.tsx` : `./components/${module.name}.tsx`,
});
```

module cache は 1 回の render call に閉じます。development edit 後は host 側で page
state を invalidation し、改めて呼び直してください。`resolveClientModule()` が返した
identity は各 island の `data-ox-module` に書かれ、`data-ox-export` も一緒に出力されます。
browser 側の loader map でも同じ key を使ってください。これにより、2 つの document が同じ
local component 名を使っても、downstream の HTML replacement なしで別 module を読めます。

diagnostics は missing component、module load failure、missing export、SSR error、
unsupported document-local import form を document/component context 付きで返します。
対応する document-local form は Markdown-module adapter と同じで、default import と
local binding 付き named import です。

browser client は別 subpath にあります:
`@ox-content/vite-plugin-solid/html-host/client`。custom host が Vite、Node helper、native
optional dependency を巻き込まずに bundle できる browser-only entry です。これは Solid
hydration ではなく fresh mount の bridge です。既存の Ox Content island payload と
authoring 時の slot HTML を読み、target を空にしてから caller の `@solidjs/web` renderer に
渡します。SSR 済み self-closing island の rendered HTML は children として渡しません。

```tsx
import { initIslands } from "@ox-content/islands";
import { render } from "@solidjs/web";
import { initSolidHtmlHost } from "@ox-content/vite-plugin-solid/html-host/client";

const modules = import.meta.glob("./{docs,components}/**/*.tsx");

initSolidHtmlHost({
  initIslands,
  modules,
  render({ component: Component, props, element, slotHtml }) {
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
  onError(error) {
    console.error(error.message);
  },
  options: { selector: ".ox-content [data-ox-island]" },
});
```

`initSolidHtmlHost()` は host が渡した `initIslands()` を呼び、その controller を返します。
island runtime を host 側で直接管理している場合は、同じ subpath の
`createSolidHtmlHostLazyHydrate()` を使い、返ってきた同期 hydrate function を
`initIslands()` に渡してください。adapter は pending lazy import を cancellation 可能にします。
module 解決前に dispose された island は stale mount せず、mounted cleanup は 1 回だけ走り、
unknown module、loader、runtime、export、render の失敗は `onError` に通知されます。

## 独自ホストの island stylesheet

server-rendered island は、client module が mount する前から CSS を必要とすることが
あります。`oxContentCustomHost()` を使う場合は public host assets context から解決し、
host が Vite manifest や development module graph を直接触らない形にします。

```ts
const styles = ctx.assets.stylesheets({
  modules: rendered.clientModules.map((module) => module.moduleId),
});

const assets = ctx.assets.document({
  islandStyles: styles.stylesheets,
  clientEntries: ["src/main.ts"],
});

return {
  html: `<!doctype html><html><head>${assets.headHtml}</head><body>${rendered.html}</body></html>`,
  dependencies: styles.dependencies,
};
```

assets context は direct / transitive island CSS を client module script より前に解決し、
`ctx.assets.document()` 側で dedupe します。dev CSS query string は保持され、stylesheet 編集で
cached custom-host route を更新する dependency path も返ります。build mode では同じ method が
Vite manifest を内部で使い、同じ module identity から emitted hashed href を返します。
manifest や module graph を意図的に自分で管理する non-custom host 向けには、
低レベルの `resolveSolidIslandStylesheets()` helper も残っています。

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
