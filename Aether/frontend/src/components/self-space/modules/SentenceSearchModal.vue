<template>
    <t-dialog
        v-model:visible="internalVisible"
        header="Link Existing Sentence"
        :footer="false"
        width="700px"
        @close="handleClose"
    >
        <div class="space-y-4">
            <!-- Search Input -->
            <t-input
                v-model="searchQuery"
                placeholder="Search for sentences..."
                clearable
                @enter="handleSearch"
                autofocus
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
            <div v-else class="max-h-[60vh] overflow-y-auto space-y-2 custom-scrollbar">
                <div v-if="results.length === 0 && !firstLoad" class="text-center py-8 text-ink/40">
                    No sentences found.
                </div>
                <div v-if="firstLoad" class="text-center py-8 text-ink/30 italic">
                    Type keywords to search global sentence repository.
                </div>

                <div 
                    v-for="item in results" 
                    :key="item.id"
                    class="p-4 rounded-lg border border-ink/5 hover:bg-ash/5 hover:border-accent/20 cursor-pointer transition-all flex flex-col gap-1 group relative"
                    @click="selectItem(item)"
                >
                    <div class="font-serif text-lg text-ink/90 pr-12 leading-relaxed">
                        {{ item.text }}
                    </div>
                    <div v-if="item.translation" class="text-sm text-ink/50 font-sans">
                        {{ item.translation }}
                    </div>
                    
                    <div class="absolute right-4 top-1/2 -translate-y-1/2 opacity-0 group-hover:opacity-100 transition-opacity">
                         <t-button size="small" theme="primary" variant="text">Select</t-button>
                    </div>
                </div>
            </div>
        </div>
    </t-dialog>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import { MessagePlugin } from 'tdesign-vue-next';
import { dictionaryApi } from '@/api/dictionary';

export interface SentenceResult {
    id: string;
    text: string;
    translation?: string;
}

const props = defineProps({
    visible: Boolean,
});

const emit = defineEmits(['update:visible', 'select']);

const internalVisible = ref(false);
const searchQuery = ref('');
const loading = ref(false);
const firstLoad = ref(true);
const results = ref<SentenceResult[]>([]);

watch(() => props.visible, (val) => {
    internalVisible.value = val;
    if (val) {
        searchQuery.value = '';
        results.value = [];
        firstLoad.value = true;
    }
});

const handleClose = () => emit('update:visible', false);

const handleSearch = async () => {
    if (!searchQuery.value.trim()) return;
    
    loading.value = true;
    firstLoad.value = false;
    
    try {
        const res = await dictionaryApi.searchSentences(searchQuery.value);
        results.value = res;
    } catch (e) {
        MessagePlugin.error('Search failed');
    } finally {
        loading.value = false;
    }
};

const selectItem = (item: SentenceResult) => {
    emit('select', item);
    handleClose();
};
</script>

<style scoped>
.custom-scrollbar::-webkit-scrollbar { width: 4px; }
.custom-scrollbar::-webkit-scrollbar-thumb { background: #e5e5e5; border-radius: 2px; }
.custom-scrollbar::-webkit-scrollbar-track { background: transparent; }
</style>
