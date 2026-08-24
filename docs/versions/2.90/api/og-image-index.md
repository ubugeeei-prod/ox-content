# index.ts

**[Source](https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/og-image/index.ts)**

> 18 documented symbols. Read the signatures first, then expand each item for parameters, return types, and examples.

<div class="ox-api-stats" aria-label="API reference summary">
<span class="ox-api-stat">
  <strong>18</strong>
  <span>symbols</span>
</span>
<span class="ox-api-stat">
  <strong>15</strong>
  <span>functions</span>
</span>
<span class="ox-api-stat">
  <strong>2</strong>
  <span>interfaces</span>
</span>
<span class="ox-api-stat">
  <strong>1</strong>
  <span>variables</span>
</span>
<span class="ox-api-stat">
  <strong>31</strong>
  <span>parameters</span>
</span>
<span class="ox-api-stat">
  <strong>5</strong>
  <span>members</span>
</span>
<span class="ox-api-stat">
  <strong>14</strong>
  <span>returns</span>
</span>
</div>

## Reference

<div class="ox-api-controls" data-ox-api-target=".ox-api-entry" role="toolbar" aria-label="Reference display controls">
<button type="button" class="ox-api-controls__button" data-ox-api-toggle="expand">Open all</button>
<button type="button" class="ox-api-controls__button" data-ox-api-toggle="collapse">Close all</button>
</div>

<details id="computetemplatesource" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">computeTemplateSource(options: ResolvedOgImageOptions, root: string): Promise&lt;string&gt;</code><span class="ox-api-entry__description">Computes a stable template source identifier for cache keys. For custom templat…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">2 params</span><span class="ox-api-badge">returns Promise&lt;string&gt;</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Computes a stable template source identifier for cache keys.</p>
<p>For custom templates, hashes the file content so cache invalidates when the template changes. For the default template, returns a fixed string.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">async function computeTemplateSource(options: ResolvedOgImageOptions, root: string): Promise&lt;string&gt;</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/og-image/index.ts#L506-L518" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">options</code>
    <code class="ox-api-entry__param-type"><a href="./og-image-types.md#resolvedogimageoptions">ResolvedOgImageOptions</a></code>
  </div>
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">root</code>
    <code class="ox-api-entry__param-type">string</code>
  </div>
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">Promise&lt;string&gt;</code>
</div>
</div>
  </div>
</details>

<details id="createsveltecompilerplugin" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">createSvelteCompilerPlugin(): import(&quot;rolldown&quot;).Plugin</code><span class="ox-api-entry__description">Creates a rolldown plugin that compiles Svelte SFCs using svelte/compiler.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">returns import(&quot;rolldown&quot;).Plugin</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Creates a rolldown plugin that compiles Svelte SFCs using svelte/compiler.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">function createSvelteCompilerPlugin(): import(&quot;rolldown&quot;).Plugin</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/og-image/index.ts#L395-L420" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">import(&quot;rolldown&quot;).Plugin</code>
</div>
</div>
  </div>
</details>

<details id="createvuecompilerplugin" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">createVueCompilerPlugin(): import(&quot;rolldown&quot;).Plugin</code><span class="ox-api-entry__description">Creates a rolldown plugin that compiles Vue SFCs using @vue/compiler-sfc.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">returns import(&quot;rolldown&quot;).Plugin</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Creates a rolldown plugin that compiles Vue SFCs using @vue/compiler-sfc.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">function createVueCompilerPlugin(): import(&quot;rolldown&quot;).Plugin</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/og-image/index.ts#L267-L319" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">import(&quot;rolldown&quot;).Plugin</code>
</div>
</div>
  </div>
</details>

