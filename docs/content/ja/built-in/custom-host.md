---
title: 独自ホスト lifecycle
description: 独自 HTML ホスト向けに、Vite loading、development routing、cache invalidation、出力書き出しを Ox Content が持つ。
---

# 独自ホスト lifecycle

サイトが layout と公開ポリシーを持ちつつ、Vite lifecycle は Ox Content に任せたい
場合は `oxContentCustomHost()` を使います。

```ts
// vite.config.ts
import { defineConfig } from "vite";
import { oxContentCustomHost } from "@ox-content/vite-plugin";

export default defineConfig({
  appType: "custom",
  plugins: [
    oxContentCustomHost({
      host: "./src/site-host.ts",
      oxContent: {
        srcDir: "content",
        redirects: { provider: "netlify" },
        ssg: {
          markdownSource: true,
          siteUrl: "https://example.com",
          siteName: "Example",
        },
      },
      themeTokens: {
        theme: colorScheme,
        include: (name) => name.startsWith("syntax-"),
      },
    }),
  ],
});
```

この factory は `oxContent({ ssg: { enabled: false } })` と独自ホスト plugin を
一緒に登録します。すでに `oxContent()` を自分で入れているホスト向けには、
低レベルの `createOxContentCustomHostPlugin()` もあります。

## ホスト module

ホスト module は route を export します。route は通常の `Response`、または
`html`、`text`、`contentType`、metadata、dependencies を持つ plain object を
返せます。

```ts
// src/site-host.ts
export default {
  routes: [
    {
      path: "/",
      inputPath: "content/index.md",
      source: "# Home\n",
      aliases: ["/old-home"],
      dependencies: ["src/data.ts"],
      async render(ctx) {
        const data = await ctx.loadModule("/src/data.ts");
        const assets = ctx.assets.document({
          head: "<title>" + data.title + "</title>",
          sharedStyles: [ctx.assets.themeTokens?.href].filter(Boolean),
          clientEntries: ["src/main.ts"],
          crossorigin: true,
        });

        return {
          html: `<!doctype html><html><head>${assets.headHtml}</head><body>${data.html}</body></html>`,
          title: data.title,
        };
      },
    },
  ],
  async outputs(ctx) {
    const content = await ctx.memo("content", () => ctx.loadModule("/src/content.ts"));
    return {
      siteDescription: content.siteDescription,
      collectionNames: ["blog", "media"],
      collections: {
        blog: content.blogFeedItems,
        media: content.mediaFeedItems,
      },
    };
  },
};
```

development では Ox Content が Vite 経由で host を SSR load し、route を dispatch
します。status と content type を保ち、HTML だけに `transformIndexHtml()` を適用し、
route も custom 404 もなければ fallthrough します。route response は promise として
cache され、宣言した dependency の変更で該当 response だけ invalidation されます。
reload は debounce され、失敗した render は次回 retry され、古い in-flight render が
新しい cache entry を消すこともありません。

production では Vite が client asset と `.vite/manifest.json` を出したあと、
`closeBundle` から一度だけ走ります。host と site module を SSR load するためだけに
一時的な middleware-mode Vite server を開き、raw server ではなく `ctx.loadModule` を
渡し、`finally` で必ず閉じます。
`outputs(ctx)` は `feeds` が有効なときだけ呼ばれます。開発 server が
`dev.feedOutputs` に opt-in しない限り build 専用です。

`ctx.memo(key, load)` は、1 回の build または development route catalogue 生成中に
高価な読み込みを共有します。`routes(ctx)` と `outputs(ctx)` で同じ key を使えば、
module-global mutable state なしで host-owned content artifact を再利用できます。
失敗した loader は memo から消えるので次回 retry できます。

`dev.feedOutputs: true` を設定すると、開発中も `outputs(ctx)` から設定済みの
RSS、Atom、JSON feed file を配信します。同じ path の明示 route がある場合は host
route が優先されます。存在しない feed path は host の `notFound()` か Vite へ fall
through し、失敗した feed render は cache しないため、source や module error を
直したあとの次の一致リクエストで再試行できます。

## Markdown renderer

すべての独自ホスト context には `ctx.markdown.render()` があります。host が
Markdown / MDX の source を自分で読みつつ、設定済みの Ox Content rendering
pipeline は重複実装したくない場合に使います。raw `source` と実際の
`documentPath` を渡してください。

