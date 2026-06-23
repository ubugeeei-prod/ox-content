# theme-renderer.ts

**[Source](https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/theme-renderer.ts)**

> 10 documented symbols. Read the signatures first, then expand each item for parameters, return types, and examples.

<div class="ox-api-stats" aria-label="API reference summary">
<span class="ox-api-stat">
  <strong>10</strong>
  <span>symbols</span>
</span>
<span class="ox-api-stat">
  <strong>5</strong>
  <span>functions</span>
</span>
<span class="ox-api-stat">
  <strong>3</strong>
  <span>interfaces</span>
</span>
<span class="ox-api-stat">
  <strong>1</strong>
  <span>types</span>
</span>
<span class="ox-api-stat">
  <strong>1</strong>
  <span>modules</span>
</span>
<span class="ox-api-stat">
  <strong>11</strong>
  <span>parameters</span>
</span>
<span class="ox-api-stat">
  <strong>16</strong>
  <span>members</span>
</span>
<span class="ox-api-stat">
  <strong>6</strong>
  <span>returns</span>
</span>
<span class="ox-api-stat">
  <strong>1</strong>
  <span>examples</span>
</span>
</div>

## Reference

<div class="ox-api-controls" data-ox-api-target=".ox-api-entry" role="toolbar" aria-label="Reference display controls">
<button type="button" class="ox-api-controls__button" data-ox-api-toggle="expand">Open all</button>
<button type="button" class="ox-api-controls__button" data-ox-api-toggle="collapse">Close all</button>
</div>

<details id="createtheme" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">createTheme(config: { layouts: Record&lt;string, ThemeComponent&gt;; defaultLayout?: string; }): ThemeComponent</code><span class="ox-api-entry__description">Creates a theme with layout switching support.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">3 params</span><span class="ox-api-badge">returns ThemeComponent</span><span class="ox-api-badge">1 example</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Creates a theme with layout switching support.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export function createTheme(config: {
  layouts: Record&lt;string, ThemeComponent&gt;;
  defaultLayout?: string;
}): ThemeComponent</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/theme-renderer.ts#L267-L291" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">config</code>
    <code class="ox-api-entry__param-type">{ layouts: Record&lt;string, <a href="#themecomponent">ThemeComponent</a>&gt;; defaultLayout?: string }</code>
  </div>
  
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">config.layouts</code>
    <code class="ox-api-entry__param-type">Record&lt;string, <a href="#themecomponent">ThemeComponent</a>&gt;</code>
  </div>
  
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">config.defaultLayout?</code>
    <code class="ox-api-entry__param-type">string</code>
  </div>
  <p class="ox-api-entry__param-description">optional</p>
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type"><a href="#themecomponent">ThemeComponent</a></code>
  
</div>
</div>
<div class="ox-api-entry__section ox-api-entry__section--examples">
<h4>Examples</h4>
<div class="ox-api-entry__example">
<div class="ox-api-entry__example-heading">Example 1</div>
<pre><code class="language-tsx">import { createTheme } from &#39;@ox-content/vite-plugin&#39;;
import { DefaultLayout } from &#39;./layouts/Default&#39;;
import { EntryLayout } from &#39;./layouts/Entry&#39;;

export default createTheme({
layouts: {
default: DefaultLayout,
entry: EntryLayout,
},
});</code></pre>

</div>
</div>
  </div>
</details>

<details id="defaulttheme" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">DefaultTheme({ children }: ThemeProps): JSXNode</code><span class="ox-api-entry__description">Default theme component. A minimal theme that renders page content with basic s…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 param</span><span class="ox-api-badge">returns JSXNode</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Default theme component. A minimal theme that renders page content with basic styling.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export function DefaultTheme({ children }: ThemeProps): JSXNode</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/theme-renderer.ts#L195-L240" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">param</code>
    <code class="ox-api-entry__param-type"><a href="#themeprops">ThemeProps</a></code>
  </div>
  
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type"><a href="./jsx-runtime.md#jsxnode">JSXNode</a></code>
  
</div>
</div>
  </div>
</details>

<details id="generatetypes" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">generateTypes(pages: PageData[], outDir: string): Promise&lt;void&gt;</code><span class="ox-api-entry__description">Generates TypeScript type definitions from page frontmatter.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">2 params</span><span class="ox-api-badge">returns Promise&lt;void&gt;</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Generates TypeScript type definitions from page frontmatter.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export async function generateTypes(pages: PageData[], outDir: string): Promise&lt;void&gt;</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/theme-renderer.ts#L178-L189" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">pages</code>
    <code class="ox-api-entry__param-type"><a href="#pagedata">PageData</a>[]</code>
  </div>
  <p class="ox-api-entry__param-description">All pages</p>
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">outDir</code>
    <code class="ox-api-entry__param-type">string</code>
  </div>
  <p class="ox-api-entry__param-description">Output directory for types</p>
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">Promise&lt;void&gt;</code>
  
</div>
</div>
  </div>
</details>

<details id="pagedata" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">PageData</code><span class="ox-api-entry__description">Page data for rendering.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">9 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Page data for rendering.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface PageData</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/theme-renderer.ts#L38-L57" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="pagedata-description">
  <td><code>description</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Page description</div></td>
</tr>
<tr id="pagedata-frontmatter">
  <td><code>frontmatter</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">Record&lt;string, unknown&gt;</code></td>
  <td><div class="ox-api-entry__member-description">Frontmatter</div></td>
</tr>
<tr id="pagedata-html">
  <td><code>html</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Rendered HTML content</div></td>
