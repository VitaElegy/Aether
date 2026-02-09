<script setup lang="ts">
import { ref, reactive, onBeforeUnmount, watch, onMounted, nextTick } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import { MessagePlugin } from 'tdesign-vue-next';
import { useDebounceFn } from '@vueuse/core';
import TopNavBar from '@/components/TopNavBar.vue';
import CollaboratorModal from './CollaboratorModal.vue';
import UniversalEditor from '@/components/editor/UniversalEditor.vue';
import EditorMetadataPanel from '@/components/editor/EditorMetadataPanel.vue';
import { usePreferencesStore } from '@/stores/preferences';
import { knowledgeApi, type KnowledgeBase } from '@/api/knowledge';
import { useContent } from '@/composables/useContent';
import { draftApi } from '@/api/draft';

const router = useRouter();
const route = useRoute();
const knowledgeBases = ref<KnowledgeBase[]>([]);
const { article, load, save, create, isSaving } = useContent();

// State
const isLiveMode = ref(true);
const isMetadataPanelOpen = ref(false);
const toc = ref<any[]>([]);
const draftId = ref<string | null>(route.params.id as string || null);
const autoSaveEnabled = ref(true);
const timestamps = ref<{ created: string | null; updated: string | null }>({ created: null, updated: null });
const isRestoring = ref(true);
const isSyncing = ref(false);

const prefStore = usePreferencesStore();
const showCommitModal = ref(false);
const showCollaboratorModal = ref(false);
const commitMessage = ref('');

const form = reactive({
  title: '',
  knowledge_base_id: null as string | null,
  parent_id: null as string | null,
  body: '',
  category: '',
  tags: [] as string[],
  visibility: 'Public',
  status: 'Draft' 
});

const editorRef = ref<InstanceType<typeof UniversalEditor> | null>(null);

// Sync form with article when loaded
watch(() => article.value, async (newArticle) => {
    isSyncing.value = true;
    if (newArticle) {
        form.title = newArticle.title || form.title;
        if (isRestoring.value) {
             form.body = (typeof newArticle.body === 'string' ? newArticle.body : form.body);
             if (editorRef.value && isLiveMode.value) {
                 await editorRef.value.load(form.body);
             }
        }
        
        form.tags = newArticle.tags || [];
        form.category = newArticle.category || '';
        form.visibility = newArticle.visibility as string || 'Public';
        form.status = newArticle.status || 'Draft';
        form.knowledge_base_id = newArticle.knowledge_base_id;
        form.parent_id = newArticle.parent_id;
        
        timestamps.value = {
            created: new Date(newArticle.created_at).toISOString(),
            updated: new Date(newArticle.updated_at).toISOString()
        };
    }
    await nextTick();
    isSyncing.value = false;
});

const handleTocUpdate = (newToc: any[]) => {
    toc.value = newToc.map(h => ({
        ...h,
        isVisible: h.level <= 3
    }));
};

// Initialization
onMounted(async () => {
  if (draftId.value) {
      await load(draftId.value);
  } else {
      timestamps.value = { created: new Date().toISOString(), updated: new Date().toISOString() };
      const intendedKbId = route.query.knowledge_base_id as string | undefined;
      const intendedParentId = route.query.parent_id as string | undefined;
      if (intendedKbId) form.knowledge_base_id = intendedKbId;
      if (intendedParentId) form.parent_id = intendedParentId;
  }

  // Check Drafts
  try {
      const serverDraft = await draftApi.get();
      if (serverDraft) {
          const isNewEntry = !draftId.value;
          const matchesCurrent = draftId.value && serverDraft.target_article_id === draftId.value;
          let shouldRestore = false;

          if (isNewEntry && !serverDraft.target_article_id) shouldRestore = true;
          else if (matchesCurrent) shouldRestore = true;
          else if (serverDraft.target_article_id) MessagePlugin.warning('You have an unsaved draft for another article.', 5000);

          if (shouldRestore) {
              isRestoring.value = true;
              form.title = serverDraft.title || form.title;
              form.body = serverDraft.body || form.body;
              form.tags = serverDraft.tags || form.tags;
              form.category = serverDraft.category || form.category;
              form.knowledge_base_id = serverDraft.knowledge_base_id || form.knowledge_base_id;
              
              if (editorRef.value && isLiveMode.value) {
                   await editorRef.value.load(form.body);
              }
              MessagePlugin.info('Restored unsaved draft.', 3000);
          }
      }
  } catch (e) {
      console.error("Failed to check drafts", e);
  }

  try {
      knowledgeBases.value = await knowledgeApi.list();
  } catch (e) {
      console.error("Failed to fetch KBs", e);
  } finally {
      setTimeout(() => { isRestoring.value = false; }, 500);
  }
});

