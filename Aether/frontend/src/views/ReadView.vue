<script setup lang="ts">
import { ref, onMounted, computed } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import axios from 'axios';
import { useAuthStore } from '../stores/auth';
import DynamicRenderer from '../components/DynamicRenderer.vue';
import CommentSection from '../components/CommentSection.vue';
import { marked } from 'marked';

const route = useRoute();
const router = useRouter();
const authStore = useAuthStore();

const id = route.params.id as string;
const post = ref<any>(null);
const loading = ref(true);
const toc = ref<{ id: string; text: string; level: number; }[]>([]);

// Generate TOC from Markdown
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
  <div class="min-h-screen w-full bg-paper flex flex-col">
    <!-- Header -->
    <header class="h-16 flex-shrink-0 flex items-center justify-between px-6 border-b border-neutral-100 bg-paper/95 backdrop-blur z-20 fixed w-full top-0">
       <div class="flex items-center gap-4">
          <button @click="router.push('/')" class="text-neutral-400 hover:text-ink transition-colors">
            <i class="ri-arrow-left-line text-xl"></i>
          </button>
          <span class="text-xs font-mono uppercase tracking-widest text-neutral-400">
            Reader
          </span>
       </div>

       <div class="flex items-center gap-4">
           <button v-if="isAuthor" @click="handleEdit" class="text-xs font-bold uppercase tracking-widest text-ink hover:text-neutral-600 transition-colors">
               Edit Entry
           </button>
       </div>
    </header>

    <div v-if="loading" class="flex-1 flex items-center justify-center pt-16">
        <div class="animate-pulse text-neutral-300 text-xs font-bold uppercase tracking-widest">Loading Transmission...</div>
    </div>

    <div v-else class="flex-1 flex pt-16">
        <!-- Sidebar (TOC) - Left -->
        <aside class="w-64 flex-shrink-0 hidden xl:flex flex-col border-r border-neutral-100 bg-paper p-8 overflow-y-auto custom-scrollbar h-[calc(100vh-64px)] sticky top-16">
            <div class="text-[10px] font-bold uppercase tracking-widest text-neutral-400 mb-6">Table of Contents</div>
            <nav v-if="toc.length > 0" class="flex flex-col gap-2">
                <a
                    v-for="(item, idx) in toc"
                    :key="idx"
                    href="#"
                    class="text-xs text-neutral-400 hover:text-ink transition-colors leading-tight"
                    :class="{ 'ml-2': item.level === 2, 'ml-4': item.level === 3 }"
                    @click.prevent="scrollToHeading(item.id)"
                >
                    {{ item.text }}
                </a>
            </nav>
             <div v-else class="text-xs text-neutral-300 italic font-mono">
                No headings found.
            </div>
        </aside>

        <!-- Main Content -->
        <main class="flex-1 overflow-y-auto relative custom-scrollbar">
            <div class="max-w-3xl mx-auto px-8 py-12">
                <!-- Meta Header -->
                <div class="mb-12 border-b border-neutral-100 pb-8">
                    <div class="flex flex-wrap gap-2 mb-6">
                         <span v-if="post.category" class="text-[10px] font-bold uppercase tracking-widest bg-neutral-100 text-neutral-500 px-2 py-1 rounded-sm">{{ post.category }}</span>
                         <span v-for="tag in post.tags" :key="tag" class="text-[10px] font-bold uppercase tracking-widest border border-neutral-200 text-neutral-400 px-2 py-1 rounded-sm">#{{ tag }}</span>
                    </div>

                    <h1 class="text-4xl md:text-5xl font-bold tracking-tight mb-6 text-ink leading-tight">{{ post.title }}</h1>

                    <div class="flex items-center gap-3">
                         <div class="w-8 h-8 bg-ash rounded-full overflow-hidden">
                             <!-- Fallback avatar logic same as HomeView -->
                             <img :src="`https://api.dicebear.com/9.x/notionists/svg?seed=${post.author_name}`" class="w-full h-full object-cover mix-blend-multiply" />
                         </div>
                         <div class="flex flex-col">
                             <span class="text-xs font-bold text-ink">{{ post.author_name }}</span>
                             <span class="text-[10px] font-mono text-neutral-400 uppercase tracking-widest">{{ post.created_at }}</span>
                         </div>
                    </div>
                </div>

                <!-- Body -->
                <div class="prose prose-neutral prose-lg max-w-none">
                    <DynamicRenderer :type="post.type" :data="post.data" />
                </div>

                <!-- Comments -->
                <CommentSection v-if="post" :content-id="post.id" :author-id="post.author_id" />
            </div>
        </main>
    </div>
  </div>
</template>

