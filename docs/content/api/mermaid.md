# mermaid.ts

**[Source](https://github.com/ubugeeei/ox-content/blob/main/npm/vite-plugin-ox-content/src/plugins/mermaid.ts)**

> 1 documented symbol. Read the signatures first, then expand each item for parameters, return types, and examples.

## Reference

<details id="transformmermaidstatic" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">transformMermaidStatic(html: string, _options?: MermaidOptions): Promise&lt;string&gt;</code><span class="ox-api-entry__description">Transforms mermaid code blocks in HTML to rendered SVG diagrams. Uses the nativ…</span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Transforms mermaid code blocks in HTML to rendered SVG diagrams. Uses the native Rust NAPI transformMermaid function.</p>
</div>
<p class="ox-api-entry__source"><a href="https://github.com/ubugeeei/ox-content/blob/main/npm/vite-plugin-ox-content/src/plugins/mermaid.ts#L76-L101">View source</a></p>
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
    <code class="ox-api-entry__param-name">_options</code>
    <code class="ox-api-entry__param-type">MermaidOptions</code>
  </div>
  <p class="ox-api-entry__param-description">optional</p>
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">Promise</code>
  
</div>
</div>
  </div>
</details>
