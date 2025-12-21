<script setup lang="ts">
import { ref } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useAuthStore } from '../stores/auth';

const route = useRoute();
const router = useRouter();
const authStore = useAuthStore();

const userId = route.params.id as string;
const user = ref({
  id: userId,
  name: userId === 'me' ? (authStore.user?.name || 'Explorer') : 'Traveler',
  role: 'Author',
  bio: 'Searching for meaning in the noise.',
  location: 'Kyoto, Japan',
  joined: '2023',
  stats: {
    entries: 42,
    collections: 5
  }
});
</script>

<template>
  <div class="min-h-screen w-full bg-paper flex items-center justify-center p-6">
    <div class="w-full max-w-2xl">
      <!-- Back -->
      <button @click="router.back()" class="mb-12 text-neutral-400 hover:text-ink transition-colors">
        <i class="ri-arrow-left-line text-xl"></i>
      </button>

      <!-- Profile Header -->
      <div class="flex flex-col md:flex-row gap-12 items-start">
         <div class="w-32 h-32 flex-shrink-0 bg-ash grayscale">
            <img :src="`https://api.dicebear.com/9.x/notionists/svg?seed=${user.name}`" class="w-full h-full object-cover mix-blend-multiply" />
         </div>

         <div class="flex-1 space-y-6">
           <div>
             <h1 class="text-4xl font-bold tracking-tighter mb-1">{{ user.name }}</h1>
             <p class="text-sm font-mono text-neutral-400 uppercase tracking-widest">{{ user.role }} • {{ user.location }}</p>
           </div>

           <p class="text-xl font-serif italic text-neutral-600 leading-relaxed border-l-2 border-ink pl-6">
             {{ user.bio }}
           </p>

           <div class="flex gap-12 pt-4">
             <div>
               <div class="text-3xl font-bold">{{ user.stats.entries }}</div>
               <div class="text-[10px] font-bold uppercase tracking-widest text-neutral-400">Entries</div>
             </div>
             <div>
               <div class="text-3xl font-bold">{{ user.stats.collections }}</div>
               <div class="text-[10px] font-bold uppercase tracking-widest text-neutral-400">Collections</div>
             </div>
           </div>
         </div>
      </div>

      <!-- Recent Activity (Placeholder) -->
      <div class="mt-24 border-t border-neutral-100 pt-12">
        <h3 class="text-xs font-bold uppercase tracking-widest mb-8">Recent Transmissions</h3>
        <div class="space-y-4">
          <div class="h-4 bg-ash w-3/4"></div>
          <div class="h-4 bg-ash w-1/2"></div>
        </div>
      </div>
    </div>
  </div>
</template>
