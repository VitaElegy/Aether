<script setup lang="ts">
import { ref } from 'vue';
import { useMemosStore } from '@/stores/memos';
import { Icon } from 'tdesign-vue-next';

const store = useMemosStore();
const content = ref('');

const expand = () => {
    store.openEditor();
}

async function handleSubmit() {
    if (!content.value.trim()) return;

    // Use current filter tag as default tag for new memo
    const tags = store.filterTags.length > 0 ? [...store.filterTags] : [];
    
    // Quick limit title
    const title = content.value.substring(0, 30) + (content.value.length > 30 ? '...' : '');

    try {
        await store.createMemo({
            title,
            content: content.value,
            tags,
            visibility: 'Public' // Default
        });
        content.value = '';
    } catch (e) {
        console.error('Failed to create memo', e);
    }
}
</script>

<template>
    <div class="border-t border-gray-100 bg-white p-4 shrink-0 shadow-sm z-10">
        <div class="max-w-4xl mx-auto items-end gap-2 flex bg-gray-50 border border-gray-200 rounded-xl p-2 focus-within:ring-2 focus-within:ring-blue-50 transition-all shadow-sm hover:shadow-md">
            
            <!-- Expand -->
            <button 
                @click="expand"
                class="p-2 text-gray-400 hover:text-gray-600 hover:bg-gray-200 rounded-lg transition-colors shrink-0"
                title="Open Full Editor"
            >
                <Icon name="expand-less" size="20px" />
            </button>

            <!-- Input -->
            <textarea
                v-model="content"
                class="flex-1 bg-transparent border-none focus:ring-0 resize-none py-2 text-sm text-gray-700 placeholder-gray-400 min-h-[40px] max-h-32 focus:outline-none custom-scrollbar"
                placeholder="What's on your mind? (Shift + Enter for new line)"
                rows="1"
                @keydown.enter.exact.prevent="handleSubmit"
            ></textarea>

            <!-- Actions -->
            <div class="flex items-center gap-1 shrink-0">
                <button class="p-2 text-gray-400 hover:text-gray-600 hover:bg-gray-200 rounded-lg transition-colors">
                    <Icon name="image-add" size="20px" />
                </button>
                
                <button 
                    @click="handleSubmit"
                    :disabled="!content.trim()"
                    class="p-2 rounded-lg transition-all flex items-center justify-center w-9 h-9"
                    :class="content.trim() ? 'bg-blue-600 text-white hover:bg-blue-700 shadow-sm' : 'bg-gray-100 text-gray-300 cursor-not-allowed'"
                >
                    <Icon name="send" size="18px" />
                </button>
            </div>
        </div>
        
        <div class="max-w-4xl mx-auto mt-2 flex justify-between text-[10px] text-gray-400 px-3 uppercase tracking-wider font-medium">
            <span>Context: {{ store.filterTags.length ? '#' + store.filterTags[0] : 'All' }}</span>
            <span>Enter to send · Shift+Enter to newline</span>
        </div>
    </div>
</template>

<style scoped>
.custom-scrollbar::-webkit-scrollbar {
  width: 4px;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
  background: #e5e7eb;
  border-radius: 4px;
}
</style>
