# docs.ts

**[Source](https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/docs.ts)**

> 5 documented symbols. Read the signatures first, then expand each item for parameters, return types, and examples.

<div class="ox-api-stats" aria-label="API reference summary">
<span class="ox-api-stat">
  <strong>5</strong>
  <span>symbols</span>
</span>
<span class="ox-api-stat">
  <strong>4</strong>
  <span>functions</span>
</span>
<span class="ox-api-stat">
  <strong>1</strong>
  <span>modules</span>
</span>
<span class="ox-api-stat">
  <strong>9</strong>
  <span>parameters</span>
</span>
<span class="ox-api-stat">
  <strong>4</strong>
  <span>returns</span>
</span>
<span class="ox-api-stat">
  <strong>2</strong>
  <span>examples</span>
</span>
</div>

## Reference

<div class="ox-api-controls" data-ox-api-target=".ox-api-entry" role="toolbar" aria-label="Reference display controls">
<button type="button" class="ox-api-controls__button" data-ox-api-toggle="expand">Open all</button>
<button type="button" class="ox-api-controls__button" data-ox-api-toggle="collapse">Close all</button>
</div>

<details id="docs" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">module</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__name">docs</code><span class="ox-api-entry__description">Source Documentation Extraction and Generation This module provides comprehensive tools for extracting JSDoc/TSDoc comm…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 example</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Source Documentation Extraction and Generation</p>
<p>This module provides comprehensive tools for extracting JSDoc/TSDoc comments from TypeScript/JavaScript source files and automatically generating Markdown documentation.</p>
<h2>Features</h2>
<ul>
<li><strong>Automatic Extraction</strong>: Parses JSDoc comments from functions, classes, interfaces, and types</li>
<li><strong>Flexible Filtering</strong>: Include/exclude patterns for selective documentation</li>
<li><strong>Markdown Generation</strong>: Converts extracted docs to organized Markdown files</li>
<li><strong>Navigation Generation</strong>: Auto-generates sidebar navigation metadata</li>
<li><strong>GitHub Links</strong>: Includes clickable links to source code on GitHub</li>
</ul>
<h2>Supported JSDoc Tags</h2>
<ul>
<li><code>@param {type} name - description</code> - Function parameter documentation</li>
<li><code>@returns {type} description</code> - Return value documentation</li>
<li><code>@example</code> - Code examples (multi-line blocks)</li>
<li><code>@private</code> - Mark item as private (excluded from docs if private=false)</li>
<li><code>@default value</code> - Default parameter value</li>
<li>Custom tags are preserved in the <code>tags</code> field</li>
</ul>
<h2>Usage Flow</h2>
<ol>
<li>Call <code>extractDocs()</code> to parse source files</li>
<li>Call <code>generateMarkdown()</code> to create Markdown content</li>
<li>Call <code>writeDocs()</code> to write files to output directory</li>
<li>Generated nav.ts can be imported for sidebar navigation</li>
</ol>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/docs.ts#L1-L50" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--examples">
<h4>Examples</h4>
<div class="ox-api-entry__example">
<div class="ox-api-entry__example-heading">Example 1</div>
<pre><code class="language-typescript">import { extractDocs, generateMarkdown, writeDocs } from &#39;./docs&#39;;

