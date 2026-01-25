<template>
  <div class="flex items-center gap-4">
    <!-- View Switcher -->
    <div class="flex bg-surface-2/50 backdrop-blur rounded-lg p-0.5 border border-border">
        <button 
          v-for="view in viewOptions" 
          :key="view.value"
          @click="store.currentView = view.value"
          class="p-1.5 rounded-md text-text-secondary hover:text-text-primary transition-colors relative"
          :class="{ 'text-primary bg-surface-0 shadow-sm': store.currentView === view.value }"
          :title="view.label"
        >
          <i :class="view.icon" class="text-lg block"></i>
        </button>
    </div>

    <div class="h-4 w-px bg-border"></div>

    <!-- Selection Mode Actions -->
    <template v-if="store.ui.selectionMode">
        <div class="flex items-center gap-2">
            <span class="text-xs font-bold text-text-secondary mr-2">
                {{ store.ui.selectedIds.size }} selected
            </span>
            <button 
                @click="store.ui.selectedIds.size > 0 ? store.batchDelete() : null"
                class="flex items-center gap-2 px-3 py-1.5 bg-red-500/10 text-red-500 rounded-md hover:bg-red-500/20 active:scale-95 transition-all text-xs font-bold uppercase tracking-widest disabled:opacity-50 disabled:cursor-not-allowed"
                :disabled="store.ui.selectedIds.size === 0"
            >
                <i class="ri-delete-bin-line text-lg" />
                <span>Delete</span>
            </button>
            <button 
                 @click="store.toggleSelectionMode(false)"
                 class="px-3 py-1.5 text-text-secondary hover:text-text-primary text-xs font-bold uppercase tracking-widest"
            >
                Cancel
            </button>
        </div>
    </template>

    <!-- Normal Actions -->
    <template v-else>
         <button 
            @click="store.toggleSelectionMode(true)"
            class="p-2 text-text-secondary hover:text-primary transition-colors rounded-md hover:bg-surface-2"
            title="Select"
         >
            <i class="ri-checkbox-multiple-line text-lg"></i>
         </button>

         <button 
            @click="store.openEditor()"
            class="flex items-center gap-2 px-3 py-1.5 bg-ink text-paper rounded-md hover:bg-neutral-800 active:scale-95 transition-all text-xs font-bold uppercase tracking-widest"
         >
            <i class="ri-add-line text-lg" />
            <span>New Memo</span>
         </button>
    </template>
  </div>
</template>

<script setup lang="ts">
import { useMemosStore } from '@/stores/memos';

const store = useMemosStore();

const viewOptions: { value: 'masonry' | 'kanban' | 'calendar' | 'timeline' | 'list', label: string, icon: string }[] = [
  { value: 'masonry', label: 'Gallery', icon: 'ri-layout-masonry-line' },
  { value: 'kanban', label: 'Board', icon: 'ri-kanban-view' },
  { value: 'calendar', label: 'Calendar', icon: 'ri-calendar-line' },
  { value: 'timeline', label: 'Timeline', icon: 'ri-time-line' },
];
</script>
