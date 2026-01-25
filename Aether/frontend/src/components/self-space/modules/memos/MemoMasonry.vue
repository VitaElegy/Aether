<template>
  <div ref="containerRef" class="h-full overflow-y-auto p-4 pb-32 custom-scrollbar">
    <div v-if="memos.length === 0" class="flex flex-col items-center justify-center h-64 text-text-tertiary">
      <i class="ri-edit-line text-4xl mb-4 text-primary/30" />
      <p class="text-sm">No memos yet.</p>
    </div>

    <!-- Computed Masonry Layout -->
    <div v-else class="flex gap-4 items-start pb-10">
      <div 
        v-for="(column, colIndex) in columns" 
        :key="colIndex" 
        class="flex-1 flex flex-col gap-4 min-w-0"
      >
        <div v-for="memo in column" :key="memo.id" class="w-full">
           <MemoCard 
              :memo="memo" 
              :selected="store.ui.selectedIds.has(memo.id)"
              :selectable="store.ui.selectionMode"
              @click="handleCardClick(memo)"
              @delete="$emit('delete', memo.id)"
              @pin="$emit('toggle-pin', memo)"
              @pin-click="$emit('toggle-pin', memo)" 
           />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { useElementSize } from '@vueuse/core';
import { useMemosStore, type Memo } from '@/stores/memos';
import MemoCard from './MemoCard.vue';

const store = useMemosStore();

const props = defineProps<{
  memos: Memo[];
}>();

const emit = defineEmits(['open', 'delete', 'toggle-pin', 'create']);

function handleCardClick(memo: Memo) {
    if (store.ui.selectionMode) {
        store.toggleSelection(memo.id);
    } else {
        emit('open', memo);
    }
}

const containerRef = ref<HTMLElement | null>(null);
const { width } = useElementSize(containerRef);

// Configurable breakpoints
const columnCount = computed(() => {
  const w = width.value;
  if (w < 640) return 1;    // Mobile
  if (w < 1024) return 2;   // Tablet
  if (w < 1280) return 3;   // Desktop
  if (w < 1536) return 3;   // Wide
  return 4;                 // Ultra Wide
});

// Distribute memos into columns (Left-to-Right Round Robin)
const columns = computed(() => {
  const cols: Memo[][] = Array.from({ length: columnCount.value }, () => []);
  props.memos.forEach((memo, index) => {
    const colIndex = index % columnCount.value;
    cols[colIndex].push(memo);
  });
  return cols;
});
</script>

<style scoped>
.custom-scrollbar::-webkit-scrollbar {
  width: 6px;
}
.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
  background-color: var(--color-border);
  border-radius: 3px;
}
</style>
