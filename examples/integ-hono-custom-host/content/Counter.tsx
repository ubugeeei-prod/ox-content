import { useState, useEffect, type Child } from "hono/jsx";
import { useFrontmatter } from "@ox-content/vite-plugin-hono";
import "./counter.css";

export default function Counter({ initial = 2, children }: { initial?: number; children?: Child }) {
  const [count, setCount] = useState(initial);
  const { title } = useFrontmatter<{ title: string }>();
  useEffect(() => {
    window.mounts = (window.mounts ?? 0) + 1;
    return () => {
      window.cleanups = (window.cleanups ?? 0) + 1;
    };
  }, []);
  return (
    <section class="counter">
      <p data-title>{title}</p>
      <button data-counter onClick={() => setCount(count + 1)}>
        Count: {count}
      </button>
      <div data-slot>{children}</div>
    </section>
  );
}