const saveDraft = async () => {
  if (!form.title && !form.body) return;
  try {
       await draftApi.save({
          target_article_id: draftId.value || null,
          title: form.title,
          body: form.body,
          tags: form.tags,
          category: form.category,
          knowledge_base_id: form.knowledge_base_id,
          parent_id: form.parent_id
      });
      timestamps.value.updated = new Date().toISOString();
  } catch (e) {
      console.error("Auto-save failed", e);
  }
};

const debouncedAutoSave = useDebounceFn(() => {
  if (autoSaveEnabled.value) saveDraft();
}, 2000);

watch(form, () => {
  if (!isRestoring.value && autoSaveEnabled.value) {
     debouncedAutoSave();
  }
}, { deep: true });

const toggleMode = () => {
  isLiveMode.value = !isLiveMode.value;
  if (isLiveMode.value) {
      nextTick(() => {
          editorRef.value?.load(form.body);
      });
  }
};

const isPublishing = ref(false);
const onPublishClick = () => {
  commitMessage.value = prefStore.defaultCommitMessage;
  showCommitModal.value = true;
};

const executePublish = async () => {
    if (isPublishing.value) return;
    showCommitModal.value = false;
    if (debouncedAutoSave && typeof (debouncedAutoSave as any).cancel === 'function') (debouncedAutoSave as any).cancel();
    if (isSaving.value) while (isSaving.value) await new Promise(resolve => setTimeout(resolve, 100));

    isPublishing.value = true;
    if (isLiveMode.value && editorRef.value) form.body = editorRef.value.getValue();

    try {
        const currentId = draftId.value || article.value?.id;
        const payload = {
              title: form.title || 'Untitled',
              body: form.body,
              tags: form.tags,
              category: form.category || '',
              visibility: form.visibility as any,
              status: 'Published' as any,
              knowledge_base_id: form.knowledge_base_id || null,
              parent_id: form.parent_id || null,
              reason: commitMessage.value,
        };

        if (!currentId) {
            const newId = await create(payload);
            draftId.value = newId;
        } else {
            if (!draftId.value) draftId.value = currentId;
            await save(payload);
        }

        await draftApi.delete();
        MessagePlugin.success('Published.');
        router.push('/');
    } catch (err: any) {
         if (err.response && err.response.status === 409) {
             MessagePlugin.warning('Article with this title already exists.', 3000);
         } else {
             MessagePlugin.error('Failed to publish.');
         }
    } finally {
        isPublishing.value = false;
    }
};

const goBack = () => {
  if (window.history.state && window.history.state.back) {
    router.back();
  } else if (draftId.value && form.knowledge_base_id) {
    router.push('/space'); 
  } else {
    router.push('/');
  }
};

const scrollToHeading = (pos: number) => {
    editorRef.value?.scrollToPosition(pos);
};

onBeforeUnmount(() => {
    if (debouncedAutoSave && typeof (debouncedAutoSave as any).cancel === 'function') {
        (debouncedAutoSave as any).cancel();
    }
});
</script>

