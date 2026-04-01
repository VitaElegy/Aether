<script setup lang="ts">
import { computed } from 'vue';
import { useMemosStore, type Memo } from '@/stores/memos';
import { format } from 'date-fns';
import { Icon } from 'tdesign-vue-next';

const props = defineProps<{
    memo: Memo
}>();

const emit = defineEmits(['click', 'tag-click', 'quick-action']);

const timeStr = computed(() => format(new Date(props.memo.created_at), 'HH:mm'));

const isCard = computed(() => {
    return props.memo.content.length > 200 || props.memo.content.includes('\n');
});

const excerpt = computed(() => {
    if (props.memo.excerpt) return props.memo.excerpt;
    const text = props.memo.content.replace(/[#*`>\-]/g, '').trim();
    return text.length > 160 ? text.substring(0, 160) + '...' : text;
});

const statusIcon = computed(() => {
    switch (props.memo.status) {
        case 'Todo': return 'circle';
        case 'Doing': return 'loading';
        case 'Done': return 'check-circle-filled';
        case 'Archived': return 'folder';
        default: return 'circle';
    }
});

const priorityColor = computed(() => {
    switch (props.memo.priority) {
        case 'P0': return 'text-red-500';
        case 'P1': return 'text-orange-400';
        case 'P2': return 'text-gray-400';
        case 'P3': return 'text-gray-300';
        default: return 'text-gray-400';
    }
});

function handleTagClick(tag: string, e: Event) {
    e.stopPropagation();
    emit('tag-click', tag);
}

function handleAction(action: string, e: Event) {
    e.stopPropagation();
    emit('quick-action', { id: props.memo.id, action });
}
</script>

<template>
    <div
        @click="$emit('click', memo)"
        class="group flex gap-3 max-w-3xl cursor-pointer"
    >
        <!-- Time (Left gutter) -->
        <div class="text-xs text-gray-400 w-10 text-right pt-2 shrink-0 group-hover:text-gray-600 tabular-nums">
            {{ timeStr }}
        </div>

        <!-- Bubble / Card Container -->
        <div
            class="rounded-lg p-3 relative transition-all border border-transparent flex-1"
            :class="[
                isCard
                    ? 'bg-white shadow-sm ring-1 ring-gray-100 hover:shadow-md'
                    : 'bg-white shadow-sm hover:shadow hover:bg-gray-50'
            ]"
        >
            <!-- Title Row (if has title) -->
            <div v-if="memo.title && memo.title !== 'Untitled Note'" class="flex items-center gap-2 mb-1">
                <Icon :name="statusIcon" size="14px" :class="priorityColor" />
                <span class="text-sm font-medium text-gray-900 truncate">{{ memo.title }}</span>
            </div>

            <!-- Content -->
            <div
                class="text-gray-700 text-sm whitespace-pre-wrap break-words leading-relaxed"
                :class="{ 'line-clamp-4': isCard }"
            >
                {{ excerpt }}
            </div>

            <!-- Tags, Channel & Meta -->
            <div class="mt-2 flex items-center gap-2 flex-wrap">
                <!-- Channel Badge -->
                <span
                    v-if="memo.channel"
                    class="text-[10px] font-semibold text-indigo-500 bg-indigo-50 px-1.5 py-0.5 rounded uppercase tracking-wider"
                >
                    {{ memo.channel }}
                </span>

                <span
                    v-for="tag in memo.tags"
                    :key="tag"
                    @click="handleTagClick(tag, $event)"
                    class="text-xs text-blue-500 hover:text-blue-700 bg-blue-50 px-1.5 py-0.5 rounded cursor-pointer"
                >
                    #{{ tag }}
                </span>

                <!-- Linked Entities Count -->
                <span
                    v-if="memo.linked_entities && memo.linked_entities.length > 0"
                    class="text-[10px] text-gray-400 flex items-center gap-0.5"
                >
                    <Icon name="link" size="10px" />
                    {{ memo.linked_entities.length }}
                </span>

                <!-- Pinned Indicator -->
                <Icon v-if="memo.is_pinned" name="pin-filled" size="12px" class="text-amber-400" />

                <!-- Due date -->
                <span v-if="memo.due_at" class="text-[10px] text-gray-400 flex items-center gap-0.5">
                    <Icon name="calendar" size="10px" />
                    {{ format(new Date(memo.due_at), 'MMM d') }}
                </span>
            </div>

            <!-- Hover Quick Actions (MEMO-01) -->
            <div class="absolute top-1 right-1 opacity-0 group-hover:opacity-100 transition-opacity flex items-center gap-0.5 bg-white/90 backdrop-blur-sm rounded-md shadow-sm px-1 py-0.5 border border-gray-100">
                <button
                    @click="handleAction('pin', $event)"
                    class="p-1 text-gray-400 hover:text-amber-500 rounded transition-colors"
                    :title="memo.is_pinned ? 'Unpin' : 'Pin'"
                >
                    <Icon :name="memo.is_pinned ? 'pin-filled' : 'pin'" size="12px" />
                </button>
                <button
                    @click="handleAction('archive', $event)"
                    class="p-1 text-gray-400 hover:text-gray-600 rounded transition-colors"
                    title="Archive"
                >
                    <Icon name="folder" size="12px" />
                </button>
                <button
                    @click="handleAction('snooze', $event)"
                    class="p-1 text-gray-400 hover:text-blue-500 rounded transition-colors"
                    title="Snooze"
                >
                    <Icon name="time" size="12px" />
                </button>
                <button
                    @click="handleAction('convert_task', $event)"
                    class="p-1 text-gray-400 hover:text-green-500 rounded transition-colors"
                    title="Convert to Task"
                >
                    <Icon name="task" size="12px" />
                </button>
            </div>
        </div>
    </div>
</template>
