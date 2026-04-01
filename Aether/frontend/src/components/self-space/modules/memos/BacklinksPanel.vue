<script setup lang="ts">
import { computed } from 'vue';
import { useMemosStore, type Memo } from '@/stores/memos';
import { Icon } from 'tdesign-vue-next';
import { format } from 'date-fns';

const store = useMemosStore();

const backlinks = computed(() => store.ui.backlinksData);
const targetId = computed(() => store.ui.backlinksTargetId);

const targetMemo = computed(() => {
    if (!targetId.value) return null;
    return store.memos.find(m => m.id === targetId.value) || null;
});

function openMemo(memo: Memo) {
    store.closeBacklinks();
    store.openEditor(memo);
}

function close() {
    store.closeBacklinks();
}
</script>

<template>
    <Transition name="slide-right">
        <div v-if="store.ui.showBacklinks" class="w-80 border-l border-gray-200 bg-white flex flex-col h-full shrink-0 z-10">
            <!-- Header -->
            <div class="px-4 py-3 border-b border-gray-100 flex items-center justify-between shrink-0">
                <div class="flex items-center gap-2">
                    <Icon name="link" size="16px" class="text-gray-400" />
                    <span class="text-sm font-semibold text-gray-700">Backlinks</span>
                    <span class="text-xs text-gray-400 bg-gray-100 px-1.5 py-0.5 rounded-full">{{ backlinks.length }}</span>
                </div>
                <button @click="close" class="p-1 rounded hover:bg-gray-100 text-gray-400 hover:text-gray-600">
                    <Icon name="close" size="16px" />
                </button>
            </div>

            <!-- Target Info -->
            <div v-if="targetMemo" class="px-4 py-3 bg-gray-50 border-b border-gray-100">
                <div class="text-[10px] text-gray-400 uppercase tracking-wider font-semibold mb-1">References to</div>
                <div class="text-sm font-medium text-gray-800 truncate">{{ targetMemo.title || 'Untitled' }}</div>
            </div>

            <!-- Backlinks List -->
            <div class="flex-1 overflow-y-auto custom-scrollbar">
                <div v-if="backlinks.length === 0" class="p-8 text-center text-gray-400 text-sm">
                    <Icon name="link-unlink" size="24px" class="mb-2 mx-auto opacity-50" />
                    <div>No backlinks found</div>
                </div>

                <div v-else class="p-2 space-y-1">
                    <button
                        v-for="memo in backlinks"
                        :key="memo.id"
                        @click="openMemo(memo)"
                        class="w-full text-left p-3 rounded-lg hover:bg-gray-50 transition-colors group"
                    >
                        <div class="text-sm font-medium text-gray-800 truncate group-hover:text-blue-600">
                            {{ memo.title || 'Untitled' }}
                        </div>
                        <div class="text-xs text-gray-500 line-clamp-2 mt-1">
                            {{ memo.content.substring(0, 120) }}{{ memo.content.length > 120 ? '...' : '' }}
                        </div>
                        <div class="flex items-center gap-2 mt-2">
                            <span
                                v-for="tag in memo.tags.slice(0, 2)"
                                :key="tag"
                                class="text-[10px] text-blue-500 bg-blue-50 px-1 py-0.5 rounded"
                            >#{{ tag }}</span>
                            <span class="text-[10px] text-gray-400 ml-auto">
                                {{ format(new Date(memo.updated_at), 'MMM d') }}
                            </span>
                        </div>
                    </button>
                </div>
            </div>

            <!-- Linked Entities of Target -->
            <div v-if="targetMemo && targetMemo.linked_entities.length > 0" class="border-t border-gray-100 px-4 py-3 shrink-0">
                <div class="text-[10px] text-gray-400 uppercase tracking-wider font-semibold mb-2">Linked Entities</div>
                <div class="space-y-1">
                    <div
                        v-for="(le, i) in targetMemo.linked_entities"
                        :key="i"
                        class="flex items-center gap-2 text-xs text-gray-600 py-1"
                    >
                        <span class="text-[10px] uppercase tracking-wider text-gray-400 bg-gray-100 px-1 py-0.5 rounded">
                            {{ le.target_type }}
                        </span>
                        <span class="truncate">{{ le.target_title }}</span>
                    </div>
                </div>
            </div>
        </div>
    </Transition>
</template>

<style scoped>
.slide-right-enter-active, .slide-right-leave-active {
    transition: transform 0.2s ease, opacity 0.2s ease;
}
.slide-right-enter-from, .slide-right-leave-to {
    transform: translateX(100%);
    opacity: 0;
}
.custom-scrollbar::-webkit-scrollbar { width: 4px; }
.custom-scrollbar::-webkit-scrollbar-track { background: transparent; }
.custom-scrollbar::-webkit-scrollbar-thumb { background: #e5e7eb; border-radius: 4px; }
</style>
