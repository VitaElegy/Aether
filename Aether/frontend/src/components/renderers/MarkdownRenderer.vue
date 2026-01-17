<script setup lang="ts">
import { computed } from 'vue';
import { marked, type Tokens } from 'marked';
import katex from 'katex';
import 'katex/dist/katex.min.css';

interface Props {
    content: string;
}

const props = defineProps<Props>();

// --- Extensions ---

// 1. Math Extension ($$ block, $ inline)
// Note: We need separate extensions for block vs inline usually, or one smart one.
// A simplified robust regex approach:

// Block Math: $$ ... $$
const mathBlockExtension = {
    name: 'mathBlock',
    level: 'block' as const,
    start(src: string) { return src.match(/\$\$/)?.index; },
    tokenizer(src: string, tokens: any) {
        const rule = /^\$\$([\s\S]+?)\$\$/;
        const match = rule.exec(src);
        if (match) {
            return {
                type: 'mathBlock', // Must match name
                raw: match[0],
                text: match[1].trim()
            };
        }
    },
    renderer(token: any) {
        return katex.renderToString(token.text, {
            displayMode: true,
            throwOnError: false
        });
    }
};

// Inline Math: $ ... $
const mathInlineExtension = {
    name: 'mathInline',
    level: 'inline' as const,
    start(src: string) { return src.match(/\$/)?.index; },
    tokenizer(src: string, tokens: any) {
        const rule = /^\$([^$\n]+?)\$/;
        const match = rule.exec(src);
        if (match) {
            return {
                type: 'mathInline',
                raw: match[0],
                text: match[1].trim()
            };
        }
    },
    renderer(token: any) {
        return katex.renderToString(token.text, {
            displayMode: false,
            throwOnError: false
        });
    }
};

// 2. Semantic Block Extension (::: type {meta} \n content \n :::)
const semanticBlockExtension = {
    name: 'semanticBlock',
    level: 'block' as const,
    start(src: string) { return src.match(/^:::\s/)?.index; },
    tokenizer(src: string, tokens: any) {
        // Regex to capture: ::: type [params] \n content \n :::
        // Group 1: Type (e.g. function-card)
        // Group 2: Header Params (e.g. { ... }) - Captures rest of line
        // Group 3: Content (Multiline)
        const rule = /^:::\s*([\w-]+)([^\n]*)\n([\s\S]*?)\n:::/;
        const match = rule.exec(src);
        if (match) {
            const blockType = match[1];
            const headerParams = match[2].trim();
            const innerContent = match[3];

            const token: any = {
                type: 'semanticBlock',
                raw: match[0],
                blockType: blockType,
                headerParams: headerParams,
                contentStr: innerContent,
                tokens: [] // For nested parsing
            };

            // Vital: Parse the inner content as Markdown blocks
            // @ts-ignore
            this.lexer.blockTokens(innerContent, token.tokens);

            return token;
        }
    },
    renderer(token: any): string {
        const type = token.blockType;
        let metaStr = token.headerParams;
        let contentStr = token.contentStr;
        let hideContent = false;

        // Fallback: If header params empty/invalid, try parsing body as JSON (Common User Pattern)
        // Specially for function-card where the entire body might be the config
        if ((!metaStr || metaStr === '') && contentStr.trim().startsWith('{')) {
             try {
                 // Try parsing the body
                 JSON.parse(contentStr);
                 // If successful, promote body to meta
                 metaStr = contentStr;
                 hideContent = true; // Don't render the raw JSON text
             } catch (e) { /* Not JSON */ }
        } else if (metaStr && !metaStr.startsWith('{')) {
            // Handle space-separated args if not JSON? (Future proofing, skip for now)
        }

        // Parse inner content (if not hidden)
        // @ts-ignore
        const innerHtml = hideContent ? '' : this.parser.parse(token.tokens);

        // --- Layout Logic Ported from MathManuscriptLayout ---
        
        // 1. Function Plots
        if (type === 'function' || type === 'function-card') {
            let fn = 'x'; // Default
            try {
                const parsedMeta = JSON.parse(metaStr || '{}');
                // Handle both "fn" (legacy) and "function" (user input) keys
                if (parsedMeta.fn) fn = parsedMeta.fn;
                if (parsedMeta.function) fn = parsedMeta.function;
            } catch (e) { /* ignore */ }

            // Encode inner content for safe transport to Lab via dataset
            // We use encodeURIComponent to handle UTF-8 (Chinese) characters correctly,
            // which btoa() alone cannot handle.
            const safeDescription = encodeURIComponent(innerHtml);

            return `
                <div class="semantic-block my-8 p-6 rounded-lg border border-orange-200 bg-orange-50/50 hover:shadow-md transition-all">
                    <div class="flex items-center justify-between mb-4">
                        <span class="text-[10px] font-black uppercase tracking-widest text-orange-600">Function Visualization</span>
                        <span class="text-xs font-mono text-orange-800/60">${fn}</span>
                    </div>
                    <!-- Hydration Target -->
                    <div 
                        class="math-function-inline w-full h-[200px] bg-white/50 rounded overflow-hidden relative" 
                        data-fn="${fn}" 
                        data-description="${safeDescription}"
                        data-hydrated="false">
                    </div>
                    <div class="mt-4 text-xs text-ink/60 italic">
                        ${innerHtml}
                    </div>
                </div>`;
        }

        // 2. Standard Semantic Blocks (Theorems, Definitions)
        const badgeColor = type === 'theorem' ? 'text-purple-600 bg-purple-50 border-purple-200' :
                          type === 'definition' ? 'text-blue-600 bg-blue-50 border-blue-200' :
                          'text-ink/70 bg-ash/30 border-ash';

        return `
            <div class="semantic-block my-8 p-6 rounded-lg border-l-4 ${badgeColor} hover:shadow-sm transition-shadow">
                <div class="flex items-center gap-2 mb-2">
                    <span class="text-xs font-black uppercase tracking-widest opacity-80">${type}</span>
                </div>
                <div class="italic text-ink/90">
                    ${innerHtml}
                </div>
            </div>`;
    }
};

// Register Extensions
marked.use({ extensions: [mathBlockExtension, mathInlineExtension, semanticBlockExtension] });

const renderedHtml = computed(() => {
    if (!props.content) return '';
    // marked handles the rendering pipeline with extensions
    return marked(props.content);
});
</script>

<template>
    <div class="prose prose-lg max-w-none" v-html="renderedHtml"></div>
</template>
