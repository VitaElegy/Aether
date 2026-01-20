<template>
  <div class="analysis-card bg-white border border-gray-100 rounded-xl shadow-sm overflow-hidden flex flex-col h-full">
    <!-- Empty State -->
    <div v-if="!word && !activeSentence" class="flex-1 flex flex-col items-center justify-center p-8 text-center text-gray-400">
        <i class="ri-cursor-line text-3xl mb-2 opacity-50"></i>
        <p class="text-sm">Select a word or sentence to see analysis.</p>
    </div>

    <!-- Active State -->
    <template v-else>
        
        <!-- SECTION 1: SENTENCE CONTEXT (Always Visible at Top if Sentence Selected) -->
        <div v-if="activeSentence" class="flex-none p-6 bg-gray-50/50 border-b border-gray-100 transition-colors duration-300">
             <div class="flex items-center justify-between mb-3">
                <div class="text-xs font-bold uppercase tracking-widest text-gray-400">Context</div>
                <!-- Carousel Controls -->
                <div v-if="activeTotal > 1" class="flex items-center gap-2 bg-white rounded-lg p-1 shadow-sm border border-gray-100">
                    <button @click="prevSlide" class="w-6 h-6 flex items-center justify-center hover:bg-gray-50 rounded text-gray-500 transition-all">
                        <i class="ri-arrow-left-s-line"></i>
                    </button>
                    <span class="text-xs font-mono text-gray-500 w-8 text-center">{{ activeIndex + 1 }}/{{ activeTotal }}</span>
                    <button @click="nextSlide" class="w-6 h-6 flex items-center justify-center hover:bg-gray-50 rounded text-gray-500 transition-all">
                        <i class="ri-arrow-right-s-line"></i>
                    </button>
                </div>
            </div>

            <!-- Interactive Sentence Card -->
            <div class="relative group">
                <div class="text-gray-700 font-serif text-lg leading-relaxed select-none">
                    <span 
                        v-for="(token, idx) in interactiveTokens" 
                        :key="idx"
                        class="hover:bg-gray-200 rounded cursor-pointer transition-colors duration-150 inline-block px-1 -mx-0.5 border border-transparent"
                        :class="{ 
                            'bg-gray-900 text-white font-medium shadow-sm border-gray-900': token.clean === displayWord
                        }"
                        @click.stop="handleWordClick(token.clean)"
                    >{{ token.raw }} </span>
                </div>
            </div>
        </div>

        <!-- SECTION 2: DETAILS PANEL (Scrollable Bottom) -->
        <div class="flex-1 flex flex-col min-h-0 bg-white relative">
            
            <!-- Details Header -->
            <div class="p-6 pb-2">
                 <h2 class="text-3xl font-bold text-gray-900 tracking-tight mb-1 serif-font">
                    {{ displayWord || 'Sentence Selected' }}
                </h2>
                <div class="flex items-center gap-2 h-5 text-sm">
                    <template v-if="loading">
                        <span class="text-xs text-gray-400 animate-pulse">Loading...</span>
                    </template>
                    <template v-else-if="existingVocab?.phonetic">
                        <span class="text-gray-500 font-mono">/{{ existingVocab.phonetic }}/</span>
                    </template>
                     <template v-else-if="!displayWord">
                        <span class="text-xs text-gray-400 uppercase tracking-widest">Select a word above</span>
                    </template>
                </div>
            </div>

            <!-- Scrollable Body -->
            <div class="flex-1 overflow-y-auto p-6 pt-4 space-y-6 custom-scrollbar">
                
                <!-- Case A: Word Details exist -->
                <div v-if="existingVocab" class="space-y-4 animate-fade-in-up">
                    <div class="space-y-1">
                        <div class="text-xs font-bold uppercase tracking-widest text-gray-400">Definition</div>
                        <p class="text-gray-800 leading-relaxed text-lg">{{ existingVocab.definition }}</p>
                        <p v-if="existingVocab.translation" class="text-gray-500 italic">{{ existingVocab.translation }}</p>
                    </div>
                    
                    <div class="flex gap-2">
                        <span class="px-2 py-0.5 rounded bg-gray-100 text-gray-600 text-xs font-medium border border-gray-200 uppercase tracking-wider">
                            {{ existingVocab.status || 'New' }}
                        </span>
                        <span v-if="existingVocab.query_count > 0" class="px-2 py-0.5 rounded bg-amber-50 text-amber-600 text-xs font-medium border border-amber-100 flex items-center gap-1">
                            <i class="ri-fire-line"></i> {{ existingVocab.query_count }}
                        </span>
                        <button @click="createNewVocab" class="ml-auto px-2 py-1 rounded hover:bg-gray-100 text-gray-400 hover:text-indigo-600 transition-colors text-xs uppercase font-bold tracking-wider" title="Edit Entry">
                            <i class="ri-edit-line"></i> Edit
                        </button>
                    </div>

                    <!-- Connect Sentence Action / Editor -->
                    <div v-if="activeSentence" class="pt-4 border-t border-gray-100">
                         <!-- If Saved Example: Show Editor -->
                         <div v-if="currentExample" class="space-y-3 bg-gray-50 p-4 rounded-lg border border-gray-200">
                            <div class="flex items-center justify-between text-xs text-gray-500 font-bold uppercase tracking-wider">
                                <span><i class="ri-link"></i> Linked Context</span>
                                <span class="text-green-600"><i class="ri-check-line"></i> Saved</span>
                            </div>
                            
                            <!-- Translation Input -->
                            <div class="space-y-1">
                                <label class="text-xs text-gray-400 font-medium">Translation</label>
                                <textarea 
                                    v-model="exampleTranslation"
                                    class="w-full text-sm p-2 bg-white border border-gray-200 rounded text-gray-700 focus:outline-none focus:border-gray-400 resize-none transition-colors"
                                    rows="1"
                                    placeholder="Add sentence translation..."
                                ></textarea>
                            </div>

                            <!-- Comment Input -->
                            <div class="space-y-1">
                                <label class="text-xs text-gray-400 font-medium">Note</label>
                                <textarea 
                                    v-model="exampleNote"
                                    class="w-full text-sm p-2 bg-white border border-gray-200 rounded text-gray-700 focus:outline-none focus:border-gray-400 resize-none transition-colors"
                                    rows="2"
                                    placeholder="Add comments or notes..."
                                ></textarea>
                            </div>

                            <!-- Update Button (Only if changed) -->
                            <button 
                                v-if="exampleChanged"
                                @click="updateExampleDetails" 
                                :disabled="saving"
                                class="w-full py-2 px-3 bg-gray-800 text-white rounded text-xs font-bold hover:bg-black transition-all flex items-center justify-center gap-2"
                            >
                                <i v-if="saving" class="ri-loader-4-line animate-spin"></i>
                                <span v-else>Save Changes</span>
                            </button>
                         </div>

                         <!-- If NOT Saved: Show Connect Button -->
                         <button 
                            v-else
                            @click="saveExample" 
                            :disabled="saving"
                            class="w-full py-2.5 px-4 bg-gray-900 text-white rounded-lg text-sm font-medium hover:bg-black transition-all shadow-sm active:scale-[0.98] disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2"
                        >
                            <i v-if="saving" class="ri-loader-4-line animate-spin"></i>
                            <span>Save Current Sentence as Example</span>
                        </button>
                    </div>
                </div>

                <!-- Case B: Word Selected but Unknown -->
                <div v-else-if="!loading && displayWord" class="space-y-4 animate-fade-in-up">
                     <div class="p-6 rounded-xl bg-gray-50 border border-gray-100 text-center space-y-3">
                         <div class="w-12 h-12 rounded-full bg-white shadow-sm flex items-center justify-center mx-auto text-gray-300">
                             <i class="ri-question-mark text-xl"></i>
                         </div>
                         <p class="text-gray-600">
                            "<span class="font-bold text-gray-800">{{ displayWord }}</span>" is not in your dictionary.
                         </p>
                         <button 
                            @click="createNewVocab" 
                            class="w-full py-2.5 bg-indigo-600 text-white shadow-md shadow-indigo-100 rounded-lg text-sm font-bold hover:bg-indigo-700 transition-all flex items-center justify-center gap-2"
                        >
                            <i class="ri-add-line"></i> Create Entry
                         </button>
                    </div>
                </div>

                <!-- Case C: No Word Selected (Sentence Only) or Shared Sentence Context -->
                <div v-if="activeSentence" class="space-y-6 pt-4 border-t border-gray-100 animate-fade-in-up">
                    
                    <!-- Header -->
                    <div class="flex items-center justify-between">
                         <div class="text-xs font-bold uppercase tracking-widest text-gray-400">Sentence Analysis</div>
                         <button 
                            @click="toggleEditMode" 
                            class="text-xs font-bold uppercase tracking-widest transition-colors"
                            :class="isEditMode ? 'text-red-500 hover:text-red-700' : 'text-gray-400 hover:text-indigo-600'"
                         >
                            <i :class="isEditMode ? 'ri-close-line' : 'ri-edit-2-line'"></i> {{ isEditMode ? 'Cancel Edit' : 'Edit Text' }}
                         </button>
                    </div>

                    <!-- SMART EDIT MODE -->
                    <div v-if="isEditMode" class="bg-red-50 p-4 rounded-lg border border-red-100 space-y-3">
                        <div class="flex items-center gap-2 text-red-600 text-xs font-bold uppercase tracking-wider">
                            <i class="ri-alert-line"></i> Careful: Changing text migrates comments
                        </div>
                        <textarea 
                            v-model="editModeText"
                            class="w-full text-lg p-3 bg-white border border-red-200 rounded-lg text-gray-900 focus:outline-none focus:border-red-400 resize-none font-serif leading-relaxed"
                            rows="4"
                        ></textarea>
                        <button 
                            @click="saveSmartEdit"
                            :disabled="saving"
                            class="w-full py-2 bg-red-600 text-white rounded-lg text-sm font-bold hover:bg-red-700 transition-all shadow-sm"
                        >
                            Confirm Text Change
                        </button>
                    </div>

                    <!-- ANNOTATION MODE (Normal) -->
                    <div v-else class="space-y-4">
                        <!-- Translation -->
                        <div class="space-y-1">
                            <label class="text-xs text-gray-400 font-medium ml-1">Translation</label>
                            <div class="relative group">
                                <textarea 
                                    v-model="localTranslation"
                                    class="w-full text-base p-3 bg-gray-50 border border-transparent hover:bg-white hover:border-gray-200 focus:bg-white focus:border-indigo-300 rounded-lg text-gray-800 focus:outline-none resize-none transition-all"
                                    rows="2"
                                    placeholder="Add translation..."
                                    @blur="autoSaveAnnotation"
                                ></textarea>
                                <div class="absolute right-2 bottom-2 text-xs text-gray-300 pointer-events-none opacity-0 group-focus-within:opacity-100 transition-opacity">
                                    Auto-saves on blur
                                </div>
                            </div>
                        </div>

                        <!-- Note -->
                        <div class="space-y-1">
                            <label class="text-xs text-gray-400 font-medium ml-1">Notes</label>
                            <textarea 
                                v-model="localNote"
                                class="w-full text-sm p-3 bg-gray-50 border border-transparent hover:bg-white hover:border-gray-200 focus:bg-white focus:border-indigo-300 rounded-lg text-gray-600 focus:outline-none resize-none transition-all"
                                rows="3"
                                placeholder="Add grammar notes or comments..."
                                @blur="autoSaveAnnotation"
                            ></textarea>
                        </div>
                    </div>

                </div>

            </div>
        </div>

    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, computed } from 'vue';
