# highlight-native.ts

**[Source](https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/highlight-native.ts)**

> 10 documented symbols. Read the signatures first, then expand each item for parameters, return types, and examples.

<div class="ox-api-stats" aria-label="API reference summary">
<span class="ox-api-stat">
  <strong>10</strong>
  <span>symbols</span>
</span>
<span class="ox-api-stat">
  <strong>7</strong>
  <span>functions</span>
</span>
<span class="ox-api-stat">
  <strong>2</strong>
  <span>interfaces</span>
</span>
<span class="ox-api-stat">
  <strong>1</strong>
  <span>modules</span>
</span>
<span class="ox-api-stat">
  <strong>10</strong>
  <span>parameters</span>
</span>
<span class="ox-api-stat">
  <strong>5</strong>
  <span>members</span>
</span>
<span class="ox-api-stat">
  <strong>7</strong>
  <span>returns</span>
</span>
</div>

## Reference

<div class="ox-api-controls" data-ox-api-target=".ox-api-entry" role="toolbar" aria-label="Reference display controls">
<button type="button" class="ox-api-controls__button" data-ox-api-toggle="expand">Open all</button>
<button type="button" class="ox-api-controls__button" data-ox-api-toggle="collapse">Close all</button>
</div>

<details id="applypendinghighlights" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">applyPendingHighlights(html: string, replacements: string[]): string</code><span class="ox-api-entry__description">Splices replacements back over the blocks the native pass left pending. Entry i…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">2 params</span><span class="ox-api-badge">returns string</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Splices <code>replacements</code> back over the blocks the native pass left pending.</p>
<p>Entry <code>i</code> is the highlighted <code>&lt;pre&gt;</code> for pending block <code>i</code>, or an empty string to leave that block alone. This keeps a page that needs one exotic grammar — a Vue SFC, a Mermaid diagram — off the HTML round trip, rather than surrendering the whole document for it.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export function applyPendingHighlights(html: string, replacements: string[]): string</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/highlight-native.ts#L164-L170" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">html</code>
    <code class="ox-api-entry__param-type">string</code>
  </div>
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">replacements</code>
    <code class="ox-api-entry__param-type">string[]</code>
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

<details id="gettextcontent" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">getTextContent(node: Element | Root): string</code><span class="ox-api-entry__description">Extract text content from a hast node.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 param</span><span class="ox-api-badge">returns string</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Extract text content from a hast node.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export function getTextContent(node: Element | Root): string</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/highlight-native.ts#L17-L31" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">node</code>
    <code class="ox-api-entry__param-type">Element | Root</code>
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

<details id="highlight-native" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">module</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__name">highlight-native</code><span class="ox-api-entry__description">The native tree-sitter highlighting path, plus the small hast helpers both engines read code blocks with. Kept apart fr…</span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>The native tree-sitter highlighting path, plus the small hast helpers both engines read code blocks with.</p>
<p>Kept apart from the Shiki plumbing so the two engines stay legible on their own, and arranged as a leaf: <code>highlight.ts</code> reaches in here, never the other way round.</p>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/highlight-native.ts#L1-L8" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
  </div>
</details>

<details id="highlightdocumentnatively" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">highlightDocumentNatively(html: string): Promise&lt;NativeDocument | null&gt;</code><span class="ox-api-entry__description">Highlights every code block in a rendered document in one native call. Returns…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 param</span><span class="ox-api-badge">returns Promise&lt;NativeDocument | null&gt;</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Highlights every code block in a rendered document in one native call.</p>
<p>Returns the rewritten HTML and the languages it declined, so the caller knows whether Shiki still has to run over the result. Returns <code>null</code> when the native module is unavailable.</p>
<p>This exists because the plumbing dwarfed the work: walking each page through an HTML parser and serializer to find <code>&lt;pre&gt;</code> elements cost 139 ms over the documentation corpus, and re-parsing each highlighted block to splice it back cost another 38 ms, against 14 ms of actual highlighting.</p>
<p>It runs off the main thread. The synchronous binding held the event loop for the whole pass, so a build asking for several pages at once still got them one at a time — <code>Promise.all</code> over the corpus measured the same as awaiting each page in turn.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export async function highlightDocumentNatively(html: string): Promise&lt;NativeDocument | null&gt;</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/highlight-native.ts#L130-L136" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
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
  <code class="ox-api-entry__return-type">Promise&lt;<a href="#nativedocument">NativeDocument</a> | null&gt;</code>
</div>
</div>
  </div>
