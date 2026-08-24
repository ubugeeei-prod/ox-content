# permalinks.ts

**[Source](https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/permalinks.ts)**

> 8 documented symbols. Read the signatures first, then expand each item for parameters, return types, and examples.

<div class="ox-api-stats" aria-label="API reference summary">
<span class="ox-api-stat">
  <strong>8</strong>
  <span>symbols</span>
</span>
<span class="ox-api-stat">
  <strong>4</strong>
  <span>functions</span>
</span>
<span class="ox-api-stat">
  <strong>3</strong>
  <span>interfaces</span>
</span>
<span class="ox-api-stat">
  <strong>1</strong>
  <span>modules</span>
</span>
<span class="ox-api-stat">
  <strong>7</strong>
  <span>parameters</span>
</span>
<span class="ox-api-stat">
  <strong>8</strong>
  <span>members</span>
</span>
<span class="ox-api-stat">
  <strong>4</strong>
  <span>returns</span>
</span>
</div>

## Reference

<div class="ox-api-controls" data-ox-api-target=".ox-api-entry" role="toolbar" aria-label="Reference display controls">
<button type="button" class="ox-api-controls__button" data-ox-api-toggle="expand">Open all</button>
<button type="button" class="ox-api-controls__button" data-ox-api-toggle="collapse">Close all</button>
</div>

<details id="escapeattribute" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">escapeAttribute(value: string): string</code><span class="ox-api-entry__description">Escapes a value for use in an HTML attribute.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 param</span><span class="ox-api-badge">returns string</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Escapes a value for use in an HTML attribute.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export function escapeAttribute(value: string): string</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/permalinks.ts#L96-L111" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">value</code>
    <code class="ox-api-entry__param-type">string</code>
  </div>
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">string</code>
</div>
</div>
  </div>
</details>

<details id="permalinks" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">module</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__name">permalinks</code><span class="ox-api-entry__description">Opt-in permalink / slug routing and <em>index frontmatter cascade. Resolution follows ox</em>content<em>ssg::resolve</em>page_routes.…</span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Opt-in permalink / slug routing and <code>_index</code> frontmatter cascade.</p>
<p>Resolution follows <code>ox_content_ssg::resolve_page_routes</code>. The Vite plugin applies those URLs during SSG and collection manifest builds.</p>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/permalinks.ts#L1-L6" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
  </div>
</details>

<details id="resolvecascadeoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">resolveCascadeOptions(value: boolean | CascadeOptions | undefined): ResolvedCascadeOptions</code><span class="ox-api-entry__description">Resolves cascade. false / omitted stays off. true / {} enables.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 param</span><span class="ox-api-badge">returns ResolvedCascadeOptions</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Resolves <code>cascade</code>. <code>false</code> / omitted stays off. <code>true</code> / <code>{}</code> enables.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export function resolveCascadeOptions(value: boolean | CascadeOptions | undefined): ResolvedCascadeOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/permalinks.ts#L45-L49" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">value</code>
    <code class="ox-api-entry__param-type">boolean | <a href="./types.md#cascadeoptions">CascadeOptions</a> | undefined</code>
  </div>
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type"><a href="./types.md#resolvedcascadeoptions">ResolvedCascadeOptions</a></code>
</div>
</div>
  </div>
</details>

<details id="resolvedroutepage" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">ResolvedRoutePage</code><span class="ox-api-entry__description">A page after cascade and optional permalink / slug rewriting.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">3 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>A page after cascade and optional permalink / slug rewriting.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface ResolvedRoutePage</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/permalinks.ts#L25-L29" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="resolvedroutepage-frontmatter">
  <td><code>frontmatter</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">Record&lt;string, unknown&gt;</code></td>
  <td></td>
