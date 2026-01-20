<template>
    <Teleport to="body">
        <Transition name="fade">
            <div v-if="visible" class="fixed inset-0 z-[100] flex items-center justify-center p-4 sm:p-8 font-sans text-ink">
                <!-- BACKDROP blur -->
                <div class="absolute inset-0 bg-black/20 backdrop-blur-sm transition-opacity" @click="close"></div>
                
                <!-- MAIN CARD: The "Paper" -->
                <div class="relative w-full max-w-3xl max-h-[90vh] flex flex-col bg-[#fdfdfd] shadow-2xl rounded-sm overflow-hidden animate-slide-up border border-white/50 ring-1 ring-black/5">
                    
                    <!-- Paper Texture Overlay (Subtle noise) -->
                    <div class="absolute inset-0 pointer-events-none opacity-[0.03]" style="background-image: url('data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSI0IiBoZWlnaHQ9IjQiPgo8cmVjdCB3aWR0aD0iNCIgaGVpZ2h0PSI0IiBmaWxsPSIjMDAwIj48L3JlY3Q+CjxyZWN0IHdpZHRoPSIxIiBoZWlnaHQ9IjEiIGZpbGw9IiNmZmYiPjwvcmVjdD4KPC9zdmc+');"></div>

                    <!-- HEADER -->
                    <div class="flex-none px-8 py-5 flex justify-between items-center border-b border-black/5 bg-white/50 backdrop-blur-sm z-10">
                        <div class="flex items-center gap-3">
                            <span class="text-xs font-bold uppercase tracking-[0.2em] text-ink/40">
                                {{ existing ? 'Editing Entry' : 'New Entry' }}
                            </span>
                        </div>
                        <div class="flex items-center gap-4">
                            <button @click="close" class="text-ink/40 hover:text-ink transition-colors text-sm font-medium tracking-wide">
                                CANCEL
                            </button>
                            <button 
                                @click="save" 
                                :disabled="saving"
                                class="bg-ink text-white px-6 py-2 text-xs font-bold uppercase tracking-widest hover:bg-black transition-all shadow-lg hover:shadow-xl active:translate-y-0.5 disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
                            >
                                <span v-if="saving" class="w-3 h-3 border-2 border-white/30 border-t-white rounded-full animate-spin"></span>
                                <span>{{ existing ? 'Save Changes' : 'Save Entry' }}</span>
                            </button>
                        </div>
                    </div>

                    <!-- SCROLLABLE CONTENT -->
                    <div class="flex-1 overflow-y-auto custom-scrollbar bg-[#fdfdfd] p-8 sm:p-12 relative z-0">
                        
                        <!-- HERO: Word & Phonetic -->
                        <div class="mb-12 text-center group">
                            <input 
                                v-model="wordInput"
                                class="w-full bg-transparent text-center text-6xl md:text-7xl font-serif font-bold text-ink placeholder:text-ink/10 outline-none border-0 p-0 transition-all selection:bg-ink/10"
                                placeholder="Word"
                            />
                            
                            <!-- Phonetic & Meta -->
                            <div class="mt-4 flex flex-col items-center gap-2">
                                <input 
                                    v-model="formData.phonetic"
                                    class="text-center font-serif italic text-xl text-ink/50 placeholder:text-ink/20 bg-transparent outline-none min-w-[100px]"
                                    placeholder="/phonetic/"
                                />
                                
                                <!-- Roots Pill List -->
                                <div class="flex flex-wrap items-center justify-center gap-2 mt-2">
                                     <div v-for="(m, i) in formData.morphology" :key="i" class="group/root relative inline-flex items-center">
                                        <div class="flex items-center gap-1 px-3 py-1 rounded-full bg-ink/5 border border-ink/5 text-xs text-ink/70">
                                            <span class="font-bold opacity-50">{{ m.type }}</span>
                                            <input 
                                                v-model="m.part" 
                                                class="bg-transparent w-16 outline-none font-serif italic text-ink border-0 p-0 text-center"
                                                placeholder="..."
                                            />
                                        </div>
                                        <button @click="removeMorphology(i)" class="absolute -top-1 -right-1 text-ink/30 hover:text-red-500 bg-white rounded-full w-4 h-4 flex items-center justify-center shadow opacity-0 group-hover/root:opacity-100 transition-opacity">
                                            <i class="ri-close-line text-[10px]"></i>
                                        </button>
                                     </div>
                                     <button 
                                        @click="addMorphology" 
                                        class="w-6 h-6 rounded-full border border-dashed border-ink/20 flex items-center justify-center text-ink/30 hover:text-ink hover:border-ink/50 transition-all"
                                        title="Add Root"
                                    >
                                        <i class="ri-add-line text-sm"></i>
                                     </button>
                                </div>
                            </div>
                        </div>

                        <hr class="border-ink/5 mb-10 mx-auto w-16" />

                        <!-- SECTION: Meanings -->
                        <div class="mb-12 max-w-2xl mx-auto">
                            <h3 class="flex items-center justify-between text-xs font-bold uppercase tracking-[0.15em] text-ink/30 mb-6">
                                <span>Definitions</span>
                                <button @click="addMeaning" class="hover:text-ink transition-colors"><i class="ri-add-line"></i> Add</button>
                            </h3>

                            <div class="space-y-8">
                                <div v-for="(item, idx) in formData.meanings" :key="idx" class="group relative pl-4 border-l-2 border-transparent hover:border-ink/10 transition-colors">
                                    <div class="flex gap-4 items-baseline">
                                        <!-- PoS Selector -->
                                        <div class="relative shrink-0">
                                            <button 
                                                @click="togglePosDropdown(idx)"
                                                class="font-bold font-serif italic text-lg text-ink/50 hover:text-ink transition-colors min-w-[3rem] text-right"
                                            >
                                                {{ item.pos }}
                                            </button>
                                            
                                            <!-- Dropdown -->
                                            <div v-if="activePosDropdown === idx" class="absolute left-0 top-full mt-2 w-32 bg-white shadow-xl rounded py-2 z-50 border border-ink/5 animate-fade-in">
                                                <div v-for="opt in ['n.', 'v.', 'adj.', 'adv.', 'prep.', 'conj.', 'phrase']" :key="opt"
                                                    class="px-4 py-2 hover:bg-ink/5 cursor-pointer font-serif italic text-sm text-ink/70"
                                                    @click="selectPos(idx, opt)">
                                                    {{ opt }}
                                                </div>
                                            </div>
                                            <div v-if="activePosDropdown === idx" class="fixed inset-0 z-40" @click="activePosDropdown = -1"></div>
                                        </div>

                                        <!-- Definition Input -->
                                        <div class="flex-1 space-y-2">
                                            <textarea 
                                                v-model="item.definition"
                                                rows="1"
                                                class="w-full bg-transparent text-lg text-ink font-serif leading-relaxed outline-none border-0 p-0 resize-none placeholder:text-ink/20"
                                                placeholder="Definition..."
                                            ></textarea>
                                            <input 
                                                v-model="item.translation"
                                                class="w-full bg-transparent text-sm text-ink/50 font-sans outline-none border-0 p-0 placeholder:text-ink/10"
                                                placeholder="Translation (optional)"
                                            />
                                        </div>

                                        <!-- Delete -->
                                        <button @click="removeMeaning(idx)" class="opacity-0 group-hover:opacity-100 text-ink/20 hover:text-red-500 transition-all">
                                            <i class="ri-delete-bin-line"></i>
                                        </button>
                                    </div>
                                </div>
                            </div>
                        </div>

                        <!-- SECTION: Journal Examples -->
                        <div class="max-w-3xl mx-auto">
                            <h3 class="flex items-center justify-between text-xs font-bold uppercase tracking-[0.15em] text-ink/30 mb-6">
                                <span>Journal</span>
                                <div class="flex gap-4">
                                    <button v-if="initialSentence && !hasInitialSentence" @click="addInitialExample" class="hover:text-ink transition-colors text-[10px] font-bold uppercase tracking-widest"><i class="ri-download-line"></i> Import Context</button>
                                    <button @click="addNewExample" class="hover:text-ink transition-colors text-[10px] font-bold uppercase tracking-widest"><i class="ri-add-line"></i> Add Entry</button>
                                </div>
                            </h3>

                            <div class="grid grid-cols-1 gap-6">
                                <div v-for="(ex, idx) in formData.examples" :key="idx" 
                                    class="relative aspect-[21/9] rounded-2xl overflow-hidden group shadow-lg ring-1 ring-black/5 hover:ring-black/20 transition-all bg-white"
                                >
                                    <!-- Background Image -->
                                    <div class="absolute inset-0 bg-ink/90 transition-all duration-700">
                                         <img v-if="ex.image_url" :src="ex.image_url" class="absolute inset-0 w-full h-full object-cover opacity-60 group-hover:scale-105 transition-transform duration-1000">
                                         <div v-else class="absolute inset-0 bg-gradient-to-br from-gray-800 to-black opacity-100"></div>
                                         
                                         <!-- Gradient Overlay -->
                                         <div class="absolute inset-0 bg-gradient-to-t from-black/90 via-black/40 to-transparent"></div>
                                    </div>
                                    
                                    <!-- Image Upload Trigger (Hidden until hover) -->
                                    <label class="absolute top-4 right-4 z-20 text-white/20 hover:text-white cursor-pointer opacity-0 group-hover:opacity-100 transition-opacity p-2 bg-black/20 backdrop-blur rounded-lg" title="Change Background Image">
                                        <i class="ri-image-add-line"></i>
                                        <input type="file" accept="image/*" class="hidden" @change="(e) => handleImageUpload(e, idx)">
                                    </label>

                                    <!-- Remove -->
                                     <button @click="removeExample(idx)" class="absolute top-4 left-4 z-20 text-white/20 hover:text-red-400 cursor-pointer opacity-0 group-hover:opacity-100 transition-opacity p-2 bg-black/20 backdrop-blur rounded-lg" title="Remove Example">
                                        <i class="ri-delete-bin-line"></i>
                                    </button>

                                    <!-- Link to Article -->
                                    <button 
                                        v-if="ex.article_id"
                                        @click="navigateToArticle(ex.article_id)"
                                        class="absolute bottom-4 right-4 z-20 text-white/40 hover:text-white cursor-pointer opacity-0 group-hover:opacity-100 transition-opacity p-2 px-3 bg-black/40 backdrop-blur rounded-full text-xs font-bold uppercase tracking-widest flex items-center gap-2" 
                                        title="View Source Article"
                                    >
                                        <i class="ri-article-line"></i> {{ ex.article_title || 'Source' }}
                                    </button>

                                    <!-- Content Overlay -->
                                    <div class="absolute inset-0 z-10 flex flex-col justify-end p-8 text-center items-center">
                                        <textarea 
                                             v-model="ex.sentence"
                                             class="w-full bg-transparent border-none text-2xl font-serif font-medium text-white placeholder:text-white/20 outline-none resize-none p-0 text-center drop-shadow-md leading-relaxed mb-4 focus:scale-105 transition-transform duration-300"
                                             placeholder="Type example sentence..."
                                             rows="2"
                                         ></textarea>
                                         
                                         <div class="h-auto w-full flex flex-col items-center gap-2 opacity-80 group-hover:opacity-100 transition-opacity duration-300">
                                             <input 
                                                v-model="ex.translation"
                                                class="w-full bg-transparent text-center text-sm text-white/60 placeholder:text-white/20 outline-none font-light tracking-wide focus:text-white transition-colors"
                                                placeholder="Translation..."
                                             />
                                             <input 
                                                v-model="ex.note"
                                                class="bg-black/40 px-3 py-1 rounded-full backdrop-blur-sm border border-white/10 text-center text-[10px] text-white/80 font-bold uppercase tracking-widest placeholder:text-white/10 outline-none w-auto min-w-[80px] hover:bg-black/60 transition-colors focus:border-white/30"
                                                placeholder="NOTE"
                                             />
                                         </div>
                                    </div>
                                </div>
                            </div>
                        </div>

                    </div>
                </div>
            </div>
        </Transition>
    </Teleport>