<details id="generateogimages" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">generateOgImages(pages: OgImagePageEntry[], options: ResolvedOgImageOptions, root: string): Promise&lt;OgImageResult[]&gt;</code><span class="ox-api-entry__description">Generates OG images for a batch of pages. Manages the full lifecycle: resolve t…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">3 params</span><span class="ox-api-badge">returns Promise&lt;OgImageResult[]&gt;</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Generates OG images for a batch of pages.</p>
<p>Manages the full lifecycle: resolve template → launch browser (with <code>using</code>) → render each page (with caching and concurrency).</p>
<p>All errors are non-fatal: failures are reported in results but never throw.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export async function generateOgImages(pages: OgImagePageEntry[], options: ResolvedOgImageOptions, root: string): Promise&lt;OgImageResult[]&gt;</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/og-image/index.ts#L528-L579" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">pages</code>
    <code class="ox-api-entry__param-type"><a href="#ogimagepageentry">OgImagePageEntry</a>[]</code>
  </div>
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">options</code>
    <code class="ox-api-entry__param-type"><a href="./og-image-types.md#resolvedogimageoptions">ResolvedOgImageOptions</a></code>
  </div>
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">root</code>
    <code class="ox-api-entry__param-type">string</code>
  </div>
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">Promise&lt;<a href="#ogimageresult">OgImageResult</a>[]&gt;</code>
</div>
</div>
  </div>
</details>

<details id="getvizejsplugin" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">getVizejsPlugin(): Promise&lt;import(&quot;rolldown&quot;).Plugin[]&gt;</code><span class="ox-api-entry__description">Loads @vizejs/vite-plugin as a rolldown plugin for Vue SFC compilation.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">returns Promise&lt;import(&quot;rolldown&quot;).Plugin[]&gt;</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Loads @vizejs/vite-plugin as a rolldown plugin for Vue SFC compilation.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">async function getVizejsPlugin(): Promise&lt;import(&quot;rolldown&quot;).Plugin[]&gt;</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/og-image/index.ts#L324-L335" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">Promise&lt;import(&quot;rolldown&quot;).Plugin[]&gt;</code>
</div>
</div>
  </div>
</details>

<details id="isbarespecifier" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">isBareSpecifier(id: string): boolean</code><span class="ox-api-entry__description">Whether id is a bare specifier, and so resolvable at runtime rather than someth…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 param</span><span class="ox-api-badge">returns boolean</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Whether <code>id</code> is a bare specifier, and so resolvable at runtime rather than something the template bundle has to inline.</p>
<p>Template bundles are written to <code>&lt;root&gt;/.cache/og-images/</code> and imported from there, so Node resolves anything left external against the project&#39;s own <code>node_modules</code>. Relative and absolute imports still bundle, which is what a template actually needs — its own components travel with it.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export function isBareSpecifier(id: string): boolean</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/og-image/index.ts#L124-L130" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">id</code>
    <code class="ox-api-entry__param-type">string</code>
  </div>
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">boolean</code>
</div>
</div>
  </div>
</details>

<details id="ogimagepageentry" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">OgImagePageEntry</code><span class="ox-api-entry__description">A single page entry for batch OG image generation.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">2 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>A single page entry for batch OG image generation.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface OgImagePageEntry</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/og-image/index.ts#L46-L51" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="ogimagepageentry-outputpath">
  <td><code>outputPath</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Absolute path to write the output PNG</div></td>
</tr>
<tr id="ogimagepageentry-props">
  <td><code>props</code></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="./og-image-types.md#ogimagetemplateprops">OgImageTemplateProps</a></code></td>
  <td><div class="ox-api-entry__member-description">Props to pass to the template</div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="ogimageresult" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">OgImageResult</code><span class="ox-api-entry__description">Result of OG image generation for a single page.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">3 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Result of OG image generation for a single page.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface OgImageResult</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/og-image/index.ts#L56-L60" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="ogimageresult-cached">
  <td><code>cached</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
<tr id="ogimageresult-error">
  <td><code>error</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="ogimageresult-outputpath">
  <td><code>outputPath</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="ox_content_package" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">variable</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">const OX_CONTENT_PACKAGE = /^@ox-content\/vite-plugin(\/.*)?$/</code><span class="ox-api-entry__description">Matches this package and every subpath it exports. A template&#39;s natural runtime…</span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Matches this package and every subpath it exports.</p>
