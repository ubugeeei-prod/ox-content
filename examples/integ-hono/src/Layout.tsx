/// <reference types="@ox-content/vite-plugin-hono/virtual" />
import "./layout.css";
import type { Child } from "hono/jsx";
import Footer, { frontmatter } from "../snippets/footer.md";
import Badge from "../snippets/badge.mdx";

export default function Layout({ children, head }: { children: Child; head: Child }) {
  return (
    <html>
      <head>{head}</head>
      <body>
        <main>{children}</main>
        <aside data-imported>
          <Footer />
          <Badge />
          <pre data-yaml>{JSON.stringify(frontmatter)}</pre>
        </aside>
      </body>
    </html>
  );
}