</template>

<script setup lang="ts">
import { ref, reactive, watch, computed } from 'vue';
import { useVocabularyStore } from '@/stores/vocabulary';
import { dictionaryApi } from '@/api/dictionary';
import { MessagePlugin } from 'tdesign-vue-next';
import { useRouter } from 'vue-router';
import axios from 'axios';

const props = defineProps<{
    visible: boolean;
    initialWord: string;
    initialSentence?: string;
    initialData?: any;
    initialContext?: { articleId: string; articleTitle?: string };
}>();

const emit = defineEmits(['update:visible', 'refresh']);
const store = useVocabularyStore();
const currentId = ref('');

// UI State
const loading = ref(false);
const saving = ref(false);
const existing = ref(false);
const wordInput = ref('');

// Dropdown State
const activePosDropdown = ref<number>(-1);

function togglePosDropdown(idx: number) {
    activePosDropdown.value = activePosDropdown.value === idx ? -1 : idx;
}

function selectPos(idx: number, opt: string) {
    formData.meanings[idx].pos = opt;
    activePosDropdown.value = -1;
}

// Interfaces
interface MorphologyItem { type: string; part: string; meaning: string; }
interface MeaningItem { pos: string; translation: string; definition: string; }
interface ExampleItem { sentence: string; translation: string; note?: string; image_url?: string; article_id?: string; article_title?: string; }

