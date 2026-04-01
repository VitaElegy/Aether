<script setup lang="ts">
import { computed, ref, onMounted } from 'vue';
import { useMemosStore, type SavedView, type ViewMode } from '@/stores/memos';
import { Icon } from 'tdesign-vue-next';
import {
    DashboardIcon,
    FormatVerticalAlignCenterIcon,
    HashtagIcon,
    PinIcon,
    PinFilledIcon
} from 'tdesign-icons-vue-next';

const store = useMemosStore();

const props = defineProps<{
    isCollapsed?: boolean
}>();

const emit = defineEmits(['toggle-collapse']);

// State
const showAllTags = ref(true);
const showSavedViews = ref(true);
const showChannels = ref(true);
const showReviewQueues = ref(true);
const newViewName = ref('');
const showNewViewInput = ref(false);

// Computed
const pinnedTags = computed(() => store.pinnedTags);
const allTags = computed(() => store.uniqueTags);

// View mode options
const viewModes: { mode: ViewMode; icon: string; label: string }[] = [
    { mode: 'stream', icon: 'view-list', label: 'Stream' },
    { mode: 'masonry', icon: 'view-module', label: 'Cards' },
    { mode: 'kanban', icon: 'view-column', label: 'Kanban' },
    { mode: 'timeline', icon: 'chart-line-data', label: 'Timeline' },
    { mode: 'calendar', icon: 'calendar', label: 'Calendar' },
];

// Actions
function selectTag(tag: string) {
    store.filterTags = [tag];
    store.searchQuery = '';
    store.activeQueue = null;
}

function selectUntagged() {
    store.filterTags = ['__untagged__'];
    store.searchQuery = '';
}

function selectChannel(ch: string) {
    store.filterChannel = ch;
    store.filterTags = [];
    store.searchQuery = '';
    store.activeQueue = null;
}

function handleCreateTag() {
    const tag = prompt("Enter new tag name:");
    if (tag && tag.trim()) {
        const cleanTag = tag.trim();
        if (!store.pinnedTags.includes(cleanTag)) {
            store.togglePinTag(cleanTag);
        }
        selectTag(cleanTag);
    }
}

function clearFilter() {
    store.clearAllFilters();
}

async function togglePin(tag: string, event: Event) {
    event.stopPropagation();
    await store.togglePinTag(tag);
}

function setViewMode(mode: ViewMode) {
    store.currentView = mode;
    store.saveUserSettings();
}

// MEMO-03: Saved Views
async function saveCurrentView() {
    if (!newViewName.value.trim()) return;
    await store.saveCurrentAsView(newViewName.value.trim());
    newViewName.value = '';
    showNewViewInput.value = false;
}

function applyView(view: SavedView) {
    store.applySavedView(view);
}

async function deleteView(viewId: string, e: Event) {
    e.stopPropagation();
    await store.deleteSavedView(viewId);
}

// MEMO-06: Review Queues
function selectQueue(queue: 'due_today' | 'overdue' | 'stale') {
    store.setActiveQueue(queue);
}

// Toggle selection mode
function toggleSelectionMode() {
    store.toggleSelectionMode(!store.ui.selectionMode);
}

onMounted(() => {
    store.fetchUserSettings();
    store.fetchSavedViews();
});
</script>

