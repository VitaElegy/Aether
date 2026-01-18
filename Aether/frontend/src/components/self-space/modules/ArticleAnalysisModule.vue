
<script setup lang="ts">
import { ref, onMounted, computed, watch, onUnmounted } from 'vue';
import axios from 'axios';
import { useDebounceFn } from '@vueuse/core';
import type { MessagePlugin } from 'tdesign-vue-next';
import EnglishArticleAnalyzer from '@/components/english/EnglishArticleAnalyzer.vue';
import EnglishEditor from '@/components/english/EnglishEditor.vue';

const props = defineProps<{
    headless?: boolean;
}>();

// --- Types ---
interface ArticleMeta {
    text: string;
    background?: string;
    references?: Array<{ title: string; url: string }>;
}

interface EnglishArticle {
    id: string;
    title: string;
    created_at: string;
    updated_at: string;
    status: 'Draft' | 'Published' | 'Archived';
    category: string;
    body: ArticleMeta; // We store JSON in the body string
    author_name?: string;
}

// --- State ---
const viewMode = ref<'list' | 'editor' | 'reader'>('list');
const articles = ref<EnglishArticle[]>([]);
const isLoading = ref(false);
const searchQuery = ref('');

// Editor State
const editorForm = ref({
    title: '',
    text: '',
    background: '',
    references: [] as Array<{ title: string; url: string }>
});

// Reader State
const currentArticle = ref<EnglishArticle | null>(null);
const showHistory = ref(false); // Toggle for Git-like history sidebar
const showComments = ref(false); // Toggle for Comments

// --- API ---
const fetchArticles = async () => {
    isLoading.value = true;
    try {
        const res = await axios.get('/api/content', {
            params: {
                category: 'English Analysis',
                limit: 50
            }
        });
        
        // Parse the body as JSON
        articles.value = res.data.map((item: any) => {
            let parsedBody: ArticleMeta = { text: '' };
            try {
                // The API returns body as { Markdown: "string" } or just "string" depending on mapping
                // Based on backend implementation: `body: ContentBody::Markdown(payload.body)`
                // API Response `item.body` will be the Enum object `{ "Markdown": "..." }` or similar.
                // Let's inspect carefully. Backend `ContentBody` serializes as untagged? 
                // Wait, `#[serde(tag = "type", content = "data")]`.
                // So it looks like `{ type: "Markdown", data: "..." }`.
                
                const rawBody = item.body?.data || ''; 
                // We stored a JSON string INSIDE the Markdown string.
                if (rawBody.startsWith('{')) {
                    parsedBody = JSON.parse(rawBody);
                } else {
                    parsedBody = { text: rawBody };
                }
            } catch (e) {
                console.warn('Failed to parse article body', e);
            }

            return {
                id: item.id,
                title: item.title,
                created_at: item.created_at,
                updated_at: item.updated_at,
                status: item.status,
                category: item.category,
                body: parsedBody,
                author_name: item.author_name
            };
        });
    } catch (e) {
        console.error('Failed to fetch articles', e);
    } finally {
        isLoading.value = false;
    }
};

const saveArticle = async () => {
    if (!editorForm.value.title || !editorForm.value.text) return;

    const payload = {
        title: editorForm.value.title,
        body: JSON.stringify({
            text: editorForm.value.text,
            background: editorForm.value.background,
            references: editorForm.value.references
        }),
        category: 'English Analysis', // Important filter
        tags: ['english-learning'],
        visibility: 'Private', // Default to private for personal study
        status: 'Draft' 
    };

    try {
        await axios.post('/api/content', payload);
        viewMode.value = 'list';
        fetchArticles();
        // Reset Form
        editorForm.value = { title: '', text: '', background: '', references: [] };
    } catch (e) {
        console.error('Failed to save article', e);
    }
};

const deleteArticle = async (id: string) => {
    if (!confirm('Are you sure you want to delete this analysis?')) return;
    try {
        await axios.delete(`/api/content/${id}`);
        fetchArticles();
    } catch (e) {
        console.error('Failed to delete', e);
    }
};

const openArticle = (article: EnglishArticle) => {
    currentArticle.value = article;
    viewMode.value = 'reader';
};

