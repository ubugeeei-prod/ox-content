# highlight-pending.ts

**[Source](https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/highlight-pending.ts)**

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
  <span>variables</span>
</span>
<span class="ox-api-stat">
  <strong>1</strong>
  <span>modules</span>
</span>
<span class="ox-api-stat">
  <strong>3</strong>
  <span>parameters</span>
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

<details id="highlight-pending" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">module</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__name">highlight-pending</code><span class="ox-api-entry__description">Highlighting the blocks the native pass could not claim. These arrive already named — the native pass reports each one&#39;…</span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Highlighting the blocks the native pass could not claim.</p>
<p>These arrive already named — the native pass reports each one&#39;s language — which lets this load exactly the grammars a page needs instead of the whole bundled set, and splice each result back without an HTML parser.</p>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/highlight-pending.ts#L1-L7" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
  </div>
</details>

<details id="highlightpendingblocks" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">highlightPendingBlocks(blocks: readonly { language: string; source: string }[], theme: string | ThemeRegistration = CSS_VARIABLES_THEME, langs: LanguageRegistration[] = []): Promise&lt;string[]&gt;</code><span class="ox-api-entry__description">Highlight the blocks the native pass left pending, in order. Entry i of the res…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">3 params</span><span class="ox-api-badge">returns Promise&lt;string[]&gt;</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Highlight the blocks the native pass left pending, in order.</p>
<p>Entry <code>i</code> of the result is the <code>&lt;pre&gt;</code> for block <code>i</code>, or an empty string when Shiki has no grammar for it either — in which case the block is left exactly as it arrived, the same outcome the tree walk reached by keeping the original element.</p>
<p>This is the whole point of the pending list: a page whose only unsupported block is a Mermaid diagram used to be handed to the tree walk in full, which re-highlighted every one of its other blocks and paid for a parse and a serialize of the page to do it.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export async function highlightPendingBlocks(blocks: readonly { language: string; source: string }[], theme: string | ThemeRegistration = CSS_VARIABLES_THEME, langs: LanguageRegistration[] = []): Promise&lt;string[]&gt;</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/highlight-pending.ts#L96-L122" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">blocks</code>
    <code class="ox-api-entry__param-type">readonly { language: string; source: string }[]</code>
  </div>
  
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">theme</code>
    <code class="ox-api-entry__param-type">string | ThemeRegistration</code>
  </div>
  <p class="ox-api-entry__param-description">optional · default: CSS<em>VARIABLES</em>THEME</p>
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">langs</code>
    <code class="ox-api-entry__param-type">LanguageRegistration[]</code>
  </div>
  <p class="ox-api-entry__param-description">optional · default: []</p>
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

<details id="lazyhighlightercache" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">variable</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">const lazyHighlighterCache = new Map&lt;string, Promise&lt;Highlighter&gt;&gt;()</code><span class="ox-api-entry__description">A highlighter that starts with no grammars and gains them on demand. getHighlig…</span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>A highlighter that starts with no grammars and gains them on demand.</p>
<p><code>getHighlighter</code> loads all two dozen <code>BUILTIN_LANGS</code> up front, because the tree walk it serves discovers a page&#39;s languages only while rewriting it. The pending list does not have that problem — it names every language it needs — and paying for two dozen TextMate grammars to highlight one Vue block is most of what a page with an exotic block costs.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">const lazyHighlighterCache = new Map&lt;string, Promise&lt;Highlighter&gt;&gt;()</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/highlight-pending.ts#L29" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
  </div>
</details>