const formData = reactive({
    phonetic: '',
    meanings: [] as MeaningItem[],
    morphology: [] as MorphologyItem[],
    examples: [] as ExampleItem[]
});

watch(() => props.visible, async (val) => {
    if (val) {
        wordInput.value = props.initialWord; // Init input
        await init();
    }
});

async function init() {
    loading.value = true;
    
    // Reset
    formData.phonetic = '';
    formData.meanings = [{ pos: 'n.', translation: '', definition: '' }];
    formData.morphology = [];
    formData.examples = [];

    try {
        let dataToLoad = props.initialData;
        
        // If not provided, try to find in store first (to enable Edit Mode)
        if (!dataToLoad) {
             try {
                const found = await store.searchWord(props.initialWord);
                if (found) dataToLoad = found;
             } catch (err) { /* ignore */ }
        }

        existing.value = !!dataToLoad;

        if (dataToLoad) {
            // Edit Mode
            wordInput.value = dataToLoad.word;
            formData.phonetic = dataToLoad.phonetic || '';
            parseMeanings(dataToLoad.definition, dataToLoad.translation);
            
            if (dataToLoad.root) parseRootString(dataToLoad.root);
            formData.examples = dataToLoad.examples ? JSON.parse(JSON.stringify(dataToLoad.examples)) : [];
            
            // If we have an ID, we need to make sure we store it for saving
            // Since props.initialData might be null now, we need a way to store the ID.
            // But we use props.initialData.id in save(). 
            // We should store the ID in a local ref or rely on dataToLoad being available (but it's local var).
            // Let's store it in a ref.
            currentId.value = dataToLoad.id;

        } else {
            // Create Mode - Dictionary Lookup
            currentId.value = '';
            const dictData = await dictionaryApi.lookup(props.initialWord);
            if (dictData) {
                formData.phonetic = dictData.phonetic || '';
                formData.meanings = [{ pos: 'n.', translation: '', definition: '' }];
            }
        }
            
        // Auto-add context sentence if new (only in Create Mode or if explicitly requested?)
        // User wants context. If we are in Create Mode, definitely add.
        // If Edit Mode? Maybe not auto-add to avoid spam, but "Import Context" button is there.
        // But if the user clicked "Edit" from a context, they might expect it.
        // Let's only auto-add for NEW entries to keep it clean, but ensure button is visible.
        if (!existing.value && props.initialSentence && formData.examples.length === 0) {
                formData.examples.push({ 
                sentence: props.initialSentence, 
                translation: '',
                article_id: props.initialContext?.articleId,
                article_title: props.initialContext?.articleTitle
                });
        }
    } catch (e) {
        console.error("Init failed", e);
    } finally {
        loading.value = false;
    }
}

