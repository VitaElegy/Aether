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

const pluginStore = usePluginStore()

// 1. Core
pluginStore.registerPlugin(ArticlesPlugin)
pluginStore.registerPlugin(KnowledgePlugin)

// 2. Memos (Standard + Legacy Alias)
console.log('[Main] Registering MemosPlugin...');
pluginStore.registerPlugin(MemosPlugin) // id: 'memo'
pluginStore.registerPlugin(Object.assign({}, MemosPlugin, { id: 'memo_std' }))
pluginStore.registerPlugin(Object.assign({}, MemosPlugin, { id: 'memo_v1' }))

// 3. Vocabulary (English)
pluginStore.registerPlugin(VocabularyPlugin) // id: 'vocabulary'
pluginStore.registerPlugin(Object.assign({}, VocabularyPlugin, { id: 'vocabulary_std' }))

// 4. English Analysis (Layouts alias to Vocabulary or ArticleAnalysis)
// "English Analysis" layout usually implies Article Analysis capabilities.
// If the user wants the "Vocabulary" module to handle it (which has tabs), we point there.
// If we want raw ArticleAnalysisModule, we point there.
// Based on VocabularyModule having 'articles' tab, it acts as a super-set.
pluginStore.registerPlugin(ArticleAnalysisPlugin) // id: 'article-analysis'
// Alias 'english_v1' and 'english' to Vocabulary so they get the full UI
pluginStore.registerPlugin(Object.assign({}, VocabularyPlugin, { id: 'english_v1_std' }))
pluginStore.registerPlugin(Object.assign({}, VocabularyPlugin, { id: 'english_v1' }))
pluginStore.registerPlugin(Object.assign({}, VocabularyPlugin, { id: 'english' }))
// Robust Aliases for legacy/deviant IDs
pluginStore.registerPlugin(Object.assign({}, ArticleAnalysisPlugin, { id: 'article_analysis' }))
pluginStore.registerPlugin(Object.assign({}, ArticleAnalysisPlugin, { id: 'english_analysis' }))
pluginStore.registerPlugin(Object.assign({}, ArticleAnalysisPlugin, { id: 'english analysis' }))

// 5. Admin
pluginStore.registerPlugin(AdminPlugin)

// 6. Math
pluginStore.registerPlugin(MathPlugin) // id: 'math'
pluginStore.registerPlugin(Object.assign({}, MathPlugin, { id: 'math_v1' }))
pluginStore.registerPlugin(Object.assign({}, MathPlugin, { id: 'math_v1_std' }))
pluginStore.registerPlugin(Object.assign({}, MathPlugin, { id: 'math_v3' }))
pluginStore.registerPlugin(Object.assign({}, MathPlugin, { id: 'math_std' }))

// 7. VRKB
pluginStore.registerPlugin(VrkbPlugin) // id: 'vrkb'
pluginStore.registerPlugin(Object.assign({}, VrkbPlugin, { id: 'vrkb_std' }))
pluginStore.registerPlugin(Object.assign({}, VrkbPlugin, { id: 'vulnerability_research' })) // Alias for template ID

import { VrkbManifest } from './components/self-space/modules/vrkb/manifest'
pluginStore.registerManifest(VrkbManifest)

// 8. PRKB (Paper Research)
import { PrkbPlugin } from './plugins/prkb'
pluginStore.registerPlugin(PrkbPlugin)


app.mount('#app')