import { useVocabularyStore } from '@/stores/vocabulary';
import { MessagePlugin } from 'tdesign-vue-next';

interface SentenceItem {
    text: string;
    uuid: string; // Backend UUID or Hash
    articleId: string;
    sid?: string; // DOM ID (Hash)
    translation?: string;
    note?: string;
}

const props = defineProps<{
    word: string;
    sentences?: Array<SentenceItem>;
    focusSid?: string | null;
}>();

const emit = defineEmits(['view-details', 'update-word', 'save-annotation', 'update-text']);

const store = useVocabularyStore();
const existingVocab = ref<any>(null);
const loading = ref(false);
const saving = ref(false);

// Local Example Editing State (For Word-Sentence Link)
const exampleTranslation = ref('');
const exampleNote = ref('');

// Local Sentence Annotation State (For Sentence Itself)
const localTranslation = ref('');
const localNote = ref('');
const isEditMode = ref(false);
const editModeText = ref('');

// Carousel State
const activeIndex = ref(0);

// Use local word selection if user clicks inside the card
const localWord = ref(''); 

// Computed effective word (Prop takes precedence on change, but local overrides on click)
const displayWord = computed(() => localWord.value || props.word);

// Sync Focus from Parent (Hover)
watch(() => props.focusSid, (sid) => {
    if (sid && props.sentences) {
        const index = props.sentences.findIndex(s => s.sid === sid);
        if (index !== -1) {
            activeIndex.value = index;
        }
    }
});

