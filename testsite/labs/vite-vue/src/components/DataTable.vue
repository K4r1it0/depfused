<template>
  <div class="data-table">
    <h3>{{ title }}</h3>
    <table>
      <thead>
        <tr>
          <th v-for="col in columns" :key="col">{{ col }}</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="(row, idx) in rows" :key="idx">
          <td v-for="col in columns" :key="col">{{ row[col] }}</td>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<script setup>
import { defineProps } from 'vue'

const props = defineProps({
  title: { type: String, default: 'Data Table' },
  columns: { type: Array, default: () => [] },
  rows: { type: Array, default: () => [] }
})
</script>

<style scoped>
.data-table {
  margin: 16px 0;
}
.data-table table {
  width: 100%;
  border-collapse: collapse;
}
.data-table th, .data-table td {
  border: 1px solid #ddd;
  padding: 8px 12px;
  text-align: left;
}
.data-table th {
  background: #f5f5f5;
  font-weight: 600;
}
</style>
