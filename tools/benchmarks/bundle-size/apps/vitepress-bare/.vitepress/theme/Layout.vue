<script setup>
import { useData } from "vitepress";

const { page, frontmatter } = useData();
</script>

<template>
  <div>
    <Content />
  </div>
</template>
