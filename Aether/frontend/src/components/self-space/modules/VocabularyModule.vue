<script setup lang="ts">
import { ref, reactive, computed, onMounted, watch, nextTick } from 'vue';
import { MessagePlugin } from 'tdesign-vue-next';
import { useDebounceFn } from '@vueuse/core';
import { dictionaryApi, type DictionaryEntry } from '@/api/dictionary';
import axios from 'axios';

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

            // 3. Length Priority (Shorter is usually more relevant for equal match types)
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
    } else {
        // Reset form for new entry
        createForm.translation = '';
        createForm.root = '';
        createForm.examples = [];
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
            // ... assignment logic
        }
    } catch (e) {
        console.error('Lookup Error:', e);
        // ... error logic
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
        await axios.post('/api/vocabulary', createForm, config);
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
        }
    } catch (e) {
        MessagePlugin.error('Delete failed');
    }
};

const fetchVocabularyList = async (query = '') => {
    listLoading.value = true;
    try {
        const token = localStorage.getItem('aether_token');
        const config = { headers: { Authorization: `Bearer ${token}` } };
        const url = query 
            ? `/api/vocabulary?limit=100&query=${query}` 
            : '/api/vocabulary?limit=100';
            
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

window.addEventListener('popstate', () => {
    if (isDetailView.value) {
        isDetailView.value = false;
    }
});

const goBack = () => {
    history.back();
};
</script>

<template>
    <div class="h-full flex flex-col relative font-sans text-ink">
        
        <!-- Header Info (Fade out when searching) -->
        <!-- Temporarily hidden per user request for clean look
        <div class="absolute top-0 left-0 right-0 z-10 flex items-center justify-between px-8 py-6 transition-all duration-500" 
             :class="{ 'opacity-0 -translate-y-4 pointer-events-none': isSpotlightActive }">
            <h1 class="text-3xl font-serif font-black tracking-tight">Vocabulary</h1>
            <div class="bg-ink/5 px-3 py-1 rounded-full text-xs font-bold text-ink/50">
                {{ vocabularyList.length }} Entries
            </div>
        </div>
        -->

        <!-- Main Scrollable Area -->
        <div class="flex-1 relative w-full h-full overflow-hidden">
            
            <!-- Collection Grid (Underneath) -->
            <!-- Temporarily hidden per user request
            <div class="w-full h-full overflow-y-auto px-8 py-32 custom-scrollbar transition-all duration-500 ease-out"
                 :class="{ 'opacity-20 scale-95 blur-sm overflow-hidden': isSpotlightActive }">
                <div v-if="vocabularyList.length === 0" class="text-center py-20 opacity-30">
                    <p class="text-4xl mb-4">✨</p>
                    Your collection is empty.
                </div>
                
                <div v-else class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-6 pb-20 max-w-7xl mx-auto">
                     <div 
                        v-for="item in vocabularyList" 
                        :key="item.id" 
                        class="group relative bg-white rounded-xl shadow-sm hover:shadow-xl hover:-translate-y-1 transition-all duration-300 h-64 overflow-hidden border border-ink/5 cursor-pointer"
                        @click="searchQuery = item.word; selectSuggestion(item.word)"
                    >
                        <div v-if="item.image_url" class="absolute inset-0">
                            <img :src="item.image_url" class="w-full h-full object-cover opacity-10 group-hover:opacity-20 transition-opacity filter grayscale">
                        </div>
                        <div class="relative z-10 p-6 h-full flex flex-col justify-between">
                            <div>
                                <h3 class="text-2xl font-serif font-bold text-ink mb-1 group-hover:text-accent transition-colors">{{ item.word }}</h3>
                                <p class="text-sm text-ink/50 line-clamp-2 h-10">{{ item.definition }}</p>
                            </div>
                            <div v-if="item.translation" class="text-xs font-bold text-accent/80 bg-accent/5 px-2 py-1 rounded w-fit">
                                {{ item.translation }}
                            </div>
                        </div>
                    </div>
                </div>
            </div>
            -->

            <!-- Unified Search Bar -->
            <!-- Position: centered by default (top-1/2), moves to top (top-24) when active -->
            <div 
                class="absolute left-0 right-0 z-50 flex justify-center transition-all duration-500 cubic-bezier(0.34, 1.56, 0.64, 1)"
                :class="[ isSpotlightActive ? 'top-24 translate-y-0' : 'top-[35%] -translate-y-1/2' ]"
            >
                <div 
                    class="relative group w-full transition-all duration-500 ease-out"
                    :class="[ isSpotlightActive ? 'max-w-4xl' : 'max-w-xl' ]"
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
                                        <h2 class="text-5xl font-serif font-black text-ink tracking-tighter mb-2">{{ previewEntry.word }}</h2>
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
            <div v-if="isDetailView && previewEntry" class="fixed inset-0 z-[100] bg-white flex flex-col overflow-hidden">
                <!-- Header -->
                <div class="px-8 py-6 border-b border-ink/5 flex justify-between items-center bg-white/80 backdrop-blur z-20">
                    <div class="flex items-center gap-4">
                        <button @click="goBack" class="w-10 h-10 rounded-full hover:bg-ink/5 flex items-center justify-center text-ink/50 hover:text-ink transition-colors">
                            <i class="ri-arrow-left-line text-xl"></i>
                        </button>
                        <h2 class="text-xl font-serif font-bold">{{ previewEntry.word }}</h2>
                    </div>
                </div>

                <!-- Content (Reusing the nice layout but centered) -->
                <div class="flex-1 overflow-y-auto custom-scrollbar bg-gray-50/30">
                    <div class="max-w-4xl mx-auto bg-white min-h-full shadow-2xl overflow-hidden flex flex-col">
                        <!-- Hero -->
                        <div class="relative bg-ink/5 p-16 pb-8">
                             <div class="flex items-end justify-between">
                                <div>
                                    <h1 class="text-8xl font-serif font-black text-ink tracking-tighter mb-4">{{ previewEntry.word }}</h1>
                                    <div class="flex items-center gap-4 text-xl">
                                        <span v-if="previewEntry.phonetic" class="font-mono text-ink/50 bg-white/50 px-3 py-1 rounded border border-ink/5">{{ previewEntry.phonetic }}</span>
                                        <span v-if="createForm.root" class="text-ink/60 font-serif italic">root: {{ createForm.root }}</span>
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
                            <h3 class="text-xs font-bold uppercase tracking-widest text-ink/30 mb-8 flex items-center gap-2">
                                <i class="ri-clapperboard-line"></i> Collected Scenes
                            </h3>
                            <div class="grid grid-cols-1 gap-12">
                                <div v-for="(ex, idx) in createForm.examples" :key="idx" 
                                     class="relative aspect-[21/9] rounded-2xl overflow-hidden shadow-2xl group transition-transform hover:scale-[1.01]"
                                >
                                    <!-- Image BG -->
                                    <img v-if="ex.image_url" :src="ex.image_url" class="absolute inset-0 w-full h-full object-cover">
                                    <div v-else class="absolute inset-0 bg-gradient-to-br from-gray-800 to-black"></div>
                                    <div class="absolute inset-0 bg-gradient-to-t from-black/90 via-black/20 to-transparent"></div>
                                    
                                    <!-- Text Content -->
                                    <div class="absolute inset-0 flex flex-col justify-end items-center p-12 text-center pb-16">
                                        <p class="text-3xl font-serif font-medium text-white mb-4 drop-shadow-xl leading-relaxed">
                                            "{{ ex.sentence }}"
                                        </p>
                                        <p v-if="ex.translation" class="text-white/60 text-lg font-light tracking-wide mb-2">{{ ex.translation }}</p>
                                        <p v-if="ex.note" class="text-accent text-xs font-bold uppercase tracking-widest bg-black/50 px-3 py-1 rounded-full backdrop-blur-md border border-accent/20">{{ ex.note }}</p>
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
