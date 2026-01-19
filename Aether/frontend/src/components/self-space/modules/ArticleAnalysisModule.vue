
<script setup lang="ts">
import { ref, onMounted, computed, watch, onUnmounted } from 'vue';
import axios from 'axios';
import { useDebounceFn } from '@vueuse/core';
import { MessagePlugin } from 'tdesign-vue-next';
import EnglishArticleAnalyzer from '@/components/english/EnglishArticleAnalyzer.vue';
import EnglishEditor from '@/components/english/EnglishEditor.vue';
import AnalysisCard from '@/components/english/AnalysisCard.vue';

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
// const showHistory = ref(false); // Removed History for now to focus on layout
const analysisSelection = ref({
    word: '',
    sentence: undefined as any
});

const handleSelection = (payload: any) => {
    analysisSelection.value = payload;
};

// --- API ---
const authStore = useAuthStore();

// --- API ---
const fetchArticles = async () => {
    isLoading.value = true;
    try {
        const params: any = {
            category: 'English Analysis',
            limit: 50
        };
        
        // If logged in, filter by 'me' to see Drafts and Private items.
        // Otherwise it defaults to 'Published Only' public feed.
        if (authStore.user?.id) {
            params.author_id = authStore.user.id;
        }

        const res = await axios.get('/api/content', { params });
        
        // Parse the body as JSON
        articles.value = res.data.map((item: any) => {
            let parsedBody: ArticleMeta = { text: '' };
            try {
                const rawBody = item.body?.data || ''; 
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
                author_name: item.author_name,
                derived_data: item.derived_data
            };
        });
    } catch (e: any) {
        console.error('Failed to fetch articles', e);
        const msg = e.response?.data?.message || e.response?.data?.error || e.message || 'Failed to load English analyses.';
        MessagePlugin.error(msg);
    } finally {
        isLoading.value = false;
    }
};

const saveArticle = async () => {
    if (!editorForm.value.title || !editorForm.value.text) {
        MessagePlugin.warning('Title and content are required.');
        return;
    }

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
        MessagePlugin.success('Analysis saved successfully.');
        viewMode.value = 'list';
        fetchArticles();
        // Reset Form
        editorForm.value = { title: '', text: '', background: '', references: [] };
    } catch (e: any) {
        console.error('Failed to save article', e);
        const msg = e.response?.data?.message || e.response?.data?.error || e.message || 'Failed to save analysis.';
        MessagePlugin.error(msg);
    }
};

const deleteArticle = async (id: string) => {
    if (!confirm('Are you sure you want to delete this analysis?')) return;
    try {
        await axios.delete(`/api/content/${id}`);
        MessagePlugin.success('Analysis deleted.');
        fetchArticles();
    } catch (e: any) {
        console.error('Failed to delete', e);
        const msg = e.response?.data?.message || e.response?.data?.error || e.message || 'Failed to delete analysis.';
        MessagePlugin.error(msg);
    }
};

const openArticle = (article: EnglishArticle) => {
    currentArticle.value = article;
    viewMode.value = 'reader';
};

const handleEditorSave = async (formData: any) => {
    console.log('handleEditorSave triggered', formData);
    try {
        let articleId = formData.id;

        // 1. Create Article Shell if new
        if (!articleId) {
            console.log('Creating new article shell...');
            const createPayload = {
                title: formData.title || 'Untitled Analysis', // Fallback title
                body: JSON.stringify({ text: '' }), 
                category: 'English Analysis', 
                tags: ['english-learning'],
                visibility: 'Private',
                status: 'Draft'
            };
            const res = await axios.post('/api/content', createPayload);
            articleId = res.data.id;
            console.log('Created article:', articleId);
            (editorForm.value as any).id = articleId; 
        }

        // 2. Save Draft (Always save to shadow draft first)
        const draftPayload = {
            title: formData.title,
            body: {
                text: formData.text,
                background: formData.background,
                references: formData.references
            }
        };
        console.log('Saving draft to shadow table...');
        await axios.post(`/api/drafts/${articleId}`, draftPayload);

        // 3. Publish (If requested)
        if (formData.status === 'Published') {
            console.log('Publishing draft...');
            await axios.post(`/api/drafts/${articleId}/publish`);
            MessagePlugin.success('Analysis published successfully!');
            console.log('Publish success, redirecting to list...');
            
            // Force refresh explicitly
            await fetchArticles();
            viewMode.value = 'list'; 
        } else {
            console.log('Draft auto-saved active.');
            // MessagePlugin.success('Draft saved.');
            // Only refresh list if we want to show 'Draft' badge update? 
            // Maybe not needed for auto-save to avoid flicker.
        }
        
    } catch (e: any) {
        console.error('Failed to save/publish article', e);
        const msg = e.response?.data?.message || e.response?.data?.error || e.message || 'Failed to save analysis.';
        MessagePlugin.error(`Error: ${msg}`);
    }
};