</tr>
<tr id="pagedata-lastupdated">
  <td><code>lastUpdated</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">number</code></td>
  <td><div class="ox-api-entry__member-description">Last git commit timestamp in milliseconds</div></td>
</tr>
<tr id="pagedata-layout">
  <td><code>layout</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Layout name</div></td>
</tr>
<tr id="pagedata-path">
  <td><code>path</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Source file path</div></td>
</tr>
<tr id="pagedata-title">
  <td><code>title</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Page title</div></td>
</tr>
<tr id="pagedata-toc">
  <td><code>toc</code></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="./types.md#tocentry">TocEntry</a>[]</code></td>
  <td><div class="ox-api-entry__member-description">Table of contents</div></td>
</tr>
<tr id="pagedata-url">
  <td><code>url</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Output URL path</div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="renderallpages" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">renderAllPages(pages: PageData[], options: ThemeRenderOptions): Promise&lt;Map&lt;string, string&gt;&gt;</code><span class="ox-api-entry__description">Renders all pages and generates type definitions.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">2 params</span><span class="ox-api-badge">returns Promise&lt;Map&lt;string, string&gt;&gt;</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Renders all pages and generates type definitions.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export async function renderAllPages(pages: PageData[], options: ThemeRenderOptions): Promise&lt;Map&lt;string, string&gt;&gt;</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/theme-renderer.ts#L152-L170" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">pages</code>
    <code class="ox-api-entry__param-type"><a href="#pagedata">PageData</a>[]</code>
  </div>
  <p class="ox-api-entry__param-description">All pages to render</p>
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">options</code>
    <code class="ox-api-entry__param-type"><a href="#themerenderoptions">ThemeRenderOptions</a></code>
  </div>
  <p class="ox-api-entry__param-description">Theme render options</p>
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">Promise&lt;Map&lt;string, string&gt;&gt;</code>
  <p class="ox-api-entry__return-description">Map of output paths to rendered HTML</p>
</div>
</div>
  </div>
</details>

<details id="renderpage" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">renderPage(page: PageData, options: ThemeRenderOptions): string</code><span class="ox-api-entry__description">Renders a page using the theme component.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">2 params</span><span class="ox-api-badge">returns string</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Renders a page using the theme component.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export function renderPage(page: PageData, options: ThemeRenderOptions): string</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/theme-renderer.ts#L84-L143" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">page</code>
    <code class="ox-api-entry__param-type"><a href="#pagedata">PageData</a></code>
  </div>
  <p class="ox-api-entry__param-description">Page data to render</p>
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">options</code>
    <code class="ox-api-entry__param-type"><a href="#themerenderoptions">ThemeRenderOptions</a></code>
  </div>
  <p class="ox-api-entry__param-description">Theme render options</p>
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">string</code>
  <p class="ox-api-entry__return-description">Rendered HTML string</p>
</div>
</div>
  </div>
</details>

<details id="theme-renderer" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">module</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__name">theme-renderer</code><span class="ox-api-entry__description">Theme Renderer for Static HTML Generation Renders JSX theme components to static HTML strings. No client-side JavaScrip…</span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Theme Renderer for Static HTML Generation</p>
<p>Renders JSX theme components to static HTML strings. No client-side JavaScript is included by default.</p>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/theme-renderer.ts#L1-L6" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
  </div>
</details>

<details id="themecomponent" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">type</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">ThemeComponent = (props: ThemeProps) =&gt; JSXNode</code><span class="ox-api-entry__description">Theme component type.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 param</span><span class="ox-api-badge">returns JSXNode</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Theme component type.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export type ThemeComponent = (props: ThemeProps) =&gt; JSXNode</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/theme-renderer.ts#L25" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">props</code>
    <code class="ox-api-entry__param-type"><a href="#themeprops">ThemeProps</a></code>
  </div>
  
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type"><a href="./jsx-runtime.md#jsxnode">JSXNode</a></code>
  
</div>
</div>
  </div>
</details>

<details id="themeprops" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">ThemeProps</code><span class="ox-api-entry__description">Props passed to the theme component.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 member</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Props passed to the theme component.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface ThemeProps</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/theme-renderer.ts#L30-L33" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="themeprops-children">
  <td><code>children</code></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="./jsx-runtime.md#jsxnode">JSXNode</a></code></td>
  <td><div class="ox-api-entry__member-description">Rendered page content as JSX</div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="themerenderoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">ThemeRenderOptions</code><span class="ox-api-entry__description">Theme render options.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">6 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Theme render options.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface ThemeRenderOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/theme-renderer.ts#L62-L75" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="themerenderoptions-base">
  <td><code>base</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Base URL path</div></td>
</tr>
<tr id="themerenderoptions-nav">
  <td><code>nav</code></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="./page-context.md#navgroup">NavGroup</a>[]</code></td>
  <td><div class="ox-api-entry__member-description">Navigation groups</div></td>
</tr>
<tr id="themerenderoptions-pages">
  <td><code>pages</code></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#pagedata">PageData</a>[]</code></td>
  <td><div class="ox-api-entry__member-description">All pages (for site context)</div></td>
</tr>
<tr id="themerenderoptions-sitename">
  <td><code>siteName</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Site name</div></td>
</tr>
<tr id="themerenderoptions-theme">
  <td><code>theme</code></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#themecomponent">ThemeComponent</a></code></td>
  <td><div class="ox-api-entry__member-description">Theme component to use</div><ul class="ox-api-entry__member-params"><li><code>props</code></li></ul></td>
</tr>
<tr id="themerenderoptions-typesoutdir">
  <td><code>typesOutDir</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Output directory for type definitions</div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>
