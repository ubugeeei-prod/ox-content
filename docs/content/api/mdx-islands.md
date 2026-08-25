# mdx-islands.ts

**[Source](https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/mdx-islands.ts)**

> 6 documented symbols. Read the signatures first, then expand each item for parameters, return types, and examples.

<div class="ox-api-stats" aria-label="API reference summary">
<span class="ox-api-stat">
  <strong>6</strong>
  <span>symbols</span>
</span>
<span class="ox-api-stat">
  <strong>4</strong>
  <span>functions</span>
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
  <strong>5</strong>
  <span>parameters</span>
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

<details id="collectmdxislandnamesfromhtml" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">collectMdxIslandNamesFromHtml(html: string): string[]</code><span class="ox-api-entry__description">Collect data-ox-island names from Rust-rendered HTML. Used when an AST walk is…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 param</span><span class="ox-api-badge">returns string[]</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Collect <code>data-ox-island</code> names from Rust-rendered HTML. Used when an AST walk is unavailable.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export function collectMdxIslandNamesFromHtml(html: string): string[]</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/mdx-islands.ts#L34-L43" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">html</code>
    <code class="ox-api-entry__param-type">string</code>
  </div>
  
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">string[]</code>
  
</div>
</div>
  </div>
</details>

<details id="collectmdxjsxnamesfromast" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">collectMdxJsxNamesFromAst(ast: unknown): string[]</code><span class="ox-api-entry__description">Collect named MDX JSX tags from a parsed mdast tree (JSON from NAPI parse()). F…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 param</span><span class="ox-api-badge">returns string[]</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Collect named MDX JSX tags from a parsed mdast tree (JSON from NAPI <code>parse()</code>). Fragments (<code>name: null</code>) and non-JSX nodes are ignored. Walks nested children so inner islands are found.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export function collectMdxJsxNamesFromAst(ast: unknown): string[]</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/mdx-islands.ts#L24-L28" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">ast</code>
    <code class="ox-api-entry__param-type">unknown</code>
  </div>
  
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">string[]</code>
  
</div>
</div>
  </div>
</details>

<details id="componentregistry" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">type</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">ComponentRegistry = Readonly&lt;Record&lt;string, unknown&gt;&gt; | ReadonlyMap&lt;string, unknown&gt; | Iterable&lt;string&gt;</code><span class="ox-api-entry__description">Global component map: object, Map, or name list.</span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Global component map: object, Map, or name list.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export type ComponentRegistry = Readonly&lt;Record&lt;string, unknown&gt;&gt; | ReadonlyMap&lt;string, unknown&gt; | Iterable&lt;string&gt;</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/mdx-islands.ts#L12-L15" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
  </div>
</details>

<details id="discoverregisteredmdxcomponents" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">discoverRegisteredMdxComponents(input: DiscoverRegisteredMdxComponentsInput): Promise&lt;string[]&gt;</code><span class="ox-api-entry__description">Resolve registered island names for an MDX document. Prefers a NAPI parse() AST…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 param</span><span class="ox-api-badge">returns Promise&lt;string[]&gt;</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Resolve registered island names for an MDX document.</p>
<p>Prefers a NAPI <code>parse()</code> AST walk. Falls back to rendered <code>data-ox-island</code> names so plugins still hydrate if #659 metadata is not present.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export async function discoverRegisteredMdxComponents(input: DiscoverRegisteredMdxComponentsInput): Promise&lt;string[]&gt;</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/mdx-islands.ts#L73-L80" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">input</code>
    <code class="ox-api-entry__param-type">DiscoverRegisteredMdxComponentsInput</code>
  </div>
  
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">Promise&lt;string[]&gt;</code>
  
</div>
</div>
  </div>
</details>

<details id="intersectregisteredcomponentnames" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">intersectRegisteredComponentNames(names: Iterable&lt;string&gt;, components: ComponentRegistry): string[]</code><span class="ox-api-entry__description">Keep names that exist on the global component map, in first-seen order.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">2 params</span><span class="ox-api-badge">returns string[]</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Keep names that exist on the global component map, in first-seen order.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export function intersectRegisteredComponentNames(names: Iterable&lt;string&gt;, components: ComponentRegistry): string[]</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/mdx-islands.ts#L46-L57" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">names</code>
    <code class="ox-api-entry__param-type">Iterable&lt;string&gt;</code>
  </div>
  
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">components</code>
    <code class="ox-api-entry__param-type"><a href="#componentregistry">ComponentRegistry</a></code>
  </div>
  
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">string[]</code>
  
</div>
</div>
  </div>
</details>

<details id="mdx-islands" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">module</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__name">mdx-islands</code><span class="ox-api-entry__description">Discover registered MDX islands from the mdast tree or rendered HTML. Framework plugins use this instead of a source re…</span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Discover registered MDX islands from the mdast tree or rendered HTML.</p>
<p>Framework plugins use this instead of a source regex when MDX is on, so nested JSX, expression attributes, and fragments stay visible. Names that are not in the global <code>components</code> map are left as static HTML.</p>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/mdx-islands.ts#L1-L7" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
  </div>
</details>

