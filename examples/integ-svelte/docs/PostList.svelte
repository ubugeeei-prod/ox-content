<script lang="ts">
  interface PostItem {
    title: string;
    href: string;
  }

  let { items = [] }: { items?: PostItem[] } = $props();
</script>

<ul class="post-list">
  {#each items as item}
    <li>
      <a href={item.href}>{item.title}</a>
    </li>
  {/each}
</ul>

<style>
  .post-list {
    display: grid;
    gap: 0.5rem;
    margin: 1rem 0;
    padding-left: 1.25rem;
  }

  a {
    color: #8bd3ff;
  }
</style>
