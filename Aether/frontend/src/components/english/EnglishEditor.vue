<script setup lang="ts">
import { ref, reactive, watch, onBeforeUnmount, computed } from 'vue';
import { useEditor, EditorContent } from '@tiptap/vue-3';
import StarterKit from '@tiptap/starter-kit';
import Placeholder from '@tiptap/extension-placeholder';
import { Markdown } from 'tiptap-markdown';
import { useDebounceFn } from '@vueuse/core';
import EnglishMetaDrawer from './EnglishMetaDrawer.vue';

const props = defineProps<{
    initialData: {
        id?: string;
        title: string;
        text: string;
        background: string;
        category: string; // Added
        references: Array<{ title: string; url: string }>;
        status: 'Draft' | 'Published';
    };
}>();

const emit = defineEmits(['save', 'cancel']);

// State
const isLiveMode = ref(true);
const showDrawer = ref(false);
const saveStatus = ref<'Saved' | 'Saving...' | 'Unsaved'>('Saved');
const lastSavedTime = ref<Date>(new Date());

const form = reactive({
    id: props.initialData.id,
    title: props.initialData.title,
    text: props.initialData.text,
    background: props.initialData.background,
    category: props.initialData.category || 'English Analysis',
    references: [...props.initialData.references],
    status: props.initialData.status
});

const stats = reactive({ chars: 0, words: 0 });

const updateStats = (text: string) => {
    stats.chars = text.length;
    stats.words = text.split(/\s+/).filter(Boolean).length;
};

// Editor Setup
const editor = useEditor({
    extensions: [
        StarterKit.configure({ heading: { levels: [1, 2, 3] } }),
        Markdown,
        Placeholder.configure({ placeholder: 'Start writing your analysis...' }),
    ],
    content: props.initialData.text,
    onUpdate: ({ editor }) => {
        form.text = (editor.storage as any).markdown.getMarkdown();
        updateStats(form.text);
    },
    editorProps: {
        attributes: { class: 'prose prose-stone prose-lg max-w-none focus:outline-none min-h-[60vh] font-sans' },
    }
});

// Watch for ID/Status injections from parent (e.g. after Create)
watch(() => props.initialData.id, (newId) => {
    if (newId) form.id = newId;
});
watch(() => props.initialData.status, (newStatus) => {
    if (newStatus) form.status = newStatus;
});

// Auto-Save Logic
const triggerSave = useDebounceFn(() => {
    if (form.status === 'Draft') {
        saveStatus.value = 'Saving...';
        emit('save', { ...form });
        // Assume success for UI snappiness, or parent can callback. 
        // ideally parent should signal completion, but for now:
        setTimeout(() => {
            saveStatus.value = 'Saved';
            lastSavedTime.value = new Date();
        }, 800);
    }
}, 2000);

// Deep Watch for Auto-Save
watch(
    () => form,
    () => {
        if (form.status === 'Draft') {
            saveStatus.value = 'Unsaved';
            triggerSave();
        }
    },
    { deep: true }
);

// Watch for manual raw edits (for stats)
watch(() => form.text, (newVal) => {
    if (!isLiveMode.value) {
        updateStats(newVal);
    }
});

const toggleMode = () => {
    isLiveMode.value = !isLiveMode.value;
    if (isLiveMode.value && editor.value) {
        editor.value.commands.setContent(form.text);
    } else if (!isLiveMode.value && editor.value) {
        form.text = (editor.value.storage as any).markdown.getMarkdown();
    }
};

const handlePublish = () => {
    form.status = 'Published';
    emit('save', { ...form });
};

const handleUnpublish = () => {
    form.status = 'Draft';
};

onBeforeUnmount(() => {
    editor.value?.destroy();
});
</script>

