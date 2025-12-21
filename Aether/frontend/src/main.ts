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

app.mount('#app')
