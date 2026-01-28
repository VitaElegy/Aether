<script setup lang="ts">
import { ref, reactive, computed, onMounted, onUnmounted, watch, nextTick } from 'vue';
import { MessagePlugin } from 'tdesign-vue-next';
import { useDebounceFn } from '@vueuse/core';
import { dictionaryApi, type DictionaryEntry } from '@/api/dictionary';
import axios from 'axios';
import ArticleAnalysisModule from './ArticleAnalysisModule.vue';
import VocabDetailModal from './VocabDetailModal.vue';

interface VocabularyExample {
    id?: string;
    sentence: string;
    translation?: string;
    note?: string;
    image_url?: string;
}

interface Vocabulary {
    id: string;
    word: string;
    definition: string;
    translation?: string;
    phonetic?: string;
    
    // New Fields
    root?: string;
    examples: VocabularyExample[];
    query_count?: number;
    is_important?: boolean;
    
    // Deprecated / Mapped
    context_sentence?: string;
    image_url?: string;
    
    language: string;
    status: string;
    created_at: string;
}

// State
const vocabularyList = ref<Vocabulary[]>([]);
const listLoading = ref(false);
const searchQuery = ref('');
const showLibrary = ref(false);
const sortBy = ref('created_at');
const sortOrder = ref('desc');

import { useNavigationStore } from '@/stores/navigation';

const navStore = useNavigationStore();
const isActive = ref(false); // [HOISTED]

// Tab State
const activeTab = ref<'vocabulary' | 'articles'>('vocabulary');

// Sync Navigation Bar State
// Sync Navigation Bar State
import { onActivated, onDeactivated } from 'vue';

// Lifecycle: Manage Navigation Bar State
const setNavState = () => {
    console.log('[Vocab] setNavState');
    isActive.value = true;
    navStore.setCustomCenter(true);
    navStore.setCustomRight(activeTab.value === 'vocabulary');
};

const clearNavState = () => {
    console.log('[Vocab] clearNavState');
    isActive.value = false;
    navStore.reset();
};

onMounted(() => {
    // Sync call for first mount
    setNavState();
});

onActivated(() => {
    nextTick(() => {
        setNavState();
        window.addEventListener('popstate', handlePopState);
    });
});

onDeactivated(() => {
    clearNavState();
    window.removeEventListener('popstate', handlePopState);
});

onUnmounted(() => {
    clearNavState();
    window.removeEventListener('popstate', handlePopState);
});

// Watch activeTab to update state WHILE active
watch(activeTab, (newTab) => {
    if (!isActive.value) return;
    navStore.setCustomCenter(true);
    // Only update right portal flag if we are active
    navStore.setCustomRight(newTab === 'vocabulary');
});

// Batch Selection State
const isSelectionMode = ref(false);
const selectedIds = ref<Set<string>>(new Set());

// Computed for Detail View
const currentVocabId = computed(() => {
    if (!previewEntry.value) return null;
    return vocabularyList.value.find(v => v.word.toLowerCase() === previewEntry.value!.word.toLowerCase())?.id;
});

const isCurrentImportant = computed(() => {
    if (!previewEntry.value) return false;
    return vocabularyList.value.find(v => v.word.toLowerCase() === previewEntry.value!.word.toLowerCase())?.is_important || false;
});

const currentQueryCount = computed(() => {
     if (!previewEntry.value) return 0;
    return vocabularyList.value.find(v => v.word.toLowerCase() === previewEntry.value!.word.toLowerCase())?.query_count || 0;
});




// Spotlight State
const isSpotlightActive = ref(false);
const searchSuggestions = ref<string[]>([]);
const previewEntry = ref<DictionaryEntry | null>(null);
const previewImage = ref('');
const isSearchingDef = ref(false);
const showAllDefinitions = ref(false);
const isDetailView = ref(false);

const totalDefinitionsCount = computed(() => {
    if (!previewEntry.value) return 0;
    return previewEntry.value.meanings.reduce((acc, m) => acc + m.definitions.length, 0);
});

// Create Form
const createForm = reactive({
    word: '',
    definition: '',
    translation: '',
    phonetic: '',
    
    // New Fields
    root: '',
    examples: [] as VocabularyExample[],
    
    // Legacy fields mapped later
    context_sentence: '', 
    image_url: '',
    
    language: 'en'
});
const isSaving = ref(false);
const isUploading = ref(false);

const inputRef = ref<HTMLInputElement | null>(null);

// Methods
const searchController = ref<AbortController | null>(null);
const debouncedSearch = useDebounceFn(async (val: string) => {
    // Cancel previous request
    if (searchController.value) {
        searchController.value.abort();
    }
    searchController.value = new AbortController();

    // 2. Local Fuzzy Search via Backend
    console.log('Fuzzy Search:', val);
    try {
        const remoteWords = await dictionaryApi.fuzzy(val, searchController.value.signal);
        console.log('Remote Words:', remoteWords);
        
        // Combine with local matches (from saved vocabulary)
        const localMatches = vocabularyList.value
            .filter(v => v.word.toLowerCase().includes(val.toLowerCase()))
            .map(v => v.word);

        // Combine matches
        const combinedSet = new Set([...localMatches, ...remoteWords]);
        let suggestionsArray = Array.from(combinedSet);

        // Sorting Logic: Exact Match > Starts With > Includes
        const lowerVal = val.toLowerCase();
        suggestionsArray.sort((a, b) => {
            const lowerA = a.toLowerCase();
            const lowerB = b.toLowerCase();

            // 1. Exact Match Priority
            if (lowerA === lowerVal) return -1;
            if (lowerB === lowerVal) return 1;

            // 2. Starts With Priority
            const aStarts = lowerA.startsWith(lowerVal);
            const bStarts = lowerB.startsWith(lowerVal);
            if (aStarts && !bStarts) return -1;
            if (!aStarts && bStarts) return 1;

            // 3. Length Priority
            return a.length - b.length;
        });

        searchSuggestions.value = suggestionsArray.slice(0, 8);
        console.log('Suggestions:', searchSuggestions.value);
    } catch (e: any) {
        if (e.name === 'CanceledError' || axios.isCancel(e)) {
            console.log('Request canceled');
            return;
        }
        console.error('Fuzzy Error:', e);
        // Fallback or just show local matches if backend fails
        const localMatches = vocabularyList.value
            .filter(v => v.word.toLowerCase().includes(val.toLowerCase()))
            .map(v => v.word);
         searchSuggestions.value = localMatches.slice(0, 8);
    }
}, 300);

