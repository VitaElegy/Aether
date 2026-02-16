<script setup lang="ts">
import { computed } from 'vue';
import { useMemosStore } from '@/stores/memos';
import { format, isToday, isYesterday, isSameDay } from 'date-fns';
import MemoBubble from './MemoBubble.vue';

const store = useMemosStore();

// Group Memos by Date
// Expected structure: [ { date: '2023-10-01', memos: [...] }, ... ]
const groupedMemos = computed(() => {
    const groups: { dateLabel: string, date: Date, memos: any[] }[] = [];
    
    if (!store.filteredMemos.length) return [];

    let currentGroup: { dateLabel: string, date: Date, memos: any[] } | null = null;

    store.filteredMemos.forEach(memo => {
        const d = new Date(memo.created_at);
        
        let label = format(d, 'yyyy-MM-dd');
        if (isToday(d)) label = 'Today';
        else if (isYesterday(d)) label = 'Yesterday';
        else label = format(d, 'MMMM d, yyyy'); // "October 1, 2023"

        if (!currentGroup || currentGroup.dateLabel !== label) {
            currentGroup = {
                dateLabel: label,
                date: d,
                memos: []
            };
            groups.push(currentGroup);
        }
        
        currentGroup.memos.push(memo);
    });

    return groups;
});

function handleMemoClick(memo: any) {
    store.openEditor(memo);
}

function handleTagClick(tag: string) {
    store.filterTags = [tag];
}
</script>

<template>
    <div class="flex-1 h-full overflow-y-auto custom-scrollbar p-6">
        <div class="max-w-4xl mx-auto space-y-8">
            <template v-for="group in groupedMemos" :key="group.dateLabel">
                <!-- Date Header -->
                <div class="flex items-center gap-4">
                    <div class="h-px bg-gray-200 flex-1"></div>
                    <div class="text-xs font-medium text-gray-400 uppercase tracking-widest">
                        {{ group.dateLabel }}
                    </div>
                    <div class="h-px bg-gray-200 flex-1"></div>
                </div>

                <!-- Memos List -->
                <div class="space-y-4">
                    <MemoBubble 
                        v-for="memo in group.memos" 
                        :key="memo.id"
                        :memo="memo"
                        @click="handleMemoClick"
                        @tag-click="handleTagClick"
                    />
                </div>
            </template>

            <div v-if="groupedMemos.length === 0" class="text-center py-20 text-gray-400">
                <div v-if="store.loading">Loading stream...</div>
                <div v-else>No thoughts found. Start typing below.</div>
            </div>
        </div>
    </div>
</template>
