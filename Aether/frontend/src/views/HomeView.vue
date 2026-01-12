<script setup lang="ts">
import { ref, onMounted, computed, watch } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import { useAuthStore } from '../stores/auth';
import axios from 'axios';
import DynamicRenderer from '../components/DynamicRenderer.vue';
import SearchBar from '../components/SearchBar.vue';

const route = useRoute();
const router = useRouter();
const authStore = useAuthStore();
const posts = ref<any[]>([]);
const currentTag = computed(() => route.query.tag as string | undefined);

const fetchContent = async () => {
    try {
        const params: any = {};
        if (currentTag.value) {
            params.tag = currentTag.value;
        }

        const res = await axios.get('/api/content', { params });
        console.log('API Response:', res.data);

        if (!Array.isArray(res.data)) {
            console.error('Invalid response format:', res.data);
            posts.value = [];
            return;
        }

        const uniqueData = Array.from(new Map(res.data.map((item: any) => [item.id, item])).values());

        posts.value = uniqueData.map((p: any) => ({
            id: p.id,
            title: p.title,
            author: p.author_name || 'Unknown',
            date: new Date(p.created_at).toLocaleDateString('en-US', { month: 'long', day: 'numeric', year: 'numeric' }),
            category: p.category || 'Uncategorized',
            type: p.body.type,
            data: p.body.type === 'Markdown' ? { content: p.body.data } : p.body.data,
            tags: p.tags,
            visibility: p.permission_mode
        }));
    } catch (err: any) {
        console.error('Failed to fetch content:', err);
        posts.value = [];
    }
};

onMounted(async () => {
    if (!authStore.user || !authStore.user.username) {
        await authStore.fetchUser();
    }
    await fetchContent();
});

watch(() => route.query.tag, () => {
    fetchContent();
});

const filterByTag = (tag: string) => {
    router.push({ query: { tag } });
};

const clearFilter = () => {
    router.push({ query: {} });
};

const goToProfile = (id: string) => router.push(`/profile/${id}`);
</script>