const definitionsRef = ref<HTMLDivElement | null>(null);
const isOverflowing = ref(false);

const checkOverflow = () => {
    nextTick(() => {
        if (definitionsRef.value) {
            isOverflowing.value = definitionsRef.value.scrollHeight > definitionsRef.value.clientHeight;
        }
    });
};

watch([() => previewEntry.value, showAllDefinitions], () => {
    checkOverflow();
});

watch(searchQuery, (val) => {
    console.log('Search Query Changed:', val);
    if (!val) {
        isSpotlightActive.value = false;
        searchSuggestions.value = [];
        return;
    }
    isSpotlightActive.value = true;
    showAllDefinitions.value = false; // Reset on new search
    debouncedSearch(val);
});

const selectSuggestion = async (word: string) => {
    console.log('Selected Suggestion:', word);
    createForm.word = word;
    const existing = vocabularyList.value.find(v => v.word.toLowerCase() === word.toLowerCase());
    
    // Always fetch dictionary info for the "Hero" section (definition, phonetic)
    // even if we have local data, to ensure we have the rich dictionary structure.
    await fetchDefinitionInfo(word);

    if (existing) {
        // Map existing data to form
        createForm.definition = existing.definition;
        createForm.translation = existing.translation || '';
        createForm.phonetic = existing.phonetic || '';
        createForm.language = existing.language;
        createForm.root = existing.root || '';
        
        if (existing.examples && existing.examples.length > 0) {
            createForm.examples = JSON.parse(JSON.stringify(existing.examples)); // Deep copy
        } else {
             // Backward compat: Map old context/image if no examples
            if (existing.context_sentence || existing.image_url) {
                createForm.examples = [{
                    sentence: existing.context_sentence || '',
                    translation: '',
                    note: '',
                    image_url: existing.image_url || ''
                }];
            } else {
                createForm.examples = [];
            }
        }
        
        // INCREMENT QUERY COUNT
        incrementQueryCount(existing.id);

    } else {
        // Reset form for new entry
        createForm.translation = '';
        createForm.root = '';
        createForm.examples = [];
    }
};

const incrementQueryCount = async (id: string) => {
    try {
        const token = localStorage.getItem('aether_token');
        await axios.post(`/api/vocabulary/${id}/increment_query`, {}, {
             headers: { Authorization: `Bearer ${token}` }
        });
        // Update local state
        const item = vocabularyList.value.find(v => v.id === id);
        if (item) {
            item.query_count = (item.query_count || 0) + 1;
        }
    } catch (e) {
        console.error('Failed to increment count', e);
    }
};

const toggleImportance = async (id: string) => {
    const item = vocabularyList.value.find(v => v.id === id);
    if (!item) return;

    const newState = !item.is_important;
    // Optimistic Update
    item.is_important = newState;

    try {
        const token = localStorage.getItem('aether_token');
        await axios.post(`/api/vocabulary/${id}/toggle_importance`, { is_important: newState }, {
             headers: { Authorization: `Bearer ${token}` }
        });
        MessagePlugin.success(newState ? 'Marked as Important' : 'Unmarked');
    } catch (e) {
        // Revert on error
        item.is_important = !newState;
        console.error('Failed to toggle importance', e);
        MessagePlugin.error('Action failed');
    }
};



const fetchDefinitionInfo = async (word: string) => {
    console.log('Fetching Definition:', word);
    isSearchingDef.value = true;
    try {
        const entry = await dictionaryApi.lookup(word);
        console.log('Lookup Result:', entry);
        
        if (entry) {
            previewEntry.value = entry;
        } else {
            // Fallback: Construct synthetic entry from local data if available
            const existing = vocabularyList.value.find(v => v.word.toLowerCase() === word.toLowerCase());
            if (existing) {
                previewEntry.value = {
                    word: existing.word,
                    phonetic: existing.phonetic,
                    meanings: [
                        {
                            partOfSpeech: 'Local',
                            definitions: [{
                                definition: existing.definition,
                                example: ''
                            }],

                        }
                    ],
                    source: 'Local',

                };
            } else {
                 previewEntry.value = null;
            }
        }
    } catch (e) {
        console.error('Lookup Error:', e);
        // Fallback on error too
        const existing = vocabularyList.value.find(v => v.word.toLowerCase() === word.toLowerCase());
        if (existing) {
             previewEntry.value = {
                word: existing.word,
                phonetic: existing.phonetic,
                meanings: [
                    {
                        partOfSpeech: 'Local',
                        definitions: [{
                            definition: existing.definition,
                            example: ''
                        }],

                    }
                ],
                source: 'Local',

            };
        } else {
             previewEntry.value = null;
        }
    } finally {
        isSearchingDef.value = false;
    }
};

const handleUpload = async (file: File) => {
    isUploading.value = true;
    const formData = new FormData();
    formData.append('file', file);
    try {
        const token = localStorage.getItem('aether_token');
        const res = await axios.post('/api/upload', formData, {
            headers: { 'Content-Type': 'multipart/form-data', Authorization: `Bearer ${token}` }
        });
        createForm.image_url = res.data.url;
        previewImage.value = res.data.url;
        MessagePlugin.success('Image uploaded');
    } catch (e) {
        MessagePlugin.error('Upload failed');
    } finally {
        isUploading.value = false;
    }
};

const onFileChange = (e: Event) => {
    const target = e.target as HTMLInputElement;
    if (target.files && target.files[0]) handleUpload(target.files[0]);
};

// Example Management
const addExample = () => {
    createForm.examples.push({
        sentence: '',
        translation: '',
        note: '',
        image_url: ''
    });
};

const removeExample = (index: number) => {
    createForm.examples.splice(index, 1);
};

const handleExampleImageUpload = async (file: File, index: number) => {
    isUploading.value = true;
    const formData = new FormData();
    formData.append('file', file);
    try {
        const token = localStorage.getItem('aether_token');
        const res = await axios.post('/api/upload', formData, {
            headers: { 'Content-Type': 'multipart/form-data', Authorization: `Bearer ${token}` }
        });
        createForm.examples[index].image_url = res.data.url;
        MessagePlugin.success('Image uploaded');
    } catch (e) {
        MessagePlugin.error('Upload failed');
    } finally {
        isUploading.value = false;
    }
};

const onExampleFileChange = (e: Event, index: number) => {
    const target = e.target as HTMLInputElement;
    if (target.files && target.files[0]) handleExampleImageUpload(target.files[0], index);
};

