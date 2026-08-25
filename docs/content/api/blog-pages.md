# blog-pages.ts

**[Source](https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/blog-pages.ts)**

> 4 documented symbols. Read the signatures first, then expand each item for parameters, return types, and examples.

<div class="ox-api-stats" aria-label="API reference summary">
<span class="ox-api-stat">
  <strong>4</strong>
  <span>symbols</span>
</span>
<span class="ox-api-stat">
  <strong>2</strong>
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
  <strong>11</strong>
  <span>parameters</span>
</span>
<span class="ox-api-stat">
  <strong>5</strong>
  <span>members</span>
</span>
<span class="ox-api-stat">
  <strong>2</strong>
  <span>returns</span>
</span>
</div>

## Reference

<div class="ox-api-controls" data-ox-api-target=".ox-api-entry" role="toolbar" aria-label="Reference display controls">
<button type="button" class="ox-api-controls__button" data-ox-api-toggle="expand">Open all</button>
<button type="button" class="ox-api-controls__button" data-ox-api-toggle="collapse">Close all</button>
</div>

<details id="appendblogpages" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">appendBlogPages(input: { generatedPages: Array&lt;{ inputPath: string; outputPath: string; html: string }&gt;; listedPages: readonly BlogSourcePage[]; options?: ResolvedBlogOptions; collections?: ResolvedCollectionsOptions; srcDir: string; outDir: string; base: string; render: (page: BlogGeneratedPage) =&gt; Promise&lt;string&gt;; errors: string[]; }): Promise&lt;void&gt;</code><span class="ox-api-entry__description">Renders index, tag, and archive pages and appends them to the build.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">10 params</span><span class="ox-api-badge">returns Promise&lt;void&gt;</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Renders index, tag, and archive pages and appends them to the build.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export async function appendBlogPages(input: {
  generatedPages: Array&lt;{ inputPath: string; outputPath: string; html: string }&gt;;
  listedPages: readonly BlogSourcePage[];
  options?: ResolvedBlogOptions;
  collections?: ResolvedCollectionsOptions;
  srcDir: string;
  outDir: string;
  base: string;
  render: (page: BlogGeneratedPage) =&gt; Promise&lt;string&gt;;
  errors: string[];
}): Promise&lt;void&gt;</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/blog-pages.ts#L106-L140" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">input</code>
    <code class="ox-api-entry__param-type">{ generatedPages: Array&lt;{ inputPath: string; outputPath: string; html: string }&gt;; listedPages: readonly <a href="./blog-html.md#blogsourcepage">BlogSourcePage</a>[]; options?: <a href="./types.md#resolvedblogoptions">ResolvedBlogOptions</a>; collections?: <a href="./types.md#resolvedcollectionsoptions">ResolvedCollectionsOptions</a>; srcDir: string; outDir: string; base: string; render: (page: <a href="#bloggeneratedpage">BlogGeneratedPage</a>) =&gt; Promise&lt;string&gt;; errors: string[] }</code>
  </div>
  
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">input.generatedPages</code>
    <code class="ox-api-entry__param-type">Array&lt;{ inputPath: string; outputPath: string; html: string }&gt;</code>
  </div>
  
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">input.listedPages</code>
    <code class="ox-api-entry__param-type">readonly <a href="./blog-html.md#blogsourcepage">BlogSourcePage</a>[]</code>
  </div>
  
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">input.options?</code>
    <code class="ox-api-entry__param-type"><a href="./types.md#resolvedblogoptions">ResolvedBlogOptions</a></code>
  </div>
  <p class="ox-api-entry__param-description">optional</p>
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">input.collections?</code>
    <code class="ox-api-entry__param-type"><a href="./types.md#resolvedcollectionsoptions">ResolvedCollectionsOptions</a></code>
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
    <code class="ox-api-entry__param-name">input.render</code>
    <code class="ox-api-entry__param-type">(page: <a href="#bloggeneratedpage">BlogGeneratedPage</a>) =&gt; Promise&lt;string&gt;</code>
  </div>
  
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">input.errors</code>
    <code class="ox-api-entry__param-type">string[]</code>
  </div>
  
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

<details id="blog-pages" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">module</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__name">blog-pages</code><span class="ox-api-entry__description">Generated blog index, tag, and archive pages.</span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Generated blog index, tag, and archive pages.</p>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/blog-pages.ts#L1-L3" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
  </div>
</details>

<details id="bloggeneratedpage" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">BlogGeneratedPage</code><span class="ox-api-entry__description">Synthetic page passed back to generateHtmlPage.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">5 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Synthetic page passed back to <code>generateHtmlPage</code>.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface BlogGeneratedPage</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/blog-pages.ts#L36-L42" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="bloggeneratedpage-content">
  <td><code>content</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="bloggeneratedpage-href">
  <td><code>href</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="bloggeneratedpage-outputpath">
  <td><code>outputPath</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="bloggeneratedpage-title">
  <td><code>title</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="bloggeneratedpage-urlpath">
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

<details id="toblogprocessresult" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">toBlogProcessResult(page: BlogGeneratedPage): { inputPath: string; routePaths: { outputPath: string; urlPath: string; href: string; ogImagePath: string; ogImageUrl: string; }; transformedHtml: string; title: string; frontmatter: Record&lt;string, unknown&gt;; toc: []; }</code><span class="ox-api-entry__description">Maps a generated blog page onto the SSG render shape.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 param</span><span class="ox-api-badge">returns object</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Maps a generated blog page onto the SSG render shape.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export function toBlogProcessResult(page: BlogGeneratedPage): {
  inputPath: string;
  routePaths: {
    outputPath: string;
    urlPath: string;
    href: string;
    ogImagePath: string;
    ogImageUrl: string;
  };
  transformedHtml: string;
  title: string;
  frontmatter: Record&lt;string, unknown&gt;;
  toc: [];
}</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/blog-pages.ts#L75-L103" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">page</code>
    <code class="ox-api-entry__param-type"><a href="#bloggeneratedpage">BlogGeneratedPage</a></code>
  </div>
  
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">object</code>
  <div class="ox-api-entry__return-members">
<div class="ox-api-entry__return-member">
<h5>frontmatter</h5>
<code class="ox-api-entry__return-member-type language-typescript">frontmatter: Record&lt;string, unknown&gt;;</code>
</div>
<div class="ox-api-entry__return-member">
<h5>inputPath</h5>
<code class="ox-api-entry__return-member-type language-typescript">inputPath: string;</code>
</div>
<div class="ox-api-entry__return-member">
<h5>routePaths</h5>
<code class="ox-api-entry__return-member-type language-typescript">routePaths: { outputPath: string; urlPath: string; href: string; ogImagePath: string; ogImageUrl: string };</code>
</div>
<div class="ox-api-entry__return-member">
<h5>title</h5>
<code class="ox-api-entry__return-member-type language-typescript">title: string;</code>
</div>
<div class="ox-api-entry__return-member">
<h5>toc</h5>
<code class="ox-api-entry__return-member-type language-typescript">toc: [];</code>
</div>
<div class="ox-api-entry__return-member">
<h5>transformedHtml</h5>
<code class="ox-api-entry__return-member-type language-typescript">transformedHtml: string;</code>
</div>
</div>
</div>
</div>
  </div>
</details>

