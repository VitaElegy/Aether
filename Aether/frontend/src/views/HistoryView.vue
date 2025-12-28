<template>
  <div class="min-h-screen bg-[var(--bg-color-page)] flex flex-col">
    <TopNavBar>
      <template #left>
        <TextAction size="sm" @click="router.back()">
           ← back to editor
        </TextAction>
      </template>

      <template #center>
        <div class="flex items-center gap-4">
             <span class="text-[10px] font-black uppercase tracking-[0.3em] text-ink/40">
                Version History
            </span>
         </div>
      </template>

      <template #right>
         <!-- Empty for now -->
      </template>
    </TopNavBar>

    <div class="flex-1 w-full max-w-4xl mx-auto p-6 md:p-12">
        <t-loading v-if="loading" size="small" />

        <div v-else-if="versions.length === 0" class="text-gray-500 py-12 text-center font-serif italic text-lg">
          No history available.
        </div>

        <div v-else class="timeline-wrapper">
          <t-timeline mode="left">
            <t-timeline-item v-for="v in versions" :key="v.id" :label="formatDate(v.created_at)">
               <t-card :title="`Version ${v.version}: ${v.reason || 'Update'}`" hover-shadow class="mb-4 cursor-pointer border-l-4 border-l-transparent hover:border-l-gray-300 transition-all font-serif" @click="viewVersion(v.version)">
                  <div class="text-sm text-gray-600 mb-2 font-sans">
                     Title: {{ v.title }}
                  </div>
                  <div class="text-xs text-gray-400 font-mono opacity-60">
                     By: {{ v.editor_id.slice(0,8) }}...
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
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { contentApi, type ContentVersionSnapshot } from '@/api/content';
import { MessagePlugin } from 'tdesign-vue-next';
import PageContainer from '@/components/ui/PageContainer.vue';
import SerifHeading from '@/components/ui/SerifHeading.vue';
import TextAction from '@/components/ui/TextAction.vue';
import TopNavBar from '@/components/TopNavBar.vue';

const route = useRoute();
const router = useRouter();
const contentId = route.params.id as string;

const versions = ref<ContentVersionSnapshot[]>([]);
const loading = ref(true);

const formatDate = (iso: string) => {
    return new Date(iso).toLocaleString();
};

const viewVersion = (v: string) => {
    router.push(`/content/${contentId}/version/${v}`);
};

const compareWithCurrent = (v: string) => {
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