// Reuse logic from previous
function parseMeanings(defStr: string, transStr: string) {
    if (!defStr) { formData.meanings = [{ pos: 'n.', translation: '', definition: '' }]; return; }
    const meanings: MeaningItem[] = [];
    const lines = defStr.split('\n').map(s => s.trim()).filter(Boolean);
    
    if (lines.some(l => l.match(/^\[(.*?)\]/))) {
        lines.forEach(line => {
             const match = line.match(/^\[(.*?)\]\s*(.*)$/);
             if (match) {
                 const pos = match[1];
                 const def = match[2];
                 let translation = '';
                 if (transStr) {
                     const tMatch = transStr.match(new RegExp(`\\[${pos}\\]\\s*([^;]+)`));
                     if (tMatch) translation = tMatch[1].trim();
                     else if (!translation && transStr.indexOf('[') === -1) translation = transStr; 
                 }
                 meanings.push({ pos, definition: def, translation });
             } else {
                 meanings.push({ pos: 'other', definition: line, translation: '' });
             }
        });
    } else {
        meanings.push({ pos: 'other', definition: defStr, translation: transStr || '' });
    }
    if (meanings.length > 0) formData.meanings = meanings;
}

// Actions
function addMeaning() { formData.meanings.push({ pos: 'n.', translation: '', definition: '' }); }
function removeMeaning(idx: number) { formData.meanings.splice(idx, 1); }