const handleEditorSave = async (formData: any) => {
    const payload = {
        title: formData.title,
        body: JSON.stringify({
            text: formData.text,
            background: formData.background,
            references: formData.references
        }),
        category: 'English Analysis', 
        tags: ['english-learning'],
        visibility: formData.status === 'Published' ? 'Public' : 'Private',
        status: formData.status
    };

    try {
        await axios.post('/api/content', payload);
        viewMode.value = 'list';
        fetchArticles();
        editorForm.value = { title: '', text: '', background: '', references: [] };
    } catch (e) {
        console.error('Failed to save article', e);
    }
};

import { useNavigationStore } from '@/stores/navigation';

const navStore = useNavigationStore();

// --- Lifecycle ---
onMounted(() => {
    fetchArticles();
    if (props.headless) {
        navStore.setCustomRight(true);
    }
});

onUnmounted(() => {
    // If headless, we should release it, BUT VocabularyModule handles the parent state.
    // However, if we switch viewMode in parent, this component might unmount?
    // Actually in VocabularyModule it is v-if="activeTab === 'articles'".
    // So when switching back to Vocabulary, this unmounts.
    // VocabularyModule watcher sets customRight(true) for vocabulary, so we are good.
    // But if we navigate away entirely? VocabularyModule unmounts and resets.
});

// --- Computed ---
const filteredArticles = computed(() => {
    // Stage 1: Absolute Safety Filter (Client-side Firewall)
    // Even if backend leaks data, we ignore anything that isn't explicitly English Analysis
    const safeList = articles.value.filter(a => a.category === 'English Analysis');

    // Stage 2: Search Query
    if (!searchQuery.value) return safeList;
    const q = searchQuery.value.toLowerCase();
    return safeList.filter(a => a.title.toLowerCase().includes(q));
});

</script>

