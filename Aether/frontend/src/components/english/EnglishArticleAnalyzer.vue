<template>
  <div class="english-article-analyzer" :class="{ 'sans-mode': vocabStore.fontMode === 'sans' }">
    <!-- Top Control Bar (Teleported to Global Nav) -->
    <Teleport to="#nav-right-portal">
        <div class="flex items-center bg-ink/5 rounded-lg p-1 gap-1">
            <button 
                @click="vocabStore.fontMode = 'serif'"
                class="w-8 h-8 rounded flex items-center justify-center transition-all"
                :class="vocabStore.fontMode === 'serif' ? 'bg-white text-ink shadow-sm' : 'text-ink/40 hover:text-ink'"
                title="Serif (Traditional)"
            >
                <i class="ri-serif text-sm"></i>
            </button>
            <button 
                @click="vocabStore.fontMode = 'sans'"
                class="w-8 h-8 rounded flex items-center justify-center transition-all"
                :class="vocabStore.fontMode === 'sans' ? 'bg-white text-ink shadow-sm' : 'text-ink/40 hover:text-ink'"
                title="Sans (Modern)"
            >
                <i class="ri-font-sans-serif text-sm"></i>
            </button>
        </div>
    </Teleport>
    
    <div class="article-container" ref="containerRef" @click="handleTextClick">
      
      <header class="article-header">
        <h1 class="serif-title">{{ article?.title }}</h1>
        <div class="meta">
            <span v-if="article?.author_name" class="author">By {{ article.author_name }}</span>
            <span class="date" v-if="article?.created_at">{{ new Date(article.created_at).toLocaleDateString(undefined, { year: 'numeric', month: 'long', day: 'numeric' }) }}</span>
        </div>
      </header>

      <div class="divider-visual"></div>

      <div class="markdown-body academic-body">
        <MarkdownRenderer :content="article?.body.data || ''" />
      </div>

    </div>

    <AcademicDrawer 
        :visible="drawerVisible"
        :word="selectedWord"
        :sentence="selectedSentence"
        @close="drawerVisible = false"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import MarkdownRenderer from '@/components/renderers/MarkdownRenderer.vue'; 
import AcademicDrawer from './AcademicDrawer.vue';
import { useVocabularyStore } from '@/stores/vocabulary';
import { useNavigationStore } from '@/stores/navigation';

const props = defineProps<{
    article: any; 
}>();

const vocabStore = useVocabularyStore(); // Global aesthetic state
const navStore = useNavigationStore();

onMounted(() => {
    navStore.setCustomRight(true);
});

onUnmounted(() => {
    navStore.setCustomRight(false);
});

const drawerVisible = ref(false);
const selectedWord = ref('');
const selectedSentence = ref<any>(null);
const containerRef = ref<HTMLElement | null>(null);

// Sentence Map from Derived Data
const sentenceMap = computed(() => {
    try {
        if (!props.article?.derived_data?.sentence_map) return {};
        // derived_data is JSON, sentence_map is inside
        return props.article.derived_data.sentence_map || {}; // Might need simpler index logic depending on structure
    } catch (e) {
        return {};
    }
});

function handleTextClick(event: MouseEvent) {
    // 1. Get the word clicked
    const selection = window.getSelection();
    if (!selection || selection.isCollapsed) {
        // Only trigger if essentially a "click" (collapsed selection) or small selection
        // Attempt to expand to word
        if (selection) {
            selection.modify('move', 'backward', 'word');
            selection.modify('extend', 'forward', 'word');
            const word = selection.toString().trim();
            
            if (word && /^[a-zA-Z]+$/.test(word)) { // Simple alpha check
                selectedWord.value = word;
                drawerVisible.value = true;
                identifySentence(event.target as HTMLElement);
                return;
            }
        }
    } else {
        // Allow user manual selection
        const word = selection.toString().trim();
         if (word) {
            selectedWord.value = word;
            drawerVisible.value = true;
            identifySentence(event.target as HTMLElement);
        }
    }
}

function identifySentence(target: HTMLElement) {
    // This is Tricky for Markdown rendered HTML
    // Ideally we match the text content to the sentence_map
    // For MVP, we pass the raw text of the closest block element or regex match
    
    // Simplest: Just use the paragraph text
    const paragraph = target.closest('p, li, blockquote');
    if (paragraph) {
        // Try to match specific sentence?
        // Let's iterate sentence map and find one that contains this text? 
        // Or just return the whole paragraph as "context" for now.
        // The backend parsing split by [.!?], so we can try to find the sentence within the paragraph text.
        
        let text = paragraph.textContent || '';
        const word = selectedWord.value;
        const index = text.indexOf(word);
        
        // Find sentence boundaries around the word
        // Simple client-side re-split for display context
        // NOTE: This might desync with backend UUID if not careful.
        // For robust UUID, we need to map the backend 'derived_data' sentences to the DOM.
        // But since we rendered Markdown blindly, we lost that mapping.
        
        // Strategy:
        // Search 'derived_data.sentence_map' values for a match of the "local sentence".
        // 1. Extract local sentence approximately
        // 2. Fuzzy match against backend map
        
        const sentences = text.match(/[^.!?]+[.!?]+(\s|$)/g) || [text];
        let matchedSentence = sentences.find(s => s.includes(word))?.trim() || text;

        // Find UUID in map
        let foundUuid = null;
        let map = sentenceMap.value;
        if (map) { // It's a Map actually, or object
             // map is actually object { map: { uuid: data } } or directly { uuid: ... } depending on Rust serialization
             // Check Rust struct: SentenceMap { map: HashMap<Uuid, SentenceData> }
             // So JSON is { "map": { "uuid": ... } }
             
             const innerMap = map.map || {};
             for (const [uuid, data] of Object.entries(innerMap) as any) {
                 if (data.text.includes(matchedSentence) || matchedSentence.includes(data.text)) {
                     // Approximate match
                     foundUuid = uuid;
                     matchedSentence = data.text; // Use backend text
                     break;
                 }
             }
        }

        selectedSentence.value = {
            text: matchedSentence,
            uuid: foundUuid, // might be null if not found
            articleId: props.article.node.id
        };
    }
}
</script>

