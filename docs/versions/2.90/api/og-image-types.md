# types.ts

**[Source](https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/og-image/types.ts)**

> 5 documented symbols. Read the signatures first, then expand each item for parameters, return types, and examples.

<div class="ox-api-stats" aria-label="API reference summary">
<span class="ox-api-stat">
  <strong>5</strong>
  <span>symbols</span>
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
  <strong>1</strong>
  <span>parameters</span>
</span>
<span class="ox-api-stat">
  <strong>18</strong>
  <span>members</span>
</span>
<span class="ox-api-stat">
  <strong>1</strong>
  <span>returns</span>
</span>
</div>

## Reference

<div class="ox-api-controls" data-ox-api-target=".ox-api-entry" role="toolbar" aria-label="Reference display controls">
<button type="button" class="ox-api-controls__button" data-ox-api-toggle="expand">Open all</button>
<button type="button" class="ox-api-controls__button" data-ox-api-toggle="collapse">Close all</button>
</div>

<details id="ogimageoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">OgImageOptions</code><span class="ox-api-entry__description">OG image generation options (user-facing).</span><span class="ox-api-entry__meta"><span class="ox-api-badge">6 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>OG image generation options (user-facing).</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface OgImageOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/og-image/types.ts#L31-L74" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="ogimageoptions-cache">
  <td><code>cache</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td><div class="ox-api-entry__member-description">Enable content-hash based caching.<br>Skips rendering when content hasn&#39;t changed.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">true</code></div></td>
</tr>
<tr id="ogimageoptions-concurrency">
  <td><code>concurrency</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">number</code></td>
  <td><div class="ox-api-entry__member-description">Number of concurrent page instances for parallel rendering.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">1</code></div></td>
</tr>
<tr id="ogimageoptions-height">
  <td><code>height</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">number</code></td>
  <td><div class="ox-api-entry__member-description">Image height in pixels.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">630</code></div></td>
</tr>
<tr id="ogimageoptions-template">
  <td><code>template</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Path to a custom template file (.ts, .vue, .svelte, .tsx/.jsx).<br>- <code>.ts</code>: default-export a function <code>(props) =&gt; string</code><br>- <code>.vue</code>: Vue SFC, rendered via SSR<br>- <code>.svelte</code>: Svelte SFC, rendered via SSR<br>- <code>.tsx</code>/<code>.jsx</code>: React Server Component, rendered via SSR<br>If not specified, the built-in default template is used.</div></td>
</tr>
<tr id="ogimageoptions-vueplugin">
  <td><code>vuePlugin</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">&quot;vitejs&quot; | &quot;vizejs&quot;</code></td>
  <td><div class="ox-api-entry__member-description">Vue plugin to use for compiling <code>.vue</code> templates.<br>- <code>&#39;vitejs&#39;</code>: Use <code>@vue/compiler-sfc</code> (official, default)<br>- <code>&#39;vizejs&#39;</code>: Use <code>@vizejs/vite-plugin</code> (Rust-based)</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">&#39;vitejs&#39;</code></div></td>
</tr>
<tr id="ogimageoptions-width">
  <td><code>width</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">number</code></td>
  <td><div class="ox-api-entry__member-description">Image width in pixels.</div><div class="ox-api-entry__member-default"><span>Default</span> <code class="language-typescript">1200</code></div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="ogimagetemplatefn" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">type</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">OgImageTemplateFn = (props: OgImageTemplateProps) =&gt; string | Promise&lt;string&gt;</code><span class="ox-api-entry__description">Template function that receives page metadata and returns an HTML string.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 param</span><span class="ox-api-badge">returns string | Promise&lt;string&gt;</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Template function that receives page metadata and returns an HTML string.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export type OgImageTemplateFn = (props: OgImageTemplateProps) =&gt; string | Promise&lt;string&gt;</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/og-image/types.ts#L26" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">props</code>
    <code class="ox-api-entry__param-type"><a href="#ogimagetemplateprops">OgImageTemplateProps</a></code>
  </div>
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">string | Promise&lt;string&gt;</code>
</div>
</div>
  </div>
</details>

<details id="ogimagetemplateprops" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">OgImageTemplateProps</code><span class="ox-api-entry__description">Props passed to OG image template functions.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">6 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Props passed to OG image template functions.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface OgImageTemplateProps</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/og-image/types.ts#L8-L21" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group ox-api-entry__member-group--indexable">
<h5>Indexable</h5>
<div class="ox-api-entry__member-details">
<section class="ox-api-entry__member-detail ox-api-entry__member-detail--indexable">
<pre><code class="language-ts">[key: string]: unknown</code></pre><div class="ox-api-entry__member-description">Custom data from frontmatter (arbitrary key-value pairs)</div></section>
</div>
</div>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="ogimagetemplateprops-author">
  <td><code>author</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Author name</div></td>
</tr>
<tr id="ogimagetemplateprops-description">
  <td><code>description</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Page description</div></td>
</tr>
<tr id="ogimagetemplateprops-sitename">
  <td><code>siteName</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Site name</div></td>
</tr>
<tr id="ogimagetemplateprops-tags">
  <td><code>tags</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string[]</code></td>
  <td><div class="ox-api-entry__member-description">Tags/categories</div></td>
</tr>
<tr id="ogimagetemplateprops-title">
  <td><code>title</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td><div class="ox-api-entry__member-description">Page title</div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="resolvedogimageoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">ResolvedOgImageOptions</code><span class="ox-api-entry__description">Resolved OG image options with all defaults applied.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">6 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Resolved OG image options with all defaults applied.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface ResolvedOgImageOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/og-image/types.ts#L79-L86" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="resolvedogimageoptions-cache">
  <td><code>cache</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">boolean</code></td>
  <td></td>
</tr>
<tr id="resolvedogimageoptions-concurrency">
  <td><code>concurrency</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">number</code></td>
  <td></td>
</tr>
<tr id="resolvedogimageoptions-height">
  <td><code>height</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">number</code></td>
  <td></td>
</tr>
<tr id="resolvedogimageoptions-template">
  <td><code>template</code><span class="ox-api-badge">optional</span></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="resolvedogimageoptions-vueplugin">
  <td><code>vuePlugin</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">&quot;vitejs&quot; | &quot;vizejs&quot;</code></td>
  <td></td>
</tr>
<tr id="resolvedogimageoptions-width">
  <td><code>width</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">number</code></td>
  <td></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="types" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">module</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__name">types</code><span class="ox-api-entry__description">Type definitions for Chromium-based OG image generation.</span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Type definitions for Chromium-based OG image generation.</p>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/og-image/types.ts#L1-L3" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
  </div>
</details>
