<template>
    <div class="h-full flex flex-col p-4 max-w-4xl mx-auto w-full">
        <div class="flex items-center justify-between mb-8">
            <div>
                <h3 class="text-2xl font-black font-serif text-ink">Project Specs</h3>
                <p class="text-ink/60 text-sm mt-1">Define research goals, scope, and technical constraints.</p>
            </div>
            <button @click="toggleEdit" class="px-4 py-2 border border-ash/20 hover:border-accent/50 rounded-lg text-xs font-bold text-ink transition-colors flex items-center gap-2 bg-white">
                <i :class="editing ? 'ri-save-line' : 'ri-edit-line'"></i>
                {{ editing ? 'Save Changes' : 'Edit Specs' }}
            </button>
        </div>

        <!-- Spec Editor / Viewer -->
        <div class="flex-1 bg-paper-2 rounded-xl border border-ash/20 p-8 shadow-sm overflow-y-auto">
            <textarea 
                v-if="editing" 
                v-model="content" 
                class="w-full h-full bg-transparent resize-none focus:outline-none font-mono text-sm leading-relaxed text-ink"
                placeholder="# Research Goals..."
            ></textarea>
            
            <div v-else class="prose prose-sm max-w-none prose-headings:font-serif prose-headings:font-bold prose-a:text-accent">
                <!-- Simple Markdown Renderer Mock -->
                <div v-html="renderMarkdown(content)"></div>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import MarkdownIt from 'markdown-it';
import { vrkbApi } from '@/api/vrkb';
import { useVrkbStore } from '@/stores/vrkb';

const store = useVrkbStore();
const md = new MarkdownIt();
const editing = ref(false);
const content = ref('');

const loadSpecs = async () => {
    if (!store.currentProject) return;
    try {
        content.value = await vrkbApi.getSpecs(store.currentProject.id);
    } catch (e) {
        console.error("Failed to load specs", e);
    }
};

const saveSpecs = async () => {
    if (!store.currentProject) return;
    try {
        await vrkbApi.saveSpecs(store.currentProject.id, "Project Spec", content.value);
        editing.value = false;
    } catch (e) {
        console.error("Failed to save specs", e);
    }
};

onMounted(() => {
    loadSpecs();
});

const toggleEdit = () => {
    if (editing.value) {
        saveSpecs();
    } else {
        editing.value = true;
    }
};

const renderMarkdown = (text: string) => {
    return md.render(text || "# No specs defined yet.");
};
</script>
