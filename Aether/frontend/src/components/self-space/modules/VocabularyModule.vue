<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue';
import axios from 'axios';
import { MessagePlugin } from 'tdesign-vue-next';

interface Vocabulary {
    id: string;
    word: string;
    definition: string;
    context_sentence?: string;
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
}

const searchQuery = ref('');
const searchResults = ref<string[]>([]);
const currentEntry = ref<DictionaryEntry | null>(null);
const isLoading = ref(false);
const isSaving = ref(false);
const contextSentence = ref('');

const vocabularyList = ref<Vocabulary[]>([]);
const listLoading = ref(false);

const searchSuggestions = async (val: string) => {
    if (!val || val.length < 2) {
        searchResults.value = [];
        return;
    }
    try {
        const res = await axios.get(`https://api.datamuse.com/sugg?s=${val}&max=5`);
        searchResults.value = res.data.map((item: any) => item.word);
    } catch (e) {
        console.error("Datamuse error", e);
    }
};

const fetchDefinition = async (word: string) => {
    isLoading.value = true;
    currentEntry.value = null;
    searchQuery.value = word; // Update input if clicked from suggestion
    searchResults.value = []; // Clear suggestions
    
    try {
        const res = await axios.get(`https://api.dictionaryapi.dev/api/v2/entries/en/${word}`);
        if (res.data && res.data.length > 0) {
            currentEntry.value = res.data[0];
            contextSentence.value = ''; // Reset context
        }
    } catch (e) {
        MessagePlugin.error('Definition not found');
    } finally {
        isLoading.value = false;
    }
};

const addToVocabulary = async () => {
    if (!currentEntry.value) return;

    isSaving.value = true;
    try {
        // Prepare simplified definition string (Take first 2 definitions)
        const defs = currentEntry.value.meanings.map(m => 
            `(${m.partOfSpeech}) ${m.definitions[0].definition}`
        ).slice(0, 2).join('; ');

        // Explicitly set header with current token (copied from EditorView logic)
        // Ideally this should be in an interceptor or global config
        const token = localStorage.getItem('aether_token');
        const config = {
            headers: { Authorization: `Bearer ${token}` }
        };

        await axios.post('/api/vocabulary', {
            word: currentEntry.value.word,
            definition: defs,
            context_sentence: contextSentence.value || undefined,
        }, config);

        MessagePlugin.success('Word saved to vocabulary!');
        currentEntry.value = null; // Clear after save
        searchQuery.value = '';
        fetchVocabularyList(); // Refresh list
    } catch (e) {
        console.error(e);
        MessagePlugin.error('Failed to save word');
    } finally {
        isSaving.value = false;
    }
};

const fetchVocabularyList = async () => {
    listLoading.value = true;
    try {
        const token = localStorage.getItem('aether_token');
        const config = {
            headers: { Authorization: `Bearer ${token}` }
        };
        const res = await axios.get('/api/vocabulary?limit=20', config);
        vocabularyList.value = res.data;
    } catch (e) {
        console.error(e);
    } finally {
        listLoading.value = false;
    }
};

const deleteVocabulary = async (id: string) => {
    try {
        const token = localStorage.getItem('aether_token');
        const config = {
            headers: { Authorization: `Bearer ${token}` }
        };
        await axios.delete(`/api/vocabulary/${id}`, config);
        MessagePlugin.success('Deleted');
        fetchVocabularyList();
    } catch (e) {
        MessagePlugin.error('Failed to delete');
    }
};

onMounted(() => {
    fetchVocabularyList();
});
</script>

