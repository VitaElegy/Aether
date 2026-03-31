import { createApp } from 'vue'
import { createPinia } from 'pinia'
import TDesign from 'tdesign-vue-next';
import { MotionPlugin } from '@vueuse/motion'
import router from './router'
import App from './App.vue'

import 'tdesign-vue-next/es/style/index.css';
import 'remixicon/fonts/remixicon.css' // Import RemixIcon
import './style.css'
import axios from 'axios'

// --- Axios Configuration for Mobile ---
// When running as an APK (production build), we can't use the Vite proxy.
// We must point to the backend server directly.
// 10.0.2.2 is the special alias for "localhost" inside the Android Emulator.
// If testing on a REAL PHONE, replace this with your computer's LAN IP (e.g., http://192.168.1.5:3000)
if (import.meta.env.PROD) {
    axios.defaults.baseURL = 'http://10.0.2.2:3000';
    console.log('[Main] Axios Base URL set to:', axios.defaults.baseURL);
}

const app = createApp(App)

app.use(createPinia())
app.use(router)
app.use(TDesign)
app.use(MotionPlugin) // Register Motion

// --- Global Error Handling ---
app.config.errorHandler = (err, instance, info) => {
    console.error('[Global Vue Error]', err);
    console.error('Instance:', instance);
    console.error('Info:', info);

    // Optional: Dispatch event for Orchestrator to pick up if it's active
    // window.dispatchEvent(new CustomEvent('aether-error', { detail: { error: err } }));
};

window.addEventListener('unhandledrejection', (event) => {
    console.error('[Global Unhandled Rejection]', event.reason);
    // event.preventDefault(); // Prevent default console error? No, let it show.
});

// Configure standard window error for syntax/script errors
window.onerror = (message, source, lineno, colno, error) => {
    console.error('[Global Window Error]', { message, source, lineno, colno, error });
};

// Register Core Plugins
import { usePluginStore } from './stores/plugins'
import { ArticlesPlugin } from './plugins/articles'
import { KnowledgePlugin } from './plugins/knowledge'
import { MemosPlugin } from './plugins/memos/index'
// import { VocabularyPlugin } from './plugins/vocabulary' // Re-import below
import { VrkbPlugin } from './plugins/vrkb'
import { AdminPlugin } from './plugins/admin'
import { MathPlugin } from './plugins/math'
import { VocabularyPlugin } from './plugins/vocabulary'
import { ArticleAnalysisPlugin } from './plugins/article-analysis'
import { PrkbPlugin } from './plugins/prkb'
import { AssetsPlugin } from './plugins/assets'

const pluginStore = usePluginStore()
console.log('[Main] Registering canonical plugins...');
[
    ArticlesPlugin,
    KnowledgePlugin,
    MemosPlugin,
    VocabularyPlugin,
    ArticleAnalysisPlugin,
    AdminPlugin,
    MathPlugin,
    VrkbPlugin,
    PrkbPlugin,
    AssetsPlugin,
].forEach((plugin) => pluginStore.registerPlugin(plugin));

import { VrkbManifest } from './components/self-space/modules/vrkb/manifest'
pluginStore.registerManifest(VrkbManifest)


app.mount('#app')
