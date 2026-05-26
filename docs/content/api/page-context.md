# page-context.ts

**[Source](https://github.com/ubugeeei/ox-content/blob/main/npm/vite-plugin-ox-content/src/page-context.ts)**

> 14 documented symbols. Read the signatures first, then expand each item for parameters, return types, and examples.

<div class="ox-api-stats" aria-label="API reference summary">
<span class="ox-api-stat">
  <strong>14</strong>
  <span>symbols</span>
</span>
<span class="ox-api-stat">
  <strong>7</strong>
  <span>functions</span>
</span>
<span class="ox-api-stat">
  <strong>6</strong>
  <span>interfaces</span>
</span>
<span class="ox-api-stat">
  <strong>1</strong>
  <span>types</span>
</span>
<span class="ox-api-stat">
  <strong>4</strong>
  <span>parameters</span>
</span>
<span class="ox-api-stat">
  <strong>24</strong>
  <span>members</span>
</span>
<span class="ox-api-stat">
  <strong>7</strong>
  <span>returns</span>
</span>
<span class="ox-api-stat">
  <strong>5</strong>
  <span>examples</span>
</span>
</div>

## Reference

<div class="ox-api-controls" data-ox-api-target=".ox-api-entry" role="toolbar" aria-label="Reference display controls">
<button type="button" class="ox-api-controls__button" data-ox-api-toggle="expand">Open all</button>
<button type="button" class="ox-api-controls__button" data-ox-api-toggle="collapse">Close all</button>
</div>

<details id="basepageprops" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">BasePageProps</code><span class="ox-api-entry__description">Base page props available for all pages.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">9 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Base page props available for all pages.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface BasePageProps</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei/ox-content/blob/main/npm/vite-plugin-ox-content/src/page-context.ts#L34-L53" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Kind</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr>
  <td><code>title</code></td>
  <td><span class="ox-api-entry__member-kind">property</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Page title from frontmatter or first heading</div></td>
</tr>
<tr>
  <td><code>description</code><span class="ox-api-badge">optional</span></td>
  <td><span class="ox-api-entry__member-kind">property</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Page description from frontmatter</div></td>
</tr>
<tr>
  <td><code>html</code></td>
  <td><span class="ox-api-entry__member-kind">property</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Rendered HTML content</div></td>
</tr>
<tr>
  <td><code>toc</code></td>
  <td><span class="ox-api-entry__member-kind">property</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">TocEntry[]</code></td>
  <td><div class="ox-api-entry__member-description">Table of contents entries</div></td>
</tr>
<tr>
  <td><code>lastUpdated</code><span class="ox-api-badge">optional</span></td>
  <td><span class="ox-api-entry__member-kind">property</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">number</code></td>
  <td><div class="ox-api-entry__member-description">Last git commit timestamp in milliseconds</div></td>
</tr>
<tr>
  <td><code>path</code></td>
  <td><span class="ox-api-entry__member-kind">property</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Source file path (relative to docs root)</div></td>
</tr>
<tr>
  <td><code>url</code></td>
  <td><span class="ox-api-entry__member-kind">property</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Output URL path</div></td>
</tr>
<tr>
  <td><code>frontmatter</code></td>
  <td><span class="ox-api-entry__member-kind">property</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">Record</code></td>
  <td><div class="ox-api-entry__member-description">Raw frontmatter object</div></td>
</tr>
<tr>
  <td><code>layout</code><span class="ox-api-badge">optional</span></td>
  <td><span class="ox-api-entry__member-kind">property</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Layout name from frontmatter</div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="frontmatterschema" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">FrontmatterSchema</code><span class="ox-api-entry__description">Schema for frontmatter type generation.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">4 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Schema for frontmatter type generation.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface FrontmatterSchema</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei/ox-content/blob/main/npm/vite-plugin-ox-content/src/page-context.ts#L256-L265" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Kind</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr>
  <td><code>name</code></td>
  <td><span class="ox-api-entry__member-kind">property</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Field name</div></td>
</tr>
<tr>
  <td><code>type</code></td>
  <td><span class="ox-api-entry__member-kind">property</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">TypeScript type</div></td>
</tr>
<tr>
  <td><code>optional</code></td>
  <td><span class="ox-api-entry__member-kind">property</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Whether the field is optional</div></td>
</tr>
<tr>
  <td><code>description</code><span class="ox-api-badge">optional</span></td>
  <td><span class="ox-api-entry__member-kind">property</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">JSDoc description</div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="generatefrontmattertypes" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">generateFrontmatterTypes(samples: Record&lt;string, unknown&gt;[], interfaceName = &quot;PageFrontmatter&quot;): string</code><span class="ox-api-entry__description">Generates TypeScript interface from frontmatter samples.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">2 params</span><span class="ox-api-badge">returns string</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Generates TypeScript interface from frontmatter samples.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export function generateFrontmatterTypes(samples: Record&lt;string, unknown&gt;[], interfaceName = &quot;PageFrontmatter&quot;): string</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei/ox-content/blob/main/npm/vite-plugin-ox-content/src/page-context.ts#L294-L335" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">samples</code>
    <code class="ox-api-entry__param-type">Record[]</code>
  </div>
  
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">interfaceName</code>
    <code class="ox-api-entry__param-type">unknown</code>
  </div>
  <p class="ox-api-entry__param-description">optional · default: &quot;PageFrontmatter&quot;</p>
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

<details id="infertype" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">inferType(value: unknown): string</code><span class="ox-api-entry__description">Infers TypeScript types from frontmatter values.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 param</span><span class="ox-api-badge">returns string</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Infers TypeScript types from frontmatter values.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export function inferType(value: unknown): string</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei/ox-content/blob/main/npm/vite-plugin-ox-content/src/page-context.ts#L270-L289" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">value</code>
    <code class="ox-api-entry__param-type">unknown</code>
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

<details id="navgroup" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">NavGroup</code><span class="ox-api-entry__description">Navigation group.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">2 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Navigation group.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface NavGroup</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei/ox-content/blob/main/npm/vite-plugin-ox-content/src/page-context.ts#L81-L84" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Kind</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr>
  <td><code>title</code></td>
  <td><span class="ox-api-entry__member-kind">property</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr>
  <td><code>items</code></td>
  <td><span class="ox-api-entry__member-kind">property</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">NavItem[]</code></td>
  <td></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="navitem" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">NavItem</code><span class="ox-api-entry__description">Navigation item.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">3 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Navigation item.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface NavItem</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei/ox-content/blob/main/npm/vite-plugin-ox-content/src/page-context.ts#L89-L93" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Kind</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr>
  <td><code>title</code></td>
  <td><span class="ox-api-entry__member-kind">property</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr>
  <td><code>path</code></td>
  <td><span class="ox-api-entry__member-kind">property</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr>
  <td><code>href</code></td>
  <td><span class="ox-api-entry__member-kind">property</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="pageprops" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">type</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">PageProps&lt;T extends Record&lt;string, unknown&gt; = Record&lt;string, unknown&gt;&gt; = BasePageProps &amp; { /** Custom frontmatter fields */ frontmatter: T &amp; Record&lt;string, unknown&gt;; }</code><span class="ox-api-entry__description">Extended page props with custom frontmatter.</span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Extended page props with custom frontmatter.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export type PageProps&lt;T extends Record&lt;string, unknown&gt; = Record&lt;string, unknown&gt;&gt; = BasePageProps &amp; {
    /** Custom frontmatter fields */
    frontmatter: T &amp; Record&lt;string, unknown&gt;;
  }</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei/ox-content/blob/main/npm/vite-plugin-ox-content/src/page-context.ts#L58-L62" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
  </div>
</details>

<details id="rendercontext" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">RenderContext&lt;T extends Record&lt;string, unknown&gt; = Record&lt;string, unknown&gt;&gt;</code><span class="ox-api-entry__description">Complete render context.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">2 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Complete render context.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface RenderContext&lt;T extends Record&lt;string, unknown&gt; = Record&lt;string, unknown&gt;&gt;</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei/ox-content/blob/main/npm/vite-plugin-ox-content/src/page-context.ts#L98-L103" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Kind</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr>
  <td><code>page</code></td>
  <td><span class="ox-api-entry__member-kind">property</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">PageProps</code></td>
  <td><div class="ox-api-entry__member-description">Current page props</div></td>
</tr>
<tr>
  <td><code>site</code></td>
  <td><span class="ox-api-entry__member-kind">property</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">SiteConfig</code></td>
  <td><div class="ox-api-entry__member-description">Site configuration</div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="siteconfig" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">SiteConfig</code><span class="ox-api-entry__description">Site-wide configuration available in context.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">4 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Site-wide configuration available in context.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface SiteConfig</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei/ox-content/blob/main/npm/vite-plugin-ox-content/src/page-context.ts#L67-L76" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Kind</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr>
  <td><code>name</code></td>
  <td><span class="ox-api-entry__member-kind">property</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Site name</div></td>
</tr>
<tr>
  <td><code>base</code></td>
  <td><span class="ox-api-entry__member-kind">property</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Base URL path</div></td>
</tr>
<tr>
  <td><code>pages</code></td>
  <td><span class="ox-api-entry__member-kind">property</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">BasePageProps[]</code></td>
  <td><div class="ox-api-entry__member-description">All pages in the site</div></td>
</tr>
<tr>
  <td><code>nav</code></td>
  <td><span class="ox-api-entry__member-kind">property</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">NavGroup[]</code></td>
  <td><div class="ox-api-entry__member-description">Navigation groups</div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="useisactive" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">useIsActive(path: string): boolean</code><span class="ox-api-entry__description">Checks if the given path is the current page.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 param</span><span class="ox-api-badge">returns boolean</span><span class="ox-api-badge">1 example</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Checks if the given path is the current page.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export function useIsActive(path: string): boolean</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei/ox-content/blob/main/npm/vite-plugin-ox-content/src/page-context.ts#L246-L249" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">path</code>
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
<div class="ox-api-entry__section ox-api-entry__section--examples">
<h4>Examples</h4>
<div class="ox-api-entry__example">
<div class="ox-api-entry__example-heading">Example 1</div>
<pre><code class="language-tsx">function NavLink({ href, children }) {
  const isActive = useIsActive(href);
  return &lt;a href={href} class={isActive ? &#39;active&#39; : &#39;&#39;}&gt;{children}&lt;/a&gt;;
}</code></pre>
</div>
</div>
  </div>
</details>

<details id="usenav" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">useNav(): NavGroup[]</code><span class="ox-api-entry__description">Gets the navigation groups.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">returns NavGroup[]</span><span class="ox-api-badge">1 example</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Gets the navigation groups.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export function useNav(): NavGroup[]</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei/ox-content/blob/main/npm/vite-plugin-ox-content/src/page-context.ts#L231-L233" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">NavGroup[]</code>
  
</div>
</div>
<div class="ox-api-entry__section ox-api-entry__section--examples">
<h4>Examples</h4>
<div class="ox-api-entry__example">
<div class="ox-api-entry__example-heading">Example 1</div>
<pre><code class="language-tsx">function Sidebar() {
  const nav = useNav();
  return (
    &lt;nav&gt;
      {each(nav, (group) =&gt; (
        &lt;div&gt;
          &lt;h3&gt;{group.title}&lt;/h3&gt;
          &lt;ul&gt;
            {each(group.items, (item) =&gt; (
              &lt;li&gt;&lt;a href={item.href}&gt;{item.title}&lt;/a&gt;&lt;/li&gt;
            ))}
          &lt;/ul&gt;
        &lt;/div&gt;
      ))}
    &lt;/nav&gt;
  );
}</code></pre>
</div>
</div>
  </div>
</details>

<details id="usepageprops" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">usePageProps&lt; T extends Record&lt;string, unknown&gt; = Record&lt;string, unknown&gt;, &gt;(): PageProps&lt;T&gt;</code><span class="ox-api-entry__description">Gets the current page props.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">returns PageProps</span><span class="ox-api-badge">1 example</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Gets the current page props.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export function usePageProps&lt;
  T extends Record&lt;string, unknown&gt; = Record&lt;string, unknown&gt;,
&gt;(): PageProps&lt;T&gt;</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei/ox-content/blob/main/npm/vite-plugin-ox-content/src/page-context.ts#L140-L150" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">PageProps</code>
  <p class="ox-api-entry__return-description">The current page props</p>
</div>
</div>
<div class="ox-api-entry__section ox-api-entry__section--examples">
<h4>Examples</h4>
<div class="ox-api-entry__example">
<div class="ox-api-entry__example-heading">Example 1</div>
<pre><code class="language-tsx">function PageTitle() {
  const page = usePageProps();
  return &lt;h1&gt;{page.title}&lt;/h1&gt;;
}</code></pre>
</div>
</div>
<div class="ox-api-entry__section ox-api-entry__section--tags">
<h4>Tags</h4>
<ul class="ox-api-entry__tags"><li><span class="ox-api-entry__tag-name">@throws</span><span class="ox-api-entry__tag-value">Error if called outside of a render context</span></li></ul>
</div>
  </div>
</details>

<details id="userendercontext" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">useRenderContext&lt; T extends Record&lt;string, unknown&gt; = Record&lt;string, unknown&gt;, &gt;(): RenderContext&lt;T&gt;</code><span class="ox-api-entry__description">Gets the full render context.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">returns RenderContext</span><span class="ox-api-badge">1 example</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Gets the full render context.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export function useRenderContext&lt;
  T extends Record&lt;string, unknown&gt; = Record&lt;string, unknown&gt;,
&gt;(): RenderContext&lt;T&gt;</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei/ox-content/blob/main/npm/vite-plugin-ox-content/src/page-context.ts#L195-L205" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">RenderContext</code>
  <p class="ox-api-entry__return-description">The complete render context</p>
</div>
</div>
<div class="ox-api-entry__section ox-api-entry__section--examples">
<h4>Examples</h4>
<div class="ox-api-entry__example">
<div class="ox-api-entry__example-heading">Example 1</div>
<pre><code class="language-tsx">function Layout({ children }) {
  const ctx = useRenderContext();
  return (
    &lt;html&gt;
      &lt;head&gt;&lt;title&gt;{ctx.page.title} - {ctx.site.name}&lt;/title&gt;&lt;/head&gt;
      &lt;body&gt;{children}&lt;/body&gt;
    &lt;/html&gt;
  );
}</code></pre>
</div>
</div>
<div class="ox-api-entry__section ox-api-entry__section--tags">
<h4>Tags</h4>
<ul class="ox-api-entry__tags"><li><span class="ox-api-entry__tag-name">@throws</span><span class="ox-api-entry__tag-value">Error if called outside of a render context</span></li></ul>
</div>
  </div>
</details>

<details id="usesiteconfig" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">useSiteConfig(): SiteConfig</code><span class="ox-api-entry__description">Gets the site configuration.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">returns SiteConfig</span><span class="ox-api-badge">1 example</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Gets the site configuration.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export function useSiteConfig(): SiteConfig</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei/ox-content/blob/main/npm/vite-plugin-ox-content/src/page-context.ts#L166-L174" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">SiteConfig</code>
  <p class="ox-api-entry__return-description">The site configuration</p>
</div>
</div>
<div class="ox-api-entry__section ox-api-entry__section--examples">
<h4>Examples</h4>
<div class="ox-api-entry__example">
<div class="ox-api-entry__example-heading">Example 1</div>
<pre><code class="language-tsx">function SiteHeader() {
  const site = useSiteConfig();
  return &lt;header&gt;{site.name}&lt;/header&gt;;
}</code></pre>
</div>
</div>
<div class="ox-api-entry__section ox-api-entry__section--tags">
<h4>Tags</h4>
<ul class="ox-api-entry__tags"><li><span class="ox-api-entry__tag-name">@throws</span><span class="ox-api-entry__tag-value">Error if called outside of a render context</span></li></ul>
</div>
  </div>
</details>
