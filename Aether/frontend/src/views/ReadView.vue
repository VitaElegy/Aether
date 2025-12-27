<script setup lang="ts">
import { ref, onMounted, computed } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import axios from 'axios';
import { useAuthStore } from '../stores/auth';
import { usePreferencesStore } from '../stores/preferences';
import DynamicRenderer from '../components/DynamicRenderer.vue';
import CommentSection from '../components/CommentSection.vue';
import { marked } from 'marked';

const route = useRoute();
const router = useRouter();
const authStore = useAuthStore();
const prefStore = usePreferencesStore();

const id = route.params.id as string;
const post = ref<any>(null);
const loading = ref(true);
const toc = ref<{ id: string; text: string; level: number; }[]>([]);

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

const scrollToHeading = (id: string) => {
    const element = document.getElementById(id);
    if (element) {
        element.scrollIntoView({ behavior: 'smooth' });
    }
};

onMounted(async () => {
    try {
        const res = await axios.get(`/api/content/${id}`);
        const data = res.data;
        post.value = {
            id: data.id,
            title: data.title,
            author_id: data.author_id,
            author_name: data.author_name || 'Unknown',
            created_at: new Date(data.created_at).toLocaleDateString('en-US', { month: 'long', day: 'numeric', year: 'numeric' }),
            category: data.category,
            tags: data.tags,
            type: data.body.type,
            data: data.body.type === 'Markdown' ? { content: data.body.data } : data.body.data,
            raw_body: data.body.type === 'Markdown' ? data.body.data : ''
        };
        if (post.value.type === 'Markdown') {
            toc.value = generateToc(post.value.raw_body);
        }
    } catch (e) {
        console.error("Failed to load article", e);
        router.push('/');
    } finally {
        loading.value = false;
    }
});

const isAuthor = computed(() => {
    return authStore.user && post.value && authStore.user.id === post.value.author_id;
});

const handleEdit = () => {
    router.push(`/editor/${post.value.id}`);
};
</script>

<template>
    <div class="min-h-screen w-full bg-paper flex flex-col transition-colors duration-700">
        <!-- Header -->
        <header
            class="h-16 flex-shrink-0 flex items-center justify-between px-6 border-b border-ash/50 bg-paper/80 backdrop-blur-xl z-20 fixed w-full top-0">
            <div class="flex items-center gap-4">
                <button @click="router.push('/')" class="text-ink/60 hover:text-accent transition-colors">
                    <i class="ri-arrow-left-line text-xl"></i>
                </button>
                <div class="h-4 w-px bg-ash mx-2"></div>
                <span class="text-[10px] font-black uppercase tracking-[0.3em] text-ink/40">
                    Transmission / Reader
                </span>
            </div>

            <div class="flex items-center gap-4">
                <button v-if="isAuthor" @click="handleEdit"
                    class="text-xs font-black uppercase tracking-widest text-accent hover:brightness-125 transition-all">
                    Modify Entry
                </button>
            </div>
        </header>

        <div v-if="loading" class="flex-1 flex items-center justify-center pt-16">
            <div class="animate-pulse text-accent text-xs font-black uppercase tracking-[0.4em]">Establishing Uplink...
            </div>
        </div>

        <div v-else class="flex-1 flex pt-16">
            <!-- Sidebar (TOC) -->
            <aside v-if="!prefStore.isSidebarCollapsed"
                class="w-80 flex-shrink-0 hidden xl:flex flex-col border-r border-ash/50 bg-paper px-10 py-16 overflow-y-auto custom-scrollbar h-[calc(100vh-64px)] sticky top-16">
                <div class="text-[10px] font-black uppercase tracking-[0.3em] text-accent mb-10">Map Of Content</div>
                <nav v-if="toc.length > 0" class="flex flex-col gap-6">
                    <a v-for="(item, idx) in toc" :key="idx" href="#"
                        class="text-xs font-bold text-ink/40 hover:text-accent transition-all leading-tight border-l-2 border-transparent hover:border-accent pl-4"
                        :class="{ 'ml-4': item.level === 2, 'ml-8': item.level === 3 }"
                        @click.prevent="scrollToHeading(item.id)">
                        {{ item.text }}
                    </a>
                </nav>
                <div v-else class="text-xs text-ink/20 italic font-mono">No structure detected.</div>
            </aside>

            <!-- Main Content -->
            <main class="flex-1 overflow-y-auto relative custom-scrollbar">
                <div class="max-w-4xl mx-auto px-12 py-20">
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

                        <div class="flex items-center gap-5">
                            <div class="w-12 h-12 bg-ash/50 rounded-full overflow-hidden border-2 border-accent/20">
                                <img :src="`https://api.dicebear.com/9.x/notionists/svg?seed=${post.author_name}`"
                                    class="w-full h-full object-cover dark:contrast-125" />
                            </div>
                            <div class="flex flex-col">
                                <span
                                    class="text-sm font-black text-ink uppercase tracking-widest">{{ post.author_name }}</span>
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
        </div>
    </div>
</template>
