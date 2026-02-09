<template>
  <div class="settings-view">
    <h2>Settings</h2>
    <div class="setting-group">
      <label>Locale:</label>
      <select v-model="locale" @change="updateLocale">
        <option value="en-US">English (US)</option>
        <option value="fr-FR">French</option>
        <option value="de-DE">German</option>
        <option value="ja-JP">Japanese</option>
      </select>
    </div>
    <div class="setting-group">
      <label>Theme:</label>
      <select v-model="theme">
        <option value="light">Light</option>
        <option value="dark">Dark</option>
      </select>
    </div>
    <div class="setting-group">
      <label>Config Status:</label>
      <span>{{ configStatus }}</span>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { init as initI18n } from '@xq9zk7823/i18n-utils'
import { init as initConfig, VERSION as configVersion } from '@xq9zk7823/config-service'
import internalUtils from 'company-internal-utils'

const locale = ref('en-US')
const theme = ref('light')
const configStatus = ref('Initializing...')

function updateLocale() {
  initI18n({ locale: locale.value })
  internalUtils.log('Locale changed to: ' + locale.value)
}

onMounted(() => {
  const result = initConfig({ env: 'production', region: 'us-east-1' })
  if (result.ready) {
    configStatus.value = 'Config Service v' + configVersion + ' ready'
  }
})
</script>

<style scoped>
.settings-view h2 {
  color: #1a1a2e;
}
.setting-group {
  display: flex;
  align-items: center;
  gap: 12px;
  margin: 16px 0;
}
.setting-group label {
  font-weight: 600;
  min-width: 120px;
}
.setting-group select {
  padding: 6px 12px;
  border: 1px solid #ddd;
  border-radius: 4px;
  font-size: 14px;
}
</style>