<template>
  <div class="h-screen w-full flex flex-col bg-paper overflow-hidden">
    <!-- Header -->
    <TopNavBar class="relative flex-none z-30">
       <template #left>
          <div class="flex items-center gap-6">
             <button @click="goBack" class="text-neutral-400 hover:text-ink transition-colors flex items-center gap-2" title="Go Back">
                <i class="ri-arrow-left-line text-xl"></i>
             </button>
             <div class="h-4 w-px bg-neutral-200/50"></div>
             <button v-if="draftId" @click="router.push(`/content/${draftId}/history`)" class="flex items-center gap-2 group cursor-pointer hover:text-ink text-neutral-400 transition-colors" title="View Version History">
                <i class="ri-history-line text-lg"></i>
                <span class="text-[10px] uppercase tracking-widest group-hover:text-neutral-600 hidden sm:inline">History</span>
             </button>
          </div>
       </template>

       <template #center>
          <div class="flex items-center gap-3">
             <span class="text-xs font-mono uppercase tracking-widest text-neutral-400 block">
               Editor / {{ form.status === 'Published' ? 'Editing Published' : (draftId ? 'Editing Draft' : 'New Entry') }}
             </span>
             <span v-if="isSaving" class="text-[10px] text-neutral-300 animate-pulse uppercase tracking-widest">Saving...</span>
             <span v-else-if="draftId" class="text-[10px] text-neutral-300 uppercase tracking-widest">
                {{ form.status === 'Published' ? 'Published' : 'Draft Saved' }}
             </span>
          </div>
       </template>

       <template #right>
          <div class="flex items-center gap-6">
             <!-- Auto Save Toggle -->
             <div class="flex items-center gap-2 mr-2 group cursor-pointer" @click="autoSaveEnabled = !autoSaveEnabled">
                <div class="w-2 h-2 rounded-full transition-colors" :class="autoSaveEnabled ? 'bg-green-500' : 'bg-neutral-300'"></div>
                <span class="text-[10px] uppercase tracking-widest text-neutral-400 group-hover:text-neutral-600">Auto-Save</span>
             </div>

             <!-- Info Toggle -->
             <button 
                @click="isMetadataPanelOpen = !isMetadataPanelOpen"
                class="text-neutral-400 hover:text-ink transition-colors"
                :class="{ 'text-ink': isMetadataPanelOpen }"
                title="Metadata"
             >
                <i class="ri-information-line text-lg"></i>
             </button>

             <div class="flex items-center gap-3">
                <button v-if="draftId" @click="showCollaboratorModal = true" class="text-xs font-bold uppercase tracking-widest text-neutral-400 hover:text-ink transition-colors mr-4">
                  Collaborators
                </button>

                <span class="text-[10px] font-bold uppercase tracking-widest text-neutral-400 transition-colors" :class="{ 'text-ink': isLiveMode }">Live</span>
                <button
                  @click="toggleMode"
                  class="w-10 h-5 rounded-full bg-ash relative transition-colors focus:outline-none"
                  :class="{ 'bg-ink': !isLiveMode }"
                >
                   <div class="absolute top-1 left-1 w-3 h-3 bg-white rounded-full transition-transform duration-200 shadow-sm" :class="{ 'translate-x-5': !isLiveMode }"></div>
                </button>
                <span class="text-[10px] font-bold uppercase tracking-widest text-neutral-400 transition-colors" :class="{ 'text-ink': !isLiveMode }">Raw</span>
             </div>

             <div class="h-4 w-px bg-neutral-200"></div>

             <button @click="onPublishClick" :disabled="isPublishing" class="bg-ink text-paper px-6 py-2 text-xs font-bold uppercase tracking-widest hover:bg-neutral-800 transition-colors disabled:opacity-50 disabled:cursor-not-allowed text-center min-w-[100px]">
               {{ isPublishing ? '...' : 'Publish' }}
             </button>
          </div>
       </template>
    </TopNavBar>

    <!-- Main Body: Relative container for flex layout -->
    <div class="flex-1 flex overflow-hidden relative">
      <!-- Left Sidebar: TOC -->
      <aside class="w-64 flex-shrink-0 hidden xl:flex flex-col border-r border-neutral-100 bg-paper p-8 overflow-y-auto custom-scrollbar z-10">
        <div class="text-[10px] font-bold uppercase tracking-widest text-neutral-400 mb-6">Table of Contents</div>
        <nav v-if="toc.length > 0" class="flex flex-col gap-1 transition-all">
           <template v-for="(item, idx) in toc" :key="idx">
             <a
              v-show="item.isVisible"
              href="#"
              @click.prevent="scrollToHeading(item.pos)"
              class="block text-xs transition-colors leading-tight py-1 border-l-2 pl-3 text-neutral-400 hover:text-neutral-600 border-transparent"
              :class="{
                  'ml-0': item.level === 1,
                  'ml-2': item.level === 2,
                  'ml-4': item.level === 3,
                  'ml-6': item.level === 4
              }"
            >
              {{ item.text }}
            </a>
           </template>
        </nav>
        <div v-else class="text-xs text-neutral-300 italic font-mono">
          Headings will appear here...
        </div>
      </aside>

      <!-- Main Editor Container -->
      <main class="flex-1 overflow-y-auto relative custom-scrollbar bg-paper">
         <div class="max-w-3xl mx-auto px-8 py-12 flex flex-col min-h-full">
            <input
              v-model="form.title"
              placeholder="Untitled Entry"
              class="text-5xl font-bold tracking-tight mb-8 placeholder:text-neutral-200 focus:outline-none w-full bg-transparent border-none p-0 flex-shrink-0"
            />

            <div class="flex-1 relative">
              <UniversalEditor
                  v-if="isLiveMode"
                  ref="editorRef"
                  v-model="form.body"
                  type="markdown"
                  @change="() => {}"
                  @update:toc="handleTocUpdate"
              />
              <textarea
                v-else
                v-model="form.body"
                placeholder="# Start typing..."
                class="w-full h-full min-h-[500px] resize-none text-base leading-relaxed text-neutral-700 dark:text-neutral-200 placeholder:text-neutral-200 focus:outline-none bg-transparent font-mono pb-32"
              ></textarea>
            </div>
         </div>
      </main>

      <!-- Floating Metadata Panel -->
      <EditorMetadataPanel 
          v-model="isMetadataPanelOpen"
          :form="form"
          :timestamps="timestamps"
          :knowledge-bases="knowledgeBases"
      />
    </div>

    <!-- Commit Modal -->
    <Transition
      enter-active-class="transition duration-200 ease-out"
      enter-from-class="opacity-0 scale-95"
      enter-to-class="opacity-100 scale-100"
      leave-active-class="transition duration-150 ease-in"
      leave-from-class="opacity-100 scale-100"
      leave-to-class="opacity-0 scale-95"
    >
      <div v-if="showCommitModal" class="fixed inset-0 z-50 flex items-center justify-center p-4">
        <!-- Backdrop -->
        <div class="absolute inset-0 bg-white/80 dark:bg-black/80 backdrop-blur-sm" @click="showCommitModal = false"></div>

        <!-- Modal Content -->
        <div class="relative bg-paper w-full max-w-md border border-neutral-100 shadow-2xl p-8 flex flex-col gap-6 animate-in fade-in zoom-in-95 duration-200">
           <div>
              <h3 class="text-xl font-bold text-ink">Save Changes</h3>
              <p class="text-xs text-neutral-400 mt-1 uppercase tracking-widest">Describes this version in history</p>
           </div>

           <textarea
             v-model="commitMessage"
             class="w-full h-32 bg-ash/30 p-4 text-sm font-mono text-ink focus:outline-none focus:ring-1 focus:ring-ink resize-none placeholder:text-neutral-400"
             placeholder="What did you change?"
             autofocus
           ></textarea>

           <div class="flex justify-end gap-4">
              <button @click="showCommitModal = false" class="text-xs font-bold uppercase tracking-widest text-neutral-400 hover:text-ink transition-colors">
                Cancel
              </button>
              <button @click="executePublish" class="bg-ink text-paper px-6 py-2 text-xs font-bold uppercase tracking-widest hover:bg-neutral-800 transition-colors">
                Commit & Publish
              </button>
           </div>
        </div>
      </div>
    </Transition>

    <CollaboratorModal 
      v-if="draftId"
      :visible="showCollaboratorModal" 
      :article-id="draftId"
      @close="showCollaboratorModal = false"
    />
  </div>
</template>

<style>
/* Scoped styles mostly handled by Tailwind */
.custom-scrollbar::-webkit-scrollbar { width: 4px; }
.custom-scrollbar::-webkit-scrollbar-thumb { background: #e5e5e5; border-radius: 2px; }
.custom-scrollbar::-webkit-scrollbar-track { background: transparent; }
</style>
