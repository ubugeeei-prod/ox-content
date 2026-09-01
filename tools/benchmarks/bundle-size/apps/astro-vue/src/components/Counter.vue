<script setup lang="ts">
import { ref } from "vue";

const count = ref(0);
</script>

<template>
  <div class="counter">
    <button @click="count--">-</button>
    <span>{{ count }}</span>
    <button @click="count++">+</button>
  </div>
</template>

<style scoped>
.counter {
  display: flex;
  gap: 1rem;
  align-items: center;
}
button {
  padding: 0.5rem 1rem;
}
</style>
