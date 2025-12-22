<script setup lang="ts">
import { computed, ref } from 'vue';
import { useAuthStore } from '../stores/auth';

// Recursive self-reference for nested comments
defineOptions({
  name: 'CommentItem'
});

const props = defineProps<{
  comment: any;
  authorId: string;
  depth: number;
}>();

const emit = defineEmits(['reply']);

const authStore = useAuthStore();
const isReplying = ref(false);
const replyText = ref('');

const isAuthor = computed(() => props.comment.user_id === props.authorId);

const formatDate = (dateStr: string) => {
    return new Date(dateStr).toLocaleDateString('en-US', { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' });
};

const startReply = () => {
    isReplying.value = !isReplying.value;
};

const submitReply = () => {
    if (!replyText.value.trim()) return;
    emit('reply', { text: replyText.value, parentId: props.comment.id });
    replyText.value = '';
    isReplying.value = false;
};
</script>

<template>
    <div class="flex gap-3 mb-6 animate-fade-in" :class="{ 'pl-4 border-l-2 border-neutral-100 ml-2': depth > 0 }">
        <!-- Avatar -->
        <div class="flex-shrink-0">
            <div class="w-8 h-8 rounded-full overflow-hidden bg-neutral-100">
                 <img :src="comment.user_avatar || `https://api.dicebear.com/9.x/notionists/svg?seed=${comment.user_name}`" class="w-full h-full object-cover" />
            </div>
        </div>

        <div class="flex-1">
            <!-- Header -->
            <div class="flex items-center gap-2 mb-1">
                <span class="text-xs font-bold text-ink">{{ comment.user_name || 'Anonymous' }}</span>
                <span v-if="isAuthor" class="text-[10px] font-bold uppercase tracking-widest bg-ink text-white px-1.5 py-0.5 rounded-sm">Author</span>
                <span class="text-[10px] text-neutral-400 font-mono">{{ formatDate(comment.created_at) }}</span>
            </div>

            <!-- Body -->
            <div class="text-sm text-neutral-600 leading-relaxed mb-2 whitespace-pre-wrap">{{ comment.text }}</div>

            <!-- Actions -->
            <div class="flex items-center gap-4">
                 <button v-if="authStore.isAuthenticated" @click="startReply" class="text-[10px] font-bold uppercase tracking-widest text-neutral-400 hover:text-ink transition-colors">
                    {{ isReplying ? 'Cancel' : 'Reply' }}
                </button>
            </div>

            <!-- Reply Input -->
            <div v-if="isReplying" class="mt-4 mb-4">
                <textarea
                    v-model="replyText"
                    class="w-full bg-neutral-50 border-none rounded-sm p-3 text-sm focus:ring-1 focus:ring-neutral-200 outline-none resize-none"
                    rows="3"
                    placeholder="Write a reply..."
                ></textarea>
                <div class="flex justify-end mt-2">
                    <button @click="submitReply" class="bg-ink text-white text-xs font-bold uppercase tracking-widest px-4 py-2 rounded-sm hover:bg-neutral-800 transition-colors">
                        Post Reply
                    </button>
                </div>
            </div>

            <!-- Nested Replies -->
            <div v-if="comment.replies && comment.replies.length > 0" class="mt-4">
                <CommentItem
                    v-for="reply in comment.replies"
                    :key="reply.id"
                    :comment="reply"
                    :author-id="authorId"
                    :depth="depth + 1"
                    @reply="(payload) => emit('reply', payload)"
                />
            </div>
        </div>
    </div>
</template>