<template>
    <div class="h-full flex flex-col gap-6 p-6 max-w-4xl mx-auto overflow-y-auto">
        <!-- Header -->
        <div class="flex flex-col gap-2">
            <h1 class="text-3xl font-black tracking-tight text-ink">Vocabulary</h1>
            <p class="text-ink/60">Search for words, get definitions, and build your personal dictionary.</p>
        </div>

        <!-- Search Section -->
        <div class="relative z-20">
            <div class="flex gap-4">
                <t-input 
                    v-model="searchQuery" 
                    placeholder="Type a word..." 
                    size="large" 
                    clearable
                    @change="searchSuggestions"
                    @enter="fetchDefinition(searchQuery)"
                >
                    <template #prefix-icon>
                        <i class="ri-search-line"></i>
                    </template>
                </t-input>
                <t-button size="large" theme="primary" :loading="isLoading" @click="fetchDefinition(searchQuery)">
                    Search
                </t-button>
            </div>
            
            <!-- Context Menu for Suggestions -->
            <div v-if="searchResults.length > 0" class="absolute top-full left-0 w-full bg-paper border border-ink/10 rounded-lg shadow-xl mt-2 overflow-hidden">
                <div 
                    v-for="word in searchResults" 
                    :key="word"
                    class="px-4 py-3 hover:bg-accent/5 cursor-pointer text-ink"
                    @click="fetchDefinition(word)"
                >
                    {{ word }}
                </div>
            </div>
        </div>

        <!-- Result Card -->
        <div v-if="currentEntry" class="bg-paper border border-ink/10 rounded-xl p-6 shadow-sm animate-fade-in-up">
            <div class="flex justify-between items-start mb-4">
                <div>
                    <h2 class="text-4xl font-serif font-bold text-ink">{{ currentEntry.word }}</h2>
                    <span v-if="currentEntry.phonetic" class="text-ink/50 font-mono mt-1 block">{{ currentEntry.phonetic }}</span>
                </div>
                <t-tag theme="primary" variant="light">New Entry</t-tag>
            </div>

            <!-- Meanings -->
            <div class="space-y-4 mb-6">
                <div v-for="(meaning, idx) in currentEntry.meanings.slice(0, 3)" :key="idx" class="text-ink/80">
                    <span class="font-bold italic opacity-70 mr-2">{{ meaning.partOfSpeech }}</span>
                    <span>{{ meaning.definitions[0].definition }}</span>
                </div>
            </div>

            <!-- Context Input -->
            <div class="bg-accent/5 p-4 rounded-lg mb-4">
                <label class="block text-xs font-bold uppercase tracking-wider text-ink/50 mb-2">My Context Sentence</label>
                <t-textarea 
                    v-model="contextSentence" 
                    placeholder="e.g. I learned this word while reading..." 
                    autosize
                />
            </div>

            <div class="flex justify-end">
                <t-button theme="primary" :loading="isSaving" @click="addToVocabulary">
                    <template #icon><i class="ri-add-line"></i></template>
                    Add to Collection
                </t-button>
            </div>
        </div>

        <!-- Saved Vocabulary List -->
        <div class="mt-8">
            <h3 class="text-xl font-bold mb-4 flex items-center gap-2">
                <i class="ri-book-mark-line text-accent"></i>
                My Collection
            </h3>
            
            <div v-if="listLoading" class="flex justify-center py-8">
                <t-loading />
            </div>
            
            <div v-else-if="vocabularyList.length === 0" class="text-center py-12 text-ink/40 bg-paper/50 rounded-xl border border-dashed border-ink/10">
                Start searching to add words!
            </div>

            <div v-else class="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div v-for="vocab in vocabularyList" :key="vocab.id" class="group bg-paper border border-ink/5 hover:border-accent/50 p-5 rounded-xl transition-all hover:shadow-lg relative">
                    <button class="absolute top-4 right-4 text-ink/20 hover:text-red-500 opacity-0 group-hover:opacity-100 transition-all" @click="deleteVocabulary(vocab.id)">
                        <i class="ri-delete-bin-line"></i>
                    </button>
                    
                    <h4 class="text-xl font-bold mb-1">{{ vocab.word }}</h4>
                    <p class="text-sm text-ink/60 line-clamp-2 mb-3 h-10">{{ vocab.definition }}</p>
                    
                    <div v-if="vocab.context_sentence" class="text-xs bg-accent/5 px-3 py-2 rounded text-ink/80 italic border-l-2 border-accent">
                        "{{ vocab.context_sentence }}"
                    </div>
                </div>
            </div>
        </div>
    </div>
</template>

<style scoped>
.animate-fade-in-up {
    animation: fadeInUp 0.4s ease-out;
}

@keyframes fadeInUp {
    from { opacity: 0; transform: translateY(10px); }
    to { opacity: 1; transform: translateY(0); }
}
</style>