<template>
    <div class="w-full h-full flex flex-col relative overflow-hidden">
        
        <!-- TOP NAV (Changes based on view) -->
        <div v-if="!headless" class="h-16 flex items-center justify-between px-8 border-b border-ink/5 bg-white/80 backdrop-blur z-20">
            <div class="flex items-center gap-4">
                <button 
                    v-if="viewMode !== 'list'" 
                    @click="viewMode = 'list'"
                    class="w-8 h-8 rounded-full hover:bg-ink/5 flex items-center justify-center transition-colors"
                >
                    <i class="ri-arrow-left-line text-lg text-ink/50"></i>
                </button>
                <h2 class="font-serif text-xl font-bold text-ink">
                    <span v-if="viewMode === 'list'">My Library</span>
                    <span v-else-if="viewMode === 'editor'">New Analysis</span>
                    <span v-else>{{ currentArticle?.title }}</span>
                </h2>
            </div>
            
            <div class="flex items-center gap-4">
                 <!-- Actions rendered inline for standalone -->
                 <div class="flex items-center gap-4">
                     <template v-if="viewMode === 'list'">
                        <div class="relative">
                            <i class="ri-search-line absolute left-3 top-1/2 -translate-y-1/2 text-ink/30"></i>
                            <input 
                                v-model="searchQuery"
                                class="pl-9 pr-4 py-1.5 bg-ink/5 rounded-full text-sm outline-none focus:ring-1 ring-accent/20 w-64 transition-all"
                                placeholder="Search articles..."
                            />
                        </div>
                        <button 
                            @click="viewMode = 'editor'"
                            class="px-4 py-1.5 bg-ink text-white rounded-full text-sm font-bold shadow-lg hover:shadow-xl hover:-translate-y-0.5 transition-all flex items-center gap-2"
                        >
                            <i class="ri-add-line"></i> New Analysis
                        </button>
                    </template>

                    <template v-if="viewMode === 'reader'">
                        <button 
                            @click="showHistory = !showHistory"
                            class="p-2 rounded hover:bg-ink/5 text-ink/40 hover:text-ink transition-colors relative"
                            title="History / Git Graph"
                        >
                            <i class="ri-git-branch-line text-lg"></i>
                        </button>
                        <button 
                            @click="showComments = !showComments"
                            class="p-2 rounded hover:bg-ink/5 text-ink/40 hover:text-ink transition-colors"
                            title="Comments"
                        >
                            <i class="ri-chat-1-line text-lg"></i>
                        </button>
                    </template>
                 </div>
            </div>
        </div>

        <!-- HEADLESS TELEPORT NAV -->
        <Teleport to="#nav-right-portal" v-if="headless">
             <div class="flex items-center gap-4 pointer-events-auto">
                 <template v-if="viewMode === 'list'">
                    <!-- Compact Search for Topbar -->
                    <div class="relative group">
                        <i class="ri-search-line absolute left-3 top-1/2 -translate-y-1/2 text-ink/30 group-focus-within:text-ink transition-colors"></i>
                        <input 
                            v-model="searchQuery"
                            class="pl-9 pr-4 py-1.5 bg-ink/5 focus:bg-white rounded-full text-xs font-medium outline-none border border-transparent focus:border-ink/10 focus:shadow-sm w-48 transition-all"
                            placeholder="Search..."
                        />
                    </div>
                    <button 
                        @click="viewMode = 'editor'"
                        class="text-xs font-bold uppercase tracking-widest text-ink/40 hover:text-ink transition-colors"
                    >
                        NEW ANALYSIS
                    </button>
                 </template>

                 <template v-if="viewMode === 'reader'">
                     <button @click="viewMode = 'list'" class="text-xs font-bold uppercase tracking-widest text-ink/40 hover:text-ink mr-4">
                        BACK
                     </button>
                    <button 
                        @click="showHistory = !showHistory"
                        class="text-xs font-bold uppercase tracking-widest transition-colors mr-4"
                        :class="showHistory ? 'text-ink' : 'text-ink/40 hover:text-ink'"
                    >
                        HISTORY
                    </button>
                    <button 
                        @click="showComments = !showComments"
                        class="text-xs font-bold uppercase tracking-widest transition-colors"
                        :class="showComments ? 'text-ink' : 'text-ink/40 hover:text-ink'"
                    >
                        COMMENTS
                    </button>
                </template>
             </div>
        </Teleport>

        <!-- CONTENT AREA -->
        <div class="flex-1 overflow-hidden relative">
            
            <!-- VIEW: LIST -->
            <div v-if="viewMode === 'list'" class="w-full h-full overflow-y-auto p-8 custom-scrollbar">
                <div v-if="isLoading" class="flex justify-center pt-20">
                    <i class="ri-loader-4-line animate-spin text-3xl text-ink/20"></i>
                </div>
                
                <div v-else-if="filteredArticles.length === 0" class="flex flex-col items-center justify-center h-full text-ink/30 gap-4">
                    <div class="w-20 h-20 rounded-full bg-ink/5 flex items-center justify-center">
                        <i class="ri-book-open-line text-4xl"></i>
                    </div>
                    <p class="font-serif italic">No analysis found. Start your first journey.</p>
                </div>

                <div v-else class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 max-w-7xl mx-auto">
                    <div 
                        v-for="article in filteredArticles" 
                        :key="article.id"
                        class="group relative aspect-[4/3] bg-white rounded-2xl shadow-sm hover:shadow-xl transition-all duration-500 overflow-hidden border border-ink/5 cursor-pointer"
                        @click="openArticle(article)"
                    >
                        <!-- Background -->
                        <div class="absolute inset-0 bg-ink/5">
                            <img 
                                v-if="article.body.background" 
                                :src="article.body.background" 
                                class="w-full h-full object-cover transition-transform duration-700 group-hover:scale-110"
                            />
                            <div class="absolute inset-0 bg-gradient-to-t from-black/80 via-black/20 to-transparent opacity-60 group-hover:opacity-80 transition-opacity"></div>
                        </div>

                        <!-- Content -->
                        <div class="absolute inset-0 p-6 flex flex-col justify-end text-white">
                            <h3 class="text-2xl font-serif font-bold leading-tight mb-2 drop-shadow-md">{{ article.title }}</h3>
                            <div class="flex items-center justify-between text-white/60 text-xs font-medium uppercase tracking-wider">
                                <span>{{ new Date(article.created_at).toLocaleDateString() }}</span>
                                <span v-if="article.status === 'Draft'" class="bg-yellow-500/20 px-2 py-0.5 rounded text-yellow-200">Analyzing</span>
                                <span v-else class="bg-green-500/20 px-2 py-0.5 rounded text-green-200">Done</span>
                            </div>
                        </div>
                        
                        <!-- Delete Action (Hover) -->
                        <button 
                            @click.stop="deleteArticle(article.id)"
                            class="absolute top-4 right-4 p-2 bg-black/20 backdrop-blur rounded-full text-white/50 hover:bg-black/40 hover:text-red-400 transition-all opacity-0 group-hover:opacity-100"
                        >
                            <i class="ri-delete-bin-line"></i>
                        </button>
                    </div>
                </div>
            </div>

            <!-- VIEW: EDITOR -->
            <div v-else-if="viewMode === 'editor'" class="w-full h-full overflow-hidden relative">
                <EnglishEditor 
                    :initial-data="{
                        title: editorForm.title,
                        text: editorForm.text,
                        background: editorForm.background,
                        references: editorForm.references,
                        status: 'Draft'
                    }"
                    @save="handleEditorSave"
                    @cancel="viewMode = 'list'"
                />
            </div>

            <!-- VIEW: READER (Analysis) -->
            <div v-else-if="viewMode === 'reader' && currentArticle" class="w-full h-full flex">
                <!-- Main Reader -->
                <div class="flex-1 h-full overflow-y-auto custom-scrollbar relative bg-[#F9F7F1]">
                    <EnglishArticleAnalyzer :article="currentArticle" />
                </div>

                 <!-- Right Sidebar (History / Comments) -->
                 <aside 
                    v-if="showHistory || showComments" 
                    class="w-96 border-l border-ink/5 bg-gray-50/50 backdrop-blur h-full overflow-y-auto flex flex-col"
                 >
                    <div class="p-4 border-b border-ink/5 font-bold uppercase tracking-wider text-xs text-ink/40">
                        {{ showHistory ? 'Version History' : 'Comments' }}
                    </div>
                    
                    <!-- History Content (Placeholder for Git-Graph) -->
                    <div v-if="showHistory" class="p-6 space-y-6">
                        <div class="relative pl-6 border-l-2 border-indigo-500/20 space-y-8">
                            <!-- Mock Data for visual -->
                            <div class="relative">
                                <div class="absolute -left-[31px] w-4 h-4 rounded-full bg-indigo-500 ring-4 ring-white"></div>
                                <div class="text-sm font-bold text-ink">Version 3 (Current)</div>
                                <div class="text-xs text-ink/50 mt-1">Updated just now</div>
                            </div>
                            <div class="relative opacity-50">
                                <div class="absolute -left-[31px] w-4 h-4 rounded-full bg-gray-300 ring-4 ring-white"></div>
                                <div class="text-sm font-bold text-ink">Version 2</div>
                                <div class="text-xs text-ink/50 mt-1">2 hours ago</div>
                            </div>
                        </div>
                        <div class="text-center text-xs text-ink/30 italic mt-8">
                            Git-like history graph will be rendered here.
                        </div>
                    </div>
                    
                    <!-- Comments Content -->
                    <div v-if="showComments" class="flex-1 flex flex-col">
                        <div class="flex-1 p-4 flex items-center justify-center text-ink/30 italic">
                            No comments yet.
                        </div>
                        <div class="p-4 border-t border-ink/5 bg-white">
                            <textarea class="w-full bg-gray-50 rounded p-2 text-sm outline-none resize-none" rows="3" placeholder="Add a comment..."></textarea>
                            <button class="w-full mt-2 py-1 bg-ink text-white rounded text-xs font-bold">Post</button>
                        </div>
                    </div>
                 </aside>
            </div>
        </div>
    </div>
</template>

<style scoped>
.prose-scholar {
    font-family: 'Noto Serif', serif;
}
.custom-scrollbar::-webkit-scrollbar {
    width: 6px;
    height: 6px;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
    background: #00000010;
    border-radius: 3px;
}
.custom-scrollbar::-webkit-scrollbar-track {
    background: transparent;
}
</style>