const docsOptions = {
enabled: true,
src: [&#39;./src&#39;],
out: &#39;./docs/api&#39;,
include: [&#39;**\/*.ts&#39;],
exclude: [&#39;**\/*.test.ts&#39;],
groupBy: &#39;file&#39;,
githubUrl: &#39;https://github.com/user/project&#39;,
};

const extracted = await extractDocs([&#39;./src&#39;], docsOptions);
const markdown = generateMarkdown(extracted, docsOptions);
await writeDocs(markdown, &#39;./docs/api&#39;, extracted, docsOptions);</code></pre>

</div>
</div>
  </div>
</details>

<details id="extractdocs" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">extractDocs(srcDirs: string[], options: ResolvedDocsOptions): Promise&lt;ExtractedDocs[]&gt;</code><span class="ox-api-entry__description">Extracts JSDoc documentation from source files in specified directories. This f…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">2 params</span><span class="ox-api-badge">returns Promise&lt;ExtractedDocs[]&gt;</span><span class="ox-api-badge">1 example</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Extracts JSDoc documentation from source files in specified directories.</p>
<p>This function recursively searches directories for source files matching the include/exclude patterns, then extracts all documented items (functions, classes, interfaces, types) from those files.</p>
<h2>Process</h2>
<ol>
<li><strong>File Discovery</strong>: Recursively walks directories, applying filters</li>
<li><strong>File Reading</strong>: Loads each matching file&#39;s content</li>
<li><strong>JSDoc Extraction</strong>: Parses JSDoc comments using the native parser</li>
<li><strong>Declaration Matching</strong>: Pairs JSDoc comments with source declarations</li>
<li><strong>Result Collection</strong>: Aggregates extracted documentation by file</li>
</ol>
<h2>Include/Exclude Patterns</h2>
<p>Patterns support:</p>
<ul>
<li><code>**</code> - Match any directory structure</li>
<li><code>*</code> - Match any filename</li>
<li>Standard glob patterns (e.g., <code>**\/*.test.ts</code>)</li>
</ul>
<h2>Performance Considerations</h2>
<ul>
<li>Uses filesystem I/O which can be slow for large codebases</li>
<li>Consider using more specific include patterns to reduce file scanning</li>
<li>Results are not cached; call once per build/dev session</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export async function extractDocs(srcDirs: string[], options: ResolvedDocsOptions): Promise&lt;ExtractedDocs[]&gt;</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/docs.ts#L139-L215" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">srcDirs</code>
    <code class="ox-api-entry__param-type">string[]</code>
  </div>
  <p class="ox-api-entry__param-description">Array of source directory paths to scan</p>
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">options</code>
    <code class="ox-api-entry__param-type"><a href="./types.md#resolveddocsoptions">ResolvedDocsOptions</a></code>
  </div>
  <p class="ox-api-entry__param-description">Documentation extraction options (filters, grouping, etc.)</p>
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">Promise&lt;<a href="./types.md#extracteddocs">ExtractedDocs</a>[]&gt;</code>
  <p class="ox-api-entry__return-description">Promise resolving to array of extracted documentation by file.<br>         Each ExtractedDocs object contains file path and array of DocEntry items.</p>
</div>
</div>
<div class="ox-api-entry__section ox-api-entry__section--examples">
<h4>Examples</h4>
<div class="ox-api-entry__example">
<div class="ox-api-entry__example-heading">Example 1</div>
<pre><code class="language-typescript">const docs = await extractDocs(
  [&#39;./packages/vite-plugin/src&#39;],
  {
    enabled: true,
    src: [],
    out: &#39;docs&#39;,
    include: [&#39;**\/*.ts&#39;],
    exclude: [&#39;**\/*.test.ts&#39;, &#39;**\/*.spec.ts&#39;],
    format: &#39;markdown&#39;,
    private: false,
    toc: true,
    groupBy: &#39;file&#39;,
    generateNav: true,
  }
);

// Returns:
// [
// {
// file: &#39;/path/to/transform.ts&#39;,
// entries: [
// { name: &#39;transformMarkdown&#39;, kind: &#39;function&#39;, ... },
// { name: &#39;loadNapiBindings&#39;, kind: &#39;function&#39;, ... },
// ]
// },
// ...
// ]</code></pre>

</div>
</div>
  </div>
</details>

<details id="generatemarkdown" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">generateMarkdown(docs: ExtractedDocs[], options: ResolvedDocsOptions): Record&lt;string, string&gt;</code><span class="ox-api-entry__description">Generates Markdown documentation from extracted docs.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">2 params</span><span class="ox-api-badge">returns Record&lt;string, string&gt;</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Generates Markdown documentation from extracted docs.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export function generateMarkdown(docs: ExtractedDocs[], options: ResolvedDocsOptions): Record&lt;string, string&gt;</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/docs.ts#L220-L255" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">docs</code>
    <code class="ox-api-entry__param-type"><a href="./types.md#extracteddocs">ExtractedDocs</a>[]</code>
  </div>
  
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">options</code>
    <code class="ox-api-entry__param-type"><a href="./types.md#resolveddocsoptions">ResolvedDocsOptions</a></code>
  </div>
  
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">Record&lt;string, string&gt;</code>
  
</div>
</div>
  </div>
</details>

<details id="resolvedocsoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">resolveDocsOptions(options: false): false</code><span class="ox-api-entry__description">Resolves docs options with defaults.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 param</span><span class="ox-api-badge">returns false</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Resolves docs options with defaults.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export function resolveDocsOptions(options: false): false</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/docs.ts#L330" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">options</code>
    <code class="ox-api-entry__param-type">false</code>
  </div>
  
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">false</code>
  
</div>
</div>
  </div>
</details>

<details id="writedocs" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">writeDocs(docs: Record&lt;string, string&gt;, outDir: string, extractedDocs?: ExtractedDocs[], options?: ResolvedDocsOptions): Promise&lt;void&gt;</code><span class="ox-api-entry__description">Writes generated documentation to the output directory.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">4 params</span><span class="ox-api-badge">returns Promise&lt;void&gt;</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Writes generated documentation to the output directory.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export async function writeDocs(docs: Record&lt;string, string&gt;, outDir: string, extractedDocs?: ExtractedDocs[], options?: ResolvedDocsOptions): Promise&lt;void&gt;</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/docs.ts#L260-L291" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">docs</code>
    <code class="ox-api-entry__param-type">Record&lt;string, string&gt;</code>
  </div>
  
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">outDir</code>
    <code class="ox-api-entry__param-type">string</code>
  </div>
  
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">extractedDocs</code>
    <code class="ox-api-entry__param-type"><a href="./types.md#extracteddocs">ExtractedDocs</a>[]</code>
  </div>
  <p class="ox-api-entry__param-description">optional</p>
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">options</code>
    <code class="ox-api-entry__param-type"><a href="./types.md#resolveddocsoptions">ResolvedDocsOptions</a></code>
  </div>
  <p class="ox-api-entry__param-description">optional</p>
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
