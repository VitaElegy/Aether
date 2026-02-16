<script setup lang="ts">
import { computed } from 'vue';
import type { Memo } from '@/stores/memos';
import { format } from 'date-fns';
import { Icon } from 'tdesign-vue-next';

const props = defineProps<{
    memo: Memo
}>();

const emit = defineEmits(['click', 'tag-click']);

// Format time: "14:30"
const timeStr = computed(() => format(new Date(props.memo.created_at), 'HH:mm'));

const isCard = computed(() => {
    // Determine if we should show as card (long content or rich media)
    // For now, simple length check
    return props.memo.content.length > 200 || props.memo.content.includes('\n');
});

function handleTagClick(tag: string, e: Event) {
    e.stopPropagation();
    emit('tag-click', tag);
}
</script>

<template>
    <div 
        @click="$emit('click', memo)"
        class="group flex gap-3 max-w-3xl cursor-pointer"
    >
        <!-- Time (Left gutter) -->
        <div class="text-xs text-gray-400 w-10 text-right pt-2 shrink-0 group-hover:text-gray-600">
            {{ timeStr }}
        </div>

        <!-- Bubble / Card Container -->
        <div 
            class="rounded-lg p-3 relative transition-all border border-transparent"
            :class="[
                isCard 
                    ? 'bg-white shadow-sm ring-1 ring-gray-100 w-full hover:shadow-md' 
                    : 'bg-white shadow-sm hover:shadow hover:bg-gray-50 inline-block'
            ]"
        >
            <!-- Content -->
            <div 
                class="text-gray-800 text-sm whitespace-pre-wrap break-words leading-relaxed"
                :class="{ 'line-clamp-6': isCard }"
            >
                {{ memo.content }}
            </div>

            <!-- Tags & Meta -->
            <div class="mt-2 flex items-center gap-2 flex-wrap">
                <span 
                    v-for="tag in memo.tags" 
                    :key="tag"
                    @click="handleTagClick(tag, $event)"
                    class="text-xs text-blue-500 hover:text-blue-700 bg-blue-50 px-1.5 py-0.5 rounded cursor-pointer"
                >
                    #{{ tag }}
                </span>
                
                <!-- Pinned Indicator -->
                <Icon v-if="memo.is_pinned" name="pin-filled" size="12px" class="text-gray-400" />
            </div>
        </div>
    </div>
</template>
