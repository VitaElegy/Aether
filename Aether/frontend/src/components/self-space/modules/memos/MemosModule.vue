<template>
  <div class="h-full flex bg-surface-0 overflow-hidden">
    <!-- Teleport Title to Nav Bar -->
    <Teleport to="#nav-center-portal">
         <div v-if="isActive" class="flex items-center gap-3">
             <i class="ri-sticky-note-line text-ink/40 text-lg"></i>
             <span class="text-[10px] font-black uppercase tracking-[0.3em] text-ink/40">
                 Self Space / Memos / {{ currentViewLabel }}
             </span>
         </div>
    </Teleport>

    <!-- Sidebar: Tag Index -->
    <aside class="w-64 flex-shrink-0 border-r border-border/40 bg-surface-1/50 flex flex-col">
        <div class="p-6 pb-4">
            <h2 class="text-sm font-bold text-text-secondary uppercase tracking-wider mb-4 px-2">Index</h2>
            
            <div class="space-y-1">
                <!-- All Notes Filter -->
                <button 
                    @click="store.filterTags = []"
                    class="w-full flex items-center justify-between px-3 py-2 rounded-lg text-sm transition-colors group"
                    :class="store.filterTags.length === 0 ? 'bg-primary/10 text-primary font-medium' : 'text-text-secondary hover:bg-surface-2'"
                >
                    <div class="flex items-center gap-2">
                        <i class="ri-function-line text-lg opacity-70" />
                        <span>All Notes</span>
                    </div>
                    <span class="text-xs opacity-50">{{ store.memos.length }}</span>
                </button>
            </div>

            <div class="mt-6 mb-2 px-2 text-xs font-bold text-text-tertiary uppercase tracking-widest opacity-60">Tags</div>

            <div class="space-y-0.5 overflow-y-auto max-h-[calc(100vh-250px)] custom-scrollbar pr-2">
                <button 
                    v-for="tag in store.uniqueTags" 
                    :key="tag.name"
                    @click="toggleTag(tag.name)"
                    class="w-full flex items-center justify-between px-3 py-1.5 rounded-md text-sm transition-all group relative"
                    :class="store.filterTags.includes(tag.name) ? 'bg-primary/10 text-primary font-medium' : 'text-text-secondary hover:bg-surface-2 hover:pl-4'"
                >
                     <div class="flex items-center gap-2 truncate">
                        <i class="ri-hashtag text-xs opacity-40 shrink-0" />
                        <span class="truncate">{{ tag.name }}</span>
                     </div>
                     <span 
                        class="text-[10px] px-1.5 py-0.5 rounded-full bg-surface-3/50 text-text-tertiary group-hover:bg-surface-3 transition-colors"
                        :class="store.filterTags.includes(tag.name) ? 'bg-primary/20 text-primary' : ''"
                     >
                        {{ tag.count }}
                     </span>
                </button>
                
                <div v-if="store.uniqueTags.length === 0" class="px-3 py-4 text-center">
                    <p class="text-xs text-text-tertiary italic">No tags yet</p>
                </div>
            </div>
        </div>
    </aside>

    <!-- Main Content Area -->
    <div class="flex-1 flex flex-col min-w-0 bg-surface-0 relative">
      <div class="flex-1 overflow-hidden relative">
        <div v-if="store.loading && store.memos.length === 0" class="absolute inset-0 flex items-center justify-center bg-surface-0/50 z-10">
          <div class="i-ph-spinner animate-spin text-2xl text-primary" />
        </div>

        <Transition mode="out-in" name="fade-slide">
          <KeepAlive>
            <component 
              :is="viewComponent"
              :memos="store.currentView !== 'kanban' ? store.filteredMemos : []"
              :columns="store.currentView === 'kanban' ? store.kanbanColumns : {}"
              @open="(m: Memo) => store.openEditor(m)"
              @delete="handleDelete"
              @toggle-pin="handleTogglePin"
              @move="handleMove"
              @create="(date: Date) => { store.ui.isCreating = true; store.ui.showEditor = true; initialDate = date; }"
              @add-column="handleAddColumn"
              @delete-column="handleDeleteColumn"
              @update-date="handleDateUpdate"
            />
          </KeepAlive>
        </Transition>
      </div>

      <!-- Quick Capture / Editor Modal -->
      <MemoEditor 
        v-if="store.ui.showEditor"
        :memo="store.ui.editingMemo"
        :is-new="store.ui.isCreating"
        :initial-date="initialDate"
        :initial-status="initialStatus"
        @close="store.closeEditor(); initialDate = undefined; initialStatus = undefined;"
        @save="handleSave"
      />

      <!-- Immersive Writer Modal (FAB Activated) -->
      <MemoWriterModal 
        v-model="showWriter"
        @save="handleQuickSave"
      />

      <!-- FAB -->
      <button
        @click="openWriter"
        class="absolute bottom-8 right-8 w-14 h-14 bg-zinc-900 dark:bg-zinc-100 text-white dark:text-zinc-900 rounded-full shadow-xl hover:shadow-2xl hover:-translate-y-1 active:scale-95 transition-all flex items-center justify-center z-50 group"
        title="Quick Note"
      >
        <i class="ri-add-line text-3xl transition-transform group-hover:rotate-90"></i>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed, onActivated, onDeactivated, nextTick } from 'vue';
