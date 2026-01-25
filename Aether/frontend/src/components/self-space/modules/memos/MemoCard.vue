<template>
    <div 
    class="memo-card relative group flex flex-col rounded-xl transition-all duration-300 hover:translate-y-[-2px] cursor-pointer select-none overflow-hidden border isolate h-full"
    :class="[
      selected ? 'ring-2 ring-primary ring-offset-2 dark:ring-offset-black' : '',
      themeClasses
    ]"
    @click="$emit('click')"
  >
    <!-- Header: Title & Pin -->
    <div class="px-5 pt-5 pb-2 flex items-start justify-between relative z-10">
      <h3 v-if="memo.title" class="font-bold text-text-primary text-base leading-tight line-clamp-2 tracking-tight">
        {{ memo.title }}
      </h3>
      <div v-else class="h-6"></div> <!-- Spacer for formatting -->
      
      <div v-if="memo.is_pinned" class="text-amber-500 shrink-0 ml-2">
        <i class="ri-pushpin-fill text-sm" />
      </div>
    </div>

    <!-- Body: Content Preview -->
    <div class="px-5 pb-4 text-[15px] leading-relaxed text-text-secondary/90 font-medium relative z-10">
      <div class="line-clamp-6 whitespace-pre-wrap decoration-clone font-sans">
         {{ previewContent }}
      </div>
    </div>

    <!-- Footer: Tags & Meta -->
    <div class="mt-auto px-5 pb-5 flex items-center justify-between text-xs pt-3 border-t border-black/5 dark:border-white/5 mx-1 relative z-10">
      <div class="flex gap-1.5 overflow-hidden flex-wrap max-h-6">
        <span 
            v-for="tag in memo.tags.slice(0, 3)" 
            :key="tag" 
            class="px-2 py-0.5 rounded-full font-semibold tracking-wide"
            :class="getTagColor(tag)"
        >
          #{{ tag }}
        </span>
      </div>
      <div class="text-text-tertiary font-medium font-mono text-[10px] uppercase tracking-wider opacity-70">
        {{ formatDate(memo.created_at) }}
      </div>
    </div>
    
    <!-- Hover Actions (Floating) -->
    <div class="absolute top-2 right-2 opacity-0 group-hover:opacity-100 transition-all duration-200 flex flex-col gap-1 translate-x-2 group-hover:translate-x-0 z-20">
       <div class="flex gap-1">
            <button 
                class="w-7 h-7 rounded-lg bg-white/50 dark:bg-zinc-800/50 shadow-sm hover:bg-white text-text-secondary hover:text-primary transition-all flex items-center justify-center backdrop-blur-sm"
                @click.stop="$emit('pin')"
                title="Pin"
            >
                <i :class="memo.is_pinned ? 'ri-pushpin-fill' : 'ri-pushpin-line'" />
            </button>
            <button 
                class="w-7 h-7 rounded-lg bg-white/50 dark:bg-zinc-800/50 shadow-sm hover:bg-white text-text-secondary hover:text-red-500 transition-all flex items-center justify-center backdrop-blur-sm"
                @click.stop="$emit('delete')"
                title="Delete"
            >
                <i class="ri-delete-bin-line" />
            </button>
       </div>
    </div>
    <!-- Selection Overlay -->
    <div 
        v-if="selectable || selected"
        class="absolute inset-0 z-30 transition-colors pointer-events-none flex items-start justify-end p-2"
        :class="selected ? 'bg-primary/5' : 'bg-transparent group-hover:bg-black/5'"
    >
        <div 
            class="w-6 h-6 rounded-full border-2 flex items-center justify-center transition-all duration-200 shadow-sm"
            :class="selected ? 'bg-primary border-primary scale-110' : 'bg-surface-0 border-border/60 opacity-0 group-hover:opacity-100 scale-90 hover:scale-100'"
        >
            <i class="ri-check-line text-white text-sm" v-if="selected"></i>
        </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import type { Memo } from '@/stores/memos';
import { formatDistanceToNow } from 'date-fns';
import { getTagColor } from '@/utils/colors';

const props = defineProps<{
  memo: Memo;
  selected?: boolean;
  selectable?: boolean;
}>();

defineEmits(['click', 'delete', 'pin']);

const previewContent = computed(() => {
    return props.memo.content || 'Empty Note';
});

const themeClasses = computed(() => {
    // Elegant Minimalism: White/Dark card with colored left border
    switch (props.memo.color) {
        case 'Yellow': return 'border-l-[4px] border-l-amber-300 dark:border-l-amber-500 bg-amber-50/30 dark:bg-amber-950/20 hover:border-black/5 dark:hover:border-white/10 shadow-sm hover:shadow-md'; 
        case 'Red': return 'border-l-[4px] border-l-red-300 dark:border-l-red-500 bg-red-50/30 dark:bg-red-950/20 hover:border-black/5 dark:hover:border-white/10 shadow-sm hover:shadow-md';
        case 'Green': return 'border-l-[4px] border-l-emerald-300 dark:border-l-emerald-500 bg-emerald-50/30 dark:bg-emerald-950/20 hover:border-black/5 dark:hover:border-white/10 shadow-sm hover:shadow-md';
        case 'Blue': return 'border-l-[4px] border-l-blue-300 dark:border-l-blue-500 bg-blue-50/30 dark:bg-blue-950/20 hover:border-black/5 dark:hover:border-white/10 shadow-sm hover:shadow-md';
        case 'Purple': return 'border-l-[4px] border-l-purple-300 dark:border-l-purple-500 bg-purple-50/30 dark:bg-purple-950/20 hover:border-black/5 dark:hover:border-white/10 shadow-sm hover:shadow-md';
        case 'Gray': return 'border-l-[4px] border-l-zinc-300 dark:border-l-zinc-500 bg-zinc-50/30 dark:bg-zinc-800/30 hover:border-black/5 dark:hover:border-white/10 shadow-sm hover:shadow-md';
        default: return 'border-l-[4px] border-l-transparent bg-white dark:bg-zinc-900 border-border/40';
    }
});

function formatDate(date: string) {
    try {
        return formatDistanceToNow(new Date(date), { addSuffix: true }).replace('about ', '');
    } catch {
        return '';
    }
}
</script>

<style scoped>
.line-clamp-6 {
  display: -webkit-box;
  -webkit-line-clamp: 6;
  -webkit-box-orient: vertical;
  overflow: hidden;
}
</style>
