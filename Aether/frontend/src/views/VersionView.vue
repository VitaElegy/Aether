<template>
  <div class="min-h-screen bg-[var(--bg-color-page)] flex flex-col">
    <TopNavBar>
      <template #left>
         <TextAction size="sm" @click="router.push(`/content/${originalId}/history`)">
            ← return to history
         </TextAction>
      </template>

      <template #center>
        <div class="flex flex-col leading-tight items-center">
          <span class="font-serif font-bold text-lg text-ink">Version {{ versionId }}</span>
          <MonoRef>Ref: {{ originalId.slice(0, 8) }}</MonoRef>
        </div>
      </template>

      <template #right>
          <TextAction
              size="sm"
              :active="isDiffMode"
              @click="toggleDiff"
          >
              {{ isDiffMode ? 'reading changes' : 'read content' }}
          </TextAction>
      </template>
    </TopNavBar>

    <div class="flex-1 w-full max-w-5xl mx-auto p-6 md:p-12">
      <t-loading v-if="loading" />

      <!-- Unified Content Card -->
      <div v-else class="relative">
        <t-card
          :title="cardTitle"
          class="min-h-[60vh] transition-all duration-300 shadow-sm border-gray-100 overflow-hidden w-full bg-paper"
          :class="{'ring-1 ring-emerald-50': isDiffMode}"
          :bordered="false"
        >
          <Transition name="fade" mode="out-in">
            <!-- View: Diff -->
            <div v-if="isDiffMode" key="diff" class="w-full">
              <DiffViewer :changes="diffContent" :empty-message="diffEmptyMessage" />
            </div>

            <!-- View: Content Reader -->
            <div v-else key="content" class="w-full">
              <div class="prose max-w-none w-full p-8 font-serif text-gray-800 leading-relaxed" v-html="renderMarkdown(contentData.body)"></div>
              <div class="meta border-t border-gray-50 mx-8 mt-4 pt-6 pb-4 text-[10px] font-mono text-gray-300 text-right">
                  Snapshot ID: {{ contentData.id }}
              </div>
            </div>
          </Transition>
        </t-card>
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
import TopNavBar from '@/components/TopNavBar.vue';

// UI Components
import PageContainer from '@/components/ui/PageContainer.vue';
import SerifHeading from '@/components/ui/SerifHeading.vue';
import MonoRef from '@/components/ui/MonoRef.vue';
import TextAction from '@/components/ui/TextAction.vue';

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

const cardTitle = computed(() => {
    if (isDiffMode.value) return 'Review Changes';
    return contentData.value.title || 'Untitled';
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

/* Smooth Fade/Slide Transition */
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.25s ease, transform 0.25s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
  transform: translateY(4px);
}
</style>
