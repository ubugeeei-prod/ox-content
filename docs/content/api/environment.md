# environment.ts

**[Source](https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/environment.ts)**

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
  <span>interfaces</span>
</span>
<span class="ox-api-stat">
  <strong>6</strong>
  <span>parameters</span>
</span>
<span class="ox-api-stat">
  <strong>3</strong>
  <span>returns</span>
</span>
<span class="ox-api-stat">
  <strong>1</strong>
  <span>examples</span>
</span>
</div>

## Reference

<div class="ox-api-controls" data-ox-api-target=".ox-api-entry" role="toolbar" aria-label="Reference display controls">
<button type="button" class="ox-api-controls__button" data-ox-api-toggle="expand">Open all</button>
<button type="button" class="ox-api-controls__button" data-ox-api-toggle="collapse">Close all</button>
</div>

<details id="createenvironmentplugins" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">createEnvironmentPlugins(options: ResolvedOptions)</code><span class="ox-api-entry__description">Environment plugin factory. Creates plugins specific to the Markdown environmen…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 param</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Environment plugin factory.</p>
<p>Creates plugins specific to the Markdown environment.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export function createEnvironmentPlugins(options: ResolvedOptions)</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/environment.ts#L150-L176" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">options</code>
    <code class="ox-api-entry__param-type">ResolvedOptions</code>
  </div>
  
</li>
</ul>
</div>
  </div>
</details>

<details id="createmarkdownenvironment" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">createMarkdownEnvironment(options: ResolvedOptions): EnvironmentOptions</code><span class="ox-api-entry__description">Creates the Markdown processing environment configuration. This environment is…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">1 param</span><span class="ox-api-badge">returns EnvironmentOptions</span><span class="ox-api-badge">1 example</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Creates the Markdown processing environment configuration.</p>
<p>This environment is used for:</p>
<ul>
<li>Server-side rendering of Markdown files</li>
<li>Static site generation</li>
<li>Pre-rendering at build time</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export function createMarkdownEnvironment(options: ResolvedOptions): EnvironmentOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/environment.ts#L33-L80" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">options</code>
    <code class="ox-api-entry__param-type">ResolvedOptions</code>
  </div>
  
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">EnvironmentOptions</code>
  
</div>
</div>
<div class="ox-api-entry__section ox-api-entry__section--examples">
<h4>Examples</h4>
<div class="ox-api-entry__example">
<div class="ox-api-entry__example-heading">Example 1</div>
<pre><code class="language-ts">// In your vite.config.ts
export default defineConfig({
  environments: {
    markdown: createMarkdownEnvironment({
      srcDir: &#39;content&#39;,
      gfm: true,
    }),
  },
});</code></pre>
</div>
</div>
  </div>
</details>

<details id="createtransformoptions" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">createTransformOptions(ctx: EnvironmentTransformContext, options: ResolvedOptions): ResolvedOptions</code><span class="ox-api-entry__description">Creates environment-aware transform options.</span><span class="ox-api-entry__meta"><span class="ox-api-badge">2 params</span><span class="ox-api-badge">returns ResolvedOptions</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Creates environment-aware transform options.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export function createTransformOptions(ctx: EnvironmentTransformContext, options: ResolvedOptions): ResolvedOptions</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/environment.ts#L113-L123" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">ctx</code>
    <code class="ox-api-entry__param-type">EnvironmentTransformContext</code>
  </div>
  
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">options</code>
    <code class="ox-api-entry__param-type">ResolvedOptions</code>
  </div>
  
</li>
</ul>
</div>
<div class="ox-api-entry__section ox-api-entry__section--returns">
<h4>Returns</h4>
<div class="ox-api-entry__return">
  <code class="ox-api-entry__return-type">ResolvedOptions</code>
  
</div>
</div>
  </div>
</details>

<details id="environmenttransformcontext" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">interface</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">EnvironmentTransformContext</code><span class="ox-api-entry__description">Environment-specific module transformer. This is called during the transform ph…</span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Environment-specific module transformer.</p>
<p>This is called during the transform phase to process Markdown files within the environment context.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export interface EnvironmentTransformContext</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/environment.ts#L88-L108" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
  </div>
</details>

<details id="prerender" class="ox-api-entry">
  <summary><span class="ox-api-entry__kind">fn</span><span class="ox-api-entry__summary-main"><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">prerender(files: string[], _options: ResolvedOptions): Promise&lt;Map&lt;string, string&gt;&gt;</code><span class="ox-api-entry__description">Runs pre-render for SSG. This function is called during build to pre-render all…</span><span class="ox-api-entry__meta"><span class="ox-api-badge">2 params</span><span class="ox-api-badge">returns Promise</span></span></span></summary>
  <div class="ox-api-entry__body">
<div class="ox-api-entry__prose">
<p>Runs pre-render for SSG.</p>
<p>This function is called during build to pre-render all Markdown files.</p>
</div>
<div class="ox-api-entry__section ox-api-entry__section--signature">
<h4>Signature</h4>
<pre><code class="language-typescript">export async function prerender(files: string[], _options: ResolvedOptions): Promise&lt;Map&lt;string, string&gt;&gt;</code></pre>
</div>
<p class="ox-api-entry__source"><a class="ox-api-entry__source-link" href="https://github.com/ubugeeei-prod/ox-content/blob/main/npm/vite-plugin-ox-content/src/environment.ts#L130-L143" target="_blank" rel="noopener noreferrer">View source<span class="ox-api-entry__source-icon" aria-hidden="true"></span></a></p>
<div class="ox-api-entry__section ox-api-entry__section--params">
<h4>Parameters</h4>
<ul class="ox-api-entry__params">
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">files</code>
    <code class="ox-api-entry__param-type">string[]</code>
  </div>
  
</li>
<li class="ox-api-entry__param">
  <div class="ox-api-entry__param-heading">
    <code class="ox-api-entry__param-name">_options</code>
    <code class="ox-api-entry__param-type">ResolvedOptions</code>
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
  </div>
</details>