const saveWord = async () => {
    isSaving.value = true;
    try {
        const token = localStorage.getItem('aether_token');
        const config = { headers: { Authorization: `Bearer ${token}` } };
        
        // Preserve existing extra fields if updating
        const existing = vocabularyList.value.find(v => v.word.toLowerCase() === createForm.word.toLowerCase());
        const payload = {
             ...createForm,
             // Note: query_count and is_important are handled by server preservation logic in save_vocabulary,
        };
        
        await axios.post('/api/vocabulary', payload, config);
        MessagePlugin.success('Saved');
        searchQuery.value = '';
        isSpotlightActive.value = false;
        fetchVocabularyList();
    } catch (e: any) {
        console.error('Save Error Details:', e.response?.data || e);
        const errorMsg = e.response?.data?.error || 'Failed to save';
        MessagePlugin.error(`Save failed: ${errorMsg}`);
    } finally {
        isSaving.value = false;
    }
};

// Aesthetic Helpers
const gradients = [
    'bg-gradient-to-br from-rose-100/40 via-orange-50/40 to-white', 
    'bg-gradient-to-bl from-indigo-100/40 via-blue-50/40 to-white',
    'bg-gradient-to-tr from-emerald-100/40 via-teal-50/40 to-white',
    'bg-gradient-to-br from-violet-100/40 via-fuchsia-50/40 to-white',
    'bg-gradient-to-tl from-amber-100/40 via-yellow-50/40 to-white',
    'bg-gradient-to-br from-cyan-100/40 via-sky-50/40 to-white',
    'bg-gradient-to-tr from-fuchsia-100/40 via-pink-50/40 to-white'
];

const getGradient = (word: string) => {
    let sum = 0;
    for (let i = 0; i < word.length; i++) {
        sum += word.charCodeAt(i);
    }
    return gradients[sum % gradients.length];
};

const deleteVocabulary = async (id: string, e?: Event) => {
    if(e) e.stopPropagation();
    try {
        const token = localStorage.getItem('aether_token');
        const config = { headers: { Authorization: `Bearer ${token}` } };
        await axios.delete(`/api/vocabulary/${id}`, config);
        MessagePlugin.success('Deleted');
        fetchVocabularyList();
        // If in preview, clear it
        if (previewEntry.value?.word === vocabularyList.value.find(v => v.id === id)?.word) {
             previewEntry.value = null;
             isDetailView.value = false;
        }
    } catch (e) {
        MessagePlugin.error('Delete failed');
    }
};

const deleteExample = async (index: number) => {
    // Determine ID to update
    const item = vocabularyList.value.find(v => v.word.toLowerCase() === createForm.word.toLowerCase());
    if (!item) {
        // Just local state if not saved yet
        removeExample(index);
        return;
    }

    // Remove from local state
    createForm.examples.splice(index, 1);
    
    // Save changes to backend
    await saveWord(); 
    MessagePlugin.success('Example removed');
};

const toggleSelection = (id: string, e?: Event) => {
    e?.stopPropagation();
    if (selectedIds.value.has(id)) {
        const newSet = new Set(selectedIds.value);
        newSet.delete(id);
        selectedIds.value = newSet;
    } else {
        const newSet = new Set(selectedIds.value);
        newSet.add(id);
        selectedIds.value = newSet;
    }
};

const deleteSelected = async () => {
    if (selectedIds.value.size === 0) return;
    
    try {
        const token = localStorage.getItem('aether_token');
        const config = { headers: { Authorization: `Bearer ${token}` } };
        
        await axios.post('/api/vocabulary/batch-delete', { ids: Array.from(selectedIds.value) }, config);
        
        MessagePlugin.success(`Deleted ${selectedIds.value.size} items`);
        
        // Reset Selection
        isSelectionMode.value = false;
        selectedIds.value = new Set();
        
        // Refresh List
        fetchVocabularyList(searchQuery.value);
        
    } catch (e) {
        console.error(e);
        MessagePlugin.error('Batch delete failed');
    }
};

const handleCardClick = (item: any) => {
    if (isSelectionMode.value) {
        toggleSelection(item.id);
    } else {
        // Switch to "Detail Mode" (Spotlight)
        searchQuery.value = item.word;
        selectSuggestion(item.word);
        showLibrary.value = false; // Close library to show spotlight/detail
    }
};

const fetchVocabularyList = async (query = '') => {
    listLoading.value = true;
    try {
        const token = localStorage.getItem('aether_token');
        const config = { headers: { Authorization: `Bearer ${token}` } };
        const url = query 
            ? `/api/vocabulary?limit=100&query=${query}&sort_by=${sortBy.value}&order=${sortOrder.value}` 
            : `/api/vocabulary?limit=100&sort_by=${sortBy.value}&order=${sortOrder.value}`;
            
        const res = await axios.get(url, config);
        vocabularyList.value = res.data;
    } catch (e) {
        console.error(e);
    } finally {
        listLoading.value = false;
    }
};

onMounted(() => {
    fetchVocabularyList();
});

// History Navigation Logic
watch(isDetailView, (val) => {
    if (val) {
        history.pushState({ popup: 'detail' }, '');
    }
});

const handlePopState = () => {
    if (isDetailView.value) {
        isDetailView.value = false;
    }
};

onMounted(() => {
    window.addEventListener('popstate', handlePopState);
});

onUnmounted(() => {
    window.removeEventListener('popstate', handlePopState);
});

const goBack = () => {
    history.back();
};

// --- Detail Modal Logic ---
const detailModalVisible = ref(false);
const detailModalWord = ref('');
const detailModalData = ref<any>(null);

const handleViewDetailsFromArticle = (payload: any) => {
    console.log("Opening Modal with:", payload);
    detailModalWord.value = payload.word;
    detailModalData.value = payload; // Can contain initialSentence or full vocab object
    detailModalVisible.value = true;
};
</script>

