<template>
  <div class="analysis-card bg-white border border-gray-100 rounded-xl shadow-sm overflow-hidden flex flex-col">
    <!-- Empty State -->
    <div v-if="!word" class="flex-1 flex flex-col items-center justify-center p-8 text-center text-gray-400">
        <i class="ri-cursor-line text-3xl mb-2 opacity-50"></i>
        <p class="text-sm">Select a word or sentence to see analysis.</p>
    </div>

    <div v-else class="flex flex-col h-full">
        <!-- Header -->
        <div class="p-6 border-b border-gray-50 bg-gray-50/50">
            <h2 class="text-3xl font-bold text-gray-900 tracking-tight mb-1">{{ word }}</h2>
            <div v-if="loading" class="text-xs text-gray-400 animate-pulse">Loading definition...</div>
            <span v-else-if="existingVocab?.phonetic" class="text-sm text-gray-500 font-mono">{{ existingVocab.phonetic }}</span>
        </div>

        <!-- Content Scroll -->
        <div class="flex-1 overflow-y-auto p-6 space-y-6 custom-scrollbar">
            
            <!-- Definition -->
            <div v-if="existingVocab" class="space-y-2">
                <div class="text-xs font-bold uppercase tracking-widest text-gray-400">Definition</div>
                <p class="text-gray-800 leading-relaxed">{{ existingVocab.definition }}</p>
                <p v-if="existingVocab.translation" class="text-gray-500 italic">{{ existingVocab.translation }}</p>
                
                <div class="flex gap-2 mt-2">
                    <span class="px-2 py-0.5 rounded bg-gray-100 text-gray-600 text-xs font-medium border border-gray-200">
                        {{ existingVocab.status || 'New' }}
                    </span>
                </div>
            </div>

            <!-- Context Sentence -->
            <div v-if="sentence" class="space-y-3 pt-4 border-t border-gray-100">
                <div class="text-xs font-bold uppercase tracking-widest text-gray-400">Context</div>
                <blockquote class="pl-3 border-l-2 border-indigo-500 text-gray-600 italic text-sm leading-relaxed">
                    "{{ sentence.text }}"
                </blockquote>
                
                <!-- Actions -->
                <button 
                    @click="saveExample" 
                    :disabled="saving || !existingVocab"
                    class="w-full mt-2 py-2 px-4 bg-gray-900 text-white rounded-lg text-sm font-medium hover:bg-black transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2"
                >
                    <i v-if="saving" class="ri-loader-4-line animate-spin"></i>
                    <span>{{ saving ? 'Saving...' : 'Save to Vocabulary' }}</span>
                </button>
            </div>
            
            <!-- New Word Action -->
            <div v-else-if="!loading && !existingVocab" class="text-center py-4">
                 <p class="text-sm text-gray-500 mb-4">Word not found in your database.</p>
                 <button @click="createNewVocab" class="text-indigo-600 text-sm font-bold hover:underline">
                    Create New Card
                 </button>
            </div>
        </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import { useVocabularyStore } from '@/stores/vocabulary';
import { MessagePlugin } from 'tdesign-vue-next';

const props = defineProps<{
    word: string;
    sentence?: { text: string; uuid: string; articleId: string };
}>();

const store = useVocabularyStore();
const existingVocab = ref<any>(null);
const loading = ref(false);
const saving = ref(false);

const fetchData = async () => {
    if (!props.word) {
        existingVocab.value = null;
        return;
    }
    loading.value = true;
    try {
        existingVocab.value = await store.searchWord(props.word);
    } catch (e) {
        console.error(e);
    } finally {
        loading.value = false;
    }
};

watch(() => props.word, fetchData, { immediate: true });

async function saveExample() {
    if (!existingVocab.value || !props.sentence) return;
    
    saving.value = true;
    try {
        await store.addExample(existingVocab.value.id, {
            sentence: props.sentence.text,
            article_id: props.sentence.articleId,
            sentence_uuid: props.sentence.uuid
        });
        MessagePlugin.success('Example added to vocabulary!');
    } catch (e) {
        MessagePlugin.error('Failed to add example.');
        console.error(e);
    } finally {
        saving.value = false;
    }
}

function createNewVocab() {
    MessagePlugin.info('Quick Create feature coming soon.');
}
</script>

<style scoped>
.analysis-card {
    height: 100%;
    max-height: calc(100vh - 8rem); /* Prevent overflow */
}

.custom-scrollbar::-webkit-scrollbar { width: 4px; }
.custom-scrollbar::-webkit-scrollbar-thumb { background: #e5e5e5; border-radius: 2px; }
.custom-scrollbar::-webkit-scrollbar-track { background: transparent; }
</style>