</tr>
<tr id="resolvedroutepage-source">
  <td><code>source</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="resolvedroutepage-urlpath">
  <td><code>urlPath</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="resolvepageroutes" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">resolvePageRoutes(input: { pages: readonly RoutePageInput[]; permalinks?: ResolvedPermalinksOptions | null; cascade?: ResolvedCascadeOptions | null; }): RouteResolveOutput</code><span class="ox-api-entry__description">Applies cascade (when on) then permalink / slug rewriting (when on). Collisions…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">4 params</span><span class="ox-api-badge">returns RouteResolveOutput</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Applies cascade (when on) then permalink / slug rewriting (when on).</p>
<p>Collisions skip the later page and keep the first. Rejected permalinks stay on the file-tree URL. Hostile non-string values are ignored.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export function resolvePageRoutes(input: {
  pages: readonly RoutePageInput[];
  permalinks?: ResolvedPermalinksOptions | null;
  cascade?: ResolvedCascadeOptions | null;
}): RouteResolveOutput</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/permalinks.ts#L57-L93" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">input</code>
    <code class="ox-api-entry__param-type">{ pages: readonly <a href="#routepageinput">RoutePageInput</a>[]; <a href="#permalinks">permalinks</a>?: <a href="./types.md#resolvedpermalinksoptions">ResolvedPermalinksOptions</a> | null; cascade?: <a href="./types.md#resolvedcascadeoptions">ResolvedCascadeOptions</a> | null }</code>
  </div>
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">input.pages</code>
    <code class="ox-api-entry__param-type">readonly <a href="#routepageinput">RoutePageInput</a>[]</code>
  </div>
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">input.permalinks?</code>
    <code class="ox-api-entry__param-type"><a href="./types.md#resolvedpermalinksoptions">ResolvedPermalinksOptions</a> | null</code>
  </div>
  <p class="ox-api-entry__param-description">optional</p>
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">input.cascade?</code>
    <code class="ox-api-entry__param-type"><a href="./types.md#resolvedcascadeoptions">ResolvedCascadeOptions</a> | null</code>
  </div>
  <p class="ox-api-entry__param-description">optional</p>
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type"><a href="#routeresolveoutput">RouteResolveOutput</a></code>
</div>
</div>
  </div>
</details>

<details id="resolvepermalinksoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">resolvePermalinksOptions(value: boolean | PermalinksOptions | undefined): ResolvedPermalinksOptions</code><span class="ox-api-entry__description">Resolves permalinks. false / omitted stays off. true / {} enables.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 param</span><span class="ox-api-badge">returns ResolvedPermalinksOptions</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Resolves <code>permalinks</code>. <code>false</code> / omitted stays off. <code>true</code> / <code>{}</code> enables.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export function resolvePermalinksOptions(value: boolean | PermalinksOptions | undefined): ResolvedPermalinksOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/permalinks.ts#L38-L42" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">value</code>
    <code class="ox-api-entry__param-type">boolean | <a href="./types.md#permalinksoptions">PermalinksOptions</a> | undefined</code>
  </div>
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type"><a href="./types.md#resolvedpermalinksoptions">ResolvedPermalinksOptions</a></code>
</div>
</div>
  </div>
</details>

<details id="routepageinput" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">RoutePageInput</code><span class="ox-api-entry__description">One page considered for cascade and permalink resolution.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">3 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>One page considered for cascade and permalink resolution.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface RoutePageInput</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/permalinks.ts#L18-L22" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="routepageinput-fileurl">
  <td><code>fileUrl</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="routepageinput-frontmatter">
  <td><code>frontmatter</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">Record&lt;string, unknown&gt;</code></td>
  <td></td>
</tr>
<tr id="routepageinput-source">
  <td><code>source</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="routeresolveoutput" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">RouteResolveOutput</code><span class="ox-api-entry__description">Resolved pages plus collision / rejection errors.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">2 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Resolved pages plus collision / rejection errors.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface RouteResolveOutput</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/permalinks.ts#L32-L35" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="routeresolveoutput-errors">
  <td><code>errors</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string[]</code></td>
  <td></td>
</tr>
<tr id="routeresolveoutput-pages">
  <td><code>pages</code></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#resolvedroutepage">ResolvedRoutePage</a>[]</code></td>
  <td></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>
