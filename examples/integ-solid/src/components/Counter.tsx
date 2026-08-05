import { createSignal } from "solid-js";

export default function Counter(props: { start?: number }) {
  const [count, setCount] = createSignal(props.start ?? 0);

  return (
    <div class="counter">
      <button onClick={() => setCount(count() - 1)}>-</button>
      <span class="count">{count()}</span>
      <button onClick={() => setCount(count() + 1)}>+</button>
    </div>
  );
}
