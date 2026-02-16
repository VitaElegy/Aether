<script setup lang="ts">
import { computed, ref, onMounted } from 'vue';
import { useMemosStore } from '@/stores/memos';
import { storeToRefs } from 'pinia'; // Wait, standard pinia usage
import { Icon } from 'tdesign-vue-next';
import { 
    DashboardIcon, 
    FormatVerticalAlignCenterIcon, 
    HashtagIcon, 
    PinIcon, 
    PinFilledIcon 
} from 'tdesign-icons-vue-next';

const store = useMemosStore();
// Access state directly or via storeToRefs if needed for reactivity on primitives?
// store.pinnedTags is a ref in the store, so store.pinnedTags array is reactive.

const props = defineProps<{
    isCollapsed?: boolean
}>();

const emit = defineEmits(['toggle-collapse']);

// State
const showAllTags = ref(true);

// Computed
const pinnedTags = computed(() => store.pinnedTags);
const allTags = computed(() => store.uniqueTags); // { name, count }[]

// Actions
function selectTag(tag: string) {
    // If tag is already selected, deselect? Or just set?
    // Store supports multiple tags filter. Smart Dock usually implies "Switch to this view".
    // Let's replace filter unless Cmd/Ctrl click?
    // For simplicity V2: Click = Exclusive Filter.
    store.filterTags = [tag];
    store.searchQuery = ''; // Clear search
}

function selectUntagged() {
    store.filterTags = ['__untagged__'];
    store.searchQuery = '';
}

function handleCreateTag() {
    // Simple prompt for now, ideally in-place input or modal
    const tag = prompt("Enter new tag name:");
    if (tag && tag.trim()) {
        const cleanTag = tag.trim();
        // Pin it to "create" it conceptually (persists in user settings)
        // If no memos have it, it won't show in allTags, so pinning is necessary
        if (!store.pinnedTags.includes(cleanTag)) {
            store.togglePinTag(cleanTag);
        }
        selectTag(cleanTag);
    }
}

function clearFilter() {
    store.filterTags = [];
}

async function togglePin(tag: string, event: Event) {
    event.stopPropagation();
    await store.togglePinTag(tag);
}

onMounted(() => {
    store.fetchUserSettings();
});
</script>

<template>
    <div 
        class="h-full bg-gray-50 border-r border-gray-200 flex flex-col transition-all duration-300 relative group"
        :class="isCollapsed ? 'w-16' : 'w-64'"
    >
        <!-- Header / Toggle -->
        <div class="p-4 flex items-center justify-between shrink-0">
            <div v-if="!isCollapsed" class="font-bold text-gray-700 flex items-center gap-2">
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

        <div class="flex-1 overflow-y-auto custom-scrollbar px-2 space-y-6">
            
            <!-- Pinned Channels -->
            <div class="space-y-1">
                <div v-if="!isCollapsed" class="px-3 text-xs font-semibold text-gray-400 uppercase tracking-wider mb-2">
                    Pinned
                </div>
                
                <!-- All Memos (Reset) -->
                <button 
                    @click="clearFilter"
                    class="w-full flex items-center gap-3 px-3 py-2 rounded-lg text-sm transition-colors"
                    :class="store.filterTags.length === 0 ? 'bg-blue-100 text-blue-700 font-medium' : 'text-gray-600 hover:bg-gray-100'"
                >
                    <Icon name="border-all" /> <!-- Grid Icon equivalent -->
                    <span v-if="!isCollapsed">All Memos</span>
                </button>

                <!-- Pinned List -->
                <button 
                    v-for="tag in pinnedTags" 
                    :key="tag"
                    @click="selectTag(tag)"
                    class="w-full flex items-center justify-between px-3 py-2 rounded-lg text-sm transition-colors group/item"
                    :class="store.filterTags.includes(tag) ? 'bg-white shadow-sm ring-1 ring-gray-200 text-gray-900 font-medium' : 'text-gray-600 hover:bg-gray-100'"
                >
                    <div class="flex items-center gap-3 overflow-hidden">
                        <HashtagIcon class="shrink-0 text-gray-400" />
                        <span v-if="!isCollapsed" class="truncate">{{ tag }}</span>
                    </div>
                    
                    <!-- Unpin Action -->
                    <div v-if="!isCollapsed" @click="togglePin(tag, $event)" class="opacity-0 group-hover/item:opacity-100 cursor-pointer text-gray-400 hover:text-red-500">
                        <Icon name="close" size="12px" />
                    </div>
                </button>
            </div>

            <!-- All Tags Tree -->
            <div class="space-y-1">
                <div 
                    v-if="!isCollapsed" 
                    @click="showAllTags = !showAllTags"
                    class="px-3 text-xs font-semibold text-gray-400 uppercase tracking-wider mb-2 cursor-pointer hover:text-gray-600 flex justify-between"
                >
                    <span>Tags</span>
                    <span>{{ showAllTags ? '-' : '+' }}</span>
                </div>

                <div v-if="!isCollapsed && showAllTags" class="space-y-0.5">
                    <!-- Create Tag Button -->
                    <div class="px-2 mb-2">
                         <button 
                            @click="handleCreateTag"
                            class="w-full flex items-center gap-2 px-2 py-1.5 rounded-md text-xs font-medium text-blue-600 bg-blue-50 hover:bg-blue-100 transition-colors border border-blue-200 border-dashed"
                        >
                            <Icon name="add" size="14px" />
                            <span>Create Tag</span>
                        </button>
                    </div>

                    <!-- Untagged Special List -->
                     <button 
                        @click="selectUntagged"
                        class="w-full flex items-center justify-between px-3 py-1.5 rounded-md text-sm transition-colors group/item"
                        :class="store.filterTags.includes('__untagged__') ? 'bg-gray-200 text-gray-900' : 'text-gray-500 hover:bg-gray-100 italic'"
                    >
                        <div class="flex items-center gap-2 overflow-hidden">
                            <span class="truncate">Untagged</span>
                        </div>
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
                             <!-- Pin Action -->
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
        
        <!-- Bottom: User/Calendar Link? -->
    </div>
</template>

<style scoped>
/* Custom Scrollbar for the dock */
.custom-scrollbar::-webkit-scrollbar {
  width: 4px;
}
.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
  background: #e5e7eb;
  border-radius: 4px;
}
.custom-scrollbar:hover::-webkit-scrollbar-thumb {
  background: #d1d5db;
}
</style>