<template>
    <div
        class="h-full bg-gray-50 border-r border-gray-200 flex flex-col transition-all duration-300 relative group"
        :class="isCollapsed ? 'w-14' : 'w-64'"
    >
        <!-- Header / Toggle -->
        <div class="p-3 flex items-center justify-between shrink-0">
            <div v-if="!isCollapsed" class="font-bold text-gray-700 flex items-center gap-2 text-sm">
                <DashboardIcon />
                <span>Memos</span>
            </div>
            <button
                @click="$emit('toggle-collapse')"
                class="p-1 rounded hover:bg-gray-200 text-gray-500"
            >
                <FormatVerticalAlignCenterIcon class="rotate-90" />
            </button>
        </div>

        <div class="flex-1 overflow-y-auto custom-scrollbar px-2 space-y-4 pb-4">

            <!-- View Mode Switcher -->
            <div v-if="!isCollapsed" class="flex gap-1 px-1">
                <button
                    v-for="vm in viewModes"
                    :key="vm.mode"
                    @click="setViewMode(vm.mode)"
                    class="flex-1 p-1.5 rounded-md text-center transition-colors"
                    :class="store.currentView === vm.mode
                        ? 'bg-white shadow-sm text-blue-600'
                        : 'text-gray-400 hover:text-gray-600 hover:bg-gray-100'"
                    :title="vm.label"
                >
                    <Icon :name="vm.icon" size="14px" />
                </button>
            </div>

            <!-- Quick Actions Bar -->
            <div v-if="!isCollapsed" class="flex gap-1 px-1">
                <button
                    @click="toggleSelectionMode"
                    class="flex-1 px-2 py-1.5 rounded-md text-[10px] font-semibold uppercase tracking-wider transition-colors"
                    :class="store.ui.selectionMode
                        ? 'bg-blue-100 text-blue-600'
                        : 'text-gray-400 hover:bg-gray-100'"
                >
                    {{ store.ui.selectionMode ? 'Exit Select' : 'Select' }}
                </button>
                <button
                    @click="store.ui.showExportDialog = true"
                    class="flex-1 px-2 py-1.5 rounded-md text-[10px] font-semibold uppercase tracking-wider text-gray-400 hover:bg-gray-100 transition-colors"
                >
                    Export
                </button>
            </div>

            <!-- Pinned / All Memos -->
            <div class="space-y-0.5">
                <div v-if="!isCollapsed" class="px-3 text-[10px] font-semibold text-gray-400 uppercase tracking-wider mb-1.5">
                    Navigate
                </div>

                <button
                    @click="clearFilter"
                    class="w-full flex items-center gap-3 px-3 py-2 rounded-lg text-sm transition-colors"
                    :class="store.filterTags.length === 0 && !store.filterChannel && !store.activeQueue && !store.activeViewId
                        ? 'bg-blue-50 text-blue-700 font-medium'
                        : 'text-gray-600 hover:bg-gray-100'"
                >
                    <Icon name="home" size="16px" />
                    <span v-if="!isCollapsed">All Memos</span>
                    <span v-if="!isCollapsed" class="ml-auto text-xs text-gray-400">{{ store.memos.length }}</span>
                </button>

                <!-- Pinned Tags -->
                <button
                    v-for="tag in pinnedTags"
                    :key="tag"
                    @click="selectTag(tag)"
                    class="w-full flex items-center justify-between px-3 py-1.5 rounded-lg text-sm transition-colors group/item"
                    :class="store.filterTags.includes(tag)
                        ? 'bg-white shadow-sm ring-1 ring-gray-200 text-gray-900 font-medium'
                        : 'text-gray-600 hover:bg-gray-100'"
                >
                    <div class="flex items-center gap-2 overflow-hidden">
                        <HashtagIcon class="shrink-0 text-gray-400" size="14px" />
                        <span v-if="!isCollapsed" class="truncate">{{ tag }}</span>
                    </div>
                    <div v-if="!isCollapsed" @click="togglePin(tag, $event)" class="opacity-0 group-hover/item:opacity-100 cursor-pointer text-gray-400 hover:text-red-500">
                        <Icon name="close" size="12px" />
                    </div>
                </button>
            </div>

            <!-- MEMO-03: Saved Views -->
            <div v-if="!isCollapsed && (store.savedViews.length > 0 || showNewViewInput)" class="space-y-0.5">
                <div
                    @click="showSavedViews = !showSavedViews"
                    class="px-3 text-[10px] font-semibold text-gray-400 uppercase tracking-wider mb-1.5 cursor-pointer hover:text-gray-600 flex justify-between items-center"
                >
                    <span>Saved Views</span>
                    <span>{{ showSavedViews ? '−' : '+' }}</span>
                </div>

                <div v-if="showSavedViews" class="space-y-0.5">
                    <button
                        v-for="view in store.savedViews"
                        :key="view.id"
                        @click="applyView(view)"
                        class="w-full flex items-center justify-between px-3 py-1.5 rounded-lg text-sm transition-colors group/item"
                        :class="store.activeViewId === view.id
                            ? 'bg-white shadow-sm ring-1 ring-gray-200 text-gray-900 font-medium'
                            : 'text-gray-600 hover:bg-gray-100'"
                    >
                        <div class="flex items-center gap-2 overflow-hidden">
                            <Icon :name="view.icon || 'bookmark'" size="14px" class="text-gray-400 shrink-0" />
                            <span class="truncate">{{ view.name }}</span>
                        </div>
                        <button @click="deleteView(view.id, $event)" class="opacity-0 group-hover/item:opacity-100 text-gray-400 hover:text-red-500">
                            <Icon name="close" size="10px" />
                        </button>
                    </button>

                    <!-- Save Current View Button -->
                    <div v-if="showNewViewInput" class="px-2">
                        <div class="flex gap-1">
                            <input
                                v-model="newViewName"
                                @keydown.enter="saveCurrentView"
                                placeholder="View name..."
                                class="flex-1 text-xs border border-gray-200 rounded px-2 py-1 focus:ring-1 focus:ring-blue-300 focus:outline-none"
                            />
                            <button @click="saveCurrentView" class="px-2 py-1 bg-blue-500 text-white rounded text-xs hover:bg-blue-600">Save</button>
                        </div>
                    </div>
                    <button
                        v-else
                        @click="showNewViewInput = true"
                        class="w-full flex items-center gap-2 px-3 py-1.5 rounded-lg text-xs text-blue-500 hover:bg-blue-50 transition-colors"
                    >
                        <Icon name="add" size="12px" />
                        <span>Save current view</span>
                    </button>
                </div>
            </div>

            <!-- Channels -->
            <div v-if="!isCollapsed && store.channels.length > 0" class="space-y-0.5">
                <div
                    @click="showChannels = !showChannels"
                    class="px-3 text-[10px] font-semibold text-gray-400 uppercase tracking-wider mb-1.5 cursor-pointer hover:text-gray-600 flex justify-between items-center"
                >
                    <span>Channels</span>
                    <span>{{ showChannels ? '−' : '+' }}</span>
                </div>
                <div v-if="showChannels" class="space-y-0.5">
                    <button
                        v-for="ch in store.channels"
                        :key="ch"
                        @click="selectChannel(ch)"
                        class="w-full flex items-center gap-2 px-3 py-1.5 rounded-lg text-sm transition-colors"
                        :class="store.filterChannel === ch
                            ? 'bg-indigo-50 text-indigo-700 font-medium'
                            : 'text-gray-600 hover:bg-gray-100'"
                    >
                        <Icon name="folder" size="14px" class="text-gray-400" />
                        <span class="truncate">{{ ch }}</span>
                    </button>
                </div>
            </div>

            <!-- MEMO-06: Review Queues -->
            <div v-if="!isCollapsed" class="space-y-0.5">
                <div
                    @click="showReviewQueues = !showReviewQueues"
                    class="px-3 text-[10px] font-semibold text-gray-400 uppercase tracking-wider mb-1.5 cursor-pointer hover:text-gray-600 flex justify-between items-center"
                >
                    <span>Review</span>
                    <span>{{ showReviewQueues ? '−' : '+' }}</span>
                </div>
                <div v-if="showReviewQueues" class="space-y-0.5">
                    <button
                        @click="selectQueue('due_today')"
                        class="w-full flex items-center justify-between px-3 py-1.5 rounded-lg text-sm transition-colors"
                        :class="store.activeQueue === 'due_today' ? 'bg-blue-50 text-blue-700 font-medium' : 'text-gray-600 hover:bg-gray-100'"
                    >
                        <div class="flex items-center gap-2">
                            <Icon name="calendar" size="14px" class="text-blue-400" />
                            <span>Due Today</span>
                        </div>
                        <span v-if="store.reviewCounts.dueToday > 0" class="text-xs font-bold text-blue-500 bg-blue-100 px-1.5 py-0.5 rounded-full min-w-[20px] text-center">
                            {{ store.reviewCounts.dueToday }}
                        </span>
                    </button>

                    <button
                        @click="selectQueue('overdue')"
                        class="w-full flex items-center justify-between px-3 py-1.5 rounded-lg text-sm transition-colors"
                        :class="store.activeQueue === 'overdue' ? 'bg-red-50 text-red-700 font-medium' : 'text-gray-600 hover:bg-gray-100'"
                    >
                        <div class="flex items-center gap-2">
                            <Icon name="error-circle" size="14px" class="text-red-400" />
                            <span>Overdue</span>
                        </div>
                        <span v-if="store.reviewCounts.overdue > 0" class="text-xs font-bold text-red-500 bg-red-100 px-1.5 py-0.5 rounded-full min-w-[20px] text-center">
                            {{ store.reviewCounts.overdue }}
                        </span>
                    </button>

                    <button
                        @click="selectQueue('stale')"
                        class="w-full flex items-center justify-between px-3 py-1.5 rounded-lg text-sm transition-colors"
                        :class="store.activeQueue === 'stale' ? 'bg-amber-50 text-amber-700 font-medium' : 'text-gray-600 hover:bg-gray-100'"
                    >
                        <div class="flex items-center gap-2">
                            <Icon name="time" size="14px" class="text-amber-400" />
                            <span>Stale</span>
                        </div>
                        <span v-if="store.reviewCounts.stale > 0" class="text-xs font-bold text-amber-500 bg-amber-100 px-1.5 py-0.5 rounded-full min-w-[20px] text-center">
                            {{ store.reviewCounts.stale }}
                        </span>
                    </button>
                </div>
            </div>

            <!-- All Tags Tree -->
            <div class="space-y-0.5">
                <div
                    v-if="!isCollapsed"
                    @click="showAllTags = !showAllTags"
                    class="px-3 text-[10px] font-semibold text-gray-400 uppercase tracking-wider mb-1.5 cursor-pointer hover:text-gray-600 flex justify-between"
                >
                    <span>Tags</span>
                    <span>{{ showAllTags ? '−' : '+' }}</span>
                </div>

                <div v-if="!isCollapsed && showAllTags" class="space-y-0.5">
                    <div class="px-2 mb-2">
                        <button
                            @click="handleCreateTag"
                            class="w-full flex items-center gap-2 px-2 py-1.5 rounded-md text-xs font-medium text-blue-600 bg-blue-50 hover:bg-blue-100 transition-colors border border-blue-200 border-dashed"
                        >
                            <Icon name="add" size="14px" />
                            <span>Create Tag</span>
                        </button>
                    </div>

                    <button
                        @click="selectUntagged"
                        class="w-full flex items-center justify-between px-3 py-1.5 rounded-md text-sm transition-colors"
                        :class="store.filterTags.includes('__untagged__') ? 'bg-gray-200 text-gray-900' : 'text-gray-500 hover:bg-gray-100 italic'"
                    >
                        <span class="truncate">Untagged</span>
                        <span class="text-xs text-gray-400">{{ store.untaggedCount }}</span>
                    </button>

                    <button
                        v-for="{ name, count } in allTags"
                        :key="name"
                        @click="selectTag(name)"
                        class="w-full flex items-center justify-between px-3 py-1.5 rounded-md text-sm transition-colors group/item"
                        :class="store.filterTags.includes(name) ? 'bg-gray-200 text-gray-900' : 'text-gray-600 hover:bg-gray-100'"
                    >
                        <div class="flex items-center gap-2 overflow-hidden">
                            <span class="truncate"># {{ name }}</span>
                        </div>
                        <div class="flex items-center gap-2">
                            <span class="text-xs text-gray-400 group-hover/item:hidden">{{ count }}</span>
                            <div
                                @click="togglePin(name, $event)"
                                class="hidden group-hover/item:block cursor-pointer"
                                :class="pinnedTags.includes(name) ? 'text-blue-500' : 'text-gray-300 hover:text-gray-600'"
                            >
                                <PinFilledIcon v-if="pinnedTags.includes(name)" size="14px" />
                                <PinIcon v-else size="14px" />
                            </div>
                        </div>
                    </button>

                    <div v-if="allTags.length === 0" class="px-3 py-4 text-center text-xs text-gray-400">
                        No tags found
                    </div>
                </div>
            </div>
        </div>
    </div>
</template>

<style scoped>
.custom-scrollbar::-webkit-scrollbar { width: 4px; }
.custom-scrollbar::-webkit-scrollbar-track { background: transparent; }
.custom-scrollbar::-webkit-scrollbar-thumb { background: #e5e7eb; border-radius: 4px; }
.custom-scrollbar:hover::-webkit-scrollbar-thumb { background: #d1d5db; }
</style>
