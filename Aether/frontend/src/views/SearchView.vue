<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import axios from 'axios';
import DynamicRenderer from '../components/DynamicRenderer.vue';
import SearchBar from '../components/SearchBar.vue';

const route = useRoute();
const router = useRouter();
const posts = ref<any[]>([]);
const loading = ref(false);

const performSearch = async () => {
    const q = route.query.q as string;
    if (!q) {
        posts.value = [];
        return;
    }

    loading.value = true;
    try {
        const res = await axios.get(`/api/search?q=${encodeURIComponent(q)}`);
        // Deduplicate and map
        const uniqueData = Array.from(new Map(res.data.map((item: any) => [item.id, item])).values());

        posts.value = uniqueData.map((p: any) => ({
            id: p.id,
            title: p.title,
            author: p.author_name || 'Unknown',
            date: new Date(p.created_at).toLocaleDateString('en-US', { month: 'long', day: 'numeric', year: 'numeric' }),
            category: p.category || 'Uncategorized',
            type: p.body.type,
            data: p.body.type === 'Markdown' ? { content: p.body.data } : p.body.data,
            tags: p.tags
        }));
    } catch (err) {
        console.error(err);
    } finally {
        loading.value = false;
    }
};

watch(() => route.query.q, performSearch);

onMounted(performSearch);
</script>

<template>
    <div
        class="min-h-screen bg-paper text-ink selection:bg-neutral-500 selection:text-white transition-colors duration-500">
        <!-- Navbar -->
        <nav
            class="fixed top-0 left-0 w-full bg-paper/80 backdrop-blur-xl border-b border-ash/50 z-50 px-6 py-4 flex justify-between items-center">
            <div class="flex items-center gap-2 cursor-pointer group" @click="router.push('/')">
                <span
                    class="font-bold tracking-tighter text-2xl group-hover:text-ink/80 transition-colors">Aether.</span>
                <span
                    class="text-[10px] font-mono uppercase bg-ash text-ink/50 px-2 py-0.5 rounded-full border border-ash/50">v2.0</span>
            </div>

            <div class="flex items-center gap-6">
                <SearchBar />
                <button @click="router.push('/')"
                    class="text-xs font-black uppercase tracking-[0.2em] hover:text-ink/70 transition-colors">Return</button>
            </div>
        </nav>

        <!-- Main Content -->
        <main class="pt-40 pb-20 max-w-5xl mx-auto px-6">

            <header class="mb-24">
                <h1 class="text-4xl md:text-6xl font-black tracking-tighter mb-6 uppercase">
                    Signals for <span class="text-ink/30 italic">"{{ route.query.q }}"</span>
                </h1>
                <div
                    class="flex items-center gap-4 text-xs font-mono uppercase tracking-[0.2em] text-ink/50 border-t border-ink/20 pt-6 max-w-sm">
                    <span class="flex items-center gap-2">
                        <span class="w-1.5 h-1.5 rounded-full"
                            :class="loading ? 'bg-accent animate-pulse' : 'bg-green-500'"></span>
                        {{ loading ? 'SCANNING...' : 'SCAN COMPLETE' }}
                    </span>
                    <span class="flex-1"></span>
                    <span>{{ posts.length }} MATCHES</span>
                </div>
            </header>

            <!-- Results Grid -->
            <div v-if="posts.length > 0" class="grid grid-cols-1 gap-12">
                <article v-for="post in posts" :key="post.id"
                    class="group relative flex flex-col md:flex-row gap-8 md:gap-16 border-t border-ash pt-16 hover:bg-ash/5 transition-all duration-500 rounded-2xl p-8 -mx-8 glow-hover">
                    <!-- Meta -->
                    <div class="md:w-1/4 flex flex-col gap-4">
                        <div class="flex flex-col gap-1">
                            <span
                                class="text-[10px] font-mono text-ink/40 uppercase tracking-widest leading-none">Author</span>
                            <span class="text-sm font-bold text-ink leading-none">{{ post.author }}</span>
                        </div>
                        <div class="flex flex-col gap-1">
                            <span
                                class="text-[10px] font-mono text-ink/40 uppercase tracking-widest leading-none">Timestamp</span>
                            <span class="text-xs font-mono text-ink/60 leading-none">{{ post.date }}</span>
                        </div>
                        <div class="mt-4">
                            <span
                                class="text-[10px] font-black uppercase tracking-[0.15em] bg-ash text-ink px-3 py-1.5 rounded-sm">{{ post.category }}</span>
                        </div>
                    </div>

                    <!-- Content -->
                    <div class="md:w-3/4">
                        <h2 @click="router.push(`/article/${post.id}`)"
                            class="text-3xl font-black tracking-tight mb-4 group-hover:text-ink/80 transition-colors cursor-pointer uppercase leading-tight">
                            {{ post.title }}
                        </h2>
                        <div class="prose prose-lg max-w-none text-ink/80 line-clamp-2 mb-6 leading-relaxed">
                            <DynamicRenderer :type="post.type" :data="post.data" />
                        </div>
                        <div class="flex flex-wrap gap-2">
                            <span v-for="tag in post.tags" :key="tag"
                                class="text-[10px] font-mono text-ink/50 border border-ash px-2 py-0.5 rounded-sm hover:border-ink/20 transition-colors">
                                #{{ tag }}
                            </span>
                        </div>
                    </div>
                </article>
            </div>

            <div v-else-if="!loading" class="py-32 text-center border-t border-ash/20">
                <p class="text-ink/30 font-mono uppercase tracking-[0.3em] text-sm mb-8 animate-pulse">NO SIGNALS
                    DETECTED.</p>
                <button @click="router.push('/')"
                    class="text-[10px] font-black uppercase tracking-[0.2em] border-b border-ink/40 pb-1 hover:text-ink transition-all">RETURN
                    TO SYSTEM FEED</button>
            </div>

        </main>
    </div>
</template>
