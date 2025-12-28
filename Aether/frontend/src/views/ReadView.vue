<script setup lang="ts">
import { ref, onMounted, computed, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import axios from 'axios';
import { useAuthStore } from '../stores/auth';
import { usePreferencesStore } from '../stores/preferences';
import DynamicRenderer from '../components/DynamicRenderer.vue';
import CommentSection from '../components/CommentSection.vue';
import { marked } from 'marked';
import { knowledgeApi } from '../api/knowledge';
import TopNavBar from '@/components/TopNavBar.vue';
import SidebarContainer from '../components/reading/SidebarContainer.vue';
import ArticleOutline from '../components/reading/ArticleOutline.vue';
import DirectoryTree from '../components/reading/DirectoryTree.vue';

const route = useRoute();
const router = useRouter();
const authStore = useAuthStore();
const prefStore = usePreferencesStore();

const id = route.params.id as string;
const post = ref<any>(null);
const kbTitle = ref<string>('');
const loading = ref(true);
const toc = ref<{ id: string; text: string; level: number; }[]>([]);

// -- Sidebar Configuration --
// Using Preferences Store for persistence
const isTreeEnabled = computed({
    get: () => prefStore.readViewShowTree && !!post.value?.knowledge_base_id,
    set: (v) => prefStore.readViewShowTree = v
});
const isOutlineEnabled = computed({
    get: () => prefStore.readViewShowOutline,
    set: (v) => prefStore.readViewShowOutline = v
});

// Layout Logic
// If Tree preference is 'left': Tree takes Left, Outline takes Right.
// If valid content is missing, the other might shift to fill if desired,
// OR we strictly follow "Primary Slot" logic.
// User requested: "Single TOC -> Left. Both -> Tree Left, TOC Right."
// This implies filling Left first.

const leftContent = computed(() => {
    const treeSide = prefStore.readViewTreeSide; // 'left' or 'right'

    // If Tree is supposed to be Left and is Enabled
    if (treeSide === 'left' && isTreeEnabled.value) {
        return 'tree';
    }

    // If Tree is NOT Left (so it's Right or Disabled)
    // AND Outline is Enabled
    // CHECK: Should Outline shift to Left if Tree is hidden?
    // User: "When I select TOC alone it shows on left". Yes.
    if (isOutlineEnabled.value) {
        // If Tree is Right and Enabled, Outline is displaced? No, Tree=Right, Outline=Left.
        // If Tree is Left but Disabled, Outline fills Left? Yes.
        // If Tree is Right and Disabled, Outline fills Left? Yes.
        // Only case Outline is NOT Left is if it's supposed to be Right and Left is occupied.
        // Left occupied by Tree? Checked above.
        // So here, Left is free. Outline takes it.
        return 'outline';
    }

    return null;
});

const rightContent = computed(() => {
    const treeSide = prefStore.readViewTreeSide;

    // If Tree is Right and Enabled
    if (treeSide === 'right' && isTreeEnabled.value) {
        return 'tree';
    }

    // If Outline is Enabled
    if (isOutlineEnabled.value) {
        // Outline wants to support floating or being on Right if Left is taken.
        // Left taken by:
        // 1. Tree (if treeSide='left' && enabled).
        // 2. Outline (if ... wait, if Left is Outline, Right shouldn't be Outline).

        if (leftContent.value === 'outline') return null; // Already rendered on left

        // If Left is Tree, Outline goes Right.
        if (leftContent.value === 'tree') return 'outline';
    }

    return null;
});

const leftMode = ref<'docked' | 'floating'>('docked');
const rightMode = ref<'docked' | 'floating'>('docked');

// Actions
const toggleTree = () => {
    console.log("[ReadView] Toggle Tree Clicked. KB ID:", post.value?.knowledge_base_id);
    if (!post.value?.knowledge_base_id) return;
    prefStore.readViewShowTree = !prefStore.readViewShowTree;
    console.log("[ReadView] Tree Enabled State:", prefStore.readViewShowTree);
};
const toggleOutline = () => {
    console.log("[ReadView] Toggle Outline Clicked");
    prefStore.readViewShowOutline = !prefStore.readViewShowOutline;
    console.log("[ReadView] Outline Enabled State:", prefStore.readViewShowOutline);
};

const toggleMode = (side: 'left' | 'right') => {
    console.log("[ReadView] Toggle Mode:", side);
    if (side === 'left') leftMode.value = leftMode.value === 'docked' ? 'floating' : 'docked';
    if (side === 'right') rightMode.value = rightMode.value === 'docked' ? 'floating' : 'docked';
};


// -- Data Loading --

const generateToc = (markdown: string) => {
    const tokens = marked.lexer(markdown);
    const headings: { id: string; text: string; level: number; }[] = [];
    tokens.forEach((token: any) => {
        if (token.type === 'heading') {
            const text = token.text;
            const id = text.toLowerCase().replace(/[^\w]+/g, '-');
            headings.push({ id, text, level: token.depth });
        }
    });
    return headings;
};



// Reusable data loader
const loadData = async (articleId: string) => {
    loading.value = true;
    try {
        const res = await axios.get(`/api/content/${articleId}`);
        const data = res.data;
        post.value = {
            id: data.id,
            title: data.title,
            author_id: data.author_id,
            author_name: data.author_name || 'Unknown',
            author_avatar: data.author_avatar,
            created_at: new Date(data.created_at).toLocaleDateString('en-US', { month: 'long', day: 'numeric', year: 'numeric' }),
            category: data.category,
            tags: data.tags,
            type: data.body.type,
            data: data.body.type === 'Markdown' ? { content: data.body.data } : data.body.data,
            raw_body: data.body.type === 'Markdown' ? data.body.data : '',
            knowledge_base_id: data.knowledge_base_id,
            parent_id: data.parent_id
        };

        if (post.value.knowledge_base_id) {
            try {
                const kb = await knowledgeApi.get(post.value.knowledge_base_id);
                kbTitle.value = kb.title;
            } catch (kbe) {
                console.warn("Failed to fetch KB details", kbe);
                kbTitle.value = 'Directory';
            }
        } else {
            kbTitle.value = '';
        }

        if (post.value.type === 'Markdown') {
            toc.value = generateToc(post.value.raw_body);
        }
        console.log("[ReadView] Loaded Article. KB:", post.value.knowledge_base_id);
    } catch (e) {
        console.error("Failed to load article", e);
    } finally {
        loading.value = false;
    }
};

onMounted(() => {
    loadData(id);
});

// Reload when ID changes (navigation within same view)
watch(() => route.params.id, (newId) => {
    if (newId) loadData(newId as string);
});

const isAuthor = computed(() => {
    return authStore.user && post.value && authStore.user.id === post.value.author_id;
});

const handleEdit = () => {
    router.push(`/editor/${post.value.id}`);
};
</script>

<template>
    <div class="h-screen w-full bg-paper flex flex-col overflow-hidden">
        <!-- Header -->
        <TopNavBar>
            <template #left>
                <button @click="router.push('/')"
                    class="text-ink/60 hover:text-accent transition-colors flex items-center gap-2 mr-4" title="Return">
                    <i class="ri-arrow-left-line text-xl"></i>
                </button>
            </template>

            <template #center>
                <div class="flex items-center gap-2">
                    <!-- Sidebar Controls (Centralized) -->
                    <div class="flex items-center bg-ash/10 rounded-lg p-1 gap-1">
                        <button v-if="post?.knowledge_base_id" @click="toggleTree"
                            class="px-3 py-1 text-[10px] font-black uppercase tracking-wider rounded transition-all flex items-center gap-2"
                            :class="isTreeEnabled ? 'bg-paper shadow-sm text-accent' : 'text-ink/40 hover:text-ink'">
                            <i class="ri-node-tree"></i>
                            <span class="hidden md:inline">Directory</span>
                        </button>

                        <button @click="toggleOutline"
                            class="px-3 py-1 text-[10px] font-black uppercase tracking-wider rounded transition-all flex items-center gap-2"
                            :class="isOutlineEnabled ? 'bg-paper shadow-sm text-accent' : 'text-ink/40 hover:text-ink'">
                            <i class="ri-list-check-2"></i>
                            <span class="hidden md:inline">Outline</span>
                        </button>
                    </div>
                </div>
            </template>

            <template #right>
                <div class="flex items-center gap-4">
                    <!-- Pin Toggles (Contextual) -->
                    <button v-if="leftContent" @click="toggleMode('left')"
                        class="text-ink/20 hover:text-accent transition-colors"
                        :class="{ 'text-accent': leftMode === 'docked' }" title="Dock/Float Left">
                        <i class="ri-pushpin-line" :class="{ 'rotate-45': leftMode === 'docked' }"></i>
                    </button>
                    <button v-if="rightContent" @click="toggleMode('right')"
                        class="text-ink/20 hover:text-accent transition-colors"
                        :class="{ 'text-accent': rightMode === 'docked' }" title="Dock/Float Right">
                        <i class="ri-pushpin-line" :class="{ 'rotate-45': rightMode === 'docked' }"></i>
                    </button>

                    <button v-if="isAuthor" @click="handleEdit"
                        class="text-xs font-black uppercase tracking-widest text-accent hover:brightness-125 transition-all">
                        Modify
                    </button>
                </div>
            </template>
        </TopNavBar>

        <div v-if="loading" class="flex-1 flex items-center justify-center">
            <div class="animate-pulse text-accent text-xs font-black uppercase tracking-[0.4em]">Establishing Uplink...
            </div>
        </div>

        <div v-else class="flex-1 flex overflow-hidden relative">
            <!-- Left Sidebar -->
            <SidebarContainer position="left" :mode="leftMode" :isOpen="!!leftContent"
                :title="leftContent === 'tree' ? (kbTitle || 'Directory') : (post?.title || 'Outline')"
                @close="leftContent === 'tree' ? toggleTree() : toggleOutline()">

                <DirectoryTree v-if="leftContent === 'tree' && post?.knowledge_base_id"
                    :knowledgeBaseId="post.knowledge_base_id" :currentArticleId="post.id" />
                <ArticleOutline v-else-if="leftContent === 'outline'" :toc="toc" />

            </SidebarContainer>

            <!-- Main Content -->
            <main
                class="flex-1 overflow-y-auto relative custom-scrollbar flex flex-col bg-paper transition-all duration-300">
                <div class="max-w-4xl mx-auto px-12 py-20 w-full">
                    <!-- Meta Header -->
                    <div class="mb-20 border-b border-ash pb-16">
                        <div class="flex flex-wrap gap-4 mb-10">
                            <span v-if="post.category"
                                class="text-[10px] font-black uppercase tracking-[0.2em] bg-accent text-paper px-4 py-1.5 rounded-sm shadow-lg shadow-accent/20">{{ post.category }}</span>
                            <span v-for="tag in post.tags" :key="tag"
                                class="text-[10px] font-mono uppercase tracking-widest border border-ash text-ink/40 px-3 py-1.5 rounded-sm hover:border-accent hover:text-accent transition-all select-none">#{{
                                    tag }}</span>
                        </div>

                        <h1
                            class="text-5xl md:text-7xl font-black tracking-tighter mb-10 text-ink leading-[0.9] uppercase">
                            {{ post.title }}
                        </h1>

                        <div class="flex items-center gap-5 cursor-pointer group"
                            @click="router.push(`/profile/${post.author_id}`)">
                            <div
                                class="w-12 h-12 bg-ash/50 rounded-full overflow-hidden border-2 border-accent/20 group-hover:border-accent transition-colors">
                                <img :src="post.author_avatar || `https://api.dicebear.com/9.x/notionists/svg?seed=${post.author_name}`"
                                    class="w-full h-full object-cover dark:contrast-125" />
                            </div>
                            <div class="flex flex-col">
                                <span
                                    class="text-sm font-black text-ink uppercase tracking-widest group-hover:text-accent transition-colors">{{ post.author_name }}</span>
                                <span
                                    class="text-[10px] font-mono text-ink/50 uppercase tracking-[0.2em]">{{ post.created_at }}</span>
                            </div>
                        </div>
                    </div>

                    <!-- Body -->
                    <div
                        class="prose prose-xl max-w-none prose-p:text-ink/90 prose-headings:text-ink prose-headings:uppercase prose-headings:font-black prose-headings:tracking-tighter prose-strong:text-accent prose-code:text-accent prose-pre:bg-ash/20 prose-hr:border-ash/50 selection:bg-accent/20">
                        <DynamicRenderer :type="post.type" :data="post.data" />
                    </div>

                    <div class="mt-40 pt-20 border-t border-ash">
                        <CommentSection v-if="post" :content-id="post.id" :author-id="post.author_id" />
                    </div>
                </div>
            </main>

            <!-- Right Sidebar -->
            <SidebarContainer position="right" :mode="rightMode" :isOpen="!!rightContent"
                :title="rightContent === 'tree' ? (kbTitle || 'Directory') : (post?.title || 'Outline')"
                @close="rightContent === 'tree' ? toggleTree() : toggleOutline()">

                <DirectoryTree v-if="rightContent === 'tree' && post?.knowledge_base_id"
                    :knowledgeBaseId="post.knowledge_base_id" :currentArticleId="post.id" />
                <ArticleOutline v-else-if="rightContent === 'outline'" :toc="toc" />

            </SidebarContainer>
        </div>
    </div>
</template>
