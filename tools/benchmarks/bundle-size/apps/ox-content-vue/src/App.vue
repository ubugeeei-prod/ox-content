<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import index from "../../../content/index.md";
import gettingStarted from "../../../content/getting-started.md";
import api from "../../../content/api.md";
import examples from "../../../content/examples.md";

const pages = {
  index,
  "getting-started": gettingStarted,
  api,
  examples,
};

const currentPage = ref("index");

const content = computed(() => {
  const page = pages[currentPage.value as keyof typeof pages];
  return page?.html || "";
});

function navigate(name: string) {
  currentPage.value = name;
}

onMounted(() => {
  document.addEventListener("click", (e) => {
    const target = e.target as HTMLElement;
    if (target.tagName === "A") {
      const href = target.getAttribute("href");
      if (href?.startsWith("./")) {
        e.preventDefault();
        const pageName = href.replace("./", "").replace(".md", "");
        navigate(pageName);
      }
    }
  });
});
</script>

<template>
  <div class="app">
    <nav>
      <button @click="navigate('index')">Home</button>
      <button @click="navigate('getting-started')">Getting Started</button>
      <button @click="navigate('api')">API</button>
      <button @click="navigate('examples')">Examples</button>
    </nav>
    <main v-html="content"></main>
  </div>
</template>
