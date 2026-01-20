<template>
  <div class="english-article-analyzer">
    <div class="article-container" ref="containerRef" @mouseup="handleSelectionEvent" @keyup="handleSelectionEvent">
      
      <header class="article-header">
        <h1 class="title">{{ article?.title }}</h1>
        <div class="meta">
            <span v-if="article?.author_name" class="author">By {{ article.author_name }}</span>
            <span class="date" v-if="article?.created_at">{{ new Date(article.created_at).toLocaleDateString(undefined, { year: 'numeric', month: 'long', day: 'numeric' }) }}</span>
        </div>
      </header>

      <div class="divider"></div>

      <div class="markdown-body standard-body">
        <MarkdownRenderer :content="article?.body.text || ''" />
      </div>

    </div>

  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch, nextTick } from 'vue';
import MarkdownRenderer from '@/components/renderers/MarkdownRenderer.vue'; 
import { useNavigationStore } from '@/stores/navigation';

const props = defineProps<{
    article: any; 
}>();

const emit = defineEmits(['selection', 'focus-change']);

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

// --- Sentence Hydration ---
import { generateContentHash } from '@/utils/text-anchoring';

function hydrateSentences() {
    if (!containerRef.value) return;
    const root = containerRef.value;
    
    // Reset counters for collision handling
    const hashCounts: Record<string, number> = {};

    const walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT, null);
    const textNodes: Text[] = [];
    while (walker.nextNode()) {
        textNodes.push(walker.currentNode as Text);
    }
    
    textNodes.forEach(node => {
        // Skip if already processed or child of article-header
        if (node.parentElement?.closest('.article-header')) return;
        if (node.parentElement?.classList.contains('sentence-entity')) return;

        const text = node.nodeValue || '';
        if (!text.trim()) return; // Skip whitespace nodes

        // Regex to split: look for punctuation followed by space or end
        const parts = text.split(/([.!?]+(?:\s|$))/);
        
        if (parts.length > 1) {
            const fragment = document.createDocumentFragment();
            let currentSentenceBuffer = '';

            // We need to reconstruct sentences. 
            // split("Hello. World.") -> ["Hello", ". ", "World", "."]
            // We want to group "Hello" + ". "
            
            for (let i = 0; i < parts.length; i++) {
                const part = parts[i];
                if (!part) continue;

                currentSentenceBuffer += part;

                // If this part is a delimiter OR it's the last part
                const isDelimiter = /[.!?]+(?:\s|$)/.test(part);
                const isLast = i === parts.length - 1;

                if (isDelimiter || isLast) {
                    // Flush buffer as a sentence
                    // Only wrap if it has meaningful content (not just a space)
                    if (currentSentenceBuffer.trim().length > 0) {
                        const cleanText = currentSentenceBuffer.trim();
                        const baseHash = generateContentHash(cleanText);
                        
                        // Handle collisions (e.g. "Yes." appearing twice)
                        if (!hashCounts[baseHash]) hashCounts[baseHash] = 0;
                        const uniqueHash = `${baseHash}-${hashCounts[baseHash]}`;
                        hashCounts[baseHash]++;

                        const span = document.createElement('span');
                        span.className = 'sentence-entity';
                        span.dataset.sid = uniqueHash; // Use Hash as SID
                        span.dataset.hash = uniqueHash;
                        span.dataset.text = cleanText; // Store text for recovery
                        span.textContent = currentSentenceBuffer;
                        span.style.cursor = 'pointer';
                        span.style.transition = 'background 0.2s, color 0.2s';
                        
                        span.addEventListener('mouseenter', () => handleEntityHover(span.dataset.sid!));
                        span.addEventListener('mouseleave', () => clearHighlight());

                        fragment.appendChild(span);
                    } else {
                        // Just Text (whitespace)
                        fragment.appendChild(document.createTextNode(currentSentenceBuffer));
                    }
                    currentSentenceBuffer = '';
                }
            }
            
            node.parentNode?.replaceChild(fragment, node);
        } else {
            // No split, wrap whole node if it looks like content
            const cleanText = text.trim();
            if (cleanText.length > 0) {
                 const baseHash = generateContentHash(cleanText);
                 if (!hashCounts[baseHash]) hashCounts[baseHash] = 0;
                 const uniqueHash = `${baseHash}-${hashCounts[baseHash]}`;
                 hashCounts[baseHash]++;

                 const span = document.createElement('span');
                 span.className = 'sentence-entity';
                 span.dataset.sid = uniqueHash;
                 span.dataset.hash = uniqueHash;
                 span.dataset.text = cleanText;
                 span.textContent = text;
                 span.addEventListener('mouseenter', () => handleEntityHover(uniqueHash));
                 span.addEventListener('mouseleave', () => clearHighlight());
                 node.parentNode?.replaceChild(span, node);
            }
        }
    });
}

function highlightGroup(sid: string) {
    if (!containerRef.value) return;
    const group = containerRef.value.querySelectorAll(`[data-sid="${sid}"]`);
    group.forEach(el => {
        (el as HTMLElement).style.backgroundColor = 'rgba(0, 82, 217, 0.1)'; 
        (el as HTMLElement).style.borderRadius = '2px';
    });
}

function clearHighlight() {
     if (!containerRef.value) return;
     const all = containerRef.value.querySelectorAll('.sentence-entity');
     all.forEach(el => {
        (el as HTMLElement).style.backgroundColor = 'transparent';
    });
}

const currentSelectionSids = ref<Set<string>>(new Set());
const currentSelectedWord = ref<string>('');

