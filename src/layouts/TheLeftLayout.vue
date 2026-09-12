<script setup lang="ts">
import TheConfigLayout from "./TheConfigLayout.vue";
import TheInfoPanel from "@/components/panels/TheInfoPanel.vue";
</script>

<template>
  <div class="left-container">
    <TheInfoPanel class="info-panel" />
    <TheConfigLayout class="config-panel" />
  </div>
</template>

<style scoped>
.left-container {
  display: flex;
  flex-direction: column;
  background-color: transparent;
}

.info-panel {
  height: 45%;
}

.config-panel {
  height: 50%;
}
</style>