const startNewAnalysis = () => {
    // Reset Form completely to avoid ID collision
    editorForm.value = {
        title: '',
        text: '',
        background: '',
        references: []
    };
    // Explicitly clear ID and Status to force new creation
    (editorForm.value as any).id = undefined;
    (editorForm.value as any).status = 'Draft';
    
    viewMode.value = 'editor';
};

const editArticle = (article: EnglishArticle) => {
    editorForm.value = {
        title: article.title,
        text: article.body.text,
        background: article.body.background || '',
        references: article.body.references || []
    };
    // Inject ID
    (editorForm.value as any).id = article.id;
    (editorForm.value as any).category = article.category;
    // CRITICAL: Always treat editing as working on a Draft
    (editorForm.value as any).status = 'Draft'; 
    
    viewMode.value = 'editor';
};

import { useNavigationStore } from '@/stores/navigation';
import { useAuthStore } from '@/stores/auth';

const navStore = useNavigationStore();

// --- Lifecycle ---
onMounted(() => {
    fetchArticles();
    if (props.headless) {
        navStore.setCustomRight(true);
    }
});

onUnmounted(() => {
    if (props.headless) {
        navStore.setCustomRight(false);
    }
});

// --- Computed ---
const filteredArticles = computed(() => {
    // Stage 1: Absolute Safety Filter (Client-side Firewall)
    // Even if backend leaks data, we ignore anything that isn't explicitly English Analysis
    // NOTE: Backend *should* handle this via category param, but we double-check.
    const safeList = articles.value.filter(a => a.category === 'English Analysis');

    // Stage 2: Search Query
    if (!searchQuery.value) return safeList;
    const q = searchQuery.value.toLowerCase();
    return safeList.filter(a => a.title.toLowerCase().includes(q));
});

</script>

<template>
    <div class="w-full h-full flex flex-col relative overflow-hidden bg-white">
        
        <!-- TOP NAV (Changes based on view) -->
        <div v-if="!headless" class="h-16 flex items-center justify-between px-8 border-b border-gray-100 bg-white z-20">
            <div class="flex items-center gap-4">
                <button 
                    v-if="viewMode !== 'list'" 
                    @click="viewMode = 'list'"
                    class="w-8 h-8 rounded-full hover:bg-gray-100 flex items-center justify-center transition-colors text-gray-500 hover:text-gray-900"
                >
                    <i class="ri-arrow-left-line text-lg"></i>
                </button>
                <h2 class="text-xl font-bold text-gray-900">
                    <span v-if="viewMode === 'list'">My Library</span>
                    <span v-else-if="viewMode === 'editor'">New Analysis</span>
                    <span v-else>{{ currentArticle?.title }}</span>
                </h2>
            </div>
            
            <div class="flex items-center gap-4">
                 <!-- Actions rendered inline for standalone -->
                 <div class="flex items-center gap-4">
                     <template v-if="viewMode === 'list'">
                        <div class="relative group">
                            <i class="ri-search-line absolute left-3 top-1/2 -translate-y-1/2 text-gray-400 group-focus-within:text-gray-600 transition-colors"></i>
                            <input 
                                v-model="searchQuery"
                                class="pl-9 pr-4 py-1.5 bg-gray-50 focus:bg-white rounded-full text-sm outline-none border border-transparent focus:border-gray-200 focus:shadow-sm w-64 transition-all"
                                placeholder="Search articles..."
                            />
                        </div>

                        <button 
                            @click="startNewAnalysis"
                            class="px-4 py-1.5 bg-gray-900 text-white rounded-full text-sm font-bold shadow hover:shadow-lg hover:-translate-y-0.5 transition-all flex items-center gap-2"
                        >
                            <i class="ri-add-line"></i> New Analysis
                        </button>
