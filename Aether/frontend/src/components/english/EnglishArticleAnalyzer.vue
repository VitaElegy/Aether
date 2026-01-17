<template>
  <div class="english-article-analyzer">
    <div class="article-container" ref="containerRef" @click="handleTextClick">
      
      <header class="article-header">
        <h1 class="serif-title">{{ article?.title }}</h1>
        <div class="meta">
            <span v-if="article?.created_at">{{ new Date(article.created_at).toLocaleDateString() }}</span>
        </div>
      </header>

      <div class="markdown-body serif-body">
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
import { ref, computed } from 'vue';
import MarkdownRenderer from '@/components/renderers/MarkdownRenderer.vue'; // Adjust path
import AcademicDrawer from './AcademicDrawer.vue';

const props = defineProps<{
    article: any; 
}>();

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
.english-article-analyzer {
    display: flex; /* For drawer integration? Actually drawer is fixed */
    background-color: #fdfbf7; /* Off white academic */
    min-height: 100vh;
    color: #2c2c2c;
}

.article-container {
    max-width: 800px;
    margin: 0 auto;
    padding: 4rem 2rem;
    cursor: text;
}

.serif-title {
    font-family: "Playfair Display", serif;
    font-size: 3rem;
    margin-bottom: 0.5rem;
}

.meta {
    font-family: "Inter", sans-serif;
    color: #888;
    font-size: 0.9rem;
    margin-bottom: 3rem;
}

.serif-body {
    font-family: "Crimson Text", "Times New Roman", serif;
    font-size: 1.25rem;
    line-height: 1.8;
}

/* Override Markdown Styles for Academic Feel */
:deep(p) {
    margin-bottom: 1.5em;
}
:deep(h2), :deep(h3) {
    font-family: "Playfair Display", serif;
    margin-top: 2em;
}
</style>
