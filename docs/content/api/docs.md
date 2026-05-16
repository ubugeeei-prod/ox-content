# docs.ts

**[Source](https://github.com/ubugeeei/ox-content/blob/main/npm/vite-plugin-ox-content/src/docs.ts)**

> 8 documented symbols. Read the signatures first, then expand each item for parameters, return types, and examples.

## Reference

<div class="ox-api-controls" data-ox-api-target=".ox-api-entry" role="toolbar" aria-label="Reference display controls">
<button type="button" class="ox-api-controls__button" data-ox-api-toggle="expand">Open all</button>
<button type="button" class="ox-api-controls__button" data-ox-api-toggle="collapse">Close all</button>
</div>

<details id="buildsymbolmap" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">buildSymbolMap(docs: ExtractedDocs[]): Map&lt;string, SymbolLocation&gt;</code><span class="ox-api-entry__description">Builds a map of all symbols to their file locations.</span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Builds a map of all symbols to their file locations.</p>
</div>
<p class="ox-api-entry__source"><a href="https://github.com/ubugeeei/ox-content/blob/main/npm/vite-plugin-ox-content/src/docs.ts#L1116-L1135">View source</a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">docs</code>
    <code class="ox-api-entry__param-type">ExtractedDocs[]</code>
  </div>
  
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">Map</code>
  
</div>
</div>
  </div>
</details>

<details id="convertsymbollinks" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">convertSymbolLinks(text: string, currentFileName: string, symbolMap: Map&lt;string, SymbolLocation&gt;): string</code><span class="ox-api-entry__description">Converts symbol links [SymbolName] to markdown links. Processes description tex…</span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Converts symbol links [SymbolName] to markdown links. Processes description text to convert cargo-docs-style symbol references <code>[SymbolName]</code> into clickable markdown links pointing to the appropriate documentation page.</p>
<h2>Examples</h2>
<p>Input: &quot;See [transformMarkdown] for usage&quot; (same file) Output: &quot;See <a href="#transformmarkdown">transformMarkdown</a> for usage&quot; Input: &quot;Uses <a href="./types.md#navitem">NavItem</a> interface&quot; (different file: types.ts) Output: &quot;Uses <a href="./types.md#navitem">NavItem</a> interface&quot;</p>
</div>
<p class="ox-api-entry__source"><a href="https://github.com/ubugeeei/ox-content/blob/main/npm/vite-plugin-ox-content/src/docs.ts#L1089-L1111">View source</a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">text</code>
    <code class="ox-api-entry__param-type">string</code>
  </div>
  <p class="ox-api-entry__param-description">Description text containing symbol references</p>
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">currentFileName</code>
    <code class="ox-api-entry__param-type">string</code>
  </div>
  <p class="ox-api-entry__param-description">Current file name (without extension) for same-file detection</p>
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">symbolMap</code>
    <code class="ox-api-entry__param-type">Map</code>
  </div>
  <p class="ox-api-entry__param-description">Map of symbol names to their file locations</p>
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">string</code>
  <p class="ox-api-entry__return-description">Text with symbol references converted to markdown links</p>
</div>
</div>
<div class="ox-api-entry__section ox-api-entry__section--tags">
<h4>Tags</h4>
<ul class="ox-api-entry__tags"><li><span class="ox-api-entry__tag-name">@internal</span><span class="ox-api-entry__tag-value"></span></li></ul>
</div>
  </div>
</details>

<details id="extractdocs" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">extractDocs(srcDirs: string[], options: ResolvedDocsOptions): Promise&lt;ExtractedDocs[]&gt;</code><span class="ox-api-entry__description">Extracts JSDoc documentation from source files in specified directories. This f…</span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Extracts JSDoc documentation from source files in specified directories. This function recursively searches directories for source files matching the include/exclude patterns, then extracts all documented items (functions, classes, interfaces, types) from those files.</p>
<h2>Process</h2>
<ol>
<li><strong>File Discovery</strong>: Recursively walks directories, applying filters</li>
<li><strong>File Reading</strong>: Loads each matching file&#39;s content</li>
<li><strong>JSDoc Extraction</strong>: Parses JSDoc comments using regex patterns</li>
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
<li>Results are not cached; call once per build/dev session Each ExtractedDocs object contains file path and array of DocEntry items.</li>
</ul>
</div>
<p class="ox-api-entry__source"><a href="https://github.com/ubugeeei/ox-content/blob/main/npm/vite-plugin-ox-content/src/docs.ts#L420-L450">View source</a></p>
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
    <code class="ox-api-entry__param-type">ResolvedDocsOptions</code>
  </div>
  <p class="ox-api-entry__param-description">Documentation extraction options (filters, grouping, etc.)</p>
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">Promise</code>
  <p class="ox-api-entry__return-description">Promise resolving to array of extracted documentation by file.</p>
</div>
</div>
<div class="ox-api-entry__section ox-api-entry__section--examples">
<h4>Examples</h4>
<pre><code class="language-ts">const docs = await extractDocs(
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
//   {
//     file: &#39;/path/to/transform.ts&#39;,
//     entries: [
//       { name: &#39;transformMarkdown&#39;, kind: &#39;function&#39;, ... },
//       { name: &#39;loadNapiBindings&#39;, kind: &#39;function&#39;, ... },
//     ]
//   },
//   ...
// ]</code></pre>
</div>
  </div>
</details>

<details id="findfiles" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">findFiles(dir: string, options: ResolvedDocsOptions): Promise&lt;string[]&gt;</code><span class="ox-api-entry__description">Recursively finds all source files matching include/exclude patterns.</span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Recursively finds all source files matching include/exclude patterns.</p>
</div>
<p class="ox-api-entry__source"><a href="https://github.com/ubugeeei/ox-content/blob/main/npm/vite-plugin-ox-content/src/docs.ts#L457-L485">View source</a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">dir</code>
    <code class="ox-api-entry__param-type">string</code>
  </div>
  
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">options</code>
    <code class="ox-api-entry__param-type">ResolvedDocsOptions</code>
  </div>
  
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">Promise</code>
  
</div>
</div>
<div class="ox-api-entry__section ox-api-entry__section--tags">
<h4>Tags</h4>
<ul class="ox-api-entry__tags"><li><span class="ox-api-entry__tag-name">@internal</span><span class="ox-api-entry__tag-value"></span></li></ul>
</div>
  </div>
</details>

<details id="generatemarkdown" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">generateMarkdown(docs: ExtractedDocs[], options: ResolvedDocsOptions): Record&lt;string, string&gt;</code><span class="ox-api-entry__description">Generates Markdown documentation from extracted docs.</span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Generates Markdown documentation from extracted docs.</p>
</div>
<p class="ox-api-entry__source"><a href="https://github.com/ubugeeei/ox-content/blob/main/npm/vite-plugin-ox-content/src/docs.ts#L691-L738">View source</a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">docs</code>
    <code class="ox-api-entry__param-type">ExtractedDocs[]</code>
  </div>
  
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">options</code>
    <code class="ox-api-entry__param-type">ResolvedDocsOptions</code>
  </div>
  
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">Record</code>
  
</div>
</div>
  </div>
</details>

<details id="generatesourcehref" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">generateSourceHref(filePath: string, githubUrl: string, lineNumber?: number, endLineNumber?: number): string</code><span class="ox-api-entry__description">Generates a GitHub source link for a file and optional line range.</span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Generates a GitHub source link for a file and optional line range.</p>
</div>
<p class="ox-api-entry__source"><a href="https://github.com/ubugeeei/ox-content/blob/main/npm/vite-plugin-ox-content/src/docs.ts#L1213-L1227">View source</a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">filePath</code>
    <code class="ox-api-entry__param-type">string</code>
  </div>
  <p class="ox-api-entry__param-description">Full path to the source file</p>
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">githubUrl</code>
    <code class="ox-api-entry__param-type">string</code>
  </div>
  <p class="ox-api-entry__param-description">Base GitHub repository URL</p>
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">lineNumber</code>
    <code class="ox-api-entry__param-type">number</code>
  </div>
  <p class="ox-api-entry__param-description">Optional start line number to link to — optional</p>
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">endLineNumber</code>
    <code class="ox-api-entry__param-type">number</code>
  </div>
  <p class="ox-api-entry__param-description">Optional end line number to link to — optional</p>
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">string</code>
  <p class="ox-api-entry__return-description">Absolute GitHub URL to source code</p>
</div>
</div>
  </div>
</details>

<details id="symbollocation" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">SymbolLocation</code><span class="ox-api-entry__description">Symbol location info for cross-file linking.</span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Symbol location info for cross-file linking.</p>
</div>
<p class="ox-api-entry__source"><a href="https://github.com/ubugeeei/ox-content/blob/main/npm/vite-plugin-ox-content/src/docs.ts#L1061-L1065">View source</a></p>
  </div>
</details>

<details id="writedocs" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">writeDocs(docs: Record&lt;string, string&gt;, outDir: string, extractedDocs?: ExtractedDocs[], options?: ResolvedDocsOptions): Promise&lt;void&gt;</code><span class="ox-api-entry__description">Writes generated documentation to the output directory.</span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Writes generated documentation to the output directory.</p>
</div>
<p class="ox-api-entry__source"><a href="https://github.com/ubugeeei/ox-content/blob/main/npm/vite-plugin-ox-content/src/docs.ts#L1140-L1199">View source</a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">docs</code>
    <code class="ox-api-entry__param-type">Record</code>
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
    <code class="ox-api-entry__param-type">ExtractedDocs[]</code>
  </div>
  <p class="ox-api-entry__param-description">optional</p>
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">options</code>
    <code class="ox-api-entry__param-type">ResolvedDocsOptions</code>
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