<template>
    <div class="english-editor-container bg-white text-ink font-sans flex flex-col h-full relative">
        
        <!-- Teleport Controls to Global Nav -->
        <Teleport to="#nav-right-portal">
            <div class="flex items-center gap-6 pointer-events-auto mr-4">
                 
                 <!-- Back / Cancel -->
                 <button 
                    @click="$emit('cancel')" 
                    class="text-xs font-bold uppercase tracking-widest text-gray-400 hover:text-gray-900 transition-colors"
                 >
                    Back
                 </button>

                 <!-- Meta / Info -->
                 <button 
                     @click="showDrawer = true" 
                     class="text-xs font-bold uppercase tracking-widest text-gray-400 hover:text-gray-900 transition-colors"
                 >
                    Info
                 </button>

                 <!-- Auto-Save Status -->
                 <div class="text-xs font-bold uppercase tracking-widest transition-colors text-gray-300 select-none">
                    {{ saveStatus }}
                 </div>

                 <!-- Publish Action (Primary Text Button) -->
                 <button 
                    v-if="form.status === 'Draft'"
                    @click="handlePublish"
                    class="text-xs font-bold uppercase tracking-widest text-gray-900 border-b-2 border-gray-900 pb-0.5 hover:opacity-70 transition-opacity"
                >
                    Publish
                 </button>
            </div>
        </Teleport>


        <!-- Main Content -->
        <div class="flex-1 overflow-y-auto custom-scrollbar">
            <div class="w-full max-w-[75ch] mx-auto px-6 py-12 flex flex-col min-h-full">
                <!-- Title -->
                <input
                    v-model="form.title"
                    placeholder="Analysis Title..."
                    class="text-4xl sm:text-5xl font-bold tracking-tight mb-8 placeholder:text-stone-300 focus:outline-none w-full bg-transparent border-none p-0 flex-shrink-0 font-sans text-ink"
                />

                <!-- Editor Body -->
                <div class="flex-1 relative pb-32">
                    <editor-content v-if="isLiveMode" :editor="editor" class="tiptap-editor h-full" />
                    <textarea
                        v-else
                        v-model="form.text"
                        placeholder="# Start writing..."
                        class="w-full h-full min-h-[60vh] resize-none text-base leading-relaxed text-stone-700 placeholder:text-stone-300 focus:outline-none bg-transparent font-mono"
                    ></textarea>
                </div>
            </div>
        </div>

        <EnglishMetaDrawer 
            :visible="showDrawer"
            :form="form"
            :stats="stats"
            @close="showDrawer = false"
            @publish="handlePublish"
            @unpublish="handleUnpublish"
        />
    </div>
</template>

<style scoped>
/* Standard Aesthetic - No Custom Fonts */

/* Tiptap Typography Overrides */
:deep(.tiptap-editor .ProseMirror) { 
    outline: none; 
    font-size: 1.1rem;
    line-height: 1.7;
    color: var(--text-primary, #333);
}

:deep(.tiptap-editor .ProseMirror h1),
:deep(.tiptap-editor .ProseMirror h2),
:deep(.tiptap-editor .ProseMirror h3) { 
    color: var(--text-primary, #111);
    margin-top: 1.5em;
    margin-bottom: 0.5em;
    font-weight: 700;
}

:deep(.tiptap-editor .ProseMirror h1) { font-size: 1.8em; line-height: 1.2; }
:deep(.tiptap-editor .ProseMirror h2) { font-size: 1.5em; }
:deep(.tiptap-editor .ProseMirror p) { margin-bottom: 1.25em; }
:deep(.tiptap-editor .ProseMirror p.is-editor-empty:first-child::before) { color: #d4d4d4; content: attr(data-placeholder); float: left; height: 0; pointer-events: none; }

.custom-scrollbar::-webkit-scrollbar { width: 4px; }
.custom-scrollbar::-webkit-scrollbar-thumb { background: #e5e5e5; border-radius: 2px; }
.custom-scrollbar::-webkit-scrollbar-track { background: transparent; }
</style>
