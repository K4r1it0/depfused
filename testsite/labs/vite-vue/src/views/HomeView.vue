<template>
  <div class="home-view">
    <h2>Welcome to AcmeCorp</h2>
    <p>Internal application dashboard. Select a section from the navigation above.</p>
    <div class="stats-grid">
      <div class="stat-card" v-for="stat in stats" :key="stat.label">
        <span class="stat-value">{{ stat.value }}</span>
        <span class="stat-label">{{ stat.label }}</span>
      </div>
    </div>
    <p class="ds-info">Design System Button: {{ buttonName }}</p>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { capitalize } from 'lodash-es'
import { Button } from '@xq9zk7823/design-system'
import { init as initDesignSystem } from '@xq9zk7823/design-system'

const buttonName = ref(Button.name)

const stats = ref([
  { label: capitalize('active users'), value: '1,234' },
  { label: capitalize('api calls today'), value: '56,789' },
  { label: capitalize('uptime'), value: '99.9%' },
  { label: capitalize('deployments'), value: '42' }
])

onMounted(() => {
  initDesignSystem({ page: 'home' })
})
</script>

<style scoped>
.home-view h2 {
  color: #1a1a2e;
}
.stats-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 16px;
  margin-top: 24px;
}
.stat-card {
  background: #f8f9fa;
  border-radius: 8px;
  padding: 24px;
  text-align: center;
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.stat-value {
  font-size: 28px;
  font-weight: 700;
  color: #4a90d9;
}
.stat-label {
  font-size: 14px;
  color: #666;
}
.ds-info {
  margin-top: 16px;
  color: #999;
  font-size: 12px;
}
</style>
