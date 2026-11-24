<!--
  Vue SFC OG Image Template

  Receives page metadata as props and renders HTML for OG image generation.
  Rendered via Vue SSR (createSSRApp + renderToString) at build time.
-->
<script setup lang="ts">
const props = defineProps<{
  title: string;
  description?: string;
  siteName?: string;
  author?: string;
  tags?: string[];
  category?: string;
  coverColor?: string;
}>();

const color = props.coverColor ?? "#6366f1";
</script>

<template>
  <div class="og">
    <div
      class="accent-bar"
      :style="{ background: `linear-gradient(90deg, ${color}, ${color}cc)` }"
    />
    <div class="body">
      <span v-if="category" class="category" :style="{ background: color }">
        {{ category }}
      </span>
      <h1 class="title">{{ title }}</h1>
      <p v-if="description" class="description">{{ description }}</p>
      <div v-if="tags?.length" class="tags">
        <span v-for="tag in tags" :key="tag" class="tag" :style="{ color }"> #{{ tag }} </span>
      </div>
    </div>
    <div class="footer">
      <div v-if="author" class="author">
        <div class="avatar" :style="{ background: color }">
          {{ author[0].toUpperCase() }}
        </div>
        <span class="author-name">{{ author }}</span>
      </div>
      <span v-if="siteName" class="site-name">{{ siteName }}</span>
    </div>
  </div>
</template>

<style scoped>
.og {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  background: #ffffff;
  font-family: system-ui, -apple-system, sans-serif;
}

.accent-bar {
  height: 8px;
}

.body {
  flex: 1;
  display: flex;
  flex-direction: column;
  justify-content: center;
  padding: 48px 64px;
  gap: 20px;
}

.category {
  display: inline-block;
  color: #fff;
  padding: 6px 16px;
  border-radius: 20px;
  font-size: 14px;
  font-weight: 600;
  letter-spacing: 0.5px;
  text-transform: uppercase;
  align-self: flex-start;
}

.title {
  font-size: 52px;
  font-weight: 800;
  color: #0f172a;
  line-height: 1.2;
  margin: 0;
}

.description {
  font-size: 22px;
  color: #475569;
  line-height: 1.5;
  margin: 0;
}

.tags {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.tag {
  font-size: 14px;
  font-weight: 500;
}

.footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 24px 64px;
  border-top: 1px solid #e2e8f0;
}

.author {
  display: flex;
  align-items: center;
  gap: 12px;
}

.avatar {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #fff;
  font-weight: 700;
  font-size: 18px;
}

.author-name {
  font-size: 16px;
  color: #64748b;
  font-weight: 500;
}

.site-name {
  font-size: 16px;
  color: #94a3b8;
  font-weight: 600;
}
</style>