// Watch for content to hydrate and reset selection
watch(() => props.article, async () => {
    currentSelectionSids.value.clear();
    currentSelectedWord.value = '';
    emit('selection', { word: '', sentences: [] });
    await nextTick();
    hydrateSentences();
}, { immediate: true, deep: true });

function handleSelectionEvent(event: Event) {
    const selection = window.getSelection();
    if (!selection) return;

    // A. Explicit Selection (Range)
    if (!selection.isCollapsed) {
        const range = selection.getRangeAt(0);
        const container = containerRef.value;
        if (!container) return;

        // Find all sentence entities in container that are touched by selection
        const entities = Array.from(container.querySelectorAll('.sentence-entity'));
        const selectedEntities = entities.filter(entity => selection.containsNode(entity, true));

        const uniqueSids = new Set<string>();
        const sentences: any[] = [];

        selectedEntities.forEach(entity => {
            const el = entity as HTMLElement;
            const sid = el.dataset.sid;
            if (sid && !uniqueSids.has(sid)) {
                uniqueSids.add(sid);
                const group = container.querySelectorAll(`[data-sid="${sid}"]`);
                let fullText = '';
                group.forEach(g => fullText += g.textContent);
                
                const sentenceData = identifySentence(el, fullText, true);
                if (sentenceData) {
                    sentences.push({ ...sentenceData, sid }); // Attach SID for focus matching
                }
            }
        });

        if (sentences.length > 0) {
            currentSelectionSids.value = uniqueSids;
            currentSelectedWord.value = '';
            // Detect if a single word is selected separately? 
            // If dragging across sentences, 'word' is usually N/A.
            emit('selection', { word: '', sentences });
            return;
        }
        
        // Fallback for non-entity text selection
        const text = selection.toString().trim();
        if (text.length > 0) {
            const sentence = identifySentence(event.target as HTMLElement, text, true);
            emit('selection', { word: '', sentences: [sentence] });
            currentSelectionSids.value.clear(); 
            currentSelectedWord.value = '';
            return;
        }
    }

    // B. Entity Click (Collapsed) - Single Sentence Focus
    const target = event.target as HTMLElement;
    const entity = target.closest('.sentence-entity') as HTMLElement;
    
    if (entity && selection.isCollapsed) {
        const sid = entity.dataset.sid;
        if (sid) {
             const group = containerRef.value?.querySelectorAll(`[data-sid="${sid}"]`);
             let fullText = '';
             group?.forEach(el => fullText += el.textContent);
             
             // Detect Word at Cursor
             let word = '';
             const s = window.getSelection();
             if (s && s.modify) {
                s.modify('move', 'backward', 'word');
                s.modify('extend', 'forward', 'word');
                word = s.toString().trim();
             }
             
             const detectedWord = (word && /^[a-zA-Z\-'’]+$/.test(word)) ? word : '';

             // TOGGLE LOGIC: If same word and same sentence, clear.
             if (detectedWord && detectedWord === currentSelectedWord.value && currentSelectionSids.value.has(sid)) {
                 currentSelectionSids.value.clear();
                 currentSelectedWord.value = '';
                 emit('selection', { word: '', sentences: [] });
                 // Clear browser selection to remove blue highlight
                 selection.removeAllRanges();
                 return;
             }

             const sentenceFn = identifySentence(target, fullText, true);
             const sentenceObj = { ...sentenceFn, sid };

             emit('selection', { 
                 word: detectedWord, 
                 sentences: [sentenceObj]
             });
             
             currentSelectionSids.value = new Set([sid]);
             currentSelectedWord.value = detectedWord;
             return;
        }
    }
    
    // Clear state if clicking empty space
    if (selection.isCollapsed && !entity) {
        currentSelectionSids.value.clear();
        currentSelectedWord.value = '';
        emit('selection', { word: '', sentences: [] });
    }
}

// Hover Handling for Focus Switch
// We attach this to the hydrator
const handleEntityHover = (sid: string) => {
    if (currentSelectionSids.value.has(sid)) {
        emit('focus-change', sid);
    }
    highlightGroup(sid);
};

function identifySentence(target: HTMLElement, textToMatch: string, isDirectSelection = false) {
    // Simplest: Just use the paragraph text
    const paragraph = target.closest('p, li, blockquote');
    if (paragraph) {
        let fullText = paragraph.textContent || '';
        
        let matchedSentence = '';
        
        if (isDirectSelection) {
            // User explicitly selected this text. Trust it primarily.
            matchedSentence = textToMatch;
        } else {
            // Context inference from a single word
            const sentences = fullText.match(/[^.!?]+[.!?]+(\s|$)/g) || [fullText];
            matchedSentence = sentences.find(s => s.includes(textToMatch))?.trim() || fullText;
        }

        // Find UUID in map (Fuzzy match)
        let foundUuid = null;
        let map = sentenceMap.value;
        if (map) { 
             const innerMap = map.map || {};
             for (const [uuid, data] of Object.entries(innerMap) as any) {
                 // Check if backend text contains our match OR our match contains backend text
                 // Use a reasonable overlap check or just simple inclusion
                 if (data.text.includes(matchedSentence) || matchedSentence.includes(data.text)) {
                     foundUuid = uuid;
                     if (!isDirectSelection) {
                         // Prefer backend text for full sentence context
                         matchedSentence = data.text; 
                     }
                     break;
                 }
             }
        }

        return {
            text: matchedSentence,
            uuid: foundUuid, 
            articleId: props.article.id
        };
    }
    // Fallback if no paragraph found (weird DOM) but direct selection
    if (isDirectSelection) {
        return {
            text: textToMatch,
            uuid: null,
            articleId: props.article.id
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