```ts
const page = await ctx.markdown.render<{
  clientModules: Array<{ moduleId: string }>;
}>({
  source,
  documentPath,
  async renderHtml(markdown) {
    const solid = await markdown.loadModule("/src/solid-renderer.ts");
    return solid.render(markdown.html, {
      documentPath: markdown.documentPath,
      imports: markdown.transform.imports,
      root: markdown.root,
      srcDir: markdown.srcDir,
      contentRoot: markdown.contentRoot,
    });
  },
});

const styles = ctx.assets.stylesheets({
  modules: page.metadata?.clientModules.map((module) => module.moduleId) ?? [],
});
```

サポートする順序は次の通りです。

1. resolved Ox Content option による Markdown / MDX syntax transform。
2. built-in HTML / embed transform。provider cache と media output の相対 path は
   Vite root から解決されます。
3. legacy `<Island>` HTML transform。
4. 任意の `renderHtml(markdown)` framework integration。
5. 呼び出し側が `readerChrome` を上書きしない限り、
   `oxContent.ssg.readerChrome` による reader-chrome HTML transform。

結果には最終 `html`、framework renderer 前の `markdownHtml`、完全な
`TransformResult`、flattened `frontmatter`、`toc`、MDX `imports`、MDX `exports`、
`components`、任意の framework `metadata`、`dependencies` が含まれます。
入力 `documentPath` は `dependencies` に入るので、返された dependency を route result
へ merge すると、開発中に source 変更で response cache が invalidation されます。
`markdown.loadModule()` で読んだ module と renderer callback 内で解決した stylesheet は、
`ctx.loadModule()` / `ctx.assets.stylesheets()` と同じ development lifecycle で追跡されます。
そのため、失敗した renderer や変更された renderer は process-global cache なしで復帰できます。

`renderHtml(markdown)` 内の `markdown.parseHtml()` は、callback に渡された HTML を
HAST fragment として返します。最初の呼び出し時に解析し、同じ render 内では同じ
mutable tree を再利用します。生成後の HTML の位置情報を省くため、element、attribute、
text だけを必要とする adapter で余分な位置計算を避けられます。tree の変更を出力へ
反映する場合は、serialize した HTML を callback から返してください。tree は render
ごとに分かれています。

Worker 内でデータを準備する場合は、programmatic Markdown entry を使えます。

```ts
import { createOxContentMarkdownRenderer } from "@ox-content/vite-plugin/markdown";

const markdown = createOxContentMarkdownRenderer({
  root: process.cwd(),
  oxContent: { srcDir: "content", embeds: false },
});
const page = await markdown.render({
  source: "# Hello",
  documentPath: "content/hello.md",
  renderHtml: ({ html, parseHtml }) => ({ html, metadata: parseHtml() }),
});
```

Renderer は各 Worker 内で作成します。Options を一度だけ解決し、Vite の server module
loader を初期化せずに同じ Custom Host Markdown pipeline を実行します。Callback が
framework module や client asset manifest を使う場合は `loadModule` と `assets` を
渡してください。Rendering のスケジュールと、返された `dependencies` の扱いは
caller が管理します。`TransformResult` を使う場合に向けて、この entry からも
`createMarkdownProcessor` と `renderMarkdown` を export しています。

framework integration は host-owned のままです。Solid HTML-string host では callback 内で
`@ox-content/vite-plugin-solid` の `createSolidHtmlHostRenderer()` を使い、返された
`html` と、host が asset / hydration に必要とする `clientModules` や diagnostics metadata を
返してください。`ctx.markdown.render()` は article HTML だけを変換します。copy control を
有効にする host は、自分の document shell に reader-chrome CSS、script、root attribute を
含める必要があります。

## 協調する出力

ホストが描画した HTML route は、既定 SSG と同じ公開 writer に接続されます。

- `writeResourceFiles()` による resource fingerprint と HTML URL 書き換え。
- `writeSelfHostedAssets()` による font と Iconify CSS。
- route `source` からの `writeMarkdownCompanions()`。
- route `aliases` / `redirect` からの `writeRedirectOutputs()`。
- 選択された route metadata からの `writeFeedFiles()` と `writeSiteMapFiles()`。

route metadata は route または render result に置きます。`title`、`description`、
`source`、`aliases`、`redirect`、`lastUpdatedPaths`、publish-state field はその HTML
page を説明します。page ではない site-output data は `outputs(ctx)` に置きます。
programmatic default `items`、名前付き feed `collections`、`collectionNames`、
`siteDescription` がそれです。Ox Content はその data を `planSsgOutputs()` と既存の
feed writer に流すので、publish-state filtering、validation、output path、diagnostic は
組み込み SSG と共有されます。
`dev.feedOutputs: true` のときは、同じ feed channel と `outputs(ctx)` の data を
file へ書かずにオンデマンドで render します。1 つの feed を配信するために全 page
を render することはありません。feed 専用の data は、host が `outputs(ctx)` から
feed collection か programmatic `items` として返してください。

