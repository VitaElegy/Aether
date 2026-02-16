<script setup lang="ts">
import { ref, computed, defineAsyncComponent } from 'vue';

const props = defineProps<{
    modelValue: any; // content
    type: string; // 'markdown', 'kanban', etc.
    readOnly?: boolean;
}>();

const emit = defineEmits<{
    (e: 'update:modelValue', value: any): void;
    (e: 'change', isDirty: boolean): void;
    (e: 'update:toc', toc: any[]): void;
}>();

const editorRef = ref<any>(null); // Reference to the adapter

// Registry of adapters
const adapters: Record<string, any> = {
    markdown: defineAsyncComponent(() => import('./adapters/MarkdownEditorAdapter.vue')),
    // kanban: ...
};

const currentAdapter = computed(() => {
    return adapters[props.type] || adapters.markdown;
});

// Proxy methods
const load = async (content: any) => {
    if (editorRef.value) await editorRef.value.load(content);
};

const getValue = () => {
    return editorRef.value ? editorRef.value.getValue() : '';
};

const exportContent = async (format: 'markdown' | 'json') => {
    return editorRef.value ? editorRef.value.export(format) : '';
};

// Scroll helper
const scrollToPosition = (pos: number) => {
    editorRef.value?.scrollToPosition(pos);
};

// Expose methods to parent (EditorView)
defineExpose({
    load,
    getValue,
    export: exportContent,
    scrollToPosition
});
</script>

<template>
    <div class="universal-editor h-full w-full">
        <component 
            :is="currentAdapter"
            ref="editorRef"
            :initial-content="modelValue"
            :read-only="readOnly"
            @update:modelValue="(val: any) => emit('update:modelValue', val)"
            @change="(val: boolean) => emit('change', val)"
            @update:toc="(val: any[]) => emit('update:toc', val)"
        />
    </div>
</template>
