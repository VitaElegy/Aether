<script setup lang="ts">
import { ref, reactive, computed, onMounted, watch, nextTick } from 'vue';
import axios from 'axios';
import { MessagePlugin } from 'tdesign-vue-next';
import { useDebounceFn } from '@vueuse/core';

interface Vocabulary {
    id: string;
    word: string;
    definition: string;
    translation?: string;
    phonetic?: string;
    context_sentence?: string;
    image_url?: string;
    language: string;
    status: string;
    created_at: string;
}

interface DictionaryEntry {
    word: string;
    phonetic?: string;
    meanings: {
        partOfSpeech: string;
        definitions: { definition: string; example?: string }[];
    }[];
    translation?: string;
    source?: string;
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

// Create Form
const createForm = reactive({
    word: '',
    definition: '',
    translation: '',
    phonetic: '',
    context_sentence: '',
    image_url: '',
    language: 'en'
});
const isSaving = ref(false);
const isUploading = ref(false);

const inputRef = ref<HTMLInputElement | null>(null);

// Methods
const debouncedSearch = useDebounceFn(async (val: string) => {
    // 2. Local Fuzzy Search via Backend
    try {
        const token = localStorage.getItem('aether_token');
        const res = await axios.get(`/api/dictionary/fuzzy?word=${val}`, {
            headers: { Authorization: `Bearer ${token}` }
        });
        // Backend returns Vec<String> directly
        const remoteWords = res.data;
        
        // Combine with local matches (from saved vocabulary)
        const localMatches = vocabularyList.value
            .filter(v => v.word.toLowerCase().includes(val.toLowerCase()))
            .map(v => v.word);

        const combined = new Set([...localMatches, ...remoteWords]);
        searchSuggestions.value = Array.from(combined).slice(0, 8);
    } catch (e) {
        // Fallback or just show local matches if backend fails
        const localMatches = vocabularyList.value
            .filter(v => v.word.toLowerCase().includes(val.toLowerCase()))
            .map(v => v.word);
         searchSuggestions.value = localMatches.slice(0, 8);
    }
}, 300);

watch(searchQuery, (val) => {
    if (!val) {
        isSpotlightActive.value = false;
        searchSuggestions.value = [];
        return;
    }
    isSpotlightActive.value = true;
    debouncedSearch(val);
});

const selectSuggestion = async (word: string) => {
    createForm.word = word;
    const existing = vocabularyList.value.find(v => v.word.toLowerCase() === word.toLowerCase());
    
    if (existing) {
        previewEntry.value = {
            word: existing.word,
            phonetic: existing.phonetic,
            meanings: [{
                partOfSpeech: 'saved',
                definitions: [{ definition: existing.definition }]
            }],
            translation: existing.translation
        };
        previewImage.value = existing.image_url || '';
        createForm.translation = existing.translation || '';
    } else {
        await fetchDefinitionInfo(word);
    }
};

const fetchDefinitionInfo = async (word: string) => {
    isSearchingDef.value = true;
    try {
        const token = localStorage.getItem('aether_token');
        // Call our new backend service instead of external API directly
        const res = await axios.get(`/api/dictionary/lookup?word=${word}`, {
            headers: { Authorization: `Bearer ${token}` }
        });
        
        if (res.data) {
            // Backend now returns normalized DictionaryEntry
            const entry: DictionaryEntry = res.data;
            previewEntry.value = entry;
            createForm.phonetic = entry.phonetic || '';
            const defs = entry.meanings.map(m => `(${m.partOfSpeech}) ${m.definitions[0].definition}`).slice(0, 2).join('\n');
            createForm.definition = defs;
            createForm.translation = entry.translation || "";
        }
    } catch (e) {
        previewEntry.value = {
             word: word,
             meanings: [{ partOfSpeech: 'unknown', definitions: [{ definition: 'No definition found.' }] }]
        };
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
    } catch (e) {
        MessagePlugin.error('Failed to save');
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

// Close spotlight when clicking outside logic if needed
// For now, let's just use the close button or clearing input
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
                            <!-- Hero Section (Word + Image) -->
                            <div class="relative h-48 bg-ink/5 flex items-end p-8 overflow-hidden group shrink-0">
                                <div v-if="previewImage" class="absolute inset-0 z-0">
                                    <img :src="previewImage" class="w-full h-full object-cover opacity-30 group-hover:opacity-40 transition-opacity grayscale group-hover:grayscale-0">
                                </div>
                                <div class="absolute top-4 right-4 z-20">
                                     <label class="cursor-pointer bg-white/20 hover:bg-white/40 text-ink/60 hover:text-ink p-2 rounded-lg backdrop-blur transition-all flex items-center gap-2 text-xs font-bold uppercase">
                                        <i class="ri-image-edit-line"></i>
                                        {{ isUploading ? '...' : 'Change Image' }}
                                        <input type="file" accept="image/*" class="hidden" @change="onFileChange">
                                    </label>
                                </div>
                                
                                <div class="relative z-10">
                                    <h2 class="text-6xl font-serif font-black text-ink mb-2 tracking-tighter">{{ previewEntry.word }}</h2>
                                    <div class="flex items-center gap-4">
                                        <span v-if="previewEntry.phonetic" class="font-mono text-ink/50 bg-white/50 px-2 py-1 rounded border border-ink/5">{{ previewEntry.phonetic }}</span>
                                        <input 
                                            v-model="createForm.translation" 
                                            placeholder="Add Translation..." 
                                            class="bg-transparent text-accent font-bold placeholder:text-accent/30 outline-none w-48 hover:bg-white/10 px-2 rounded transition-colors"
                                        />
                                    </div>
                                </div>
                            </div>

                            <!-- Details Scroll -->
                            <div class="flex-1 overflow-y-auto p-8 space-y-8 custom-scrollbar">
                                <div class="space-y-6">
                                    <div v-for="(m, idx) in previewEntry.meanings.slice(0, 5)" :key="idx" class="pl-4 border-l-2 border-accent/20">
                                        <span class="text-xs font-bold uppercase text-ink/40 mb-1 block">{{ m.partOfSpeech }}</span>
                                        <p class="text-lg leading-relaxed text-ink/80">{{ m.definitions[0].definition }}</p>
                                    </div>
                                    <div v-if="previewEntry.source" class="text-xs text-ink/30 italic text-right mt-2">
                                        Source: {{ previewEntry.source }}
                                    </div>
                                </div>

                                <div class="bg-gray-50 p-6 rounded-xl">
                                    <label class="block text-xs font-bold uppercase tracking-wider text-ink/40 mb-3 flex items-center gap-2">
                                        <i class="ri-chat-quote-line"></i> Context Sentence
                                    </label>
                                    <textarea 
                                        v-model="createForm.context_sentence"
                                        class="w-full bg-transparent border-none text-xl font-serif italic text-ink/80 placeholder:text-ink/20 outline-none resize-none"
                                        placeholder="Where did you find this word? Type a sentence..."
                                        rows="3"
                                    ></textarea>
                                </div>
                            </div>

                            <div class="p-6 border-t border-ink/5 flex justify-between items-center bg-gray-50/50 shrink-0">
                                <t-button 
                                    v-if="vocabularyList.find(v => v.word === previewEntry?.word)" 
                                    theme="danger" variant="text" 
                                    @click="deleteVocabulary(vocabularyList.find(v => v.word === previewEntry?.word)!.id)"
                                >
                                    Delete Entry
                                </t-button>
                                <div v-else></div>

                                <t-button size="large" theme="primary" :loading="isSaving" @click="saveWord">
                                    <template #icon><i class="ri-save-3-line"></i></template>
                                    Save to Collection
                                </t-button>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

        </div>
    </div>
</template>

<style scoped>
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
