<style scoped>
.history-page {
    background-color: var(--td-bg-color-page);
    min-height: 100vh;
}
.history-tree {
    position: relative;
    padding-left: 20px;
}
.history-tree::before {
    content: '';
    position: absolute;
    left: 29px; /* Align with dots */
    top: 0;
    bottom: 0;
    width: 2px;
    background: var(--td-component-stroke);
    z-index: 0;
}
</style>

<template>
  <div class="history-page p-6">
    <div class="max-w-4xl mx-auto">
        <div class="header mb-8 flex justify-between items-center bg-white p-4 rounded-lg shadow-sm border border-gray-100">
            <div>
                <h1 class="text-2xl font-bold text-gray-800">Version History</h1>
                <p class="text-gray-500 text-sm mt-1">Track changes and revisions for this article.</p>
            </div>
            <t-button variant="outline" shape="round" @click="router.back()">
                <template #icon><arrow-left-icon /></template>
                Back to Editor
            </t-button>
        </div>

        <t-loading v-if="loading" size="small" class="flex justify-center py-12" />

        <div v-else-if="versions.length === 0" class="text-center py-16 bg-white rounded-lg border border-dashed border-gray-300">
            <div class="text-gray-400 mb-2">No history available for this document.</div>
        </div>

        <div v-else class="history-tree space-y-8">
            <div v-for="(v, index) in versions" :key="v.id" class="relative z-10">
                <!-- Timeline Dot -->
                <div class="absolute -left-8 top-6 w-5 h-5 rounded-full border-4 border-white shadow-sm flex items-center justify-center"
                     :class="index === 0 ? 'bg-blue-600 ring-2 ring-blue-100' : 'bg-gray-400'">
                </div>

                <div class="pl-4">
                     <span class="text-xs font-mono text-gray-400 mb-1 block">{{ formatDate(v.created_at) }}</span>
                    <t-card :bordered="false" class="transition-all duration-200 hover:shadow-md border border-gray-200 group relative overflow-hidden">
                        <!-- Version Badge -->
                        <div class="absolute right-0 top-0 p-0">
                             <div class="bg-gray-100 text-gray-500 text-xs px-3 py-1 rounded-bl-lg font-mono">v{{ v.version }}</div>
                        </div>

                        <div class="flex items-start gap-4">
                             <!-- Action Buttons (Hover) -->
                             <div class="flex-1">
                                <h3 class="font-semibold text-lg text-gray-800 mb-1 flex items-center gap-2">
                                    {{ v.reason || 'Update' }}
                                    <t-tag v-if="index === 0" theme="primary" variant="light" size="small">Latest</t-tag>
                                </h3>
                                <div class="text-sm text-gray-600 mb-3 line-clamp-1">
                                    <span class="font-medium text-gray-900">{{ v.title }}</span>
                                </div>

                                <div class="flex items-center gap-3 text-xs text-gray-400">
                                    <div class="flex items-center gap-1 bg-gray-50 px-2 py-1 rounded-full">
                                        <user-icon size="12px" />
                                        <span>Editor: {{ v.editor_id.slice(0,6) }}</span>
                                    </div>
                                    <span>{{ timeAgo(v.created_at) }}</span>
                                </div>
                             </div>
                        </div>

                        <template #actions>
                            <div class="flex gap-2 opacity-100 sm:opacity-0 sm:group-hover:opacity-100 transition-opacity">
                                <t-button size="small" variant="text" theme="primary" @click="viewVersion(v.version)">
                                    <template #icon><file-icon /></template> Review
                                </t-button>
                                <t-button v-if="index !== versions.length - 1" size="small" variant="text" theme="default" @click="compareWithPrevious(v.version)">
                                    <template #icon><swap-icon /></template> Compare Prev
                                </t-button>
                            </div>
                        </template>
                    </t-card>
                </div>
            </div>
        </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { contentApi, type ContentVersionSnapshot } from '@/api/content';
import { MessagePlugin } from 'tdesign-vue-next';
import { ArrowLeftIcon, UserIcon, FileIcon, SwapIcon } from 'tdesign-icons-vue-next';

const route = useRoute();
const router = useRouter();
const contentId = route.params.id as string;

const versions = ref<ContentVersionSnapshot[]>([]);
const loading = ref(true);

const formatDate = (iso: string) => {
    return new Date(iso).toLocaleString(undefined, {
        year: 'numeric', month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit'
    });
};

const timeAgo = (iso: string) => {
    const date = new Date(iso);
    const now = new Date();
    const diff = (now.getTime() - date.getTime()) / 1000;
    if (diff < 60) return 'Just now';
    if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
    if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
    return `${Math.floor(diff / 86400)}d ago`;
};

const viewVersion = (v: number) => {
    router.push(`/content/${contentId}/version/${v}`);
};

const compareWithPrevious = (v: number) => {
    // Current assumption: Compare v with v-1.
    // Need backend support or generic diff support.
    // The current backend `get_content_diff_handler` takes (v1, v2).
    // So we can route to a compare view or just the version view with diff param.
    // Let's rely on the VersionView having the ability to default to comparing with previous
    // or we construct a URL manually if we had a dedicated compare route.
    // For now, reuse existing pattern: /version/:v?diff=true (implied compare with prev?)
    // Actually, `VersionView` needs to handle logic.
    // Let's stick to the previous logic but clearer naming.
    router.push(`/content/${contentId}/version/${v}?diff=true`);
};

onMounted(async () => {
    try {
        versions.value = await contentApi.getHistory(contentId);
        // Sort descending just in case (newest first)
        versions.value.sort((a, b) => b.version - a.version);
    } catch (e) {
        MessagePlugin.error('Failed to load history');
    } finally {
        loading.value = false;
    }
});
</script>
