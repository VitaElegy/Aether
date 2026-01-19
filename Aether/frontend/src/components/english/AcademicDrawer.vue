<template>
  <div class="academic-drawer-container">
    <transition name="slide-fade">
      <div v-if="visible" class="academic-drawer">
        <!-- Close Button -->
        <button class="close-btn" @click="$emit('close')">
            <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18"></line><line x1="6" y1="6" x2="18" y2="18"></line></svg>
        </button>

        <div class="content">
            <!-- Header: Word -->
            <div class="word-header">
                <h2 class="word">{{ word }}</h2>
                <span v-if="existingVocab?.phonetic" class="phonetic">{{ existingVocab.phonetic }}</span>
            </div>
            
            <div class="divider"></div>

            <!-- Definition -->
            <div class="section definition-section">
                <div v-if="loading" class="loading-state">Looking up...</div>
                <div v-else-if="existingVocab">
                    <p class="definition">{{ existingVocab.definition }}</p>
                    <p class="translation" v-if="existingVocab.translation">{{ existingVocab.translation }}</p>
                    
                    <div class="status-badge" :class="existingVocab.status.toLowerCase()">
                        {{ existingVocab.status }}
                    </div>
                </div>
                <div v-else class="new-word-state">
                    <p class="text-sm text-gray-500">New word found.</p>
                </div>
            </div>

            <div class="divider"></div>

            <!-- Context Sentence -->
            <div class="section context-section" v-if="sentence">
                <h3 class="section-title">Context</h3>
                <blockquote class="context-quote">
                    "{{ sentence.text }}"
                </blockquote>
                
                <div class="actions">
                     <button class="action-btn primary" @click="saveExample" :disabled="saving || !existingVocab">
                        {{ saving ? 'Saving...' : 'Save as Example' }}
                     </button>
                     <button class="action-btn secondary" v-if="!existingVocab" @click="createNewVocab">
                        Create Card
                     </button>
                </div>
            </div>

        </div>
      </div>
    </transition>
    <div v-if="visible" class="backdrop" @click="$emit('close')"></div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import { useVocabularyStore } from '@/stores/vocabulary';

const props = defineProps<{
    visible: boolean;
    word: string;
    sentence?: { text: string; uuid: string; articleId: string };
}>();

const emit = defineEmits(['close']);

const store = useVocabularyStore();
const existingVocab = ref<any>(null);
const loading = ref(false);
const saving = ref(false);

watch(() => props.word, async (newWord) => {
    if (newWord && props.visible) {
        loading.value = true;
        existingVocab.value = await store.searchWord(newWord);
        loading.value = false;
    }
});

watch(() => props.visible, async (isVisible) => {
    if (isVisible && props.word) {
        loading.value = true;
        existingVocab.value = await store.searchWord(props.word);
        loading.value = false;
    }
});

async function saveExample() {
    if (!existingVocab.value || !props.sentence) return;
    
    saving.value = true;
    try {
        await store.addExample(existingVocab.value.id, {
            sentence: props.sentence.text,
            article_id: props.sentence.articleId,
            sentence_uuid: props.sentence.uuid
        });
        alert('Example added!');
    } catch (e) {
        alert('Failed to add example');
    } finally {
        saving.value = false;
    }
}

function createNewVocab() {
    alert('Create Card feature coming soon (use main Vocabulary view)');
}

</script>

<style scoped>
.academic-drawer-container {
    position: fixed;
    top: 0;
    right: 0;
    height: 100vh;
    z-index: 2000;
    pointer-events: none;
}

.backdrop {
    position: absolute;
    inset: 0;
    width: 100vw;
    background: rgba(0,0,0,0.2);
    pointer-events: auto;
}

.academic-drawer {
    pointer-events: auto;
    width: 400px;
    height: 100%;
    background: rgba(255, 255, 255, 0.95);
    backdrop-filter: blur(16px);
    box-shadow: -5px 0 30px rgba(0,0,0,0.1);
    position: absolute;
    right: 0;
    display: flex;
    flex-direction: column;
    padding: 2rem;
    border-left: 1px solid rgba(0,0,0,0.05);
}

/* Animations */
.slide-fade-enter-active,
.slide-fade-leave-active {
  transition: all 0.3s cubic-bezier(0.16, 1, 0.3, 1);
}

.slide-fade-enter-from,
.slide-fade-leave-to {
  transform: translateX(100%);
  opacity: 0;
}

/* Typography & Layout */
.word-header .word {
    font-size: 2.5rem;
    font-weight: 700;
    color: var(--text-primary, #1d1d1d);
    margin: 0;
    letter-spacing: -0.01em;
}

.phonetic {
    font-family: 'Inter', sans-serif;
    color: var(--text-secondary, #888);
    margin-top: 0.5rem;
    display: block;
    font-size: 0.9rem;
}

.divider {
    height: 1px;
    background: var(--border-level-1, #f0f0f0);
    margin: 1.5rem 0;
}

.content {
    flex: 1;
    overflow-y: auto;
}

.definition {
    font-size: 1.1rem;
    line-height: 1.6;
    color: var(--text-primary, #333);
}

.translation {
    font-size: 1rem;
    color: var(--text-secondary, #666);
    margin-top: 0.5rem;
}

.section-title {
    font-size: 0.85rem;
    text-transform: uppercase;
    color: var(--text-secondary, #888);
    font-weight: 600;
    letter-spacing: 0.05em;
    margin-bottom: 0.5rem;
}

.context-quote {
    border-left: 3px solid var(--brand-color, #0052d9);
    padding-left: 1rem;
    margin: 1rem 0;
    font-style: italic;
    color: var(--text-secondary, #555);
}

/* Actions */
.action-btn {
    width: 100%;
    padding: 0.8rem;
    margin-top: 0.5rem;
    border: none;
    cursor: pointer;
    font-size: 0.95rem;
    font-weight: 500;
    transition: all 0.2s;
    border-radius: 6px;
}

.action-btn.primary {
    background: var(--brand-color, #2c2c2c);
    color: #fff;
}
.action-btn.primary:hover {
    opacity: 0.9;
    transform: translateY(-1px);
}
.action-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
    transform: none;
}

.action-btn.secondary {
    background: transparent;
    border: 1px solid var(--border-level-2, #ddd);
    color: var(--text-primary, #2c2c2c);
}
.action-btn.secondary:hover {
    background: rgba(0,0,0,0.02);
}

.close-btn {
    position: absolute;
    top: 1rem;
    right: 1rem;
    background: none;
    border: none;
    cursor: pointer;
    color: #999;
    transition: color 0.2s;
}
.close-btn:hover {
    color: #333;
}
</style>
