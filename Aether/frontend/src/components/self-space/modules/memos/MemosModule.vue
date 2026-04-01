<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { useMemosStore } from '@/stores/memos';
import { useNavigationStore } from '@/stores/navigation';
import { storeToRefs } from 'pinia';
import MemoMasonry from './MemoMasonry.vue';
import MemoKanban from './MemoKanban.vue';
import MemoTimeline from './MemoTimeline.vue';
import MemoCalendar from './MemoCalendar.vue';
import MemoStream from './stream/MemoStream.vue';
import SmartDock from './smart-dock/SmartDock.vue';
import ComposeBar from './input/ComposeBar.vue';
import MemoEditor from './MemoEditor.vue';
import BacklinksPanel from './BacklinksPanel.vue';
import PortabilityDialog from './PortabilityDialog.vue';
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
    store.fetchSavedViews();
    navStore.setCustomRight(true);
});

onUnmounted(() => {
    navStore.setCustomRight(false);
});

function handleOpenEditor() {
    store.openEditor();
}

async function handleEditorSave(payload: any) {
    if (ui.value.isCreating) {
        await store.createMemo(payload);
    } else if (ui.value.editingMemo) {
        await store.updateMemo(ui.value.editingMemo.id, payload);
    }
    store.closeEditor();
}
</script>

<template>
    <div class="h-full flex overflow-hidden bg-white">
        <!-- Left: Smart Dock -->
        <SmartDock
            :isCollapsed="isDockCollapsed"
            @toggle-collapse="toggleDock"
            class="shrink-0 z-20"
        />

        <!-- Center: Main Stream / View -->
        <div class="flex-1 flex flex-col min-w-0 relative">
            <!-- Content Area -->
            <div class="flex-1 overflow-hidden relative bg-white flex flex-col">
                <div class="flex-1 overflow-y-auto custom-scrollbar p-0">
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
                        @update-date="(id: string, date: string) => store.updateMemo(id, { due_at: date })"
                    />
                </div>
            </div>

            <!-- Bottom: Compose Bar -->
            <div class="shrink-0 z-10 relative">
                <ComposeBar @expand="handleOpenEditor" />
            </div>
        </div>

        <!-- Right: Backlinks Panel (MEMO-05) -->
        <BacklinksPanel />

        <!-- Overlays -->

        <!-- Editor Modal -->
        <MemoEditor
            v-if="ui.showEditor"
            :memo="ui.editingMemo"
            :isNew="ui.isCreating"
            @close="store.closeEditor"
            @save="handleEditorSave"
        />

        <!-- Portability Dialog (MEMO-07) -->
        <PortabilityDialog />
    </div>
</template>
