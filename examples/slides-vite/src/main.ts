const app = document.querySelector<HTMLDivElement>("#app");

if (app) {
  app.innerHTML = `
    <main style="font-family: system-ui, sans-serif; padding: 24px; line-height: 1.6;">
      <h1>ox-content slides example</h1>
      <p>Open <code>/slides/</code> to view the deck.</p>
      <p>Open <code>/slides/presenter/1/</code> for presenter mode.</p>
      <p>Open <code>/slides/editor/</code> for the dev editor.</p>
    </main>
  `;
}
