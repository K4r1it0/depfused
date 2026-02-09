<template>
  <footer class="app-footer">
    <p>{{ footerText }}</p>
  </footer>
</template>

<script setup>
import { computed } from 'vue'
import internalUtils from 'company-internal-utils'

const footerText = computed(() => {
  return internalUtils.format('  AcmeCorp Internal Dashboard - Confidential  ')
})
</script>

<style scoped>
.app-footer {
  margin-top: 40px;
  padding: 16px 0;
  border-top: 1px solid #e0e0e0;
  text-align: center;
  color: #888;
  font-size: 14px;
}
</style>
