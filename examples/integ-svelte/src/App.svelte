<script>
  // Import Markdown document as Svelte component
  import IndexDoc from '../docs/index.md';
  import PageTemplate from '../docs/page-template.mdx';

  const posts = [
    { title: 'Ox Content with Svelte', href: '/posts/ox-content-svelte' },
    { title: 'MDX page templates', href: '/posts/mdx-page-templates' },
  ];
</script>

<div class="app">
  <header>
    <h1>🦀 Ox Content + Svelte</h1>
    <p>Embed Svelte components directly in Markdown</p>
  </header>
  <main>
    <IndexDoc />
    <PageTemplate title="Blog" pathname="/blog" items={posts} />
  </main>
</div>

<style>
  :global(*) {
    margin: 0;
    padding: 0;
    box-sizing: border-box;
  }

  :global(body) {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    background: #0f0f0f;
    color: #e5e5e5;
    min-height: 100vh;
  }

  .app {
    max-width: 900px;
    margin: 0 auto;
    padding: 2rem;
  }

  header {
    text-align: center;
    margin-bottom: 3rem;
    padding-bottom: 2rem;
    border-bottom: 1px solid #333;
  }

  header h1 {
    color: #ff6b35;
    font-size: 2.5rem;
    margin-bottom: 0.5rem;
  }

  header p {
    color: #888;
    font-size: 1.1rem;
  }

  main {
    line-height: 1.8;
  }

  :global(main h1),
  :global(main h2),
  :global(main h3) {
    color: #ff6b35;
    margin: 2rem 0 1rem;
  }

  :global(main h1) { font-size: 2rem; }
  :global(main h2) { font-size: 1.5rem; }
  :global(main h3) { font-size: 1.25rem; }

  :global(main p) {
    margin: 1rem 0;
  }

  :global(main code) {
    background: #1a1a1a;
    padding: 0.2rem 0.4rem;
    border-radius: 4px;
    font-family: 'Fira Code', monospace;
  }

  :global(main pre) {
    background: #1a1a1a;
    padding: 1rem;
    border-radius: 8px;
    overflow-x: auto;
    margin: 1rem 0;
  }

  :global(main pre code) {
    background: transparent;
    padding: 0;
  }

  :global(main ul),
  :global(main ol) {
    margin: 1rem 0;
    padding-left: 2rem;
  }

  :global(main li) {
    margin: 0.5rem 0;
  }
</style>
