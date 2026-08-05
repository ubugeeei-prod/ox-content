// Import Markdown document as a Solid component
import IndexDoc from "../docs/index.md";

export default function App() {
  return (
    <div class="app">
      <header>
        <h1>🦀 Ox Content + Solid</h1>
        <p>Embed Solid components directly in Markdown</p>
      </header>
      <main>
        <IndexDoc />
      </main>
    </div>
  );
}
