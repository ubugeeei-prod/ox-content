# apply-permalinks.ts

**[Source](https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/apply-permalinks.ts)**

> 5 documented symbols. Read the signatures first, then expand each item for parameters, return types, and examples.

<div class="ox-api-stats" aria-label="API reference summary">
<span class="ox-api-stat">
  <strong>5</strong>
  <span>symbols</span>
</span>
<span class="ox-api-stat">
  <strong>3</strong>
  <span>functions</span>
</span>
<span class="ox-api-stat">
  <strong>1</strong>
  <span>interfaces</span>
</span>
<span class="ox-api-stat">
  <strong>1</strong>
  <span>modules</span>
</span>
<span class="ox-api-stat">
  <strong>15</strong>
  <span>parameters</span>
</span>
<span class="ox-api-stat">
  <strong>3</strong>
  <span>members</span>
</span>
<span class="ox-api-stat">
  <strong>3</strong>
  <span>returns</span>
</span>
</div>

## Reference

<div class="ox-api-controls" data-ox-api-target=".ox-api-entry" role="toolbar" aria-label="Reference display controls">
<button type="button" class="ox-api-controls__button" data-ox-api-toggle="expand">Open all</button>
<button type="button" class="ox-api-controls__button" data-ox-api-toggle="collapse">Close all</button>
</div>

<details id="apply-permalinks" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">module</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__name">apply-permalinks</code><span class="ox-api-entry__description">Applies resolved permalinks / cascade to SSG pages and collection entries.</span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Applies resolved permalinks / cascade to SSG pages and collection entries.</p>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/apply-permalinks.ts#L1-L3" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
  </div>
</details>

<details id="applycollectionroutes" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">applyCollectionRoutes(manifest: CollectionManifest, permalinks?: ResolvedPermalinksOptions | null, cascade?: ResolvedCascadeOptions | null): { manifest: CollectionManifest; errors: string[] }</code><span class="ox-api-entry__description">Rewrites collection path / stem / inherited frontmatter.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">3 params</span><span class="ox-api-badge">returns object</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Rewrites collection <code>path</code> / <code>stem</code> / inherited frontmatter.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export function applyCollectionRoutes(manifest: CollectionManifest, permalinks?: ResolvedPermalinksOptions | null, cascade?: ResolvedCascadeOptions | null): { manifest: CollectionManifest; errors: string[] }</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/apply-permalinks.ts#L87-L128" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">manifest</code>
    <code class="ox-api-entry__param-type"><a href="./types.md#collectionmanifest">CollectionManifest</a></code>
  </div>
  
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">permalinks</code>
    <code class="ox-api-entry__param-type"><a href="./types.md#resolvedpermalinksoptions">ResolvedPermalinksOptions</a> | null</code>
  </div>
  <p class="ox-api-entry__param-description">optional</p>
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">cascade</code>
    <code class="ox-api-entry__param-type"><a href="./types.md#resolvedcascadeoptions">ResolvedCascadeOptions</a> | null</code>
  </div>
  <p class="ox-api-entry__param-description">optional</p>
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">object</code>
  <div class="ox-api-entry__return-members">
<div class="ox-api-entry__return-member">
<h5>errors</h5>
<code class="ox-api-entry__return-member-type language-typescript">errors: string[];</code>
</div>
<div class="ox-api-entry__return-member">
<h5>manifest</h5>
<code class="ox-api-entry__return-member-type language-typescript">manifest: <a href="./types.md#collectionmanifest">CollectionManifest</a>;</code>
</div>
</div>
</div>
</div>
  </div>
</details>

