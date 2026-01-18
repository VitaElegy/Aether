<script setup lang="ts">
import { ref, reactive, watch, onBeforeUnmount } from 'vue';
import { useEditor, EditorContent } from '@tiptap/vue-3';
import StarterKit from '@tiptap/starter-kit';
import Placeholder from '@tiptap/extension-placeholder';
import { Markdown } from 'tiptap-markdown';
import EnglishMetaDrawer from './EnglishMetaDrawer.vue';

const props = defineProps<{
    initialData: {
        title: string;
        text: string;
        background: string;
        references: Array<{ title: string; url: string }>;
        status: 'Draft' | 'Published';
    };
}>();

const emit = defineEmits(['save', 'cancel']);

// State
const isLiveMode = ref(true);
const showDrawer = ref(false);
const form = reactive({
    title: props.initialData.title,
    text: props.initialData.text,
    background: props.initialData.background,
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
        attributes: { class: 'prose prose-stone prose-lg max-w-none focus:outline-none min-h-[60vh] serif-font' },
    }
});

// Watch for manual raw edits
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

const handleSave = () => {
    emit('save', { ...form });
};

const handlePublish = () => {
    form.status = 'Published';
    // Optionally auto-save on publish action
};

const handleUnpublish = () => {
    form.status = 'Draft';
};

onBeforeUnmount(() => {
    editor.value?.destroy();
});
</script>

<template>
    <div class="english-editor-container bg-paper text-ink font-serif flex flex-col h-full relative">
        
        <!-- Teleport Controls to Global Nav -->
        <Teleport to="#nav-right-portal">
            <div class="flex items-center gap-6 pointer-events-auto">
                 <button @click="$emit('cancel')" class="text-xs font-bold uppercase tracking-widest text-ink/40 hover:text-ink transition-colors mr-2">
                    BACK
                 </button>

                 <!-- Status -->
                 <span class="text-xs font-mono uppercase tracking-widest text-ink/30">
                    {{ form.status === 'Published' ? 'PUB' : 'DRAFT' }}
                 </span>

                 <!-- Mode Toggle -->
                 <div class="flex items-center gap-2">
                    <button 
                        @click="toggleMode" 
                        class="text-xs font-bold uppercase tracking-widest transition-colors"
                        :class="isLiveMode ? 'text-ink' : 'text-ink/40 hover:text-ink'"
                    >
                        {{ isLiveMode ? 'LIVE' : 'RAW' }}
                    </button>
                 </div>

                 <!-- Meta Trigger -->
                 <button 
                    @click="showDrawer = true" 
                    class="text-xs font-bold uppercase tracking-widest text-ink/40 hover:text-ink transition-colors"
                >
                    INFO
                 </button>

                 <button 
                    @click="handleSave"
                    class="text-xs font-bold uppercase tracking-widest text-ink hover:text-ink/60 transition-colors"
                >
                    SAVE
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
                    class="text-4xl sm:text-5xl font-bold tracking-tight mb-8 placeholder:text-stone-300 focus:outline-none w-full bg-transparent border-none p-0 flex-shrink-0 serif-font text-ink"
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
.font-serif {
    font-family: "Playfair Display", "Times New Roman", serif;
}
.serif-font {
    font-family: "Playfair Display", "Times New Roman", serif;
}

/* Tiptap Typography Overrides */
:deep(.tiptap-editor .ProseMirror) { 
    outline: none; 
    font-family: 'Crimson Text', serif;
    font-size: 1.15rem;
    line-height: 1.8;
    color: #333;
}

:deep(.tiptap-editor .ProseMirror h1),
:deep(.tiptap-editor .ProseMirror h2),
:deep(.tiptap-editor .ProseMirror h3) { 
    font-family: 'Playfair Display', serif;
    color: #111;
    margin-top: 1.5em;
    margin-bottom: 0.5em;
}

:deep(.tiptap-editor .ProseMirror h1) { font-size: 2em; line-height: 1.2; }
:deep(.tiptap-editor .ProseMirror h2) { font-size: 1.5em; }
:deep(.tiptap-editor .ProseMirror p) { margin-bottom: 1.2em; }
:deep(.tiptap-editor .ProseMirror p.is-editor-empty:first-child::before) { color: #d4d4d4; content: attr(data-placeholder); float: left; height: 0; pointer-events: none; }

.custom-scrollbar::-webkit-scrollbar { width: 4px; }
.custom-scrollbar::-webkit-scrollbar-thumb { background: #e5e5e5; border-radius: 2px; }
.custom-scrollbar::-webkit-scrollbar-track { background: transparent; }
</style>
