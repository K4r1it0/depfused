<template>
  <div class="dashboard-view">
    <h2>Dashboard</h2>
    <p>API Status: {{ apiStatus }}</p>
    <DataTable
      title="Recent API Calls"
      :columns="tableColumns"
      :rows="tableRows"
    />
    <button @click="fetchData">Refresh Data</button>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { sortBy } from 'lodash-es'
import axios from 'axios'
import DataTable from '../components/DataTable.vue'

const apiStatus = ref('Loading...')
const tableColumns = ref(['endpoint', 'method', 'status', 'latency'])
const tableRows = ref([])

async function fetchData() {
  // Dynamic import of @xq9zk7823/api-client (lazy loaded)
  const apiClient = await import('@xq9zk7823/api-client')
  apiClient.init({ baseUrl: 'https://api.internal.acmecorp.com' })
  apiStatus.value = 'Connected (v' + apiClient.VERSION + ')'

  // Simulate some data rows
  const rawRows = [
    { endpoint: '/users', method: 'GET', status: 200, latency: '45ms' },
    { endpoint: '/orders', method: 'POST', status: 201, latency: '120ms' },
    { endpoint: '/products', method: 'GET', status: 200, latency: '32ms' },
    { endpoint: '/auth/login', method: 'POST', status: 200, latency: '89ms' },
    { endpoint: '/analytics', method: 'GET', status: 200, latency: '67ms' },
  ]
  tableRows.value = sortBy(rawRows, ['latency'])
}

onMounted(() => {
  fetchData()
})
</script>

<style scoped>
.dashboard-view h2 {
  color: #1a1a2e;
}
.dashboard-view button {
  margin-top: 16px;
  padding: 8px 24px;
  background: #4a90d9;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
}
.dashboard-view button:hover {
  background: #357abd;
}
</style>
