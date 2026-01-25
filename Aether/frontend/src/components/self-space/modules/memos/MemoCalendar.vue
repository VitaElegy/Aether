<template>
  <div class="h-full flex flex-col p-4 overflow-hidden">
    <!-- Header: Month Navigation -->
    <div class="flex items-center justify-between mb-4 shrink-0">
      <div class="flex items-center gap-4">
        <h2 class="text-xl font-bold text-text-primary font-serif">
          {{ format(currentDate, 'MMMM yyyy') }}
        </h2>
        <div class="flex items-center bg-surface-2 rounded-lg p-0.5">
            <button @click="prevMonth" class="p-1 hover:bg-surface-3 rounded-md transition-colors">
                <i class="ri-arrow-left-s-line text-lg" />
            </button>
            <button @click="jumpToToday" class="px-3 text-xs font-bold uppercase tracking-widest text-text-tertiary hover:text-text-primary transition-colors">
                Today
            </button>
            <button @click="nextMonth" class="p-1 hover:bg-surface-3 rounded-md transition-colors">
                <i class="ri-arrow-right-s-line text-lg" />
            </button>
        </div>
      </div>
      
      <!-- Optional: View Mode (Month/Week) - Keeping simple Month for now -->
    </div>

    <!-- Calendar Grid -->
    <div class="flex-1 grid grid-cols-7 grid-rows-[auto_1fr] border border-border rounded-xl bg-surface-1 overflow-hidden shadow-sm">
      <!-- Weekday Headers -->
      <div v-for="day in weekDays" :key="day" class="py-2 text-center text-[10px] font-bold uppercase tracking-widest text-text-tertiary border-b border-border bg-surface-2/50">
        {{ day }}
      </div>

      <!-- Days -->
      <!-- We need 6 rows to cover all possible month spans (max 42 days) or just flex -->
      <!-- Subgrid for days: grid-rows-6 (approx) -->
      <div class="col-span-7 grid grid-cols-7 grid-rows-6 h-full">
         <div 
            v-for="(day, idx) in calendarDays" 
            :key="idx"
            class="border-r border-b border-border relative group flex flex-col overflow-hidden transition-colors"
            :class="[
                day.isCurrentMonth ? 'bg-surface-1' : 'bg-surface-2/30 text-text-tertiary',
                isToday(day.date) ? 'bg-primary/5' : ''
            ]"
            @dragover.prevent
            @drop="onDrop($event, day.date)"
         >
            <!-- Day Number -->
            <div class="p-2 flex items-center justify-between shrink-0">
                <span 
                    class="text-xs font-medium w-6 h-6 flex items-center justify-center rounded-full"
                    :class="isToday(day.date) ? 'bg-primary text-white' : ''"
                >
                    {{ format(day.date, 'd') }}
                </span>
                
                <!-- Add Button (Hover) -->
                <button 
                    @click="$emit('create', day.date)"
                    class="opacity-0 group-hover:opacity-100 p-1 hover:bg-surface-3 rounded transition-opacity text-text-tertiary hover:text-primary"
                >
                    <i class="ri-add-line" />
                </button>
            </div>

            <!-- Memos List -->
            <div class="flex-1 overflow-y-auto px-1 pb-1 space-y-1 custom-scrollbar">
                <div 
                    v-for="memo in getMemosForDate(day.date)" 
                    :key="memo.id"
                    @click.stop="$emit('open', memo)"
                    draggable="true"
                    @dragstart="onDragStart($event, memo)"
                    class="text-[10px] p-1.5 rounded border mb-0.5 cursor-pointer shadow-sm hover:shadow-md transition-all truncate bg-surface-0 border-border hover:border-primary/50"
                    :class="[ statusColorClass(memo) ]"
                >
                    <span v-if="memo.due_at" class="mr-1 text-primary">●</span>
                    {{ memo.title || 'Untitled' }}
                </div>
            </div>
         </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { 
    format, 
    startOfMonth, 
    endOfMonth, 
    startOfWeek, 
    endOfWeek, 
    eachDayOfInterval, 
    isSameMonth, 
    isSameDay, 
    addMonths, 
    subMonths 
} from 'date-fns';
import type { Memo } from '@/stores/memos';

const props = defineProps<{
    memos: Memo[];
}>();

const emit = defineEmits(['open', 'create', 'update-date']);

const currentDate = ref(new Date());

const weekDays = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'];

// Calendar Logic
const calendarDays = computed(() => {
    const start = startOfWeek(startOfMonth(currentDate.value));
    const end = endOfWeek(endOfMonth(currentDate.value));
    
    // Ensure always 6 weeks (42 days) for consistent grid height
    // Actually `eachDayOfInterval` just gives the range. 
    // If we want fixed 6 rows, we might need to pad.
    // Let's generate range mostly, and let grid layout handle it.
    // CSS grid-rows-6 requires exactly 42 items? 
    // Actually, `startOfWeek` and `endOfWeek` usually give 4, 5, or 6 weeks.
    // To keep UI stable, let's just use what `eachDayOfInterval` gives, but force row height to be dynamic `1fr`.
    
    const days = eachDayOfInterval({ start, end });
    return days.map(date => ({
        date,
        isCurrentMonth: isSameMonth(date, currentDate.value)
    }));
});

function isToday(date: Date) {
    return isSameDay(date, new Date());
}

function prevMonth() {
    currentDate.value = subMonths(currentDate.value, 1);
}

function nextMonth() {
    currentDate.value = addMonths(currentDate.value, 1);
}

function jumpToToday() {
    currentDate.value = new Date();
}

function getMemosForDate(date: Date) {
    return props.memos.filter(memo => {
        // Preference: check due_at first, then created_at?
        // Usually Calendar View is for Due Dates (GTD).
        // If no due_at, maybe don't show? Or show on created_at?
        // Let's stick to: "Show if due_at is on this day".
        // If we want a journal view, we use created_at.
        // Let's support both logic via props later or just mix them?
        // For now: Due Date is priority.
        if (memo.due_at) {
            return isSameDay(new Date(memo.due_at), date);
        }
        return false; 
    });
}

function statusColorClass(memo: Memo) {
    if (memo.status === 'Done') return 'opacity-50 line-through decoration-text-tertiary text-text-tertiary';
    // Priority colors?
    return 'text-text-primary';
}

// Drag and Drop (Reschedule)
function onDragStart(event: DragEvent, memo: Memo) {
    if (event.dataTransfer) {
        event.dataTransfer.dropEffect = 'move';
        event.dataTransfer.effectAllowed = 'move';
        event.dataTransfer.setData('application/json', memo.id);
    }
}

function onDrop(event: DragEvent, date: Date) {
    if (event.dataTransfer) {
        const id = event.dataTransfer.getData('application/json');
        if (id) {
            // Update Memo Due Date
            // We need to keep the time if it exists? Or reset to EOD?
            // Simple: Set date at 9:00 AM or just Date part if Backend supports it.
            // Backend is ISO string. Let's keep it simple.
            const newDate = new Date(date);
            newDate.setHours(9, 0, 0, 0); // Default 9 AM
            emit('update-date', id, newDate.toISOString());
        }
    }
}
</script>

<style scoped>
.custom-scrollbar::-webkit-scrollbar {
  width: 4px;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
  background-color: var(--color-border);
  border-radius: 2px;
}
</style>
