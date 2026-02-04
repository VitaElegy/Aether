<template>
    <t-dialog
        v-model:visible="internalVisible"
        :header="title"
        :footer="false"
        width="600px"
        @close="handleClose"
    >
        <div class="space-y-4">
            <!-- Search Input -->
            <t-input
                v-model="searchQuery"
                placeholder="Search resources (e.g. 'Documentation', 'API Guide')..."
                clearable
                @enter="handleSearch"
            >
                <template #suffix>
                    <t-icon name="search" class="cursor-pointer" @click="handleSearch" />
                </template>
            </t-input>

            <!-- Loading State -->
            <div v-if="loading" class="py-10 text-center text-ink/40">
                <t-loading />
            </div>

            <!-- Results List -->
            <div v-else class="max-h-[60vh] overflow-y-auto space-y-2">
                <div v-if="results.length === 0 && !firstLoad" class="text-center py-8 text-ink/40">
                    No resources found.
                </div>

                <div 
                    v-for="item in results" 
                    :key="item.id"
                    class="p-3 rounded-lg border border-ink/5 hover:bg-ash/5 hover:border-accent/20 cursor-pointer transition-all flex items-center gap-3 group"
                    @click="selectItem(item)"
                >
                    <!-- Icon based on type -->
                    <div class="w-10 h-10 rounded bg-ash/10 flex items-center justify-center shrink-0">
                        <t-icon :name="getIcon(item.entity_type)" />
                    </div>

                    <div class="flex-1 min-w-0">
                        <div class="font-medium text-ink truncate">{{ item.title }}</div>
                        <div class="text-xs text-ink/40 flex items-center gap-2">
                            <span class="uppercase tracking-wide text-[10px] bg-ash/10 px-1.5 rounded">{{ item.entity_type }}</span>
                            <span v-if="item.path" class="text-ink/30 truncate">{{ item.path }}</span>
                        </div>
                    </div>
                    
                    <t-button size="small" variant="text" theme="primary" class="opacity-0 group-hover:opacity-100">
                        Select
                    </t-button>
                </div>
            </div>
        </div>
    </t-dialog>
</template>

<script setup lang="ts">
import { ref, watch, computed } from 'vue';
import { MessagePlugin } from 'tdesign-vue-next';
import { SearchIcon } from 'tdesign-icons-vue-next';

// Define Interface
export interface SearchResult {
    id: string;
    title: string;
    entity_type: 'knowledge_base' | 'article' | 'folder';
    path?: string; // Optional context path
}

const props = defineProps({
    visible: Boolean,
    title: { type: String, default: 'Select Resource' }
});

const emit = defineEmits(['update:visible', 'select']);

const internalVisible = ref(false);
const searchQuery = ref('');
const loading = ref(false);
const firstLoad = ref(true);
const results = ref<SearchResult[]>([]);

watch(() => props.visible, (val) => {
    internalVisible.value = val;
    if (val) {
        searchQuery.value = '';
        results.value = [];
        firstLoad.value = true;
        // Optionally load recents?
    }
});

const handleClose = () => emit('update:visible', false);

const handleSearch = async () => {
    if (!searchQuery.value.trim()) return;
    
    loading.value = true;
    firstLoad.value = false;
    
    try {
        // Mock Search API for now - replace with real API call
        // In real impl, call /api/search?q=...
        // For MVP, we simulated delay and return mock data if no API avail.
        // Assuming we have knowledgeApi.search() 
        
        await new Promise(r => setTimeout(r, 600)); // Sim delay
        
        // Mock Results adapted to query
        results.value = [
            { id: '1', title: 'Backend Documentation', entity_type: 'knowledge_base' },
            { id: '2', title: 'API Specification', entity_type: 'article', path: 'Backend / API' },
            { id: '3', title: 'Deployment Guide', entity_type: 'article', path: 'Ops / Guides' },
            { id: '4', title: 'Project Specification', entity_type: 'folder', path: 'Archive' },
        ].filter(i => i.title.toLowerCase().includes(searchQuery.value.toLowerCase())) as SearchResult[];
        
    } catch (e) {
        MessagePlugin.error('Search failed');
    } finally {
        loading.value = false;
    }
};

const selectItem = (item: SearchResult) => {
    emit('select', item);
    handleClose();
};

const getIcon = (type: string) => {
    switch(type) {
        case 'knowledge_base': return 'book';
        case 'folder': return 'folder';
        case 'article': return 'file-text';
        default: return 'file';
    }
};
</script>