</details>

<details id="highlightnatively" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">highlightNatively(code: string, lang: string): string | null</code><span class="ox-api-entry__description">Highlights with the native tree-sitter engine, or null when it has no grammar f…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">2 params</span><span class="ox-api-badge">returns string | null</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Highlights with the native tree-sitter engine, or <code>null</code> when it has no grammar for <code>lang</code>.</p>
<p>Parsing once and walking the tree is roughly eight times faster than matching TextMate patterns line by line — 10.5 ms against 81.5 ms over the documentation corpus&#39;s code blocks — and it emits the same <code>--octc-shiki-*</code> markup, so themes are unaffected.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export function highlightNatively(code: string, lang: string): string | null</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/highlight-native.ts#L54-L60" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">code</code>
    <code class="ox-api-entry__param-type">string</code>
  </div>
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">lang</code>
    <code class="ox-api-entry__param-type">string</code>
  </div>
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">string | null</code>
</div>
</div>
  </div>
</details>

<details id="languageof" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">languageOf(codeElement: Element): string | null</code><span class="ox-api-entry__description">The language-* class on a &lt;code&gt; element, if it carries one.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 param</span><span class="ox-api-badge">returns string | null</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>The <code>language-*</code> class on a <code>&lt;code&gt;</code> element, if it carries one.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export function languageOf(codeElement: Element): string | null</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/highlight-native.ts#L106-L111" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">codeElement</code>
    <code class="ox-api-entry__param-type">Element</code>
  </div>
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">string | null</code>
</div>
</div>
  </div>
</details>

<details id="nativedocument" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">NativeDocument</code><span class="ox-api-entry__description">What highlightDocumentNatively produced for a page.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">3 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>What <a href="#highlightdocumentnatively">highlightDocumentNatively</a> produced for a page.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface NativeDocument</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/highlight-native.ts#L145-L154" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="nativedocument-html">
  <td><code>html</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="nativedocument-pending">
  <td><code>pending</code></td>
  <td><code class="ox-api-entry__member-type language-typescript"><a href="#pendingblock">PendingBlock</a>[]</code></td>
  <td><div class="ox-api-entry__member-description">Well-formed blocks whose language has no native grammar, in order.</div></td>
</tr>
<tr id="nativedocument-skipped">
  <td><code>skipped</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string[]</code></td>
  <td><div class="ox-api-entry__member-description">Languages of elements the native pass could not read. Non-empty means the<br>page has to be produced by the HTML-parser-based highlighter instead.</div></td>
</tr>
</tbody>
</table>
</div>
</div>
  </div>
</details>

<details id="nativesupports" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">nativeSupports(lang: string): boolean</code><span class="ox-api-entry__description">Whether the native engine claims lang.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 param</span><span class="ox-api-badge">returns boolean</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Whether the native engine claims <code>lang</code>.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export function nativeSupports(lang: string): boolean</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/highlight-native.ts#L63-L69" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">lang</code>
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

<details id="pendingblock" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">PendingBlock</code><span class="ox-api-entry__description">A block the native pass left for another highlighter.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">2 members</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>A block the native pass left for another highlighter.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface PendingBlock</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/highlight-native.ts#L139-L142" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--members">
<h4>Members</h4>
<div class="ox-api-entry__member-group">
<h5>Properties</h5>
<table class="ox-api-entry__members-table">
<thead><tr><th>Name</th><th>Type</th><th>Description</th></tr></thead>
<tbody>
<tr id="pendingblock-language">
  <td><code>language</code></td>
  <td><code class="ox-api-entry__member-type language-typescript">string</code></td>
  <td></td>
</tr>
<tr id="pendingblock-source">
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

<details id="treeneedsshiki" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">treeNeedsShiki(tree: Root, nativeThemeApplies: boolean): boolean</code><span class="ox-api-entry__description">Whether any block in this tree still needs Shiki. Creating a Shiki highlighter…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">2 params</span><span class="ox-api-badge">returns boolean</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Whether any block in this tree still needs Shiki.</p>
<p>Creating a Shiki highlighter parses two dozen TextMate grammars, about 190 ms, so a document whose languages are all covered natively must not touch it at all.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export function treeNeedsShiki(tree: Root, nativeThemeApplies: boolean): boolean</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/highlight-native.ts#L78-L103" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">tree</code>
    <code class="ox-api-entry__param-type">Root</code>
  </div>
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">nativeThemeApplies</code>
    <code class="ox-api-entry__param-type">boolean</code>
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
