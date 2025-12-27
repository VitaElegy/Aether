<template>
  <div class="history-container max-w-4xl mx-auto p-6">
    <div class="header mb-6 flex justify-between items-center">
      <h1 class="text-2xl font-bold">Version History</h1>
      <t-button variant="outline" @click="router.back()">Back to Editor</t-button>
    </div>

    <t-loading v-if="loading" size="small" />

    <div v-else-if="versions.length === 0" class="text-gray-500">
      No history available.
    </div>

    <div v-else class="timeline-wrapper">
      <t-timeline mode="left">
        <t-timeline-item v-for="v in versions" :key="v.id" :label="formatDate(v.created_at)">
           <t-card :title="`Version ${v.version}: ${v.reason || 'Update'}`" hover-shadow class="mb-4 cursor-pointer" @click="viewVersion(v.version)">
              <div class="text-sm text-gray-600 mb-2">
                 Title: {{ v.title }}
              </div>
              <div class="text-xs text-gray-400">
                 Saved by Editor ID: {{ v.editor_id.slice(0,8) }}...
              </div>
              <template #actions>
                 <t-button size="small" variant="text" @click.stop="viewVersion(v.version)">View</t-button>
                 <t-button size="small" variant="text" @click.stop="compareWithCurrent(v.version)">Diff</t-button>
              </template>
           </t-card>
        </t-timeline-item>
      </t-timeline>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { contentApi, type ContentVersionSnapshot } from '@/api/content';
import { MessagePlugin } from 'tdesign-vue-next';

const route = useRoute();
const router = useRouter();
const contentId = route.params.id as string;

const versions = ref<ContentVersionSnapshot[]>([]);
const loading = ref(true);

const formatDate = (iso: string) => {
    return new Date(iso).toLocaleString();
};

const viewVersion = (v: number) => {
    router.push(`/content/${contentId}/version/${v}`);
};

const compareWithCurrent = (v: number) => {
    // We need to know current version?
    // Usually "current" is the head.
    // If we want diff between v and v-1, or v and HEAD.
    // Let's assume user wants to see what changed in THIS version compared to previous?
    // Or compare THIS version to CURRENT?
    // Let's just go to a Diff view where we can select.
    // For now, let's link to the Version View which will have diff capabilities.
    router.push(`/content/${contentId}/version/${v}?diff=true`);
};

onMounted(async () => {
    try {
        versions.value = await contentApi.getHistory(contentId);
    } catch (e) {
        MessagePlugin.error('Failed to load history');
    } finally {
        loading.value = false;
    }
});
</script>

<style scoped>
.history-container {
    background: var(--bg-color-page);
    min-height: 100vh;
}
</style>