<template>
    <div class="min-h-screen bg-paper text-ink selection:bg-accent/20">
        <!-- Navbar -->
        <nav
            class="fixed top-0 left-0 w-full bg-paper/80 backdrop-blur-xl border-b border-ash/50 z-50 px-6 py-4 flex justify-between items-center transition-all duration-500">
            <div class="flex items-center gap-2">
                <span class="font-black tracking-tighter text-2xl text-ink">Aether.</span>
                <span
                    class="text-[9px] font-mono font-bold uppercase bg-accent/10 text-accent px-2 py-0.5 rounded-full border border-accent/20">v2.0</span>
            </div>

            <div class="flex items-center gap-6">
                <SearchBar />

                <button v-if="authStore.isAuthenticated" @click="router.push('/space')"
                    class="text-xs font-bold uppercase tracking-widest text-ink hover:text-accent transition-colors">Self
                    Space</button>
                <div class="h-4 w-px bg-ash mx-2"></div>

                <div class="flex items-center gap-4">
                    <button v-if="authStore.isAuthenticated" @click="router.push('/editor')"
                        class="flex items-center gap-2 group">
                        <div
                            class="w-8 h-8 rounded-full bg-ink text-paper flex items-center justify-center group-hover:bg-accent group-hover:scale-110 transition-all">
                            <i class="ri-add-line"></i>
                        </div>
                    </button>

                    <div class="w-8 h-8 bg-ash rounded-full overflow-hidden cursor-pointer hover:ring-2 ring-accent transition-all border border-ash/50"
                        @click="goToProfile(authStore.user?.id || 'me')">
                        <img :src="authStore.user?.avatar_url || `https://api.dicebear.com/9.x/notionists/svg?seed=${authStore.user?.username || 'User'}`"
                            class="w-full h-full object-cover dark:contrast-125" />
                    </div>
                </div>
            </div>
        </nav>

        <!-- Main Content -->
        <main class="pt-40 pb-20 max-w-5xl mx-auto px-6">
            <header class="mb-24 border-b border-ash/50 pb-16">
                <h1 class="text-6xl md:text-8xl font-bold tracking-tighter mb-6 leading-[0.9]">
                    Digital <br /> <span class="text-neutral-300">Consciousness.</span>
                </h1>
                <div
                    class="flex items-center gap-6 text-[10px] font-mono uppercase tracking-[0.3em] text-ink/50 pt-8 max-w-md">
                    <span class="flex items-center gap-2 text-green-500">
                        <span class="w-2 h-2 rounded-full bg-current animate-pulse"></span>
                        Status: Active
                    </span>
                    <span class="flex-1"></span>
                    <span class="text-ink/80 font-bold">{{ posts.length }} ARCHIVES FOUND</span>
                </div>
            </header>

            <!-- Filter Status -->
            <div v-if="currentTag" class="mb-12 flex items-center justify-between bg-accent/5 border border-accent/20 p-4 rounded-xl">
                 <div class="flex items-center gap-3">
                    <span class="text-accent text-xs font-black uppercase tracking-widest">Active Filter:</span>
                    <span class="bg-accent text-paper px-3 py-1 text-xs font-bold rounded-full">#{{ currentTag }}</span>
                 </div>
                 <button @click="clearFilter" class="text-ink/40 hover:text-red-500 transition-colors text-xs font-bold uppercase tracking-widest flex items-center gap-2">
                    <i class="ri-close-circle-line text-lg"></i>
                    Clear
                 </button>
            </div>

            <!-- Feed Grid -->
            <div v-if="posts.length > 0" class="grid grid-cols-1 gap-12">
                <article v-for="post in posts" :key="post.id"
                    class="group relative flex flex-col md:flex-row gap-8 md:gap-16 border-t border-ash/50 pt-16 pb-8 hover:bg-ash/5 transition-all duration-700 rounded-3xl p-8 -mx-8 glow-hover">

                    <!-- Meta (Left) -->
                    <div class="md:w-1/4 flex flex-col gap-6">
                        <div class="flex flex-col gap-1">
                            <span class="text-[10px] font-black uppercase tracking-[0.2em] text-accent">Author</span>
                            <span class="text-base font-bold text-ink">{{ post.author }}</span>
                        </div>
                        <div class="flex flex-col gap-1">
                            <span class="text-[10px] font-black uppercase tracking-[0.2em] text-ink/40">Timestamp</span>
                            <span class="text-xs font-mono text-ink/80">{{ post.date }}</span>
                        </div>
                        <div class="flex gap-2">
                            <span
                                class="text-[10px] font-black uppercase tracking-[0.1em] border border-accent/30 text-accent px-3 py-1 rounded-sm bg-accent/5">{{ post.category }}</span>
                            <!-- Visibility Badge -->
                            <span v-if="post.visibility && post.visibility !== 'Public'"
                                class="text-[10px] font-black uppercase tracking-[0.1em] border px-3 py-1 rounded-sm"
                                :class="{
                                    'border-red-500/30 text-red-500 bg-red-500/5': post.visibility === 'Private',
                                    'border-blue-500/30 text-blue-500 bg-blue-500/5': post.visibility === 'Internal'
                                }">
                                {{ post.visibility }}
                            </span>
                        </div>
                        <div class="mt-auto hidden md:block">
                            <button @click="router.push(`/article/${post.id}`)"
                                class="text-[10px] font-black uppercase tracking-[0.3em] flex items-center gap-3 text-ink group-hover:text-accent transition-all">
                                READ ENTRY <i class="ri-arrow-right-line"></i>
                            </button>
                        </div>
                    </div>

                    <!-- Content (Right) -->
                    <div class="md:w-3/4">
                        <h2 @click="router.push(`/article/${post.id}`)"
                            class="text-4xl md:text-5xl font-black tracking-tight mb-6 group-hover:translate-x-2 transition-transform cursor-pointer leading-tight uppercase">
                            {{ post.title }}
                        </h2>

                        <div
                            class="prose prose-lg max-w-none text-ink/90 line-clamp-3 mb-8 leading-relaxed font-medium">
                            <DynamicRenderer :type="post.type" :data="post.data" />
                        </div>

                        <div class="flex flex-wrap gap-2">
                            <span v-for="tag in post.tags" :key="tag"
                                @click.stop="filterByTag(tag)"
                                class="text-[10px] font-bold text-ink/60 bg-ash/30 px-3 py-1 rounded-full hover:bg-accent/10 hover:text-accent transition-all cursor-crosshair z-10 relative">
                                #{{ tag.toUpperCase() }}
                            </span>
                        </div>
                    </div>

                    <!-- Detail Link (Mobile) -->
                    <div class="md:hidden mt-4">
                        <button @click="router.push(`/article/${post.id}`)" class="btn-accent w-full text-center">Open
                            entry</button>
                    </div>
                </article>
            </div>

            <div v-else class="py-32 text-center">
                <p class="text-ink/20 font-mono uppercase tracking-[0.5em] text-sm animate-pulse">Transmission Void
                    Detected</p>
            </div>
        </main>
    </div>
</template>
