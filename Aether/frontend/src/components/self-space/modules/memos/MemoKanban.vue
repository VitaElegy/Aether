<template>
  <div class="h-full overflow-x-auto overflow-y-hidden p-4 flex gap-4 custom-scrollbar">
    <div 
      v-for="(memos, status) in columns" 
      :key="status"
      class="flex-shrink-0 w-72 flex flex-col bg-surface-1 rounded-xl border border-border h-full max-h-full"
      @dragover.prevent
      @drop="onDrop($event, String(status))"
    >
      <!-- Column Header -->
      <div class="p-3 border-b border-border flex items-center justify-between shrink-0 font-medium text-text-secondary select-none">
        <div class="flex items-center gap-2">
           <div class="w-2 h-2 rounded-full" :class="statusColor(String(status))" />
           <span>{{ status }}</span>
           <span class="text-xs bg-surface-3 px-1.5 rounded-full text-text-tertiary">{{ memos.length }}</span>
        </div>
        <div class="i-ph-dots-three text-lg cursor-pointer hover:text-text-primary" />
      </div>

      <!-- Column Body -->
      <div class="flex-1 overflow-y-auto p-2 space-y-2 custom-scrollbar">
        <div 
            v-for="memo in memos" 
            :key="memo.id"
            draggable="true"
            @dragstart="onDragStart($event, memo)"
            class="cursor-grab active:cursor-grabbing"
        >
             <MemoCard 
                :memo="memo" 
                @click="$emit('open', memo)"
                @delete="$emit('delete', memo.id)"
                @pin="$emit('toggle-pin', memo)"
                class="hover:ring-1 hover:ring-border/50"
             />
        </div>
        
        <!-- Empty Placeholder for Drop Target visibility -->
        <div v-if="memos.length === 0" class="h-24 border-2 border-dashed border-border rounded-lg flex items-center justify-center text-text-tertiary text-xs">
          Drop here
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { Memo } from '@/stores/memos';
import MemoCard from './MemoCard.vue';

defineProps<{
  columns: Record<string, Memo[]>;
}>();

const emit = defineEmits(['open', 'delete', 'move', 'toggle-pin']);

function statusColor(status: string) {
    switch (status) {
        case 'Todo': return 'bg-gray-400';
        case 'Doing': return 'bg-blue-500';
        case 'Done': return 'bg-green-500';
        case 'Archived': return 'bg-purple-500';
        default: return 'bg-gray-400';
    }
}

function onDragStart(event: DragEvent, memo: Memo) {
    if (event.dataTransfer) {
        event.dataTransfer.dropEffect = 'move';
        event.dataTransfer.effectAllowed = 'move';
        event.dataTransfer.setData('application/json', memo.id);
    }
}

function onDrop(event: DragEvent, status: string) {
    if (event.dataTransfer) {
        const id = event.dataTransfer.getData('application/json');
        if (id) {
            emit('move', id, status);
        }
    }
}
</script>

<style scoped>
.custom-scrollbar::-webkit-scrollbar {
    width: 4px;
    height: 6px;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
    background: var(--color-border);
    border-radius: 4px;
}
</style>
