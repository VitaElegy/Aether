<script setup lang="ts">
import { ref, onMounted, computed } from 'vue';
import axios from 'axios';
import { useRouter } from 'vue-router';
import TagInput from '../../common/TagInput.vue'; // We might use a read-only variant or just manual rendering

const router = useRouter();
const articles = ref<any[]>([]);
const loading = ref(true);

// -- Filters --
const searchQuery = ref('');
const selectedTag = ref<string | null>(null);

// -- Computed Stats --
const totalArticles = computed(() => articles.value.length);
const totalWords = computed(() => {
    return articles.value.reduce((acc, curr) => {
        const text = curr.body?.data || "";
        return acc + text.length; // Simplified char count like CJK usually does
    }, 0);
});

const allTags = computed(() => {
    const tags = new Set<string>();
    articles.value.forEach(a => {
        if (a.tags && Array.isArray(a.tags)) {
            a.tags.forEach((t: string) => tags.add(t));
        }
    });
    return Array.from(tags).sort();
});

const filteredArticles = computed(() => {
    return articles.value.filter(article => {
        // 1. Text Search
        const q = searchQuery.value.toLowerCase();
        const matchesSearch = !q || article.title.toLowerCase().includes(q) || (article.body?.data || "").toLowerCase().includes(q);
        
        // 2. Tag Filter
        const matchesTag = !selectedTag.value || (article.tags && article.tags.includes(selectedTag.value));

        return matchesSearch && matchesTag;
    });
});

const calculateReadTime = (content: string) => {
    const wpm = 200; // Average
    const words = content.length; // Approximate for CJK
    const min = Math.ceil(words / wpm);
    return `${min} min read`;
};

onMounted(async () => {
    try {
        const res = await axios.get('/api/content');
        // Ensure data integrity
        articles.value = Array.isArray(res.data) 
            ? res.data.map(a => ({
                ...a,
                tags: a.tags || [],
                body: a.body || { data: '' } 
            })) 
            : [];
    } catch (e) {
        console.error("Failed to fetch articles", e);
    } finally {
        loading.value = false;
    }
});
</script>

<template>
    <div class="h-full flex flex-col p-8 max-w-5xl mx-auto w-full animate-in fade-in slide-in-from-bottom-4 duration-700">
        
        <!-- MINIMALIST HEADER -->
        <header class="mb-16 flex flex-col md:flex-row md:items-end justify-between gap-8">
            <div>
                <h1 class="text-8xl font-serif font-black text-ink mb-2 tracking-tighter">Writing</h1>
                <p class="text-ink/40 font-serif italic text-lg ml-1">Thoughts, drafts, and published works.</p>
            </div>
            
            <!-- Minimal Search & Filter -->
            <div class="flex flex-col items-end gap-4 w-full md:w-auto">
                <div class="relative group w-full md:w-64">
                    <input v-model="searchQuery" 
                        type="text" 
                        class="w-full bg-transparent border-b border-ink/20 py-2 text-ink font-serif placeholder:text-ink/20 focus:outline-none focus:border-accent transition-colors"
                        placeholder="Search..." />
                    <i class="ri-search-line absolute right-0 top-2 text-ink/20"></i>
                </div>
                
                 <!-- Tag Cloud (Text Only) -->
                 <div class="flex flex-wrap justify-end gap-3">
                     <button @click="selectedTag = null"
                        class="text-xs font-bold uppercase tracking-widest transition-colors"
                        :class="!selectedTag ? 'text-accent' : 'text-ink/30 hover:text-ink/60'">
                        All
                    </button>
                    <button v-for="tag in allTags" :key="tag"
                         @click="selectedTag = tag"
                         class="text-xs font-bold uppercase tracking-widest transition-colors"
                         :class="selectedTag === tag ? 'text-accent' : 'text-ink/30 hover:text-ink/60'">
                        #{{ tag }}
                    </button>
                </div>
            </div>
        </header>

        <!-- MINIMALIST LIST -->
        <div v-if="loading" class="space-y-8">
             <div v-for="i in 3" :key="i" class="h-32 bg-ash/5 animate-pulse rounded-lg"></div>
        </div>

        <TransitionGroup 
            name="list" 
            tag="div" 
            class="flex flex-col space-y-12 pb-24"
            v-else-if="filteredArticles.length > 0"
        >
            <article v-for="article in filteredArticles" :key="article.id" 
                @click="router.push(`/article/${article.id}`)"
                class="group relative cursor-pointer flex flex-col gap-2 hover:-translate-x-[-8px] transition-transform duration-500 will-change-transform">
                
                <!-- Meta -->
                <div class="flex items-center gap-3 text-xs font-mono text-ink/30 uppercase tracking-wider group-hover:text-ink/50 transition-colors">
                    <span>{{ new Date(article.created_at).toLocaleDateString('en-US', { month: 'long', day: 'numeric', year: 'numeric' }) }}</span>
                    <span class="w-px h-3 bg-ink/10"></span>
                    <span>{{ article.category || 'Muse' }}</span>
                    <span v-if="article.status === 'Published'" class="text-green-500/50">• Live</span>
                    <span v-else class="text-orange-400/50">• Draft</span>
                </div>

                <!-- Title -->
                <h2 class="text-5xl font-serif font-black text-ink group-hover:text-accent transition-colors leading-tight tracking-tight">
                    {{ article.title }}
                </h2>
                
                <!-- Excerpt (Fade in on hover or keep subtle) -->
                <p class="text-ink/40 font-serif text-lg leading-relaxed line-clamp-2 max-w-2xl group-hover:text-ink/60 transition-colors">
                    {{ article.body?.data || "No content..." }}
                </p>

                <!-- Reading Time -->
                 <div class="mt-2 text-[10px] font-bold uppercase tracking-widest text-ink/20 opacity-0 group-hover:opacity-100 transition-opacity transform translate-y-2 group-hover:translate-y-0 duration-300">
                    {{ calculateReadTime(article.body?.data || '') }} Read <i class="ri-arrow-right-line ml-1"></i>
                 </div>

            </article>
        </TransitionGroup>

        <div v-else class="flex flex-col items-center justify-center py-32 text-ink/20">
            <p class="text-2xl font-serif italic">Silence is golden.</p>
            <button @click="searchQuery = ''; selectedTag = null" class="mt-4 text-accent hover:underline text-xs font-bold uppercase tracking-widest">Clear Filters</button>
        </div>
    </div>
</template>

<style scoped>
.list-move, /* apply transition to moving elements */
.list-enter-active,
.list-leave-active {
  transition: all 0.5s ease;
}

.list-enter-from,
.list-leave-to {
  opacity: 0;
  transform: translateY(30px);
}

/* Ensure leaving items are taken out of layout flow so others can move smoothly */
.list-leave-active {
  position: absolute; 
}
</style>
