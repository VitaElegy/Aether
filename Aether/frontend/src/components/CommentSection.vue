<script setup lang="ts">
import { ref, onMounted } from 'vue';
import axios from 'axios';
import { useAuthStore } from '../stores/auth';
import CommentItem from './CommentItem.vue';

const props = defineProps<{
  contentId: string;
  authorId: string;
}>();

const authStore = useAuthStore();
const comments = ref<any[]>([]);
const loading = ref(true);
const newCommentText = ref('');

const fetchComments = async () => {
    try {
        const res = await axios.get(`/api/content/${props.contentId}/comments`);
        comments.value = buildTree(res.data);
    } catch (e) {
        console.error("Failed to fetch comments", e);
    } finally {
        loading.value = false;
    }
};

const buildTree = (flatComments: any[]) => {
    const map = new Map();
    const roots: any[] = [];

    // Deep clone to avoid mutating if re-run, and ensure replies array exists
    const nodes = flatComments.map(c => ({ ...c, replies: [] }));

    nodes.forEach(c => map.set(c.id, c));

    nodes.forEach(c => {
        if (c.parent_id && map.has(c.parent_id)) {
            map.get(c.parent_id).replies.push(c);
        } else {
            roots.push(c);
        }
    });

    // Sort roots by date descending (newest first) or ascending? Usually oldest first for threads, newest first for streams.
    // Let's go with Oldest First (Chronological) for discussion threads so you can read from top to bottom.
    // Backend sorts by CreatedAt ASC.
    return roots;
};

const handlePostComment = async () => {
    if (!newCommentText.value.trim()) return;

    try {
        await axios.post(`/api/content/${props.contentId}/comments`, {
            text: newCommentText.value
        });
        newCommentText.value = '';
        await fetchComments();
    } catch (e) {
        console.error("Failed to post comment", e);
        alert("Failed to post comment. You probably need to log in.");
    }
};

const handleReply = async (payload: { text: string; parentId: string }) => {
    try {
         await axios.post(`/api/content/${props.contentId}/comments`, {
            text: payload.text,
            parent_id: payload.parentId
        });
        await fetchComments();
    } catch (e) {
        console.error("Failed to post reply", e);
         alert("Failed to post reply.");
    }
};

onMounted(() => {
    fetchComments();
});
</script>

<template>
    <div class="border-t border-neutral-100 pt-12 mt-12">
        <h3 class="text-sm font-bold uppercase tracking-widest text-neutral-400 mb-8">Discussion</h3>

        <!-- New Comment -->
        <div v-if="authStore.isAuthenticated" class="mb-10 flex gap-4">
             <div class="w-8 h-8 rounded-full overflow-hidden bg-neutral-100 flex-shrink-0">
                 <!-- Current user avatar fallback -->
                 <img :src="authStore.user?.avatar_url || `https://api.dicebear.com/9.x/notionists/svg?seed=${authStore.user?.username}`" class="w-full h-full object-cover" />
             </div>
             <div class="flex-1">
                 <textarea
                    v-model="newCommentText"
                    class="w-full bg-white border border-neutral-200 rounded-sm p-4 text-sm focus:border-ink focus:ring-0 outline-none resize-none transition-colors"
                    rows="3"
                    placeholder="Add to the discussion..."
                ></textarea>
                <div class="flex justify-end mt-2">
                    <button @click="handlePostComment" class="bg-ink text-white text-xs font-bold uppercase tracking-widest px-6 py-2 rounded-sm hover:bg-neutral-800 transition-colors">
                        Post Comment
                    </button>
                </div>
             </div>
        </div>
        <div v-else class="mb-10 p-6 bg-neutral-50 text-center rounded-sm">
            <p class="text-sm text-neutral-500 mb-4">Log in to join the discussion.</p>
            <router-link to="/login" class="text-xs font-bold uppercase tracking-widest text-ink border-b-2 border-ink pb-0.5 hover:text-neutral-600 hover:border-neutral-300 transition-colors">
                Login / Register
            </router-link>
        </div>

        <!-- Comments List -->
        <div v-if="loading" class="text-center py-8">
            <span class="text-xs font-mono text-neutral-400 animate-pulse">Loading comments...</span>
        </div>
        <div v-else-if="comments.length === 0" class="text-center py-8">
            <span class="text-sm text-neutral-400 italic">No comments yet. Be the first to critique.</span>
        </div>
        <div v-else>
            <CommentItem
                v-for="comment in comments"
                :key="comment.id"
                :comment="comment"
                :author-id="authorId"
                :depth="0"
                @reply="handleReply"
            />
        </div>
    </div>
</template>
