<script>
  let { start = 0 } = $props();
  let count = $state(0);

  $effect(() => {
    count = start;
  });
</script>

<div class="counter">
  <button onclick={() => count--}>-</button>
  <span class="count">{count}</span>
  <button onclick={() => count++}>+</button>
</div>

<style>
  .counter {
    display: inline-flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem;
    border: 1px solid #333;
    border-radius: 0.5rem;
    background: #1a1a1a;
  }

  .count {
    min-width: 2rem;
    text-align: center;
    font-weight: 600;
    color: #fff;
  }

  button {
    padding: 0.25rem 0.75rem;
    border: none;
    border-radius: 0.25rem;
    background: #ff6b35;
    color: white;
    cursor: pointer;
  }

  button:hover {
    background: #e85d2c;
  }
</style>
