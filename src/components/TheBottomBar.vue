<script setup lang="ts">
defineProps<{
  version: string;
  author: string;
}>();
</script>

<template>
  <div class="version-info">
    <p>by {{ author }} v{{ version }}</p>
  </div>
</template>

<style scoped>
.version-info {
  padding-right: 2.5vw;
  background-color: transparent;
  display: flex;
  align-items: center;
  justify-content: flex-end;
}
</style>