<style scoped>
@import url('https://fonts.googleapis.com/css2?family=Crimson+Text:ital,wght@0,400;0,600;0,700;1,400&family=Playfair+Display:ital,wght@0,400;0,600;0,700;1,400&display=swap');

.english-article-analyzer {
    background-color: #F9F7F1; /* Warm Ivory "London Paper" */
    min-height: 100%;
    color: #2C3E50; /* Ink Blue/Grey */
    position: relative;
    padding-bottom: 5rem;
}

/* Control Bar */
.control-bar {
    position: sticky;
    top: 0;
    right: 0;
    left: 0;
    padding: 1rem 2rem;
    display: flex;
    justify-content: flex-end;
    z-index: 10;
    pointer-events: none; /* Let clicks pass through to content behind if needed */
}

.font-toggle {
    pointer-events: auto;
    background: rgba(255,255,255,0.8);
    backdrop-filter: blur(8px);
    border: 1px solid rgba(0,0,0,0.05);
    padding: 0.25rem;
    border-radius: 999px;
    display: flex;
    align-items: center;
    gap: 0.5rem;
    box-shadow: 0 4px 12px rgba(0,0,0,0.03);
}

.font-toggle .label {
    font-size: 0.7rem;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    font-weight: 700;
    color: #888;
    padding-left: 0.75rem;
    padding-right: 0.25rem;
}

.toggle-switch {
    display: flex;
    align-items: center;
    background: rgba(0,0,0,0.03);
    border-radius: 999px;
    padding: 2px;
}

.toggle-switch button {
    width: 2rem;
    height: 2rem;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    border: none;
    background: transparent;
    cursor: pointer;
    color: #999;
    transition: all 0.2s;
}

.toggle-switch button.active {
    background: #fff;
    color: #2C3E50;
    box-shadow: 0 2px 4px rgba(0,0,0,0.05);
}

.toggle-switch .divider {
    width: 1px;
    height: 12px;
    background: rgba(0,0,0,0.1);
    margin: 0 2px;
}

/* Article Container */
.article-container {
    max-width: 800px;
    margin: 0 auto;
    padding: 2rem 2rem 6rem 2rem;
    cursor: text;
}

.article-header {
    text-align: center;
    margin-bottom: 2rem;
}

.serif-title {
    font-family: "Playfair Display", serif;
    font-size: 3.5rem;
    line-height: 1.1;
    margin-bottom: 1.5rem;
    font-weight: 700;
    letter-spacing: -0.02em;
    color: #1a1a1a;
}

.meta {
    font-family: "Inter", sans-serif;
    color: #666; /* Muted Ink */
    font-size: 0.85rem;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    display: flex;
    justify-content: center;
    gap: 1.5rem;
}

.meta .author {
    font-weight: 600;
    color: #444;
}

.divider-visual {
    width: 40px;
    height: 4px;
    background: #2C3E50;
    margin: 0 auto 4rem auto;
    border-radius: 2px;
}

/* Typography Modes */
.academic-body {
    font-size: 1.25rem;
    line-height: 1.8;
    color: #2C3E50;
    transition: font-family 0.3s ease;
}

/* Serif Mode (Default) */
.academic-body {
    font-family: "Crimson Text", "Times New Roman", serif;
}

/* Sans Mode Override */
.sans-mode .academic-body {
    font-family: "Inter", -apple-system, BlinkMacSystemFont, sans-serif;
    font-size: 1.15rem; /* Slight optical adjustment */
    line-height: 1.7;
    letter-spacing: -0.01em;
}

/* Override Markdown Styles for Academic Feel */
:deep(p) {
    margin-bottom: 1.5em;
    text-align: justify; /* Justify for that newspaper feel? Maybe left is better for web */
    text-align: left; 
}

:deep(h2) {
    font-family: "Playfair Display", serif;
    font-size: 2rem;
    margin-top: 2.5em;
    margin-bottom: 1em;
    border-bottom: 1px solid rgba(44, 62, 80, 0.1);
    padding-bottom: 0.5rem;
    color: #1a1a1a;
}

:deep(h3) {
    font-family: "Playfair Display", serif;
    font-size: 1.5rem;
    margin-top: 2em;
    margin-bottom: 0.5em;
    color: #2c2c2c;
}

:deep(blockquote) {
    border-left: 3px solid #d4af37; /* Gold accent */
    margin: 2em 0;
    padding-left: 1.5em;
    font-style: italic;
    color: #555;
    background: transparent;
}
</style>