<template>
    <div class="h-full flex flex-col relative font-sans text-ink">
        
        <!-- Header Info (Fade out when searching) -->
        <!-- Implicit Library Toggle (Top Right) -->
        <!-- Navigation Teleports -->
        <Teleport to="#nav-center-portal">
             <div v-if="isActive && !isDetailView && !isSpotlightActive" class="flex items-center gap-3 font-serif text-sm pointer-events-auto">
                <button 
                    @click="activeTab = 'vocabulary'"
                    class="transition-all duration-300 relative group"
                    :class="activeTab === 'vocabulary' ? 'text-ink font-bold scale-105' : 'text-ink/30 hover:text-ink/60'"
                >
                    Words
                    <span 
                        class="absolute -bottom-1 left-1/2 -translate-x-1/2 w-1 h-1 bg-ink rounded-full opacity-0 transition-opacity"
                        :class="{'opacity-100': activeTab === 'vocabulary'}"
                    ></span>
                </button>
                <span class="text-ink/10 text-xs">/</span>
                <button 
                    @click="activeTab = 'articles'"
                    class="transition-all duration-300 relative group"
                    :class="activeTab === 'articles' ? 'text-ink font-bold scale-105' : 'text-ink/30 hover:text-ink/60'"
                >
                    Articles
                     <span 
                        class="absolute -bottom-1 left-1/2 -translate-x-1/2 w-1 h-1 bg-ink rounded-full opacity-0 transition-opacity"
                        :class="{'opacity-100': activeTab === 'articles'}"
                    ></span>
                </button>
            </div>
        </Teleport>

        <Teleport to="#nav-right-portal">
            <!-- Library Toggle (Only for Vocab Tab) -->
            <template v-if="isActive && !isDetailView && !isSpotlightActive">
                <button 
                    v-if="activeTab === 'vocabulary'"
                @click="showLibrary = !showLibrary"
                class="w-8 h-8 rounded-full flex items-center justify-center transition-all duration-300 group ml-2"
                :class="showLibrary ? 'bg-ink text-white shadow-xl rotate-90' : 'text-ink/40 hover:bg-ink/5 hover:text-ink'"
                title="Toggle Library"
            >
                <i :class="showLibrary ? 'ri-close-line' : 'ri-book-3-line'" class="text-xl"></i>
            </button>
            </template>
        </Teleport>

        <!-- content: ARTICLES TAB -->
        <div v-if="activeTab === 'articles'" class="w-full h-full">
            <ArticleAnalysisModule :headless="true" />
        </div>

        <!-- content: VOCABULARY TAB -->
        <div v-else class="flex-1 relative w-full h-full overflow-hidden">
            
            <!-- Collection Grid (Underneath) -->
            <div 
                 v-if="showLibrary"
                 class="w-full h-full overflow-y-auto px-8 py-32 custom-scrollbar transition-all duration-500 ease-out animate-fade-in"
                 :class="{ 'opacity-20 scale-95 blur-sm overflow-hidden': isSpotlightActive }"
            >
                <!-- Library Header/Close -->
                <div class="max-w-7xl mx-auto mb-8 flex justify-between items-end">
                    <div>
                        <h2 class="text-2xl font-serif font-bold text-ink">My Words</h2>
                        <span class="text-xs font-bold uppercase tracking-wider text-ink/40">{{ vocabularyList.length }} Entries</span>
                    </div>
                    <div class="flex items-center gap-2">
                        <!-- Batch Delete Controls -->
                         <template v-if="isSelectionMode">
                             <span class="text-sm font-bold text-ink/60 mr-2">{{ selectedIds.size }} Selected</span>
                             
                             <button 
                                @click="deleteSelected"
                                :disabled="selectedIds.size === 0"
                                class="px-3 py-1 rounded bg-red-500 text-white font-bold text-xs hover:bg-red-600 disabled:opacity-50 transition-colors"
                             >
                                Delete
                             </button>
                             <button 
                                @click="isSelectionMode = false; selectedIds.clear()"
                                class="px-3 py-1 rounded bg-gray-100 text-ink/60 font-bold text-xs hover:bg-gray-200 transition-colors"
                             >
                                Cancel
                             </button>
                         </template>
                         
                         <!-- Sort Controls (Hidden in Selection Mode) -->
                         <template v-else>
                             <button
                                @click="isSelectionMode = true"
                                class="px-3 py-1 rounded bg-white border border-ink/10 text-ink/60 hover:text-ink hover:border-ink/30 transition-all font-bold text-xs mr-2"
                             >
                                Select
                             </button>
                             
                             <t-select 
                                v-model="sortBy" 
                                size="small" 
                                class="w-32" 
                                :popup-props="{ overlayClassName: 'text-xs font-bold' }"
                                @change="fetchVocabularyList(searchQuery)"
                             >
                                <t-option value="created_at" label="Date Added" />
                                <t-option value="query_count" label="Most Searched" />
                                <t-option value="is_important" label="Important" />
                             </t-select>

                             <button 
                                @click="sortOrder = sortOrder === 'asc' ? 'desc' : 'asc'; fetchVocabularyList(searchQuery)"
                                class="w-8 h-8 rounded bg-white border border-ink/10 flex items-center justify-center text-ink/60 hover:text-ink hover:border-ink/30 transition-all font-bold text-xs"
                             >
                                {{ sortOrder === 'asc' ? 'A-Z' : 'Z-A' }}
                             </button>
                         </template>
                    </div>
                </div>

                <div v-if="vocabularyList.length === 0" class="text-center py-20 opacity-30">
                    <p class="text-4xl mb-4">✨</p>
                    Your collection is empty.
                </div>
                
                <div v-else class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-6 pb-20 max-w-7xl mx-auto">
                     <div 
                        v-for="item in vocabularyList" 
                        :key="item.id" 
                         class="group relative bg-white rounded-2xl shadow-sm hover:shadow-2xl hover:-translate-y-1 transition-all duration-300 overflow-hidden border border-ink/5 cursor-pointer flex flex-col"
                         :class="{ 'ring-2 ring-accent ring-offset-2': isSelectionMode && selectedIds.has(item.id) }"
                         @click="handleCardClick(item)"
                     >
                         <!-- Image Header (Fixed Height) -->
                         <div class="h-40 w-full relative overflow-hidden" :class="item.image_url ? 'bg-ink/5' : getGradient(item.word)">
                            <img 
                                v-if="item.image_url" 
                                :src="item.image_url" 
                                class="absolute inset-0 w-full h-full object-cover transition-transform duration-700 ease-out group-hover:scale-110 group-hover:-rotate-1"
                            >
                            <div v-else class="absolute inset-0 flex items-center justify-center">
                                <!-- Background Big Letter (Parallax Back) -->
                                <span class="text-9xl font-serif italic text-ink/5 absolute -bottom-8 -right-8 select-none transition-transform duration-700 ease-out group-hover:scale-150 group-hover:rotate-12 group-hover:translate-x-4">
                                    {{ item.word.charAt(0) }}
                                </span>
                                <!-- Foreground Letter (Parallax Front) -->
                                <span class="text-5xl font-serif font-bold text-ink/20 relative z-10 transition-transform duration-500 ease-out group-hover:-translate-y-4 group-hover:scale-110">
                                    {{ item.word.charAt(0) }}
                                </span>
                            </div>
                            
                            <!-- Overlay Gradient -->
                            <div class="absolute inset-0 bg-gradient-to-t from-black/40 via-transparent to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-500"></div>
                            
                            <!-- Selection Checkbox (Top Right) -->
                             <div v-if="isSelectionMode" class="absolute top-3 right-3 z-30">
                                 <div 
                                    class="w-6 h-6 rounded-full border-2 flex items-center justify-center transition-all shadow-md"
                                    :class="selectedIds.has(item.id) ? 'border-accent bg-accent text-white scale-110' : 'border-white bg-white/80 text-transparent'"
                                 >
                                     <i class="ri-check-line text-xs font-bold"></i>
                                 </div>
                             </div>
                         </div>

                         <!-- Content Body -->
                         <div class="p-5 flex-1 flex flex-col relative bg-white">
                             
                             <!-- Title & Meta Row -->
                             <div class="flex justify-between items-start mb-2">
                                <h3 class="text-xl font-serif font-bold text-ink group-hover:text-accent transition-colors truncate pr-2">{{ item.word }}</h3>
                                
                                <div class="flex gap-1 shrink-0">
                                    <!-- Dynamic Sort Badge -->
                                    <span v-if="sortBy === 'query_count'" class="text-[10px] uppercase font-bold bg-amber-50 text-amber-600 px-2 py-0.5 rounded-full flex items-center gap-1 border border-amber-100">
                                        <i class="ri-fire-fill"></i> {{ item.query_count }}
                                    </span>
                                    <span v-else-if="sortBy === 'created_at'" class="text-[10px] uppercase font-bold bg-slate-50 text-slate-500 px-2 py-0.5 rounded-full flex items-center gap-1 border border-slate-100">
                                        {{ new Date(item.created_at).toLocaleDateString(undefined, { month: 'numeric', day: 'numeric'}) }}
                                    </span>
                                    
                                    <!-- Important Star -->
                                    <i v-if="item.is_important" class="ri-star-fill text-yellow-400 text-sm"></i>
                                </div>
                             </div>

                             <!-- Translation Tag -->
                             <div class="mb-3">
                                <span v-if="item.translation" class="inline-block text-xs font-bold text-ink/50 bg-ink/5 px-2 py-1 rounded-md">
                                    {{ item.translation }}
                                </span>
                             </div>

                             <!-- Definition (Truncated) -->
                             <p class="text-sm text-ink/60 line-clamp-2 leading-relaxed h-10 mb-2">
                                {{ item.definition || 'No definition provided.' }}
                             </p>
                             
                             <!-- Footer Actions (Hover Reveal) -->
                             <div class="mt-auto pt-4 border-t border-ink/5 flex justify-between items-center opacity-0 group-hover:opacity-100 transition-opacity transform translate-y-2 group-hover:translate-y-0 duration-300">
                                <span class="text-xs font-bold text-accent uppercase tracking-wider flex items-center gap-1">
                                    View Details <i class="ri-arrow-right-line"></i>
                                </span>
                             </div>
                         </div>
                     </div>
                </div>
            </div>


            <!-- Unified Search Bar -->
            <!-- Position: centered by default (top-1/2), moves moves up if Library is open OR Spotlight is active -->
            <div 
                class="absolute left-0 right-0 z-50 flex justify-center transition-all duration-500 cubic-bezier(0.34, 1.56, 0.64, 1)"
                :class="[ (isSpotlightActive || showLibrary) ? 'top-16 translate-y-0' : 'top-[40%] -translate-y-1/2' ]"
            >
                <div 
                    class="relative group w-full transition-all duration-500 ease-out"
                    :class="[ (isSpotlightActive || showLibrary) ? 'max-w-4xl' : 'max-w-xl' ]"
                >
                    <i class="ri-search-line absolute left-6 top-1/2 -translate-y-1/2 text-2xl transition-colors duration-300"
                       :class="isSpotlightActive ? 'text-accent' : 'text-ink/30'"></i>
                    
                    <input 
                        ref="inputRef"
                        v-model="searchQuery" 
                        type="text" 
                        placeholder="Type a word to search or create..." 
                        class="w-full bg-white/80 backdrop-blur-xl shadow-2xl rounded-2xl py-5 pl-16 pr-12 text-xl font-bold text-ink outline-none ring-1 ring-ink/5 transition-all duration-300 placeholder:text-ink/20 placeholder:font-normal"
                        :class="[ isSpotlightActive ? 'ring-2 ring-accent/20 bg-white' : 'hover:ring-accent/10 hover:shadow-xl' ]"
                    />

                    <!-- Clear Button -->
                    <button 
                        v-if="searchQuery"
                        @click="searchQuery = ''; isSpotlightActive = false"
                        class="absolute right-4 top-1/2 -translate-y-1/2 text-ink/30 hover:text-ink transition-colors p-1"
                    >
                        <i class="ri-close-circle-fill text-xl"></i>
                    </button>
                </div>
            </div>

            <!-- Active Search Results Overlay -->
            <div 
                v-if="isSpotlightActive"
                class="absolute inset-x-0 bottom-0 top-44 z-40 overflow-hidden flex justify-center"
            >
                <div class="w-full max-w-6xl h-full grid grid-cols-12 gap-6 px-8 pb-8 animate-fade-in-up">
                    
                    <!-- Suggestions List (Left) -->
                    <div class="col-span-12 lg:col-span-4 bg-white/90 backdrop-blur rounded-2xl border border-ink/5 shadow-lg overflow-hidden flex flex-col max-h-[calc(100%-2rem)]">
                        <div class="p-4 border-b border-ink/5 text-xs font-bold uppercase tracking-wider text-ink/40 bg-gray-50/50">Suggestions</div>
                        <div class="flex-1 overflow-y-auto p-2 space-y-1 custom-scrollbar">
                            <div 
                                v-for="word in searchSuggestions" 
                                :key="word"
                                class="px-4 py-3 rounded-lg hover:bg-accent/5 hover:text-accent cursor-pointer transition-all text-lg font-medium text-ink/70 flex justify-between items-center group"
                                :class="{'bg-accent/5 text-accent': createForm.word === word}"
                                @click="selectSuggestion(word)"
                            >
                                {{ word }}
                                <i class="ri-arrow-right-line opacity-0 group-hover:opacity-100 text-sm"></i>
                            </div>
                            <div v-if="searchSuggestions.length === 0" class="p-4 text-center text-ink/30 italic">
                                No suggestions found.
                            </div>
                        </div>
                    </div>

                    <!-- Preview / Editor Panel (Right) -->
                    <div class="col-span-12 lg:col-span-8 bg-white/90 backdrop-blur rounded-2xl shadow-xl border border-ink/5 overflow-hidden flex flex-col h-[calc(100%-2rem)]">
                        
                        <!-- Empty State -->
                        <div v-if="!previewEntry" class="flex-1 flex flex-col items-center justify-center text-ink/20 gap-4">
                            <div v-show="isSearchingDef" class="animate-spin text-3xl"><i class="ri-loader-4-line"></i></div>
                            <div v-show="!isSearchingDef" class="w-20 h-20 rounded-full bg-ink/5 flex items-center justify-center">
                                <i class="ri-translate-2 text-4xl"></i>
                            </div>
                            <p v-show="!isSearchingDef">Select a word to view details</p>
                        </div>

                        <!-- Content -->
                        <div v-else class="flex flex-col h-full animate-fade-in-up">
                            <!-- Hero Section (Word + Root) -->
                            <div class="relative bg-ink/5 p-8 pb-4">
                                <!-- Expand Button -->
                                <button @click="isDetailView = true" class="absolute top-4 right-4 z-20 text-ink/30 hover:text-accent transition-colors p-2" title="Full Screen Detail">
                                    <i class="ri-expand-diagonal-line text-xl"></i>
                                </button>
                                
                                <div class="flex items-end justify-between">
                                    <div>
                                        <div class="flex items-center gap-3 mb-2">
                                            <h2 class="text-5xl font-serif font-black text-ink tracking-tighter">{{ previewEntry.word }}</h2>
                                            <!-- Important Star (Quick View) -->
                                            <button 
                                                v-if="currentVocabId"
                                                @click="toggleImportance(currentVocabId)"
                                                class="text-2xl transition-all duration-300 hover:scale-110"
                                                :class="isCurrentImportant ? 'text-yellow-400' : 'text-ink/10 hover:text-yellow-400'"
                                                title="Toggle Importance"
                                            >
                                                <i :class="isCurrentImportant ? 'ri-star-fill' : 'ri-star-line'"></i>
                                            </button>
                                        </div>
                                        <div class="flex items-center gap-3">
                                            <span v-if="previewEntry.phonetic" class="font-mono text-xs text-ink/50 bg-white/50 px-2 py-1 rounded border border-ink/5">{{ previewEntry.phonetic }}</span>
                                            
                                            <!-- Root Input Inline -->
                                            <div class="flex items-center gap-1 bg-white/50 px-2 py-1 rounded border border-ink/5 group focus-within:ring-2 ring-accent/10 focus-within:bg-white transition-all">
                                                <i class="ri-seedling-line text-xs text-ink/30"></i>
                                                <input 
                                                    v-model="createForm.root"
                                                    class="bg-transparent text-sm font-bold text-ink/60 placeholder:text-ink/20 outline-none w-24"
                                                    placeholder="Root..."
                                                />
                                            </div>
                                            <!-- Query Count Badge -->
                                            <span v-if="currentQueryCount > 0" class="flex items-center gap-1 text-xs font-bold text-accent bg-accent/5 px-2 py-1 rounded-full">
                                                <i class="ri-search-line"></i> {{ currentQueryCount }}
                                            </span>
                                        </div>
                                    </div>
                                    
                                    <!-- Translation -->
                                    <div class="flex flex-col items-end">
                                         <input 
                                            v-model="createForm.translation" 
                                            placeholder="Translation" 
                                            class="bg-transparent text-right text-3xl font-serif font-bold text-accent placeholder:text-accent/20 outline-none w-48 hover:bg-white/10 rounded transition-colors"
                                        />
                                    </div>
                                </div>
                            </div>

                            <!-- Scrollable Body -->
                            <div class="flex-1 overflow-y-auto custom-scrollbar">
                                
                                <!-- Compact Definitions -->
                                <div class="px-8 py-6 border-b border-ink/5 relative bg-white">
                                    <div 
                                        ref="definitionsRef"
                                        class="space-y-4" 
                                        :class="{ 'max-h-32 overflow-hidden': !showAllDefinitions }"
                                    >
                                        <div v-for="(m, idx) in previewEntry.meanings" :key="idx" class="flex gap-6">
                                            <span class="text-xs font-bold uppercase text-ink/30 w-16 shrink-0 pt-1 text-right">{{ m.partOfSpeech }}</span>
                                            <div class="space-y-2 flex-1">
                                                <div v-for="(def, dIdx) in m.definitions" :key="dIdx" class="text-base text-ink/80 leading-relaxed font-medium">
                                                     <!-- Full Definition Rendering -->
                                                     <div v-if="def.definition.includes('\n')" class="space-y-1">
                                                         <div v-for="(line, lIdx) in def.definition.split('\n')" :key="lIdx">
                                                             {{ line }}
                                                         </div>
                                                     </div>
                                                     <span v-else>{{ def.definition }}</span>
                                                </div>
                                            </div>
                                        </div>
                                    </div>
                                    
                                    <!-- Toggle Show More (Only if overflowing or was overflowing) -->
                                    <div v-if="isOverflowing || showAllDefinitions" class="relative z-10">
                                        <div v-if="!showAllDefinitions" class="absolute bottom-0 left-0 right-0 h-24 bg-gradient-to-t from-white via-white/90 to-transparent flex items-end justify-center pb-2 cursor-pointer hover:text-accent transition-colors" @click="showAllDefinitions = true">
                                            <span class="text-xs font-bold uppercase tracking-wide flex items-center gap-1"><i class="ri-arrow-down-double-line"></i> View Full Definition</span>
                                        </div>
                                        <div v-else class="text-center pt-4">
                                            <span class="text-xs font-bold uppercase text-ink/20 cursor-pointer hover:text-ink" @click="showAllDefinitions = false">Show Less</span>
                                        </div>
                                    </div>
                                </div>

                                <!-- Atmospheric Examples -->
                                <div class="p-8 space-y-6 bg-gray-50/50 min-h-[300px]">
                                    <label class="block text-xs font-bold uppercase tracking-wider text-ink/40 flex items-center justify-between mb-4">
                                        <span class="flex items-center gap-2"><i class="ri-clapperboard-line"></i> Cinematic Examples</span>
                                        <button @click="addExample" class="text-accent hover:text-accent/80 transition-colors">
                                            <i class="ri-add-line"></i>
                                        </button>
                                    </label>

                                    <div class="grid grid-cols-1 gap-6">
                                        <div v-for="(ex, idx) in createForm.examples" :key="idx" 
                                             class="relative aspect-[21/9] rounded-2xl overflow-hidden group shadow-lg ring-1 ring-black/5 hover:ring-accent/20 transition-all bg-white"
                                        >
                                            <!-- Background Image -->
                                            <div class="absolute inset-0 bg-ink/90 transition-all duration-700">
                                                 <img v-if="ex.image_url" :src="ex.image_url" class="absolute inset-0 w-full h-full object-cover opacity-60 group-hover:scale-105 transition-transform duration-1000">
                                                 <div v-else class="absolute inset-0 bg-gradient-to-br from-gray-800 to-black opacity-100"></div>
                                                 
                                                 <!-- Gradient Overlay -->
                                                 <div class="absolute inset-0 bg-gradient-to-t from-black/90 via-black/40 to-transparent"></div>
                                            </div>
                                            
                                            <!-- Image Upload Trigger (Hidden until hover) -->
                                            <label class="absolute top-4 right-4 z-20 text-white/20 hover:text-white cursor-pointer opacity-0 group-hover:opacity-100 transition-opacity p-2 bg-black/20 backdrop-blur rounded-lg">
                                                <i class="ri-image-add-line"></i>
                                                <input type="file" accept="image/*" class="hidden" @change="(e) => onExampleFileChange(e, idx)">
                                            </label>

                                            <!-- Remove -->
                                             <button @click="removeExample(idx)" class="absolute top-4 left-4 z-20 text-white/20 hover:text-red-400 cursor-pointer opacity-0 group-hover:opacity-100 transition-opacity">
                                                <i class="ri-delete-bin-line"></i>
                                            </button>

                                            <!-- Content Overlay -->
                                            <div class="absolute inset-0 z-10 flex flex-col justify-end p-8 text-center items-center">
                                                <textarea 
                                                     v-model="ex.sentence"
                                                     class="w-full bg-transparent border-none text-2xl font-serif font-medium text-white placeholder:text-white/20 outline-none resize-none p-0 text-center drop-shadow-md leading-relaxed"
                                                     placeholder="Type a sentence..."
                                                     rows="2"
                                                 ></textarea>
                                                 
                                                 <div class="h-0 group-hover:h-auto overflow-hidden transition-all duration-300 w-full flex flex-col items-center gap-2 opacity-0 group-hover:opacity-100">
                                                     <input 
                                                        v-model="ex.translation"
                                                        class="w-full bg-transparent text-center text-sm text-white/60 placeholder:text-white/20 outline-none"
                                                        placeholder="Translation..."
                                                     />
                                                     <input 
                                                        v-model="ex.note"
                                                        class="w-full bg-transparent text-center text-xs text-accent font-bold uppercase tracking-widest placeholder:text-white/10 outline-none"
                                                        placeholder="ADD NOTE"
                                                     />
                                                 </div>
                                            </div>
                                        </div>
                                        
                                        <!-- Add Button (Big) -->
                                        <div v-if="createForm.examples.length === 0" @click="addExample" class="aspect-[21/9] rounded-2xl border-2 border-dashed border-ink/10 flex flex-col items-center justify-center text-ink/20 hover:border-accent/30 hover:text-accent/50 cursor-pointer transition-all">
                                            <i class="ri-film-line text-4xl mb-2"></i>
                                            <span class="font-serif italic">Add an atmospheric example scene</span>
                                        </div>
                                    </div>
                                </div>
                                
                                <div class="h-20"></div> <!-- Spacer -->
                            </div>
                            
                            <!-- Save Bar -->
                             <div class="p-6 border-t border-ink/5 flex justify-between items-center bg-white/80 backdrop-blur absolute bottom-0 left-0 right-0 z-20">
                                <t-button 
                                    v-if="vocabularyList.find(v => v.word === previewEntry?.word)" 
                                    theme="danger" variant="text" 
                                    @click="deleteVocabulary(vocabularyList.find(v => v.word === previewEntry?.word)!.id)"
                                >
                                    Delete
                                </t-button>
                                <div v-else></div>

                                <t-button theme="primary" :loading="isSaving" @click="saveWord">
                                    <template #icon><i class="ri-save-3-line"></i></template>
                                    Save Record
                                </t-button>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

        </div>

        <!-- Full Screen Detail Overlay -->
        <Transition name="fade">
            <div v-if="isDetailView && previewEntry" class="fixed inset-0 z-[200] bg-white flex flex-col overflow-hidden">
                <!-- Header -->
                <div class="px-8 py-6 border-b border-ink/5 flex justify-between items-center bg-white/80 backdrop-blur z-20">
                    <div class="flex items-center gap-4">
                        <button @click="goBack" class="w-10 h-10 rounded-full hover:bg-ink/5 flex items-center justify-center text-ink/50 hover:text-ink transition-colors">
                            <i class="ri-arrow-left-line text-xl"></i>
                        </button>
                        <h2 class="text-xl font-serif font-bold">{{ previewEntry.word }}</h2>
                    </div>
                    
                    <!-- Detail View Actions -->
                    <div class="flex items-center gap-4">
                        <button 
                            @click="saveWord"
                            class="flex items-center gap-2 px-4 py-2 rounded-lg text-white bg-ink hover:bg-ink/90 font-bold transition-all text-sm shadow-lg"
                        >
                            <i class="ri-save-3-line"></i> Save Changes
                        </button>
                        
                        <template v-if="currentVocabId">
                            <button 
                                @click="deleteVocabulary(currentVocabId)"
                                class="flex items-center gap-2 px-4 py-2 rounded-lg text-red-500 hover:bg-red-50 font-bold transition-all text-sm"
                            >
                                <i class="ri-delete-bin-line"></i>
                            </button>
                        </template>
                    </div>
                </div>

                <!-- Content (Reusing the nice layout but centered) -->
                <div class="flex-1 overflow-y-auto custom-scrollbar bg-gray-50/30">
                    <div class="max-w-4xl mx-auto bg-white min-h-full shadow-2xl overflow-hidden flex flex-col">
                        <!-- Hero -->
                        <div class="relative bg-ink/5 p-16 pb-8">
                             <div class="flex items-end justify-between">
                                <div>
                                    <div class="flex items-center gap-4 mb-4">
                                         <h1 class="text-8xl font-serif font-black text-ink tracking-tighter">{{ previewEntry.word }}</h1>
                                        <!-- Important Star (Detail) -->
                                        <button 
                                            v-if="currentVocabId"
                                            @click="toggleImportance(currentVocabId)"
                                            class="text-4xl transition-all duration-300 hover:scale-110"
                                            :class="isCurrentImportant ? 'text-yellow-400' : 'text-ink/10 hover:text-yellow-400'"
                                            title="Toggle Importance"
                                        >
                                            <i :class="isCurrentImportant ? 'ri-star-fill' : 'ri-star-line'"></i>
                                        </button>
                                    </div>
                                    <div class="flex items-center gap-4 text-xl">
                                        <span v-if="previewEntry.phonetic" class="font-mono text-ink/50 bg-white/50 px-3 py-1 rounded border border-ink/5">{{ previewEntry.phonetic }}</span>
                                        <span v-if="createForm.root" class="text-ink/60 font-serif italic">root: {{ createForm.root }}</span>
                                        <!-- Query Count Badge -->
                                        <span v-if="currentQueryCount > 0" class="flex items-center gap-1 text-sm font-bold text-accent bg-accent/5 px-2 py-1 rounded-full">
                                            <i class="ri-search-line"></i> {{ currentQueryCount }} Lookups
                                        </span>
                                    </div>
                                </div>
                                <div class="text-4xl font-serif font-bold text-accent">{{ createForm.translation }}</div>
                            </div>
                        </div>

                        <!-- Definitions -->
                         <div class="p-16 border-b border-ink/5">
                            <h3 class="text-xs font-bold uppercase tracking-widest text-ink/30 mb-8">Dictionary Definition</h3>
                            <div class="space-y-8">
                                <div v-for="(m, idx) in previewEntry.meanings" :key="idx" class="flex gap-8">
                                    <div class="w-24 text-right pt-1">
                                        <span class="text-sm font-bold uppercase text-ink/40 bg-ink/5 px-2 py-1 rounded">{{ m.partOfSpeech }}</span>
                                    </div>
                                    <div class="flex-1 space-y-4">
                                        <div v-for="(def, dIdx) in m.definitions" :key="dIdx" class="text-xl text-ink/80 leading-relaxed font-serif">
                                             <span>{{ def.definition }}</span>
                                             <div v-if="def.example" class="text-base text-ink/40 mt-2 italic px-4 border-l-2 border-ink/10 pl-4">
                                                 "{{ def.example }}"
                                             </div>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </div>

                        <!-- Examples Gallery -->
                        <div class="p-16 bg-gray-50">
                            <h3 class="text-xs font-bold uppercase tracking-widest text-ink/30 mb-8 flex items-center justify-between">
                                <span class="flex items-center gap-2"><i class="ri-clapperboard-line"></i> Collected Scenes</span>
                                <button @click="addExample" class="text-accent hover:text-accent/80 transition-colors text-xs flex items-center gap-1">
                                    <i class="ri-add-line"></i> Add Scene
                                </button>
                            </h3>
                            <div class="grid grid-cols-1 gap-12">
                                <div v-for="(ex, idx) in createForm.examples" :key="idx" 
                                    class="relative aspect-[21/9] rounded-2xl overflow-hidden shadow-2xl group transition-transform hover:scale-[1.01]"
                                >
                                    <!-- Delete Button (Top Right) -->
                                    <button 
                                        v-if="currentVocabId"
                                        @click="deleteExample(idx)"
                                        class="absolute top-4 right-4 z-50 w-10 h-10 rounded-full bg-black/40 backdrop-blur text-white/50 hover:bg-red-500 hover:text-white flex items-center justify-center transition-all opacity-0 group-hover:opacity-100"
                                        title="Delete Scene"
                                    >
                                        <i class="ri-delete-bin-line"></i>
                                    </button>
                                    <!-- Image BG (Cinematic Zoom) -->
                                    <div class="absolute inset-0 overflow-hidden rounded-2xl">
                                        <img 
                                            v-if="ex.image_url" 
                                            :src="ex.image_url" 
                                            class="absolute inset-0 w-full h-full object-cover transition-transform duration-[2000ms] ease-out group-hover:scale-110"
                                        >
                                        <div v-else class="absolute inset-0 bg-gradient-to-br from-gray-800 to-black transition-colors duration-700 group-hover:from-gray-900 group-hover:to-gray-900"></div>
                                    </div>
                                    <div class="absolute inset-0 bg-gradient-to-t from-black/90 via-black/20 to-transparent transition-opacity duration-500 group-hover:opacity-80"></div>
                                    
                                    <!-- Text Content (Editable + Parallax) -->
                                    <div class="absolute inset-0 flex flex-col justify-end items-center p-12 text-center pb-16 z-20 transition-transform duration-500 ease-out group-hover:-translate-y-2">
                                        <textarea 
                                            v-model="ex.sentence"
                                            class="w-full bg-transparent border-none text-3xl font-serif font-medium text-white placeholder:text-white/20 outline-none resize-none p-0 text-center drop-shadow-xl leading-relaxed mb-4 scrollbar-hide focus:scale-105 transition-transform duration-300"
                                            placeholder="Type a sentence..."
                                            rows="2"
                                        ></textarea>
                                        
                                        <div class="h-auto overflow-hidden transition-all duration-500 w-full flex flex-col items-center gap-2 opacity-80 group-hover:opacity-100 translate-y-4 group-hover:translate-y-0">
                                            <input 
                                                v-model="ex.translation"
                                                class="w-full bg-transparent text-center text-lg text-white/80 placeholder:text-white/30 outline-none font-light tracking-wide mb-2 focus:text-white transition-colors"
                                                placeholder="Translation..."
                                            />
                                            <input 
                                                v-model="ex.note"
                                                class="bg-black/50 px-4 py-1 rounded-full backdrop-blur-md border border-accent/20 text-center text-xs text-accent font-bold uppercase tracking-widest placeholder:text-white/10 outline-none w-auto min-w-[100px] hover:bg-black/70 transition-colors"
                                                placeholder="ADD NOTE"
                                            />
                                        </div>
                                    </div>
                                </div>
                            </div>
                            
                            <div v-if="createForm.examples.length === 0" class="text-center py-20 text-ink/20 font-serif italic text-xl">
                                No examples collected yet.
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </Transition>


    <!-- Restored Detail Modal -->
    <VocabDetailModal
        v-model:visible="detailModalVisible"
        :initial-word="detailModalWord"
        :initial-sentence="detailModalData?.initialSentence"
        :initial-data="detailModalData"
        @refresh="fetchVocabularyList"
    />
    </div>
</template>

<style scoped>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

.animate-fade-in-up {
    animation: fadeInUp 0.4s ease-out forwards;
}

@keyframes fadeInUp {
    from { opacity: 0; transform: translateY(10px); }
    to { opacity: 1; transform: translateY(0); }
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
