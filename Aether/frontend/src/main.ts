import { createApp } from 'vue'
import { createPinia } from 'pinia'
import TDesign from 'tdesign-vue-next';
import { MotionPlugin } from '@vueuse/motion'
import router from './router'
import App from './App.vue'

import 'tdesign-vue-next/es/style/index.css';
import 'remixicon/fonts/remixicon.css' // Import RemixIcon
import './style.css'

const app = createApp(App)

app.use(createPinia())
app.use(router)
app.use(TDesign)
app.use(MotionPlugin) // Register Motion

// Register Core Plugins
import { usePluginStore } from './stores/plugins'
import { ArticlesPlugin } from './plugins/articles'
import { KnowledgePlugin } from './plugins/knowledge'
import { MemosPlugin } from './plugins/memos'
import { VocabularyPlugin } from './plugins/vocabulary'
import { VrkbPlugin } from './plugins/vrkb'

const pluginStore = usePluginStore()
pluginStore.registerPlugin(ArticlesPlugin)
pluginStore.registerPlugin(KnowledgePlugin)
console.log('[Main] Registering MemosPlugin...');
pluginStore.registerPlugin(MemosPlugin)
pluginStore.registerPlugin(VocabularyPlugin)
pluginStore.registerPlugin(VrkbPlugin)

app.mount('#app')
