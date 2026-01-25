<template>
  <div 
    class="memo-card relative group flex flex-col bg-surface-1 border border-border rounded-xl transition-all duration-200 hover:shadow-lg cursor-pointer select-none overflow-hidden"
    :class="[
      selected ? 'ring-2 ring-primary' : '',
      colorClass
    ]"
    @click="$emit('click')"
  >
    <!-- Header: Title & Pin -->
    <div v-if="memo.title || memo.is_pinned" class="p-3 pb-0 flex items-start justify-between">
      <h3 v-if="memo.title" class="font-bold text-text-primary text-sm line-clamp-2">
        {{ memo.title }}
      </h3>
      <div v-if="memo.is_pinned" class="text-xs text-primary">
        <i class="ri-pushpin-fill" />
      </div>
    </div>

    <!-- Body: Content Preview -->
    <div class="p-3 text-sm text-text-secondary">
      <div class="line-clamp-6 whitespace-pre-wrap font-serif">
         {{ previewContent }}
      </div>
    </div>

    <!-- Footer: Tags & Meta -->
    <div class="mt-auto p-3 pt-0 flex items-center justify-between text-xs text-text-tertiary">
      <div class="flex gap-1 overflow-hidden">
        <span v-for="tag in memo.tags.slice(0, 3)" :key="tag" class="bg-surface-2 px-1.5 py-0.5 rounded-md">
          #{{ tag }}
        </span>
      </div>
      <div>
        {{ formatDate(memo.created_at) }}
      </div>
    </div>
    
    <!-- Hover Actions -->
    <div class="absolute top-2 right-2 opacity-0 group-hover:opacity-100 transition-opacity flex gap-1">
       <button class="p-1 rounded-md bg-surface-2 hover:bg-surface-3 text-text-primary transition-colors" @click.stop="$emit('pin')">
         <i :class="memo.is_pinned ? 'ri-pushpin-fill' : 'ri-pushpin-line'" />
       </button>
       <button class="p-1 rounded-md bg-surface-2 hover:bg-surface-3 text-red-500 hover:text-red-600 transition-colors" @click.stop="$emit('delete')">
         <i class="ri-delete-bin-line" />
       </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import type { Memo } from '@/stores/memos';
import { formatDistanceToNow } from 'date-fns';

const props = defineProps<{
  memo: Memo;
  selected?: boolean;
}>();

defineEmits(['click', 'delete', 'pin']);

const previewContent = computed(() => {
    // Simple text preview. If content is JSON, parse and extract text? 
    // Requirement said content is "Block-First" but stored as JSON string.
    // However, if we simply use raw text for phase 1, creating memo sends string.
    // If backend stores JSONB, frontend CreateMemo sends string. Backend API (CreateMemoRequest) expects string.
    // memo.rs -> Handler maps payload.content (String) to memo.content (String) to detail.content (String/JSONB).
    // If DB has JSONB, saving a raw string might fail if not valid JSON.
    // Ah, Step 217 `set(memo.content)` where `memo.content` came from `payload.content` (String).
    // If DB expects JSONB, this string MUST be valid JSON.
    // OR we wrap it: `serde_json::json!({ "text": memo.content })`.
    // Given Block-First requirement, ideally we wrap it.
    // But for now let's assume it's just text to get it working, OR assume the input IS valid JSON.
    // Let's assume raw text for the preview logic for now.
    return props.memo.content;
});

const colorClass = computed(() => {
    switch (props.memo.color) {
        case 'Red': return 'bg-red-50 dark:bg-red-900/20 border-red-200 dark:border-red-800/30';
        case 'Green': return 'bg-green-50 dark:bg-green-900/20 border-green-200 dark:border-green-800/30';
        case 'Blue': return 'bg-blue-50 dark:bg-blue-900/20 border-blue-200 dark:border-blue-800/30';
        case 'Purple': return 'bg-purple-50 dark:bg-purple-900/20 border-purple-200 dark:border-purple-800/30';
        case 'Yellow': return 'bg-yellow-50 dark:bg-yellow-900/20 border-yellow-200 dark:border-yellow-800/30';
        case 'Gray': return 'bg-gray-50 dark:bg-gray-800 border-gray-200 dark:border-gray-700';
        default: return 'bg-surface-1';
    }
});

function formatDate(date: string) {
    try {
        return formatDistanceToNow(new Date(date), { addSuffix: true });
    } catch {
        return '';
    }
}
</script>

<style scoped>
.line-clamp-6 {
  display: -webkit-box;
  -webkit-line-clamp: 6;
  -webkit-box-orient: vertical;
  overflow: hidden;
}
</style>