// Reset index when sentences list changes substantially
watch(() => props.sentences, (newVal, oldVal) => {
    if (newVal?.length !== oldVal?.length) {
         if (!props.focusSid) activeIndex.value = 0;
    }
}, { deep: true });

const activeSentence = computed(() => {
    if (!props.sentences || props.sentences.length === 0) return null;
    return props.sentences[activeIndex.value];
});

// Watch activeSentence to hydrate annotation inputs
watch(activeSentence, (newVal) => {
    if (newVal) {
        localTranslation.value = newVal.translation || '';
        localNote.value = newVal.note || '';
        editModeText.value = newVal.text || '';
        isEditMode.value = false; // Reset edit mode on switch
    }
}, { immediate: true });

// Check if current sentence is already an example
const currentExample = computed(() => {
    if (!existingVocab.value || !activeSentence.value) return null;
    return existingVocab.value.examples?.find((ex: any) => ex.sentence === activeSentence.value?.text);
});

// Sync local inputs when currentExample changes
watch(currentExample, (ex) => {
    if (ex) {
        exampleTranslation.value = ex.translation || '';
        exampleNote.value = ex.note || '';
    } else {
        exampleTranslation.value = '';
        exampleNote.value = '';
    }
}, { immediate: true });

const exampleChanged = computed(() => {
    if (!currentExample.value) return false;
    return exampleTranslation.value !== (currentExample.value.translation || '') ||
           exampleNote.value !== (currentExample.value.note || '');
});

