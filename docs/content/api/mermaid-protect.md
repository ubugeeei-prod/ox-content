# mermaid-protect.ts

**[Source](https://github.com/ubugeeei/ox-content/blob/main/npm/vite-plugin-ox-content/src/plugins/mermaid-protect.ts)**

> 3 documented symbols. Read the signatures first, then expand each item for parameters, return types, and examples.

## Reference

<div class="ox-api-controls" data-ox-api-target=".ox-api-entry" role="toolbar" aria-label="Reference display controls">
<button type="button" class="ox-api-controls__button" data-ox-api-toggle="expand">Open all</button>
<button type="button" class="ox-api-controls__button" data-ox-api-toggle="collapse">Close all</button>
</div>

<details id="mermaidsvgprotection" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">MermaidSvgProtection</code><span class="ox-api-entry__description">Protects mermaid SVG content from rehype HTML5 parser corruption. rehypeParse +…</span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Protects mermaid SVG content from rehype HTML5 parser corruption. rehypeParse + rehypeStringify converts <code>&lt;br /&gt;</code> in SVG foreignObject to <code>&lt;br&gt;&lt;/br&gt;</code>, which HTML5 interprets as 2 &lt;br&gt; elements. Each rehype pass doubles them: 1 → 2 → 4 → 8 → 16. This module extracts ox-mermaid SVG blocks into placeholders before rehype processing and restores them after.</p>
</div>
<p class="ox-api-entry__source"><a href="https://github.com/ubugeeei/ox-content/blob/main/npm/vite-plugin-ox-content/src/plugins/mermaid-protect.ts#L12-L15">View source</a></p>
  </div>
</details>

<details id="protectmermaidsvgs" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">protectMermaidSvgs(html: string): MermaidSvgProtection</code><span class="ox-api-entry__description">Extract <code>&lt;div class=&quot;ox-mermaid&quot;&gt;...&lt;/div&gt;</code> blocks and replace with HTML commen…</span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Extract <code>&lt;div class=&quot;ox-mermaid&quot;&gt;...&lt;/div&gt;</code> blocks and replace with HTML comment placeholders that rehype will preserve.</p>
</div>
<p class="ox-api-entry__source"><a href="https://github.com/ubugeeei/ox-content/blob/main/npm/vite-plugin-ox-content/src/plugins/mermaid-protect.ts#L21-L64">View source</a></p>
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
  <code class="ox-api-entry__return-type">MermaidSvgProtection</code>
  
</div>
</div>
  </div>
</details>

<details id="restoremermaidsvgs" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">restoreMermaidSvgs(html: string, svgs: Map&lt;string, string&gt;): string</code><span class="ox-api-entry__description">Restore protected mermaid SVG blocks from placeholders.</span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Restore protected mermaid SVG blocks from placeholders.</p>
</div>
<p class="ox-api-entry__source"><a href="https://github.com/ubugeeei/ox-content/blob/main/npm/vite-plugin-ox-content/src/plugins/mermaid-protect.ts#L69-L75">View source</a></p>
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
    <code class="ox-api-entry__param-name">svgs</code>
    <code class="ox-api-entry__param-type">Map</code>
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