const hasInitialSentence = computed(() => {
    if (!props.initialSentence) return false;
    return formData.examples.some(e => e.sentence.trim() === props.initialSentence?.trim());
});

function addInitialExample() {
    if (props.initialSentence) {
        formData.examples.push({ 
            sentence: props.initialSentence, 
            translation: '',
            article_id: props.initialContext?.articleId,
            article_title: props.initialContext?.articleTitle
        });
    }
}
function addNewExample() { formData.examples.push({ sentence: '', translation: '' }); }
function removeExample(idx: number) { formData.examples.splice(idx, 1); }

// Morphology
function addMorphology() { formData.morphology.push({ type: 'Root', part: '', meaning: '' }); }
function removeMorphology(idx: number) { formData.morphology.splice(idx, 1); }
function buildRootString(): string | undefined {
    if (formData.morphology.length === 0) return undefined;
    return formData.morphology.map(m => `[${m.type}] ${m.part} : ${m.meaning}`).join(' | ');
}
function parseRootString(str: string) {
    if (!str) return;
    try {
        const parts = str.split('|').map(s => s.trim());
        formData.morphology = parts.map(p => {
            const match = p.match(/^\[(.*?)\]\s*(.*?)\s*:\s*(.*)$/);
            if (match) return { type: match[1], part: match[2], meaning: match[3] };
            return { type: 'Root', part: p, meaning: '' }; 
        });
    } catch (e) {
        console.error("Failed to parse root string", e);
    }
}

// Image Upload
async function handleImageUpload(e: Event, idx: number) {
    const target = e.target as HTMLInputElement;
    if (!target.files?.[0]) return;
    
    try {
        const file = target.files[0];
        const token = localStorage.getItem('aether_token');
        const form = new FormData();
        form.append('file', file);
        
        const res = await axios.post('/api/upload', form, {
            headers: { 'Content-Type': 'multipart/form-data', Authorization: `Bearer ${token}` }
        });
        formData.examples[idx].image_url = res.data.url;
        MessagePlugin.success('Image added to scene');
    } catch (err) {
        MessagePlugin.error('Upload failed');
    }
}

const router = useRouter();
function navigateToArticle(articleId: string) {
    // Save current work? Maybe prompt? 
    // for now just go
    router.push(`/article/${articleId}`);
}

function close() { emit('update:visible', false); }

async function save() {
    saving.value = true;
    try {
        const validMeanings = formData.meanings.filter(m => m.definition.trim() || m.translation.trim());
        let fullDefinition = validMeanings.map(m => `[${m.pos}] ${m.definition}`).join('\n');
        let fullTranslation = validMeanings.map(m => `[${m.pos}] ${m.translation}`).join('; ');

        const payload = {
            word: wordInput.value,
            definition: fullDefinition,
            translation: fullTranslation,
            phonetic: formData.phonetic,
            examples: formData.examples.filter(e => e.sentence.trim().length > 0),
            root: buildRootString() 
        };

        if (existing.value && currentId.value) {
             await store.saveVocabulary({ ...payload, id: currentId.value } as any);
             MessagePlugin.success('Entry updated');
        } else {
             await store.saveVocabulary(payload as any);
             MessagePlugin.success('Entry created');
        }
        
        emit('refresh', wordInput.value);
        close();
    } catch (e) {
        console.error(e);
        MessagePlugin.error('Failed to save');
    } finally {
        saving.value = false;
    }
}
</script>

<style scoped>
.animate-slide-up { animation: slideUp 0.6s cubic-bezier(0.19, 1, 0.22, 1) forwards; opacity: 0; transform: translateY(40px); }
@keyframes slideUp { 
    to { transform: translateY(0); opacity: 1; } 
}

.custom-scrollbar::-webkit-scrollbar { width: 4px; }
.custom-scrollbar::-webkit-scrollbar-thumb { background: #e5e5e5; border-radius: 2px; }
.custom-scrollbar::-webkit-scrollbar-track { background: transparent; }

/* Font Setup overrides if global isn't set */
input, textarea {
    font-family: inherit;
}
</style>
