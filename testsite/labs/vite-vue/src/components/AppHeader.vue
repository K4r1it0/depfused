<template>
  <header class="app-header">
    <h1>{{ title }}</h1>
    <span class="version-badge">v{{ designSystemVersion }}</span>
  </header>
</template>

<script setup>
import { ref } from 'vue'
import { VERSION as designSystemVersion } from '@xq9zk7823/design-system'

const title = ref('AcmeCorp Dashboard')
</script>

<style scoped>
.app-header {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 16px 0;
}
.app-header h1 {
  margin: 0;
  font-size: 24px;
  color: #1a1a2e;
}
.version-badge {
  background: #4a90d9;
  color: white;
  padding: 2px 8px;
  border-radius: 12px;
  font-size: 12px;
}
</style>