<p>A template&#39;s natural runtime is whatever renders it, and for the framework-less kinds that is this package: <code>renderToString</code>, <code>raw</code>, <code>when</code> and <code>each</code> live at its root, and the JSX runtime under <code>./jsx-runtime</code>. Inlining them instead drags the entire plugin — chokidar, fsevents and all — into the template bundle, which is what made importing it fail outright.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">const OX_CONTENT_PACKAGE = /^@ox-content\/vite-plugin(\/.*)?$/</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/og-image/index.ts#L113" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
  </div>
</details>

<details id="rendersinglepage" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">renderSinglePage(entry: OgImagePageEntry, templateFn: OgImageTemplateFn, templateSource: string, options: ResolvedOgImageOptions, cacheDir: string, session: OgBrowserSession, publicDir?: string): Promise&lt;OgImageResult&gt;</code><span class="ox-api-entry__description">Renders a single page to PNG, with cache support.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">7 params</span><span class="ox-api-badge">returns Promise&lt;OgImageResult&gt;</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Renders a single page to PNG, with cache support.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">async function renderSinglePage(entry: OgImagePageEntry, templateFn: OgImageTemplateFn, templateSource: string, options: ResolvedOgImageOptions, cacheDir: string, session: OgBrowserSession, publicDir?: string): Promise&lt;OgImageResult&gt;</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/og-image/index.ts#L616-L673" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">entry</code>
    <code class="ox-api-entry__param-type"><a href="#ogimagepageentry">OgImagePageEntry</a></code>
  </div>
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">templateFn</code>
    <code class="ox-api-entry__param-type"><a href="./og-image-types.md#ogimagetemplatefn">OgImageTemplateFn</a></code>
  </div>
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">templateSource</code>
    <code class="ox-api-entry__param-type">string</code>
  </div>
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">options</code>
    <code class="ox-api-entry__param-type"><a href="./og-image-types.md#resolvedogimageoptions">ResolvedOgImageOptions</a></code>
  </div>
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">cacheDir</code>
    <code class="ox-api-entry__param-type">string</code>
  </div>
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">session</code>
    <code class="ox-api-entry__param-type"><a href="./browser.md#ogbrowsersession">OgBrowserSession</a></code>
  </div>
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">publicDir</code>
    <code class="ox-api-entry__param-type">string</code>
  </div>
  <p class="ox-api-entry__param-description">optional</p>
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">Promise&lt;<a href="#ogimageresult">OgImageResult</a>&gt;</code>
</div>
</div>
  </div>
</details>

<details id="resolveogimageoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">resolveOgImageOptions(options: OgImageOptions | undefined): ResolvedOgImageOptions</code><span class="ox-api-entry__description">Resolves user-provided OG image options with defaults.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 param</span><span class="ox-api-badge">returns ResolvedOgImageOptions</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Resolves user-provided OG image options with defaults.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export function resolveOgImageOptions(options: OgImageOptions | undefined): ResolvedOgImageOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/og-image/index.ts#L32-L41" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">options</code>
    <code class="ox-api-entry__param-type"><a href="./og-image-types.md#ogimageoptions">OgImageOptions</a> | undefined</code>
  </div>
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type"><a href="./og-image-types.md#resolvedogimageoptions">ResolvedOgImageOptions</a></code>
</div>
</div>
  </div>
</details>

<details id="resolvereacttemplate" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">resolveReactTemplate(templatePath: string, root: string): Promise&lt;OgImageTemplateFn&gt;</code><span class="ox-api-entry__description">Resolves a React (.tsx/.jsx) template via SSR. Bundles with rolldown (JSX trans…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">2 params</span><span class="ox-api-badge">returns Promise&lt;OgImageTemplateFn&gt;</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Resolves a React (.tsx/.jsx) template via SSR.</p>
<p>Bundles with rolldown (JSX transform), then wraps with react-dom/server renderToReadableStream for async Server Component support.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">async function resolveReactTemplate(templatePath: string, root: string): Promise&lt;OgImageTemplateFn&gt;</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/og-image/index.ts#L428-L498" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">templatePath</code>
    <code class="ox-api-entry__param-type">string</code>
  </div>
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">root</code>
    <code class="ox-api-entry__param-type">string</code>
  </div>
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">Promise&lt;<a href="./og-image-types.md#ogimagetemplatefn">OgImageTemplateFn</a>&gt;</code>
</div>
</div>
  </div>
