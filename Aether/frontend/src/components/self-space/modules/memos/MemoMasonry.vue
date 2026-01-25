<template>
  <div class="h-full overflow-y-auto p-4 custom-scrollbar">
    <div v-if="memos.length === 0" class="flex flex-col items-center justify-center h-64 text-text-tertiary">
      <i class="ri-edit-line text-4xl mb-4 text-primary/30" />
      <p class="text-sm">No memos yet.</p>
    </div>

    <!-- Masonry via CSS Columns -->
    <div class="columns-1 sm:columns-2 md:columns-3 lg:columns-4 xl:columns-5 gap-4 space-y-4">
      <div v-for="memo in memos" :key="memo.id" class="break-inside-avoid mb-4">
         <MemoCard 
            :memo="memo" 
            @click="$emit('open', memo)"
            @delete="$emit('delete', memo.id)"
            @pin="$emit('toggle-pin', memo)"
         />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { Memo } from '@/stores/memos';
import MemoCard from './MemoCard.vue';

defineProps<{
  memos: Memo[];
}>();

defineEmits(['open', 'delete', 'toggle-pin', 'create']);
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
