<template>
  <div class="h-full overflow-x-auto overflow-y-hidden p-6 pb-32 flex gap-6 custom-scrollbar bg-surface-0/50">
    <div 
      v-for="(memos, status) in columns" 
      :key="status"
      class="flex-shrink-0 w-80 flex flex-col bg-surface-1/80 backdrop-blur-md rounded-2xl border border-border/50 h-full max-h-full shadow-sm hover:shadow-md transition-shadow duration-300"
      @dragover.prevent
      @drop="onDrop($event, String(status))"
    >
      <!-- Column Header -->
      <div 
        class="p-4 border-b border-border/50 flex items-center justify-between shrink-0 select-none bg-gradient-to-b from-white/50 to-transparent dark:from-white/5 relative group/header"
        :class="headerBorderClass(String(status))"
      >
        <div class="flex items-center gap-2.5">
           <div class="w-2.5 h-2.5 rounded-full shadow-sm" :class="statusColor(String(status))" />
           <span class="font-bold text-text-primary tracking-wide text-sm">{{ status }}</span>
           <span class="text-[10px] font-bold bg-surface-3/80 px-2 py-0.5 rounded-full text-text-secondary border border-border/50">{{ memos.length }}</span>
        </div>
        
        <div class="flex items-center gap-1">
            <button 
                @click="$emit('create', String(status))"
                class="w-6 h-6 rounded flex items-center justify-center text-text-tertiary hover:bg-surface-3 hover:text-text-primary opacity-0 group-hover/header:opacity-100 transition-all scale-90 active:scale-95"
                title="Add Memo"
            >
                <i class="ri-add-line" />
            </button>
            <div class="i-ph-dots-three text-lg cursor-pointer text-text-tertiary hover:text-text-primary transition-colors" />
        </div>
      </div>

      <!-- Column Body -->
      <div class="flex-1 overflow-y-auto p-2 space-y-1.5 custom-scrollbar relative">
        <TransitionGroup name="kanban-list">
            <div 
                v-for="memo in memos" 
                :key="memo.id"
                draggable="true"
                @dragstart="onDragStart($event, memo)"
                class="cursor-grab active:cursor-grabbing group"
            >
                 <MemoCard 
                    :memo="memo" 
                    @click="$emit('open', memo)"
                    @delete="$emit('delete', memo.id)"
                    @pin="$emit('toggle-pin', memo)"
                    class="hover:shadow-md hover:-translate-y-0.5 transition-all duration-200 border-border/60 hover:border-primary/30"
                 />
            </div>
        </TransitionGroup>
        
        <!-- Empty Placeholder -->
        <div v-if="memos.length === 0" class="h-32 border-2 border-dashed border-border/40 rounded-xl flex flex-col items-center justify-center text-text-tertiary text-xs gap-2 select-none group-hover:border-primary/20 transition-colors">
          <div class="i-ph-tray text-2xl opacity-50" />
          <span>{{ status }} is empty</span>
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

const emit = defineEmits(['open', 'delete', 'move', 'toggle-pin', 'create']);

function statusColor(status: string) {
    switch (status) {
        case 'Todo': return 'bg-zinc-400';
        case 'Doing': return 'bg-blue-500 shadow-blue-500/20';
        case 'Done': return 'bg-emerald-500 shadow-emerald-500/20';
        case 'Archived': return 'bg-purple-500 shadow-purple-500/20';
        default: return 'bg-zinc-400';
    }
}

function headerBorderClass(status: string) {
    // Subtle top accent
    // switch (status) {
    //     case 'Doing': return 'border-t-2 border-t-blue-500/30';
    //     default: return '';
    // }
    return '';
}

function onDragStart(event: DragEvent, memo: Memo) {
    if (event.dataTransfer) {
        event.dataTransfer.dropEffect = 'move';
        event.dataTransfer.effectAllowed = 'move';
        event.dataTransfer.setData('application/json', memo.id);
        // Clean ghost image? Browser default usually OK.
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
.custom-scrollbar::-webkit-scrollbar-track {
    background: transparent;
}

/* Kanban List Transitions */
.kanban-list-move,
.kanban-list-enter-active,
.kanban-list-leave-active {
  transition: all 0.3s cubic-bezier(0.25, 0.8, 0.5, 1);
}

.kanban-list-enter-from,
.kanban-list-leave-to {
  opacity: 0;
  transform: translateY(10px) scale(0.95);
}

.kanban-list-leave-active {
  position: absolute;
  width: 100%; /* Ensure layout doesn't collapse during leave */
}
</style>
