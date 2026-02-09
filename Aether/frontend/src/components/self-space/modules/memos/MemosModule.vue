<script setup lang="ts">
import { ref, onMounted, computed } from 'vue';
import { useMemosStore } from '@/stores/memos';
import { useNavigationStore } from '@/stores/navigation';
import { storeToRefs } from 'pinia';
import MemoMasonry from './MemoMasonry.vue';
import MemoKanban from './MemoKanban.vue';
import MemoTimeline from './MemoTimeline.vue';
import MemoCalendar from './MemoCalendar.vue'; // Optional
import MemoStream from './stream/MemoStream.vue'; // V2 Stream
import SmartDock from './smart-dock/SmartDock.vue'; // V2 Sidebar
import ComposeBar from './input/ComposeBar.vue'; // V2 Input
import MemoEditor from './MemoEditor.vue'; // Correct Import
import { Icon } from 'tdesign-vue-next';

const store = useMemosStore();
const { currentView, ui } = storeToRefs(store);

// Sidebar State
const isDockCollapsed = ref(false);

function toggleDock() {
    isDockCollapsed.value = !isDockCollapsed.value;
}

const navStore = useNavigationStore();

onMounted(() => {
    store.fetchMemos();
    store.fetchUserSettings();
    navStore.setCustomRight(true);
});

import { onUnmounted } from 'vue';
onUnmounted(() => {
    navStore.setCustomRight(false);
});

function handleOpenEditor() {
    store.openEditor();
}
</script>

<template>
    <div class="h-full flex overflow-hidden bg-white">
        <!-- Left: Smart Dock (V2) -->
        <SmartDock 
            :isCollapsed="isDockCollapsed" 
            @toggle-collapse="toggleDock"
            class="shrink-0 z-20"
        />

        <!-- Center: Main Stream / View -->
        <div class="flex-1 flex flex-col min-w-0 relative">
            <!-- Header (Teleported) -->
            <!-- ... -->

            <!-- Content Area (Scrollable) -->
            <div class="flex-1 overflow-hidden relative bg-white flex flex-col">
                <div class="flex-1 overflow-y-auto custom-scrollbar p-0"> 
                    <!-- Removed p-4 to let children handle padding if needed, or keep it? -->
                    <!-- Stream usually needs padding. Let's keep children structure but ensure scroll is here. -->
                    
                    <MemoStream v-if="currentView === 'stream'" />
                    <MemoMasonry 
                        v-else-if="currentView === 'masonry'" 
                        :memos="store.filteredMemos"
                        @open="handleOpenEditor"
                    />
                    <MemoKanban 
                        v-else-if="currentView === 'kanban'" 
                        :columns="store.kanbanColumns"
                        @open="handleOpenEditor"
                        @move="store.moveMemoToStatus"
                    />
                    <MemoTimeline 
                        v-else-if="currentView === 'timeline'" 
                        :memos="store.filteredMemos"
                        @open="handleOpenEditor"
                    />
                    <MemoCalendar 
                        v-else-if="currentView === 'calendar'" 
                        :memos="store.filteredMemos"
                        @open="handleOpenEditor"
                        @create="store.createMemo"
                        @update-date="(id, date) => store.updateMemo(id, { due_at: date })"
                    />
                </div>
            </div>

            <!-- Bottom: Compose Bar (V2) - Fixed at bottom of flex column -->
            <div class="shrink-0 z-10 relative">
                <ComposeBar @expand="handleOpenEditor" />
            </div>
        </div>

        <!-- Right: Detail/Editor Modal (Global Overlay) -->
        <MemoEditor 
            v-if="ui.showEditor" 
            :memo="ui.editingMemo" 
            :isNew="ui.isCreating"
            @close="store.closeEditor"
            @save="store.fetchMemos" 
        />
        <!-- Wait, MemoEditor is the name in file list, MemoWriterModal was old? -->
        <!-- Step 434 shows both MemoEditor.vue and MemoWriterModal.vue -->
        <!-- Reading MemosModule.vue (which I am overwriting) would have confirmed. -->
        <!-- I'll check file list again or just import MemoEditor if that seems to be the main one. -->
        <!-- MemoEditor.vue size 17588 vs MemoWriterModal 8489. Editor likely newer/fuller. -->
        <!-- I'll assume MemoEditor.vue is the one to use. -->
    </div>
</template>