import { useMemosStore, type Memo, type CreateMemoPayload, type UpdateMemoPayload } from '@/stores/memos';
import { useNavigationStore } from '@/stores/navigation';
import MemoMasonry from './MemoMasonry.vue';
import MemoKanban from './MemoKanban.vue';
import MemoEditor from './MemoEditor.vue';
import MemoCalendar from './MemoCalendar.vue';
import MemoTimeline from './MemoTimeline.vue';
import MemoWriterModal from './MemoWriterModal.vue';

const store = useMemosStore();
const navStore = useNavigationStore();
const initialDate = ref<Date | undefined>(undefined);
const initialStatus = ref<string | undefined>(undefined);
const showWriter = ref(false);
const isActive = ref(false); // Track visibility for teleport content

// View Component Map
const viewComponent = computed(() => {
    switch (store.currentView) {
        case 'kanban': return MemoKanban;
        case 'calendar': return MemoCalendar;
        case 'timeline': return MemoTimeline;
        default: return MemoMasonry;
    }
});

const currentViewLabel = computed(() => {
    switch(store.currentView) {
        case 'masonry': return 'Gallery';
        case 'kanban': return 'Board';
        case 'calendar': return 'Calendar';
        case 'timeline': return 'Timeline';
        default: return 'Gallery';
    }
});

onMounted(() => {
  store.fetchMemos();
  // If not using KeepAlive, we'd set active here too.
  // But usually wrapped in KeepAlive, so use onActivated.
  // Wait, MemosModule IS wrapped in KeepAlive in SelfSpaceView.
});

onActivated(() => {
    isActive.value = true;
    navStore.setCustomCenter(true);
});

onDeactivated(() => {
    isActive.value = false;
    navStore.setCustomCenter(false);
});

// Fallback if not kept alive (nav safe)
import { onBeforeUnmount } from 'vue';
onBeforeUnmount(() => {
    navStore.setCustomCenter(false);
});

function openWriter() {
    showWriter.value = true;
}

async function handleQuickSave(payload: CreateMemoPayload) {
    await store.createMemo(payload);
    // showWriter is handled by v-model in the modal component or auto-closed
}

async function handleSave(payload: CreateMemoPayload | UpdateMemoPayload) {
  if (store.ui.isCreating) {
    if (initialDate.value) {
        // Pre-fill date from calendar click
        payload.due_at = initialDate.value.toISOString();
    }
    await store.createMemo(payload as CreateMemoPayload);
  } else if (store.ui.editingMemo) {
    await store.updateMemo(store.ui.editingMemo.id, payload as UpdateMemoPayload);
  }
  store.closeEditor();
  initialDate.value = undefined;
}

async function handleDelete(id: string) {
  if (confirm('Are you sure you want to delete this memo?')) {
    await store.deleteMemo(id);
  }
}

async function handleTogglePin(memo: Memo) {
  await store.updateMemo(memo.id, { is_pinned: !memo.is_pinned });
}

async function handleMove(id: string, status: string) {
    await store.moveMemoToStatus(id, status);
}

async function handleDateUpdate(id: string, dateIso: string) {
    await store.updateMemo(id, { due_at: dateIso });
}

function toggleTag(tag: string) {
    const idx = store.filterTags.indexOf(tag);
    if (idx === -1) {
        store.filterTags.push(tag);
    } else {
        store.filterTags.splice(idx, 1);
    }
}

async function handleAddColumn() {
    const name = prompt('Enter new column name:');
    if (name && name.trim()) {
        const cleanName = name.trim();
        if (!store.workflow.includes(cleanName)) {
            await store.saveWorkflow([...store.workflow, cleanName]);
        }
    }
}

async function handleDeleteColumn(status: string) {
    if (confirm(`Delete column "${status}"? Memos in this column will be moved to the first column visually.`)) {
        const newWorkflow = store.workflow.filter(s => s !== status);
        await store.saveWorkflow(newWorkflow);
    }
}
</script>

<style scoped>
.fade-slide-enter-active,
.fade-slide-leave-active {
  transition: opacity 0.3s ease, transform 0.3s ease;
}

.fade-slide-enter-from {
  opacity: 0;
  transform: translateY(10px);
}

.fade-slide-leave-to {
  opacity: 0;
  transform: translateY(-10px);
}
</style>