組み込み SSG page には `ssg.minifyHtml: true`、独自ホスト plugin には
`build.minifyHtml: true` を指定すると production HTML を minify します。独自ホストの
build option を省略したときは `oxContent.ssg.minifyHtml` に従います。minify は
`transformIndexHtml()` と resource URL rewrite のあと、HTML file を書く直前に走ります。
XML feed、JSON feed、Markdown companion、text output、binary asset は変換しません。

route の出力 path が重複すると、どの route 同士が衝突したかを示して build を
失敗させます。公開対象の選択はホストが持ち、Ox Content はホストが返した route
だけを書きます。

## コレクションアセット

collection に付随する project image や download file を host が所有し、それを同じ manifest で
Ox Content に dev 配信 / build 書き出しさせたい場合は `collectionAssets` を使います。

```ts
oxContentCustomHost({
  host: "./src/site-host.ts",
  collectionAssets: {
    manifest: async (ctx) =>
      planCollectionAssets({
        root: ctx.root,
        assets: [
          {
            sourcePath: "content/projects/cover.jpg",
            publicPath: "/projects/cover.jpg",
          },
        ],
      }),
    watch: [{ path: "content/projects", kind: "directory" }],
    ownedPrefixes: ["/assets/content"],
  },
});
```

development では middleware が route rendering より前に alias と content-addressed target を
配信します。owned prefix 配下で file が見つからない URL は host router に fallthrough せず
404 になります。watch 対象 file や manifest source file が変わると、Ox Content は manifest を
re-plan し、成功した plan のあとで cached route response を消して reload します。失敗した
re-plan は retry 可能で、最後に成功した manifest を置き換えません。production では
`collectionAssets.write` が `false` でない限り、collection asset は協調 output file と一緒に
書き出されます。

route、render、output hook は、同じ configured snapshot を
`ctx.assets.collectionManifest()` から読めます。HTML rewrite や structured page data の alias
解決が必要な場合に使います。writer と development middleware はその resolved manifest を
再利用するため、`collectionAssets.manifest` を二度呼びません。

```ts
const manifest = await ctx.assets.collectionManifest();
const body = manifest
  ? rewriteCollectionAssetUrls({
      html,
      pagePath: ctx.url.pathname,
      manifest,
    }).html
  : html;
```

既に公開する Markdown / MDX document の集合が決まっているなら、拡張子 allowlist や広い glob
ではなく `planCollectionAssetsFromDocuments()` を使えます。

```ts
collectionAssets: {
  manifest: async (ctx) => {
    const result = await planCollectionAssetsFromDocuments({
      root: ctx.root,
      contentRoot: "content",
      documents: [{ documentPath: "content/posts/hello.mdx", pagePath: "/posts/hello" }],
      extraAssets: [{ sourcePath: "content/posts/og.png", publicPath: "/og/hello.png" }],
      publicPath: (reference) => [reference.publicPath, `/legacy${reference.publicPath}`],
    });
    if (result.diagnostics.length > 0) {
      throw new Error(result.diagnostics.map((diagnostic) => diagnostic.message).join("\n"));
    }
    return result.manifest;
  },
}
```

この helper は選択された document だけから、relative な Markdown image/link URL と、
literal な HTML / MDX JSX の `href`、`src`、`poster` 属性を見つけます。file は実際の
document path から解決し、既存の `CollectionAssetManifest` を返します。外部 URL、
root-absolute URL、data/blob、fragment-only link などの non-local URL は対象外です。
missing file と `contentRoot` 外への参照は document-scoped diagnostic になり、query
string と fragment は後続の `rewriteCollectionAssetUrls()` で保持できます。

Solid HTML-string host は、同じ選択済み route / document set から browser island registry を
生成できます。`@ox-content/vite-plugin-solid` の
`createSolidHtmlHostIslandRegistry()` を使い、client entry では directory 全体の
`import.meta.glob()` の代わりに `virtual:ox-content-solid/html-host/modules` を
import します。生成 module には選択された island dynamic-import root だけが入り、
Vite はその transitive dependency を保持します。

## SSR stylesheet

server-rendered page や layout module が blocking CSS を import する場合は、
小さな SSR stylesheet root の集合を先に設定し、render 時にその route が選んだ root だけを
解決します。