<!-- ... -->

                    </template>

                    <template v-if="viewMode === 'reader'">
                        <button 
                            @click="editArticle(currentArticle!)"
                            class="flex items-center gap-2 px-3 py-1.5 bg-gray-100 hover:bg-gray-200 text-gray-700 rounded-lg transition-colors text-sm font-medium"
                            title="Edit this article"
                        >
                            <i class="ri-edit-line"></i> Edit
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
                        <i class="ri-search-line absolute left-3 top-1/2 -translate-y-1/2 text-gray-400 group-focus-within:text-gray-600 transition-colors"></i>
                        <input 
                            v-model="searchQuery"
                            class="pl-9 pr-4 py-1.5 bg-gray-50 focus:bg-white rounded-full text-xs font-medium outline-none border border-transparent focus:border-gray-200 w-48 transition-all"
                            placeholder="Search..."
                        />
                    </div>
                    <button 
                        @click="startNewAnalysis"
                        class="text-xs font-bold uppercase tracking-widest text-gray-400 hover:text-gray-900 transition-colors"
                    >
                        NEW ANALYSIS
                    </button>

                 </template>

                 <template v-if="viewMode === 'reader'">
                     <button 
                        @click="editArticle(currentArticle!)"
                        class="text-xs font-bold uppercase tracking-widest text-gray-400 hover:text-gray-900 mr-4 transition-colors"
                     >
                        EDIT
                     </button>
                     <button @click="viewMode = 'list'" class="text-xs font-bold uppercase tracking-widest text-gray-400 hover:text-gray-900">
                        BACK
                     </button>
                </template>
             </div>
        </Teleport>

        <!-- CONTENT AREA -->
        <div class="flex-1 overflow-hidden relative">
            
            <!-- VIEW: LIST -->
            <div v-if="viewMode === 'list'" class="w-full h-full overflow-y-auto p-8 custom-scrollbar">
                <div v-if="isLoading" class="flex justify-center pt-20">
                    <i class="ri-loader-4-line animate-spin text-3xl text-gray-300"></i>
                </div>
                
                <div v-else-if="filteredArticles.length === 0" class="flex flex-col items-center justify-center h-full text-gray-400 gap-4">
                    <div class="w-20 h-20 rounded-full bg-gray-50 flex items-center justify-center">
                        <i class="ri-book-open-line text-4xl"></i>
                    </div>
                    <p class="font-medium">No analysis found. Start your first journey.</p>
                </div>

                <div v-else class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 max-w-7xl mx-auto">
                    <div 
                        v-for="article in filteredArticles" 
                        :key="article.id"
                        class="group relative aspect-[4/3] bg-white rounded-2xl shadow-sm hover:shadow-xl transition-all duration-300 overflow-hidden border border-gray-100 cursor-pointer"
                        @click="openArticle(article)"
                    >
                        <!-- Background -->
                        <div class="absolute inset-0 bg-gray-50">
                            <img 
                                v-if="article.body.background" 
                                :src="article.body.background" 
                                class="w-full h-full object-cover transition-transform duration-700 group-hover:scale-105"
                                loading="lazy"
                            />
                            <!-- Elegant Fallback Gradient if no image -->
                            <div v-else class="w-full h-full bg-gradient-to-br from-slate-800 to-slate-900 group-hover:scale-105 transition-transform duration-700 relative">
                                <!-- Abstract Pattern (CSS based) -->
                                <div class="absolute inset-0 opacity-10" style="background-image: radial-gradient(circle at 2px 2px, white 1px, transparent 0); background-size: 20px 20px;"></div>
                            </div>
                            
                            <!-- Overlay -->
                            <div class="absolute inset-0 bg-gradient-to-t from-black/90 via-black/20 to-transparent opacity-80 group-hover:opacity-90 transition-opacity"></div>
                        </div>

                        <!-- Content -->
                        <div class="absolute inset-0 p-6 flex flex-col justify-end text-white">
                            <h3 class="text-xl font-bold leading-tight mb-2 drop-shadow-md line-clamp-2 text-white">{{ article.title }}</h3>
                            <div class="flex items-center justify-between text-white/80 text-xs font-medium uppercase tracking-wider">
                                <span>{{ new Date(article.created_at).toLocaleDateString() }}</span>
                                <span v-if="article.status === 'Draft'" class="bg-yellow-500/30 backdrop-blur px-2 py-0.5 rounded text-yellow-100 border border-yellow-500/20">Draft</span>
                                <span v-else class="bg-green-500/30 backdrop-blur px-2 py-0.5 rounded text-green-100 border border-green-500/20">Published</span>
                            </div>
                        </div>
                        
                        <!-- Delete Action (Hover) -->
                        <button 
                            @click.stop="deleteArticle(article.id)"
                            class="absolute top-4 right-4 p-2 bg-black/40 backdrop-blur rounded-full text-white/70 hover:bg-red-500 hover:text-white transition-all opacity-0 group-hover:opacity-100 transform translate-y-[-10px] group-hover:translate-y-0"
                            title="Delete Analysis"
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
                        id: (editorForm as any).id,
                        title: editorForm.title,
                        text: editorForm.text,
                        background: editorForm.background,
                        category: (editorForm as any).category || 'English Analysis',
                        references: editorForm.references,
                        status: (editorForm as any).status || 'Draft'
                    }"
                    @save="handleEditorSave"
                    @cancel="viewMode = 'list'"
                />
            </div>

            <!-- VIEW: READER (Analysis) -->
            <div v-else-if="viewMode === 'reader' && currentArticle" class="w-full h-full flex bg-gray-50/50">
                <!-- Main Content (Centered) -->
                <div class="flex-1 h-full overflow-y-auto custom-scrollbar relative">
                    <div class="max-w-4xl mx-auto bg-white min-h-full shadow-sm border-x border-gray-100/50">
                        <EnglishArticleAnalyzer 
                            :article="currentArticle" 
                            @selection="handleSelection"
                        />
                    </div>
                </div>

                 <!-- Right Sidebar (Analysis Card) -->
                 <aside class="w-96 border-l border-gray-200 bg-white h-full relative z-30 flex-shrink-0">
                    <div class="absolute inset-0 overflow-y-auto p-6">
                        <AnalysisCard 
                            :word="analysisSelection.word"
                            :sentence="analysisSelection.sentence"
                        />
                    </div>
                 </aside>
            </div>
        </div>
    </div>
</template>

<style scoped>
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
