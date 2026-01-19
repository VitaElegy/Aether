<template>
  <div class="english-article-analyzer">
    <div class="article-container" ref="containerRef" @click="handleTextClick">
      
      <header class="article-header">
        <h1 class="title">{{ article?.title }}</h1>
        <div class="meta">
            <span v-if="article?.author_name" class="author">By {{ article.author_name }}</span>
            <span class="date" v-if="article?.created_at">{{ new Date(article.created_at).toLocaleDateString(undefined, { year: 'numeric', month: 'long', day: 'numeric' }) }}</span>
        </div>
      </header>

      <div class="divider"></div>

      <div class="markdown-body standard-body">
        <MarkdownRenderer :content="article?.body.data || ''" />
      </div>

    </div>

  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import MarkdownRenderer from '@/components/renderers/MarkdownRenderer.vue'; 
import { useNavigationStore } from '@/stores/navigation';

const props = defineProps<{
    article: any; 
}>();

const emit = defineEmits(['selection']);

const navStore = useNavigationStore();
const containerRef = ref<HTMLElement | null>(null);

// ... existing sentenceMap computation ...

// Sentence Map from Derived Data
const sentenceMap = computed(() => {
    try {
        if (!props.article?.derived_data?.sentence_map) return {};
        // derived_data is JSON, sentence_map is inside
        return props.article.derived_data.sentence_map || {}; 
    } catch (e) {
        return {};
    }
});

function handleTextClick(event: MouseEvent) {
    // 1. Get the word clicked
    const selection = window.getSelection();
    let word = '';
    let target = event.target as HTMLElement;

    if (!selection || selection.isCollapsed) {
        // Only trigger if essentially a "click" (collapsed selection) or small selection
        if (selection) {
            selection.modify('move', 'backward', 'word');
            selection.modify('extend', 'forward', 'word');
            word = selection.toString().trim();
        }
    } else {
        word = selection.toString().trim();
    }

    if (word && /^[a-zA-Z]+$/.test(word)) { // Simple alpha check
        const sentence = identifySentence(target, word);
        emit('selection', { word, sentence });
        return;
    }
}

function identifySentence(target: HTMLElement, word: string) {
    // Simplest: Just use the paragraph text
    const paragraph = target.closest('p, li, blockquote');
    if (paragraph) {
        let text = paragraph.textContent || '';
        const sentences = text.match(/[^.!?]+[.!?]+(\s|$)/g) || [text];
        let matchedSentence = sentences.find(s => s.includes(word))?.trim() || text;

        // Find UUID in map
        let foundUuid = null;
        let map = sentenceMap.value;
        if (map) { 
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

        return {
            text: matchedSentence,
            uuid: foundUuid, 
            articleId: props.article.node.id
        };
    }
    return null;
}
</script>

<style scoped>
/* Standard Aesthetic - No Custom Fonts */

.english-article-analyzer {
    background-color: var(--bg-body, #ffffff); 
    min-height: 100%;
    color: var(--text-primary, #1d1d1d);
    position: relative;
    padding-bottom: 5rem;
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

.title {
    font-size: 2.5rem; /* Standard H1 size */
    line-height: 1.2;
    margin-bottom: 1rem;
    font-weight: 700;
    letter-spacing: -0.01em;
    color: var(--text-primary, #1d1d1d);
}

.meta {
    font-size: 0.875rem;
    color: var(--text-secondary, #808080);
    display: flex;
    justify-content: center;
    gap: 1.5rem;
}

.meta .author {
    font-weight: 600;
}

.divider {
    width: 60px;
    height: 2px;
    background: var(--border-level-1, #e7e7e7);
    margin: 0 auto 3rem auto;
    border-radius: 2px;
}

/* Typography Modes */
.standard-body {
    font-size: 1.1rem;
    line-height: 1.7;
    color: var(--text-primary, #1d1d1d);
}

/* Override Markdown Styles for Standard Feel */
:deep(p) {
    margin-bottom: 1.25em;
    text-align: left;
}

:deep(h2) {
    font-size: 1.75rem;
    margin-top: 2em;
    margin-bottom: 0.75em;
    font-weight: 700;
    color: var(--text-primary, #1d1d1d);
}

:deep(h3) {
    font-size: 1.4rem;
    margin-top: 1.5em;
    margin-bottom: 0.5em;
    font-weight: 600;
    color: var(--text-primary, #1d1d1d);
}

:deep(blockquote) {
    border-left: 3px solid var(--brand-color, #0052d9); /* Standard Brand Color */
    margin: 1.5em 0;
    padding-left: 1rem;
    font-style: italic;
    color: var(--text-secondary, #666);
    background: transparent;
}
</style>
