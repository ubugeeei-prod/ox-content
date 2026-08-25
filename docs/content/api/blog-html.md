# blog-html.ts

**[Source](https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/blog-html.ts)**

> 3 documented symbols. Read the signatures first, then expand each item for parameters, return types, and examples.

<div class="ox-api-stats" aria-label="API reference summary">
<span class="ox-api-stat">
  <strong>3</strong>
  <span>symbols</span>
</span>
<span class="ox-api-stat">
  <strong>1</strong>
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
  <strong>1</strong>
  <span>parameters</span>
</span>
<span class="ox-api-stat">
  <strong>5</strong>
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

<details id="blog-html" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">module</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__name">blog-html</code><span class="ox-api-entry__description">Escaped blog index, tag, archive, and post-meta HTML.</span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Escaped blog index, tag, archive, and post-meta HTML.</p>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/blog-html.ts#L1-L3" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
  </div>
</details>

<details id="blogsourcepage" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">BlogSourcePage</code><span class="ox-api-entry__description">One listed post considered for blog surfaces.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">5 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>One listed post considered for blog surfaces.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface BlogSourcePage</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/blog-html.ts#L9-L15" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="blogsourcepage-frontmatter">
  <td><code>frontmatter</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">Record&lt;string, unknown&gt;</code></td>
  <td></td>
</tr>
<tr id="blogsourcepage-inputpath">
  <td><code>inputPath</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="blogsourcepage-routepaths">
  <td><code>routePaths</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">{ href: string }</code></td>
  <td></td>
</tr>
<tr id="blogsourcepage-title">
  <td><code>title</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="blogsourcepage-transformedhtml">
  <td><code>transformedHtml</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="issafeblogurl" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">isSafeBlogUrl(value: string): boolean</code><span class="ox-api-entry__description">https: or a same-origin path starting with / but not //.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 param</span><span class="ox-api-badge">returns boolean</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p><code>https:</code> or a same-origin path starting with <code>/</code> but not <code>//</code>.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export function isSafeBlogUrl(value: string): boolean</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/blog-html.ts#L30-L45" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
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
  <code class="ox-api-entry__return-type">boolean</code>
  
</div>
</div>
  </div>
</details>

