<template>
    <div class="flex flex-col h-full bg-paper-1 rounded-lg overflow-hidden border border-ink-border">
        <!-- Toolbar -->
        <div class="flex items-center justify-between p-2 border-b border-ink-border bg-paper-2">
            <div class="flex items-center gap-2">
                <button 
                    @click="editor?.chain().focus().toggleBold().run()" 
                    class="p-1 rounded hover:bg-ash text-ink-muted hover:text-ink transition-colors" 
                    :class="{ 'bg-accent text-white': editor?.isActive('bold') }"
                >
                     <i class="ri-bold text-sm"></i>
                </button>
                <button 
                    @click="editor?.chain().focus().toggleItalic().run()" 
                    class="p-1 rounded hover:bg-ash text-ink-muted hover:text-ink transition-colors" 
                    :class="{ 'bg-accent text-white': editor?.isActive('italic') }"
                >
                     <i class="ri-italic text-sm"></i>
                </button>
                <button 
                    @click="editor?.chain().focus().toggleCodeBlock().run()" 
                    class="p-1 rounded hover:bg-ash text-ink-muted hover:text-ink transition-colors" 
                    :class="{ 'bg-accent text-white': editor?.isActive('codeBlock') }"
                >
                     <i class="ri-code-line text-sm"></i>
                </button>
            </div>
            <div class="text-xs text-ink-muted">Markdown Support</div>
        </div>

        <!-- Editor Area -->
        <editor-content :editor="editor" class="flex-1 overflow-y-auto p-4 prose prose-sm max-w-none text-ink focus:outline-none" />
    </div>
</template>

<script setup lang="ts">
import { useEditor, EditorContent } from '@tiptap/vue-3';
import StarterKit from '@tiptap/starter-kit';
import { watch, onBeforeUnmount } from 'vue';

const props = defineProps<{
    modelValue: string;
}>();

const emit = defineEmits(['update:modelValue']);

const editor = useEditor({
    content: props.modelValue,
    extensions: [StarterKit],
    editorProps: {
        attributes: {
            class: 'focus:outline-none h-full',
        },
    },
    onUpdate: () => {
        if (editor.value) {
            // Basic JSON serialization or HTML? 
            // For now emitting HTML/String. 
            // Ideally backend stores JSON but we used serde_json::Value in model.
            // Let's emit HTML string for simplicity or JSON if we want strictness.
            // The prompt/store uses 'any', typical for JSON. 
            // Let's settle on: We store the JSON object from Tiptap.
            emit('update:modelValue', editor.value.getJSON());
        }
    },
});

// Watch for external changes
watch(() => props.modelValue, (newVal) => {
    if (editor.value && newVal) {
        // Only update if content is different to avoid cursor jumps
        // For complex JSON objects deep compare is needed, here we just set it if editor is empty or fully replaced
        // This is a naive implementation.
        // For MVP: Assumes one-way binding on load.
    }
});

onBeforeUnmount(() => {
    editor.value?.destroy();
});
</script>

<style scoped>
</style>
