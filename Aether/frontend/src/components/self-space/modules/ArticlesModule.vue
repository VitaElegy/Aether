<script setup lang="ts">
import { ref, onMounted } from 'vue';
import axios from 'axios';
import { useRouter } from 'vue-router';

const router = useRouter();
const articles = ref<any[]>([]);
const loading = ref(true);

onMounted(async () => {
    try {
        const res = await axios.get('/api/content');
        articles.value = res.data;
    } catch (e) {
        console.error("Failed to fetch articles", e);
    } finally {
        loading.value = false;
    }
});
</script>

<template>
    <div class="max-w-4xl mx-auto py-24 px-6 animate-in fade-in slide-in-from-bottom-4 duration-700">
        <header class="mb-16">
            <h1 class="text-3xl font-bold tracking-tight mb-2">My Writings</h1>
            <p class="text-neutral-400 font-mono text-xs uppercase tracking-widest">
                {{ articles.length }} Entries • System Active
            </p>
        </header>

        <div v-if="loading" class="space-y-4">
            <div class="h-24 w-full bg-ash/30 animate-pulse rounded-lg"></div>
            <div class="h-24 w-full bg-ash/30 animate-pulse rounded-lg"></div>
        </div>

        <div v-else class="grid gap-6">
            <article v-for="article in articles" :key="article.id" @click="router.push(`/article/${article.id}`)"
                class="group p-6 rounded-2xl border border-ash/50 hover:border-accent/30 hover:bg-ash/5 transition-all cursor-pointer bg-paper/50">
                <div class="flex justify-between items-start mb-4">
                    <span
                        class="text-[10px] font-bold uppercase tracking-widest px-2 py-0.5 rounded-full bg-ash/20 text-neutral-500">
                        {{ article.category || 'General' }}
                    </span>
                    <span class="text-xs font-mono text-neutral-300">
                        {{ new Date(article.created_at).toLocaleDateString() }}
                    </span>
                </div>

                <h2 class="text-xl font-bold mb-2 group-hover:text-accent transition-colors">{{ article.title }}</h2>
                <div class="text-sm text-neutral-400 line-clamp-2 leading-relaxed">
                    {{ article.body?.data?.substring(0, 150) }}...
                </div>
            </article>
        </div>
    </div>
</template>
