<template>
  <div class="h-full overflow-y-auto px-4 py-8 custom-scrollbar">
    <div class="max-w-2xl mx-auto pb-32">
        <div class="space-y-4">
            <div v-for="(group, dateLabel) in groupedMemos" :key="dateLabel">
                <!-- Day Node Accordion Header -->
                <MemoDayNode 
                    :dateLabel="dateLabel"
                    :memos="group"
                    :isExpanded="expandedDate === dateLabel"
                    @toggle="toggleDate(dateLabel)"
                />

                <!-- Expanded Content (Micro Timeline) -->
                <div 
                    v-if="expandedDate === dateLabel"
                    class="mt-2 ml-4 pl-4 border-l border-border/40 space-y-1 animate-in slide-in-from-top-2 duration-300"
                >
                     <MemoMicroItem 
                        v-for="memo in group" 
                        :key="memo.id"
                        :memo="memo"
                        @select="$emit('open', memo)"
                     />
                </div>
            </div>
        </div>

        <!-- Empty State -->
        <div v-if="memos.length === 0" class="text-center py-20 opacity-50">
            <i class="ri-inbox-line text-4xl mb-2 block"></i>
            <span class="text-sm">Time flows, memories wait.</span>
        </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import type { Memo } from '@/stores/memos';
import { format, isToday, isYesterday, isSameYear } from 'date-fns';
import MemoDayNode from './MemoDayNode.vue';
import MemoMicroItem from './MemoMicroItem.vue';

const props = defineProps<{
  memos: Memo[];
}>();

defineEmits(['open']);

// Accordion State
const expandedDate = ref<string | null>(null);

// Grouping Logic (Same as before but key is crucial for identification)
const groupedMemos = computed(() => {
    const groups: Record<string, Memo[]> = {};
    
    // Sort desc first
    // Use stable sort
    const sorted = [...props.memos].sort((a, b) => 
        new Date(b.created_at).getTime() - new Date(a.created_at).getTime()
    );

    sorted.forEach(memo => {
        const date = new Date(memo.created_at);
        let label = '';
        
        // Use a consistent, comparable key for the label initially, display format handled by sub-component?
        // Actually MemoDayNode takes a label. Let's make the key user-friendly but unique per day.
        // Or better: Use ISO date YYYY-MM-DD as key for sorting, and MemoDayNode formats it?
        // But the previous implementation used formatted strings as keys.
        // Let's stick to the formatted label for display simplicity in the v-for loop, assuming unique days.
        // To ensure strict ordering of keys in the object, we shouldn't rely on Object key order.
        // Wait, groupedMemos returns an Object. v-for iteration order on objects is generally insertion order in modern JS engines, but Map is safer.
        // However, for Vue reactivity, standard object is fine if we construct it in order? 
        // No, constructing in order doesn't guarantee v-for order.
        // Let's just use the string format logic we had, it seemed to work.
        
        if (isToday(date)) label = 'Today';
        else if (isYesterday(date)) label = 'Yesterday';
        else if (isSameYear(date, new Date())) label = format(date, 'MMMM d');
        else label = format(date, 'MMM d, yyyy');

        if (!groups[label]) groups[label] = [];
        groups[label].push(memo);
    });

    return groups;
});

// Auto-expand Today on mount or data change?
// Let's do it on mount.
onMounted(() => {
    // If 'Today' exists in groups, expand it.
    if (groupedMemos.value['Today']) {
        expandedDate.value = 'Today';
    } else {
        // Expand the first available group (most recent)
        const keys = Object.keys(groupedMemos.value);
        if (keys.length > 0) expandedDate.value = keys[0];
    }
});

function toggleDate(label: string) {
    if (expandedDate.value === label) {
        expandedDate.value = null; // Collapse if same
    } else {
        expandedDate.value = label; // Expand new
    }
}
</script>

<style scoped>
.custom-scrollbar::-webkit-scrollbar {
  width: 4px;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
  background-color: var(--color-border);
  border-radius: 4px;
}
</style>
