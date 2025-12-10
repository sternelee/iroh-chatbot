import { createApp } from 'vue'
import '@mcp-ui/client/ui-resource-renderer.wc.js'
import { initialize } from './tauri-fetch'

import App from './App.vue'
import './style.css'

initialize()

createApp(App).mount('#app')
