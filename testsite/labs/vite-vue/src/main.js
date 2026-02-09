import { createApp } from 'vue'
import { debounce } from 'lodash-es'
import { init as initDesignSystem, VERSION as dsVersion } from '@xq9zk7823/design-system'
import { init as initConfig } from '@xq9zk7823/config-service'
import { init as initI18n } from '@xq9zk7823/i18n-utils'
import internalUtils from 'company-internal-utils'
import App from './App.vue'
import router from './router/index.js'

// Initialize internal packages
initDesignSystem({ theme: 'dark' })
initConfig({ env: 'production' })
initI18n({ locale: 'en-US' })

console.log('Design System Version:', dsVersion)
internalUtils.log('App bootstrapping...')

// Use lodash debounce for a global resize handler
const onResize = debounce(() => {
  console.log('Window resized')
}, 300)
window.addEventListener('resize', onResize)

const app = createApp(App)
app.use(router)
app.mount('#app')
