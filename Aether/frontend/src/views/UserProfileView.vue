<script setup lang="ts">
import { ref, onMounted, computed, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useAuthStore } from '../stores/auth';
import axios from 'axios';

const route = useRoute();
const router = useRouter();
const authStore = useAuthStore();

const userId = computed(() => route.params.id as string);
const isCurrentUser = computed(() => userId.value === 'me' || userId.value === authStore.user?.id);

const remoteProfile = ref<any>(null);

const profile = computed(() => {
    if (isCurrentUser.value && authStore.user) {
         return {
             id: authStore.user.id,
             display_name: authStore.user.display_name || authStore.user.username,
             username: authStore.user.username,
             role: 'Author',
             bio: authStore.user.bio || 'Searching for meaning in the noise.',
             location: 'Unknown',
             avatar_url: authStore.user.avatar_url || '',
             stats: { entries: 0, collections: 0 }
         };
    } else if (remoteProfile.value) {
        return {
             id: remoteProfile.value.id,
             display_name: remoteProfile.value.display_name || remoteProfile.value.username,
             username: remoteProfile.value.username,
             role: 'Author',
             bio: remoteProfile.value.bio || 'Searching for meaning in the noise.',
             location: 'Unknown',
             avatar_url: remoteProfile.value.avatar_url || '',
             stats: { entries: 0, collections: 0 }
        };
    }

    return {
        id: '',
        display_name: 'Loading...',
        username: '',
        role: 'Author',
        bio: '...',
        location: 'Unknown',
        avatar_url: '',
        stats: { entries: 0, collections: 0 }
    };
});

const loadData = async () => {
    if (isCurrentUser.value) {
        // Check if we are actually logged in
        if (!authStore.isAuthenticated) {
            // GUEST MODE
            try {
                // Try to get IP
                const ipRes = await axios.get('https://api.ipify.org?format=json');
                const ip = ipRes.data.ip;

                remoteProfile.value = {
                    id: 'guest',
                    username: 'guest',
                    display_name: 'Visitor',
                    bio: `Exploring the network from ${ip}`,
                    location: 'Deep Space',
                    avatar_url: '', // Will fall back to dicebear
                    stats: { entries: 0, collections: 0 }
                };
            } catch (e) {
                // Fallback if IP fetch fails
                remoteProfile.value = {
                    id: 'guest',
                    username: 'guest',
                    display_name: 'Visitor',
                    bio: 'Exploring the network anonymously.',
                    location: 'Deep Space',
                    avatar_url: '',
                    stats: { entries: 0, collections: 0 }
                };
            }
        } else if (!authStore.user || !authStore.user.username) {
            await authStore.fetchUser();
        }
    } else {
        try {
            const res = await axios.get(`/api/users/${userId.value}`);
            remoteProfile.value = res.data;
        } catch (e) {
            console.error("Failed to load profile", e);
        }
    }
};

onMounted(async () => {
    await loadData();
});

watch(() => route.params.id, () => {
    loadData();
});

const handleLogout = () => {
  authStore.logout();
  router.push('/login');
};
</script>

<template>
  <div class="min-h-screen w-full bg-paper flex items-center justify-center p-6">
    <div class="w-full max-w-2xl relative">
      <!-- Top Actions -->
      <div class="flex justify-between items-center mb-12">
        <button @click="router.back()" class="text-neutral-400 hover:text-ink transition-colors">
          <i class="ri-arrow-left-line text-xl"></i>
        </button>

        <div class="flex gap-4">
            <button v-if="isCurrentUser" @click="router.push('/settings')" class="text-xs font-bold uppercase tracking-widest text-neutral-400 hover:text-ink transition-colors">
              Settings
            </button>
            <button v-if="isCurrentUser" @click="handleLogout" class="text-xs font-bold uppercase tracking-widest text-red-500 hover:text-red-700 transition-colors">
              Logout
            </button>
        </div>
      </div>

      <!-- Profile Header -->
      <div class="flex flex-col md:flex-row gap-12 items-start">
         <div class="w-32 h-32 flex-shrink-0 bg-ash grayscale overflow-hidden">
            <img :src="profile.avatar_url || `https://api.dicebear.com/9.x/notionists/svg?seed=${profile.username}`" class="w-full h-full object-cover mix-blend-multiply" />
         </div>

         <div class="flex-1 space-y-6">
           <div>
             <h1 class="text-4xl font-bold tracking-tighter mb-1">{{ profile.display_name }}</h1>
             <p class="text-sm font-mono text-neutral-400 uppercase tracking-widest">@{{ profile.username }} • {{ profile.location }}</p>
           </div>

           <p class="text-xl font-serif italic text-neutral-600 leading-relaxed border-l-2 border-ink pl-6">
             {{ profile.bio }}
           </p>

           <div class="flex gap-12 pt-4">
             <div>
               <div class="text-3xl font-bold">{{ profile.stats.entries }}</div>
               <div class="text-[10px] font-bold uppercase tracking-widest text-neutral-400">Entries</div>
             </div>
             <div>
               <div class="text-3xl font-bold">{{ profile.stats.collections }}</div>
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