const activeTotal = computed(() => props.sentences?.length || 0);

// Simple Tokenizer for Interactive Sentence
const interactiveTokens = computed(() => {
    if (!activeSentence.value) return [];
    const raw = activeSentence.value.text.split(' ');
    return raw.map(t => {
        const clean = t.replace(/^[^\w]+|[^\w]+$/g, '');
        return {
            raw: t, 
            clean: clean
        };
    });
});

function handleWordClick(clickedWord: string) {
    if (!clickedWord) return;
    localWord.value = clickedWord;
    emit('update-word', clickedWord);
}

// Watch effective word changes to fetch data
watch(displayWord, (newWord) => {
    if (newWord) fetchData(newWord);
    else existingVocab.value = null;
}, { immediate: true });

// Also watch prop word changes to reset local override
watch(() => props.word, (val) => {
    if (val) localWord.value = ''; 
});

function nextSlide() {
    if (activeTotal.value > 1) {
        activeIndex.value = (activeIndex.value + 1) % activeTotal.value;
    }
}

function prevSlide() {
    if (activeTotal.value > 1) {
        activeIndex.value = (activeIndex.value - 1 + activeTotal.value) % activeTotal.value;
    }
}

const fetchData = async (w: string) => {
    loading.value = true;
    try {
        existingVocab.value = await store.searchWord(w);
    } catch (e) {
        existingVocab.value = null;
    } finally {
        loading.value = false;
    }
};

