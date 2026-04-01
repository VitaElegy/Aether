<script setup lang="ts">
import { computed } from 'vue';
import { useMemosStore } from '@/stores/memos';
import { format, isToday, isYesterday } from 'date-fns';
import MemoBubble from './MemoBubble.vue';

const store = useMemosStore();

// Group Memos by Date
const groupedMemos = computed(() => {
    const groups: { dateLabel: string, date: Date, memos: any[] }[] = [];

    if (!store.filteredMemos.length) return [];

    let currentGroup: { dateLabel: string, date: Date, memos: any[] } | null = null;

    store.filteredMemos.forEach(memo => {
        const d = new Date(memo.created_at);

        let label = format(d, 'yyyy-MM-dd');
        if (isToday(d)) label = 'Today';
        else if (isYesterday(d)) label = 'Yesterday';
        else label = format(d, 'MMMM d, yyyy');

        if (!currentGroup || currentGroup.dateLabel !== label) {
            currentGroup = { dateLabel: label, date: d, memos: [] };
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

async function handleQuickAction(payload: { id: string; action: string }) {
    if (payload.action === 'pin') {
        const memo = store.memos.find(m => m.id === payload.id);
        await store.quickAction(payload.id, memo?.is_pinned ? 'unpin' : 'pin');
    } else {
        await store.quickAction(payload.id, payload.action);
    }
}

// Active queue label
const queueLabel = computed(() => {
    switch (store.activeQueue) {
        case 'due_today': return '📅 Due Today';
        case 'overdue': return '🔴 Overdue';
        case 'stale': return '💤 Stale';
        case 'unresolved': return '⏰ Snoozed (Unresolved)';
        default: return null;
    }
});
</script>

<template>
    <div class="flex-1 h-full overflow-y-auto custom-scrollbar p-6">
        <div class="max-w-4xl mx-auto space-y-8">
            <!-- Active Queue Banner -->
            <div v-if="queueLabel" class="flex items-center gap-3 bg-amber-50 border border-amber-200 rounded-lg px-4 py-2 text-sm text-amber-700">
                <span class="font-medium">{{ queueLabel }}</span>
                <span class="text-amber-500">({{ store.filteredMemos.length }} items)</span>
                <button @click="store.setActiveQueue(null)" class="ml-auto text-xs text-amber-600 hover:text-amber-800 underline">Clear</button>
            </div>

            <!-- Selection Mode Bar -->
            <div v-if="store.ui.selectionMode" class="sticky top-0 z-10 bg-blue-50 border border-blue-200 rounded-lg px-4 py-2 flex items-center gap-3 text-sm">
                <span class="text-blue-700 font-medium">{{ store.ui.selectedIds.size }} selected</span>
                <button @click="store.selectAll()" class="text-blue-600 hover:underline text-xs">Select All</button>
                <button @click="store.deselectAll()" class="text-blue-600 hover:underline text-xs">Deselect</button>
                <div class="ml-auto flex items-center gap-2">
                    <button @click="store.bulkUpdate(Array.from(store.ui.selectedIds), { status: 'Archived' })" class="px-2 py-1 bg-gray-100 rounded text-xs hover:bg-gray-200">Archive</button>
                    <button @click="store.bulkDelete(Array.from(store.ui.selectedIds))" class="px-2 py-1 bg-red-50 text-red-600 rounded text-xs hover:bg-red-100">Delete</button>
                    <button @click="store.mergeMemos(Array.from(store.ui.selectedIds))" v-if="store.ui.selectedIds.size >= 2" class="px-2 py-1 bg-purple-50 text-purple-600 rounded text-xs hover:bg-purple-100">Merge</button>
                    <button @click="store.toggleSelectionMode(false)" class="px-2 py-1 text-gray-500 text-xs hover:text-gray-700">Cancel</button>
                </div>
            </div>

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
                        @quick-action="handleQuickAction"
                    />
                </div>
            </template>

            <div v-if="groupedMemos.length === 0" class="text-center py-20 text-gray-400">
                <div v-if="store.loading" class="flex items-center justify-center gap-2">
                    <div class="w-4 h-4 border-2 border-gray-300 border-t-blue-500 rounded-full animate-spin"></div>
                    Loading stream...
                </div>
                <div v-else class="space-y-2">
                    <div class="text-lg">✨</div>
                    <div>No thoughts found. Start typing below.</div>
                </div>
            </div>
        </div>
    </div>
</template>
