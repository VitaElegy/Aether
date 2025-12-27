<template>
  <div class="version-view p-6 max-w-5xl mx-auto">
    <div class="header mb-6 flex justify-between items-center">
      <div class="flex flex-col">
         <h1 class="text-3xl font-serif text-gray-900 tracking-wide italic">
            Version {{ versionId }}
         </h1>
         <div class="text-gray-400 text-[10px] font-mono mt-1 tracking-widest uppercase opacity-50 pl-1">
            Ref: {{ originalId.slice(0, 8) }}
         </div>
      </div>

      <div class="flex items-center gap-3">
         <button
            class="text-gray-500 hover:text-gray-800 font-serif italic text-lg transition-colors duration-200"
            @click="router.push(`/content/${originalId}/history`)"
         >
            ← return to history
         </button>

         <div class="w-px h-4 bg-gray-200 rotate-12"></div>

         <button
            class="font-serif italic text-lg transition-colors duration-200"
            :class="isDiffMode ? 'text-gray-900 underline decoration-1 underline-offset-4' : 'text-gray-400 hover:text-gray-600'"
            @click="toggleDiff"
         >
            {{ isDiffMode ? 'reading changes' : 'read content' }}
         </button>
      </div>
    </div>

    <t-loading v-if="loading" />
    <div v-else>
         <!-- Diff View -->
         <div v-if="isDiffMode">
             <t-card title="Diff with Previous Version" class="mb-4">
                 <!-- GitHub-style Diff Table -->
                 <DiffViewer :changes="diffContent" :empty-message="diffEmptyMessage" />
             </t-card>
         </div>

         <!-- Content View -->
         <div v-else class="content-preview">
            <t-card :title="contentData.title || 'Untitled'" class="mb-4">
               <div class="prose max-w-none p-4" v-html="renderMarkdown(contentData.body)"></div>
            </t-card>
            <div class="meta text-xs text-gray-400 mt-2">
               Raw Data: {{ contentData }}
            </div>
         </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { contentApi } from '@/api/content';
import { MessagePlugin } from 'tdesign-vue-next';
import { ArrowLeftIcon, HistoryIcon, ComponentCheckboxIcon, FileIcon } from 'tdesign-icons-vue-next';
import MarkdownIt from 'markdown-it';
import DiffViewer from '@/components/DiffViewer.vue';

const route = useRoute();
const router = useRouter();
const originalId = route.params.id as string;
const versionId = parseInt(route.params.version as string);
const isDiffMode = ref(route.query.diff === 'true');

const loading = ref(true);
const contentData = ref<any>({});
const diffContent = ref<any[] | null>(null);

const md = new MarkdownIt();

const renderMarkdown = (body: any) => {
    if (!body) return '';
    if (typeof body === 'string') return md.render(body);
    if (body.type === 'Markdown' && typeof body.data === 'string') {
        return md.render(body.data);
    }
    return JSON.stringify(body);
};

const diffEmptyMessage = computed(() => {
    if (versionId === 1) {
        return 'This is the first version, nothing to compare against.';
    }
    return 'No differences found.';
});

const loadData = async () => {
    try {
        loading.value = true;

        // Load Version Content
        const data = await contentApi.getVersion(originalId, versionId);
        contentData.value = typeof data === 'string' ? JSON.parse(data) : data;

        // Load Diff if requested
        if (isDiffMode.value && versionId > 1) {
             const diff = await contentApi.getDiff(originalId, versionId - 1, versionId);
             diffContent.value = diff.changes;
        } else if (isDiffMode.value && versionId === 1) {
             diffContent.value = []; // Set empty for v1
        }

    } catch (e) {
        console.error(e);
        MessagePlugin.error('Failed to load version data');
    } finally {
        loading.value = false;
    }
};

const toggleDiff = async () => {
    isDiffMode.value = !isDiffMode.value;

    // If switching to diff mode and data not loaded yet
    if (isDiffMode.value && !diffContent.value) {
         if (versionId <= 1) {
             diffContent.value = []; // Mark as loaded but empty
             return;
         }

         try {
             loading.value = true;
             const diff = await contentApi.getDiff(originalId, versionId - 1, versionId);
             diffContent.value = diff.changes;
         } catch (e) {
             MessagePlugin.error('Failed to load diff');
         } finally {
             loading.value = false;
         }
    }
};

onMounted(loadData);
</script>

<style scoped>
.version-view {
    background: var(--bg-color-page);
    min-height: 100vh;
}
</style>
