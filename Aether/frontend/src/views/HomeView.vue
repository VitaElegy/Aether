<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { useAuthStore } from '../stores/auth';
import axios from 'axios';
import DynamicRenderer from '../components/DynamicRenderer.vue';

const router = useRouter();
const authStore = useAuthStore();
const posts = ref<any[]>([]);

onMounted(async () => {
  // Ensure we have user data for the avatar
  if (!authStore.user || !authStore.user.username) {
      await authStore.fetchUser();
  }

  try {
    const res = await axios.get('/api/content');

    // Deduplicate posts based on ID (Temporary fix for data issues)
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
  }
});

const goToProfile = (id: string) => router.push(`/profile/${id}`);
</script>

<template>
  <div class="min-h-screen bg-paper text-ink selection:bg-black selection:text-white">
    <!-- Navbar -->
    <nav class="fixed top-0 left-0 w-full bg-paper/90 backdrop-blur-sm border-b border-neutral-100 z-50 px-6 py-4 flex justify-between items-center">
      <div class="flex items-center gap-2">
         <span class="font-bold tracking-tighter text-xl">Aether.</span>
         <span class="text-[10px] font-mono uppercase bg-neutral-100 px-2 py-0.5 rounded-full text-neutral-500">v2.0</span>
      </div>

      <div class="flex items-center gap-6">
        <button class="text-xs font-bold uppercase tracking-widest hover:text-neutral-500 transition-colors">Journal</button>
        <button class="text-xs font-bold uppercase tracking-widest text-neutral-400 hover:text-ink transition-colors">Archive</button>
        <button class="text-xs font-bold uppercase tracking-widest text-neutral-400 hover:text-ink transition-colors">About</button>

        <div class="h-4 w-px bg-neutral-200 mx-2"></div>

        <button @click="router.push('/editor')" class="flex items-center gap-2 group">
          <div class="w-8 h-8 rounded-full bg-ink text-paper flex items-center justify-center group-hover:bg-neutral-800 transition-colors">
            <i class="ri-add-line"></i>
          </div>
        </button>

        <div class="w-8 h-8 bg-neutral-100 rounded-full overflow-hidden cursor-pointer hover:ring-2 ring-neutral-200 transition-all" @click="goToProfile(authStore.user?.id || 'me')">
           <img :src="authStore.user?.avatar_url || `https://api.dicebear.com/9.x/notionists/svg?seed=${authStore.user?.username || 'User'}`" class="w-full h-full object-cover mix-blend-multiply" />
        </div>
      </div>
    </nav>

    <!-- Main Content -->
    <main class="pt-32 pb-20 max-w-5xl mx-auto px-6">

      <!-- Header Section -->
      <header class="mb-20">
        <h1 class="text-6xl md:text-8xl font-bold tracking-tighter mb-6 leading-[0.9]">
          Digital <br/> <span class="text-neutral-300">Consciousness.</span>
        </h1>
        <div class="flex items-center gap-4 text-xs font-mono uppercase tracking-widest text-neutral-500 border-t border-black pt-4 max-w-xs">
           <span>System Status: Online</span>
           <span class="flex-1"></span>
           <span>{{ posts.length }} Entries</span>
        </div>
      </header>

      <!-- Feed Grid -->
      <div class="grid grid-cols-1 gap-16">
        <article v-for="post in posts" :key="post.id" class="group relative flex flex-col md:flex-row gap-8 md:gap-16 border-t border-neutral-100 pt-16">

          <!-- Meta (Left) -->
          <div class="md:w-1/4 flex flex-col gap-2">
            <span class="text-xs font-mono text-neutral-400 uppercase tracking-widest">{{ post.author }}</span>
            <span class="text-[10px] font-mono text-neutral-300 uppercase tracking-widest">{{ post.date }}</span>
            <div class="flex flex-wrap gap-2">
              <span class="text-[10px] font-bold uppercase tracking-widest bg-neutral-100 px-2 py-1">{{ post.category }}</span>
            </div>
            <div class="mt-auto hidden md:block">
               <button @click="router.push(`/article/${post.id}`)" class="text-xs font-bold uppercase tracking-widest flex items-center gap-2 opacity-0 group-hover:opacity-100 transition-opacity -ml-4 group-hover:ml-0 duration-300">
                 Read Entry <i class="ri-arrow-right-line"></i>
               </button>
            </div>
          </div>

          <!-- Content (Right) -->
          <div class="md:w-3/4">
             <h2 @click="router.push(`/article/${post.id}`)" class="text-3xl font-bold tracking-tight mb-4 group-hover:text-neutral-600 transition-colors cursor-pointer">{{ post.title }}</h2>

             <!-- Preview Content (Truncated) -->
             <div class="prose prose-neutral prose-lg max-w-none text-neutral-600 line-clamp-3 mb-6">
                <DynamicRenderer :type="post.type" :data="post.data" />
             </div>

             <!-- Tags -->
             <div class="flex flex-wrap gap-3">
               <span v-for="tag in post.tags" :key="tag" class="text-xs text-neutral-400">#{{ tag }}</span>
             </div>
          </div>

        </article>
      </div>

    </main>
  </div>
</template>