```ts
// vite.config.ts
oxContentCustomHost({
  host: "./src/host.ts",
  ssrStylesheets: {
    modules: ["src/layout.tsx", "src/pages/**/page.tsx"],
  },
});

// host.ts
const routes = [{ path: "/about", pageModule: "/src/pages/about/page.tsx" }];

export default {
  routes: routes.map((route) => ({
    path: route.path,
    async render(ctx) {
      const layout = await ctx.loadModule("/src/layout.tsx");
      const page = await ctx.loadModule(route.pageModule);
      const ssrStyles = ctx.assets.ssrStylesheets({
        modules: ["/src/layout.tsx", route.pageModule],
      });
      const assets = ctx.assets.document({
        pageStyles: ssrStyles.stylesheets,
        clientEntries: ["src/main.ts"],
      });

      return {
        html: `<!doctype html><html><head>${assets.headHtml}</head><body>${layout.render(page)}</body></html>`,
        dependencies: ssrStyles.dependencies,
      };
    },
  })),
};
```

rendered HTML string から source module id は推論できないため、host が module identity を
明示的に渡します。route descriptor には `path` や `inputPath` と同じように page module id を
保持できます。現在の route が選んだ layout / page root だけを渡してください。route catalogue
を作るために import された全 page module から style を導出すると、sibling route の CSS まで
含むため、この問題は解決しません。

development では、選択した SSR root を `ctx.loadModule()` で読んだあとに
`ctx.assets.ssrStylesheets()` を呼びます。resolver は Vite module graph から blocking
stylesheet href と dependency file を返し、cached route の invalidation に使えます。
`ssrStylesheets.modules` も watch されるため、設定された glob に一致する page module の追加や
削除で route catalogue を更新できます。

production build では、plugin が設定された local JavaScript / TypeScript SSR root を静的に辿り、
local static import を追跡して、直接 import された plain CSS file から hashed custom-host
CSS artifact を書きます。external package JavaScript は辿りません。local dynamic import、
missing root、未解決の local import、Vite root 外に解決された CSS は、黙って style を落とさず
diagnostic として返します。順序は source import order と route の `modules` order に従います。
返された style は `ctx.assets.document()` に渡すと、他の returned stylesheet descriptor との
重複を document asset API 側で dedupe できます。Vite plugin transform が必要な preprocessed
stylesheet は、明示的な browser / CSS entry に残して `ctx.assets.stylesheets()` を使ってください。

## Island stylesheet

描画済み route が SSR-visible island の browser module id を知っている場合は、
blocking CSS を `ctx.assets.stylesheets()` から解決します。

```ts
const islandStyles = ctx.assets.stylesheets({
  modules: rendered.clientModules.map((module) => module.moduleId),
});

const assets = ctx.assets.document({
  islandStyles: islandStyles.stylesheets,
  clientEntries: ["src/main.ts"],
});

return {
  html: `<!doctype html><html><head>${assets.headHtml}</head><body>${rendered.html}</body></html>`,
  dependencies: islandStyles.dependencies,
};
```

development では、host が `ctx.loadModule()` で island を描画したあと、描画済み browser module id
に対応する内部 Vite module graph を辿り、direct / transitive CSS を依存順に返します。
CSS query string は保持され、source file dependency も返るため cached route を invalidation
できます。build では Vite manifest を使い、同じ module identity から emitted hashed stylesheet
href を返します。entry が見つからない場合は style を黙って落とさず diagnostic に出ます。
返された style は `ctx.assets.document()` に渡すと、document-level dedupe、nonce、base、
shared CSS、page CSS と同じ場所で合成できます。

critical CSS を明示的に inline したい build host は、public href から filesystem path を
逆算せず、同じ resolved descriptor から emitted bytes を読めます。

```ts
const islandStyles = ctx.assets.stylesheets({
  modules: rendered.clientModules.map((module) => module.moduleId),
});
const critical = await ctx.assets.stylesheetContent({
  stylesheets: islandStyles.stylesheets,
});

const assets = ctx.assets.document({
  inlineStyles: critical.stylesheets.map((style) => ({
    key: `critical:${style.href}`,
    content: style.content,
  })),
});
```

`stylesheetContent()` は Vite が emit した stylesheet 向けの build-only API です。
development では `unavailable` diagnostic を返し、build では descriptor に artifact がない、
または file を読めない場合に `missing-artifact` diagnostic を返します。順序は input
descriptor に従うため、home-only inline、linked stylesheet、dedupe の policy は host 側で
維持できます。

## テーマトークン stylesheet

`themeTokens` は小さな stylesheet を書き出して dev でも配信します。既定 href は
`/__ox_theme_tokens__/theme-tokens.css` です。`ctx.assets.themeTokens.href` を
`ctx.assets.document()` に渡せば、local Vite transform なしで syntax token CSS を
使えます。

## 関連

- [Document assets](./document-assets.md)
- [SSG 出力プリミティブ](./ssg-output.md)
- [テーマ](../theming.md)
- 追跡: [#1281](https://github.com/ubugeeei-prod/ox-content/issues/1281)