<details id="applyssgpageroutes" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">applySsgPageRoutes(input: { pages: readonly SsgRoutablePage[]; permalinks?: ResolvedPermalinksOptions | null; cascade?: ResolvedCascadeOptions | null; srcDir: string; outDir: string; base: string; extension: string; siteUrl?: string; }): { pages: SsgRoutablePage[]; errors: string[] }</code><span class="ox-api-entry__description">Rewrites SSG routePaths from resolved permalinks / slugs.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">9 params</span><span class="ox-api-badge">returns object</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Rewrites SSG <code>routePaths</code> from resolved permalinks / slugs.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export function applySsgPageRoutes(input: {
  pages: readonly SsgRoutablePage[];
  permalinks?: ResolvedPermalinksOptions | null;
  cascade?: ResolvedCascadeOptions | null;
  srcDir: string;
  outDir: string;
  base: string;
  extension: string;
  siteUrl?: string;
}): { pages: SsgRoutablePage[]; errors: string[] }</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/apply-permalinks.ts#L44-L84" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">input</code>
    <code class="ox-api-entry__param-type">{ pages: readonly <a href="#ssgroutablepage">SsgRoutablePage</a>[]; <a href="./permalinks.md#permalinks">permalinks</a>?: <a href="./types.md#resolvedpermalinksoptions">ResolvedPermalinksOptions</a> | null; cascade?: <a href="./types.md#resolvedcascadeoptions">ResolvedCascadeOptions</a> | null; srcDir: string; outDir: string; base: string; extension: string; siteUrl?: string }</code>
  </div>
  
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">input.pages</code>
    <code class="ox-api-entry__param-type">readonly <a href="#ssgroutablepage">SsgRoutablePage</a>[]</code>
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
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">input.srcDir</code>
    <code class="ox-api-entry__param-type">string</code>
  </div>
  
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">input.outDir</code>
    <code class="ox-api-entry__param-type">string</code>
  </div>
  
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">input.base</code>
    <code class="ox-api-entry__param-type">string</code>
  </div>
  
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">input.extension</code>
    <code class="ox-api-entry__param-type">string</code>
  </div>
  
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">input.siteUrl?</code>
    <code class="ox-api-entry__param-type">string</code>
  </div>
  <p class="ox-api-entry__param-description">optional</p>
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">object</code>
  <div class="ox-api-entry__return-members">
<div class="ox-api-entry__return-member">
<h5>errors</h5>
<code class="ox-api-entry__return-member-type language-typescript">errors: string[];</code>
</div>
<div class="ox-api-entry__return-member">
<h5>pages</h5>
<code class="ox-api-entry__return-member-type language-typescript">pages: <a href="#ssgroutablepage">SsgRoutablePage</a>[];</code>
</div>
</div>
</div>
</div>
  </div>
</details>

<details id="remapnavgroups" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">remapNavGroups&lt;T extends NavGroup&gt;(nav: T[], kept: readonly { fileUrl: string; urlPath: string; href: string }[], skippedFileUrls: readonly string[]): T[]</code><span class="ox-api-entry__description">Updates auto-nav hrefs after permalinks change a page URL.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">3 params</span><span class="ox-api-badge">returns T[]</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Updates auto-nav hrefs after permalinks change a page URL.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export function remapNavGroups&lt;T extends NavGroup&gt;(nav: T[], kept: readonly { fileUrl: string; urlPath: string; href: string }[], skippedFileUrls: readonly string[]): T[]</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/apply-permalinks.ts#L131-L141" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">nav</code>
    <code class="ox-api-entry__param-type">T[]</code>
  </div>
  
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">kept</code>
    <code class="ox-api-entry__param-type">readonly { fileUrl: string; urlPath: string; href: string }[]</code>
  </div>
  
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">skippedFileUrls</code>
    <code class="ox-api-entry__param-type">readonly string[]</code>
  </div>
  
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">T[]</code>
  
</div>
</div>
  </div>
</details>

<details id="ssgroutablepage" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">SsgRoutablePage</code><span class="ox-api-entry__description">SSG page shape that can have its routePaths rewritten.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">3 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>SSG page shape that can have its <code>routePaths</code> rewritten.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface SsgRoutablePage</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/apply-permalinks.ts#L15-L25" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="ssgroutablepage-frontmatter">
  <td><code>frontmatter</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">Record&lt;string, unknown&gt;</code></td>
  <td></td>
</tr>
<tr id="ssgroutablepage-inputpath">
  <td><code>inputPath</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="ssgroutablepage-routepaths">
  <td><code>routePaths</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">{ outputPath: string; urlPath: string; href: string; ogImagePath: string; ogImageUrl: string }</code></td>
  <td></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

