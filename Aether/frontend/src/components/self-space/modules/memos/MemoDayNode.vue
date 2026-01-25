<template>
  <div 
    class="group cursor-pointer rounded-xl bg-surface-1 hover:bg-surface-2 border border-black/5 dark:border-white/5 transition-all duration-200 overflow-hidden"
    :class="{ 'ring-2 ring-primary/10 shadow-sm': isExpanded }"
    @click="$emit('toggle')"
  >
    <!-- Heatmap Bar (Left Edge) -->
    <div class="h-1.5 w-full flex">
       <div 
          v-for="(color, idx) in heatmapColors" 
          :key="idx" 
          class="flex-1 h-full opacity-80"
          :class="color"
       ></div>
    </div>

    <div class="px-5 py-4 flex items-center justify-between">
      <!-- Left: Date info -->
      <div class="flex items-center gap-4">
        <!-- Date Block -->
        <div class="flex flex-col items-center leading-none">
           <span class="text-[10px] uppercase font-bold text-text-tertiary mb-0.5">{{ weekDay }}</span>
           <span class="text-xl font-bold font-serif text-text-primary">{{ dayNumber }}</span>
        </div>

        <div class="h-8 w-px bg-border/40 mx-1"></div>

        <!-- Meta -->
        <div class="flex flex-col gap-1">
           <span class="text-sm font-medium text-text-primary">{{ fullDateLabel }}</span>
           <div class="flex items-center gap-2">
              <span class="text-xs text-text-tertiary">{{ count }} memos</span>
              <!-- Emotion Dots or Priority dots could go here -->
           </div>
        </div>
      </div>

      <!-- Right: Chevron -->
      <div 
        class="w-8 h-8 rounded-full flex items-center justify-center text-text-tertiary transition-transform duration-300"
        :class="{ 'rotate-180 bg-black/5 dark:bg-white/10 text-primary': isExpanded }"
      >
        <i class="ri-arrow-down-s-line text-lg"></i>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { format, isToday, isYesterday } from 'date-fns';
import type { Memo } from '@/stores/memos';

const props = defineProps<{
  dateLabel: string; // "YYYY-MM-DD" or similar key
  memos: Memo[];
  isExpanded: boolean;
}>();

defineEmits(['toggle']);

const dateObj = computed(() => {
    // Attempt to parse the label if it's not "Today"/"Yesterday" or parse from first memo
    if (props.memos.length > 0) {
        return new Date(props.memos[0].created_at);
    }
    return new Date();
});

const weekDay = computed(() => format(dateObj.value, 'EEE'));
const dayNumber = computed(() => format(dateObj.value, 'd'));
const fullDateLabel = computed(() => {
    if (isToday(dateObj.value)) return 'Today';
    if (isYesterday(dateObj.value)) return 'Yesterday';
    return format(dateObj.value, 'MMMM yyyy');
});

const count = computed(() => props.memos.length);

// Generate a simple heatmap bar based on memo colors/priorities
const heatmapColors = computed(() => {
    if (props.memos.length === 0) return ['bg-gray-200 dark:bg-zinc-800'];
    // Take max 10 slices to avoid dom bloat
    const slice = props.memos.slice(0, 20);
    return slice.map(m => {
        switch(m.color) {
            case 'Yellow': return 'bg-yellow-400';
            case 'Red': return 'bg-red-400';
            case 'Green': return 'bg-emerald-400';
            case 'Blue': return 'bg-sky-400';
            case 'Purple': return 'bg-purple-400';
            default: return 'bg-zinc-300 dark:bg-zinc-600';
        }
    });
});
</script>