</details>

<details id="resolvesveltetemplate" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">resolveSvelteTemplate(templatePath: string, root: string): Promise&lt;OgImageTemplateFn&gt;</code><span class="ox-api-entry__description">Resolves a Svelte SFC template via SSR. Compiles the SFC with svelte/compiler (…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">2 params</span><span class="ox-api-badge">returns Promise&lt;OgImageTemplateFn&gt;</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Resolves a Svelte SFC template via SSR.</p>
<p>Compiles the SFC with svelte/compiler (server mode + runes), bundles with rolldown, then wraps with svelte/server render().</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">async function resolveSvelteTemplate(templatePath: string, root: string): Promise&lt;OgImageTemplateFn&gt;</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/og-image/index.ts#L343-L390" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">templatePath</code>
    <code class="ox-api-entry__param-type">string</code>
  </div>
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">root</code>
    <code class="ox-api-entry__param-type">string</code>
  </div>
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">Promise&lt;<a href="./og-image-types.md#ogimagetemplatefn">OgImageTemplateFn</a>&gt;</code>
</div>
</div>
  </div>
</details>

<details id="resolvetemplate" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">resolveTemplate(options: ResolvedOgImageOptions, root: string): Promise&lt;OgImageTemplateFn&gt;</code><span class="ox-api-entry__description">Resolves the template function from options. Dispatches by file extension: - .v…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">2 params</span><span class="ox-api-badge">returns Promise&lt;OgImageTemplateFn&gt;</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Resolves the template function from options.</p>
<p>Dispatches by file extension:</p>
<ul>
<li><code>.vue</code>  → Vue SFC (SSR via vue/server-renderer)</li>
<li><code>.svelte</code> → Svelte SFC (SSR via svelte/server)</li>
<li><code>.tsx</code>/<code>.jsx</code> → React Server Component (SSR via react-dom/server)</li>
<li>others → TypeScript template (direct function export)</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">async function resolveTemplate(options: ResolvedOgImageOptions, root: string): Promise&lt;OgImageTemplateFn&gt;</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/og-image/index.ts#L71-L102" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">options</code>
    <code class="ox-api-entry__param-type"><a href="./og-image-types.md#resolvedogimageoptions">ResolvedOgImageOptions</a></code>
  </div>
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">root</code>
    <code class="ox-api-entry__param-type">string</code>
  </div>
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">Promise&lt;<a href="./og-image-types.md#ogimagetemplatefn">OgImageTemplateFn</a>&gt;</code>
</div>
</div>
  </div>
</details>

<details id="resolvetstemplate" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">resolveTsTemplate(templatePath: string, options: ResolvedOgImageOptions, root: string): Promise&lt;OgImageTemplateFn&gt;</code><span class="ox-api-entry__description">Resolves a plain TypeScript template (existing behavior).</span><span class="ox-api-entry__meta"><span class="ox-api-badge">3 params</span><span class="ox-api-badge">returns Promise&lt;OgImageTemplateFn&gt;</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Resolves a plain TypeScript template (existing behavior).</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">async function resolveTsTemplate(templatePath: string, options: ResolvedOgImageOptions, root: string): Promise&lt;OgImageTemplateFn&gt;</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/og-image/index.ts#L152-L181" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">templatePath</code>
    <code class="ox-api-entry__param-type">string</code>
  </div>
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">options</code>
    <code class="ox-api-entry__param-type"><a href="./og-image-types.md#resolvedogimageoptions">ResolvedOgImageOptions</a></code>
  </div>
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">root</code>
    <code class="ox-api-entry__param-type">string</code>
  </div>
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">Promise&lt;<a href="./og-image-types.md#ogimagetemplatefn">OgImageTemplateFn</a>&gt;</code>
</div>
</div>
  </div>