async function saveExample() {
    if (!existingVocab.value || !activeSentence.value) return;
    
    saving.value = true;
    try {
        await store.addExample(existingVocab.value.id, {
            sentence: activeSentence.value.text,
            article_id: activeSentence.value.articleId,
            sentence_uuid: activeSentence.value.uuid
        });
        await fetchData(displayWord.value); 
        MessagePlugin.success('Example added to vocabulary!');
    } catch (e) {
        MessagePlugin.error('Failed to add example.');
        console.error(e);
    } finally {
        saving.value = false;
    }
}

async function updateExampleDetails() {
    if (!existingVocab.value || !currentExample.value) return;

    saving.value = true;
    try {
        const updatedVocab = JSON.parse(JSON.stringify(existingVocab.value));
        const idx = updatedVocab.examples.findIndex((e: any) => e.id === currentExample.value.id);
        if (idx !== -1) {
            updatedVocab.examples[idx].translation = exampleTranslation.value;
            updatedVocab.examples[idx].note = exampleNote.value;
            await store.saveVocabulary(updatedVocab);
            await fetchData(displayWord.value);
            MessagePlugin.success('Example updated!');
        }
    } catch (e) {
        MessagePlugin.error('Failed to update example.');
        console.error(e);
    } finally {
        saving.value = false;
    }
}

function createNewVocab() {
    emit('view-details', { 
        word: displayWord.value, 
        initialSentence: activeSentence.value?.text,
        articleId: activeSentence.value?.articleId
    });
}

function refresh(newWord?: string) {
    if (newWord) {
        handleWordClick(newWord);
    } else {
        if (displayWord.value) fetchData(displayWord.value);
    }
}

// --- Sentence Annotation Logic ---

function autoSaveAnnotation() {
    if (!activeSentence.value) return;
    // Check if changed
    if (localTranslation.value === activeSentence.value.translation && 
        localNote.value === activeSentence.value.note) {
        return;
    }

    emit('save-annotation', {
        sid: activeSentence.value.sid,
        hash: activeSentence.value.sid, // SID is Hash now
        translation: localTranslation.value,
        note: localNote.value
    });
}

function toggleEditMode() {
    isEditMode.value = !isEditMode.value;
    if (isEditMode.value && activeSentence.value) {
        editModeText.value = activeSentence.value.text;
    }
}

function saveSmartEdit() {
    if (!activeSentence.value) return;
    if (!editModeText.value.trim()) {
        MessagePlugin.warning("Text cannot be empty");
        return;
    }
    
    emit('update-text', {
        oldText: activeSentence.value.text,
        newText: editModeText.value,
        oldHash: activeSentence.value.sid // SID is old Hash
    });
    isEditMode.value = false;
}

defineExpose({ refresh });
</script>

<style scoped>
.analysis-card {
    height: 100%;
}

.serif-font {
    font-family: "Playfair Display", "Times New Roman", serif;
}

.custom-scrollbar::-webkit-scrollbar { width: 4px; }
.custom-scrollbar::-webkit-scrollbar-thumb { background: #e5e5e5; border-radius: 2px; }
.custom-scrollbar::-webkit-scrollbar-track { background: transparent; }

@keyframes fadeInUp {
    from { opacity: 0; transform: translateY(10px); }
    to { opacity: 1; transform: translateY(0); }
}
.animate-fade-in-up {
    animation: fadeInUp 0.3s ease-out forwards;
}
</style>


