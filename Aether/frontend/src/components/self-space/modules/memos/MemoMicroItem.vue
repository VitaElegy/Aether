<template>
  <div 
    class="relative pl-6 py-2 group cursor-pointer transition-colors"
    @click="$emit('select')"
  >
    <!-- Timeline Vertical Line (Continuing) -->
    <div class="absolute left-[3.5px] top-0 bottom-0 w-px bg-border/20 group-hover:bg-primary/20 transition-colors"></div>

    <!-- Dot -->
    <div 
        class="absolute left-0 top-1/2 -translate-y-1/2 w-2 h-2 rounded-full border border-border bg-surface-1 z-10 transition-all duration-200 group-hover:scale-125 group-hover:border-primary group-hover:bg-primary"
        :class="statusColor"
    ></div>

    <!-- Content Row -->
    <div class="flex items-baseline gap-3 text-sm">
        <span class="font-mono text-xs text-text-tertiary shrink-0 w-10 text-right">{{ timeStr }}</span>
        
        <div class="flex-1 min-w-0 flex items-center gap-2">
            <span class="font-medium text-text-primary truncate transition-colors group-hover:text-primary">
                {{ memo.title || memo.content }}
            </span>
            
            <!-- Tags (optional) -->
             <div v-if="memo.tags && memo.tags.length > 0" class="flex shrink-0 gap-1 opacity-0 group-hover:opacity-100 transition-opacity duration-200">
                <span v-for="tag in memo.tags.slice(0, 2)" :key="tag" class="text-[9px] px-1.5 py-px rounded-full bg-surface-3 text-text-secondary">#{{ tag }}</span>
             </div>
        </div>

        <!-- Priority Icon -->
        <div v-if="memo.priority === 'P0' || memo.priority === 'P1'" class="shrink-0 text-red-400 text-xs">
            <i class="ri-flag-fill"></i>
        </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import type { Memo } from '@/stores/memos';
import { format } from 'date-fns';

const props = defineProps<{
  memo: Memo;
}>();

defineEmits(['select']);

const timeStr = computed(() => format(new Date(props.memo.created_at), 'HH:mm'));

const statusColor = computed(() => {
    switch(props.memo.status) {
        case 'Done': return 'bg-green-500 border-green-500';
        case 'Doing': return 'bg-blue-500 border-blue-500 ring-2 ring-blue-500/20';
        default: return '';
    }
});
</script>