</details>

<details id="resolvevuetemplate" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">resolveVueTemplate(templatePath: string, options: ResolvedOgImageOptions, root: string): Promise&lt;OgImageTemplateFn&gt;</code><span class="ox-api-entry__description">Resolves a Vue SFC template via SSR. Compiles the SFC with @vue/compiler-sfc (o…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">3 params</span><span class="ox-api-badge">returns Promise&lt;OgImageTemplateFn&gt;</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Resolves a Vue SFC template via SSR.</p>
<p>Compiles the SFC with @vue/compiler-sfc (or @vizejs/vite-plugin), bundles with rolldown, then wraps with createSSRApp + renderToString.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">async function resolveVueTemplate(templatePath: string, options: ResolvedOgImageOptions, root: string): Promise&lt;OgImageTemplateFn&gt;</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/og-image/index.ts#L189-L262" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">templatePath</code>
    <code class="ox-api-entry__param-type">string</code>
  </div>
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">options</code>
    <code class="ox-api-entry__param-type"><a href="./og-image-types.md#resolvedogimageoptions">ResolvedOgImageOptions</a></code>
  </div>
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">root</code>
    <code class="ox-api-entry__param-type">string</code>
  </div>
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">Promise&lt;<a href="./og-image-types.md#ogimagetemplatefn">OgImageTemplateFn</a>&gt;</code>
</div>
</div>
  </div>
</details>

<details id="tryserveallfromcache" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">tryServeAllFromCache(pages: OgImagePageEntry[], templateSource: string, options: ResolvedOgImageOptions, cacheDir: string): Promise&lt;OgImageResult[] | null&gt;</code><span class="ox-api-entry__description">Tries to serve all pages from cache. Returns results if ALL pages are cached, n…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">4 params</span><span class="ox-api-badge">returns Promise&lt;OgImageResult[] | null&gt;</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Tries to serve all pages from cache. Returns results if ALL pages are cached, null otherwise.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">async function tryServeAllFromCache(pages: OgImagePageEntry[], templateSource: string, options: ResolvedOgImageOptions, cacheDir: string): Promise&lt;OgImageResult[] | null&gt;</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/og-image/index.ts#L585-L611" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">pages</code>
    <code class="ox-api-entry__param-type"><a href="#ogimagepageentry">OgImagePageEntry</a>[]</code>
  </div>
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">templateSource</code>
    <code class="ox-api-entry__param-type">string</code>
  </div>
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">options</code>
    <code class="ox-api-entry__param-type"><a href="./og-image-types.md#resolvedogimageoptions">ResolvedOgImageOptions</a></code>
  </div>
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">cacheDir</code>
    <code class="ox-api-entry__param-type">string</code>
  </div>
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">Promise&lt;<a href="#ogimageresult">OgImageResult</a>[] | null&gt;</code>
</div>
</div>
  </div>
</details>

<details id="tstemplatebundleoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">tsTemplateBundleOptions(templatePath: string)</code><span class="ox-api-entry__description">Rolldown input options for a .ts template bundle. A .ts template is the framewo…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 param</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Rolldown input options for a <code>.ts</code> template bundle.</p>
<p>A <code>.ts</code> template is the framework-less kind, so it has no single runtime to externalize the way the <code>.vue</code>, <code>.svelte</code> and <code>.tsx</code> paths do — anything from <code>node_modules</code> is better resolved at import time than inlined. Nothing on this path has a compiler plugin, so nothing here needed bundling to be loadable in the first place.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export function tsTemplateBundleOptions(templatePath: string)</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/og-image/index.ts#L141-L147" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">templatePath</code>
    <code class="ox-api-entry__param-type">string</code>
  </div>
</li>
</ul>
</div>
  </div>
</details>
