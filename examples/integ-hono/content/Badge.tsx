import { useState } from "hono/jsx";
export function Badge({ label }: { label: string }) {
  const [count, setCount] = useState(0);
  return (
    <button data-badge onClick={() => setCount(count + 1)}>
      {label}: {count}
    </button>
  );
}
