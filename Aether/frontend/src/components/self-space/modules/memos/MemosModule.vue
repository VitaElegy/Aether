<template>
  <div class="h-full flex flex-col bg-surface-0">
    <!-- Main Content -->
    <div class="flex-1 overflow-hidden relative">
      <div v-if="store.loading && store.memos.length === 0" class="absolute inset-0 flex items-center justify-center bg-surface-0/50 z-10">
        <div class="i-ph-spinner animate-spin text-2xl text-primary" />
      </div>

      <Transition mode="out-in" name="fade-slide">
        <KeepAlive>
          <component 
            :is="store.currentView === 'masonry' ? MemoMasonry : store.currentView === 'kanban' ? MemoKanban : MemoCalendar"
            :memos="store.currentView === 'masonry' || store.currentView === 'calendar' ? store.filteredMemos : []"
            :columns="store.currentView === 'kanban' ? store.kanbanColumns : {}"
            @open="(m: Memo) => store.openEditor(m)"
            @delete="handleDelete"
            @toggle-pin="handleTogglePin"
            @move="handleMove"
            @create="(date: Date) => { store.ui.isCreating = true; store.ui.showEditor = true; initialDate = date; }"
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
      @close="store.closeEditor(); initialDate = undefined;"
      @save="handleSave"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useMemosStore, type Memo, type CreateMemoPayload, type UpdateMemoPayload } from '@/stores/memos';
import MemoMasonry from './MemoMasonry.vue';
import MemoKanban from './MemoKanban.vue';
import MemoEditor from './MemoEditor.vue';
import MemoCalendar from './MemoCalendar.vue';

const store = useMemosStore();
const initialDate = ref<Date | undefined>(undefined);

onMounted(() => {
  store.fetchMemos();
});

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
