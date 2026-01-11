<script setup lang="ts">
import { ref, onMounted, computed, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useAuthStore } from '../stores/auth';
import axios from 'axios';
import { knowledgeApi } from '../api/knowledge';
import { memoApi } from '../api/memos';

const route = useRoute();
const router = useRouter();
const authStore = useAuthStore();

const userId = computed(() => route.params.id as string);
const isCurrentUser = computed(() => userId.value === 'me' || userId.value === authStore.user?.id);

const remoteProfile = ref<any>(null);
const entries = ref<any[]>([]);
const kbs = ref<any[]>([]);
const memos = ref<any[]>([]);

const loadingEntries = ref(false);
const loadingKbs = ref(false);
const loadingMemos = ref(false);

const activeTab = ref('transmissions');

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
            experience: authStore.user.experience || [],
            stats: { entries: entries.value.length, collections: kbs.value.length }
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
            experience: remoteProfile.value.experience || [],
            stats: { entries: entries.value.length, collections: kbs.value.length }
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
        experience: [],
        stats: { entries: entries.value.length, collections: kbs.value.length }
    };
});

const loadData = async () => {
    entries.value = [];
    kbs.value = [];
    memos.value = [];

    let uid = userId.value;

    if (isCurrentUser.value) {
        if (!authStore.isAuthenticated) {
            // GUEST logic...
            try {
                const ipRes = await axios.get('https://api.ipify.org?format=json');
                const ip = ipRes.data.ip;
                remoteProfile.value = { id: 'guest', username: 'guest', display_name: 'Visitor', bio: `Exploring from ${ip}`, location: 'Deep Space', avatar_url: '', stats: { entries: 0, collections: 0 } };
            } catch (e) {
                remoteProfile.value = { id: 'guest', username: 'guest', display_name: 'Visitor', bio: 'Exploring anonymously.', location: 'Deep Space', avatar_url: '', stats: { entries: 0, collections: 0 } };
            }
            return; // Guest has no content
        } else if (!authStore.user || !authStore.user.username) {
            await authStore.fetchUser();
        }
        if (authStore.user) uid = authStore.user.id;
    } else {
        try {
            const res = await axios.get(`/api/users/${uid}`);
            remoteProfile.value = res.data;
        } catch (e) {
            console.error("Failed to load profile", e);
        }
    }

    if (uid && uid !== 'guest') {
        await Promise.all([
            loadArticles(uid),
            loadKbs(uid),
            loadMemos(uid)
        ]);
    }
};

const loadArticles = async (uid: string) => {
    loadingEntries.value = true;
    try {
        const res = await axios.get(`/api/content`, { params: { author_id: uid } });
        entries.value = res.data;
    } catch (e) {
        console.error("Failed to load articles", e);
    } finally {
        loadingEntries.value = false;
    }
};

const loadKbs = async (uid: string) => {
    loadingKbs.value = true;
    try {
        // Updated API call supporting filtering
        // Using axios directly for list to be safe with the new backend API query param
        const res = await axios.get('/api/knowledge-bases', { params: { author_id: uid } });
        kbs.value = res.data;
    } catch (e) {
        console.error("Failed to load KBs", e);
    } finally {
        loadingKbs.value = false;
    }
};

const loadMemos = async (uid: string) => {
    loadingMemos.value = true;
    try {
        const res = await memoApi.list({ author_id: uid });
        memos.value = res;
    } catch (e) {
        console.error("Failed to load memos", e);
    } finally {
        loadingMemos.value = false;
    }
};

const openKb = (kb: any) => {
    // Navigate to KB view or first article?
    // For now, maybe just log or do nothing?
    // Requirement said: "can view public database"
    // Usually implies traversing the directory.
    // We don't have a generic "KB View" page yet, usually we view *articles* IN a KB.
    // Ideally we should open the Directory Tree for this KB.
    // ReadView supports opening with KB context?
    // Maybe we just don't have a route for "View KB" yet.
    // Let's just do nothing or expand later.
    // Wait, the user wants to "check the public database".
    console.log("Open KB", kb);
    // Maybe just go to read view of the first article?
    // TODO: contentApi to list KB content?
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
                    <button v-if="isCurrentUser" @click="router.push('/settings')"
                        class="text-xs font-bold uppercase tracking-widest text-neutral-400 hover:text-ink transition-colors">
                        Settings
                    </button>
                    <button v-if="isCurrentUser" @click="handleLogout"
                        class="text-xs font-bold uppercase tracking-widest text-red-500 hover:text-red-700 transition-colors">
                        Logout
                    </button>
                </div>
            </div>

            <!-- Profile Header -->
            <div class="flex flex-col md:flex-row gap-12 items-start mb-16">
                <div class="w-32 h-32 flex-shrink-0 bg-ash grayscale overflow-hidden">
                    <img :src="profile.avatar_url || `https://api.dicebear.com/9.x/notionists/svg?seed=${profile.username}`"
                        class="w-full h-full object-cover mix-blend-multiply" />
                </div>

                <div class="flex-1 space-y-6">
                    <div>
                        <h1 class="text-4xl font-bold tracking-tighter mb-1">{{ profile.display_name }}</h1>
                        <p class="text-sm font-mono text-neutral-400 uppercase tracking-widest">@{{ profile.username }}
                            • {{ profile.location }}</p>
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
                            <div class="text-[10px] font-bold uppercase tracking-widest text-neutral-400">Collections
                            </div>
                        </div>
                    </div>
                </div>

                <div v-if="profile.experience && profile.experience.length > 0" class="mt-8 pt-8 border-t border-dashed border-neutral-200">
                    <h3 class="text-[10px] font-bold uppercase tracking-widest text-neutral-400 mb-4">Journey</h3>
                    <div class="space-y-6 border-l border-neutral-200 ml-1 pl-6 relative">
                        <div v-for="(exp, i) in profile.experience" :key="i" class="relative group">
                            <!-- Timeline Dot -->
                            <div class="absolute -left-[31px] top-1.5 w-2.5 h-2.5 rounded-full border border-neutral-300 bg-paper group-hover:border-ink group-hover:bg-ink transition-colors"></div>
                            
                            <div class="flex flex-col sm:flex-row sm:items-baseline gap-1 sm:gap-2 mb-1">
                                <span class="font-bold text-ink">{{ exp.title }}</span>
                                <span class="text-neutral-400 text-sm">at</span>
                                <span class="font-serif italic text-neutral-600">{{ exp.organization }}</span>
                            </div>
                            <div class="text-[10px] font-mono text-neutral-400 uppercase tracking-widest mb-1">{{ exp.start_date }} — {{ exp.end_date || 'Present' }}</div>
                            <p v-if="exp.description" class="text-sm text-neutral-500 leading-relaxed">{{ exp.description }}</p>
                        </div>
                    </div>
                </div>
            </div>


            <!-- Tabs -->
            <div class="flex border-b border-ash/50 mb-12">
                <button @click="activeTab = 'transmissions'"
                    :class="['pb-4 pr-8 text-xs font-bold uppercase tracking-widest transition-colors', activeTab === 'transmissions' ? 'text-ink border-b-2 border-ink' : 'text-neutral-400 hover:text-ink']">
                    Transmissions
                </button>
                <button @click="activeTab = 'knowledge'"
                    :class="['pb-4 px-8 text-xs font-bold uppercase tracking-widest transition-colors', activeTab === 'knowledge' ? 'text-ink border-b-2 border-ink' : 'text-neutral-400 hover:text-ink']">
                    Knowledge Bases
                </button>
                <button @click="activeTab = 'memos'"
                    :class="['pb-4 px-8 text-xs font-bold uppercase tracking-widest transition-colors', activeTab === 'memos' ? 'text-ink border-b-2 border-ink' : 'text-neutral-400 hover:text-ink']">
                    Memos
                </button>
            </div>

            <!-- Tab Content -->
            <div>

                <!-- Transmissions Tab -->
                <div v-if="activeTab === 'transmissions'"
                    class="animate-in fade-in slide-in-from-bottom-4 duration-300">
                    <div v-if="loadingEntries" class="space-y-4">
                        <div class="h-4 bg-ash w-3/4 animate-pulse"></div>
                        <div class="h-4 bg-ash w-1/2 animate-pulse"></div>
                    </div>

                    <div v-else-if="entries.length === 0" class="text-neutral-400 italic">
                        No transmissions detected.
                    </div>

                    <div v-else class="space-y-8">
                        <div v-for="post in entries" :key="post.id" class="group cursor-pointer"
                            @click="router.push(`/article/${post.id}`)">
                            <h4 class="text-xl font-bold group-hover:text-accent transition-colors">{{ post.title }}
                            </h4>
                            <div class="flex gap-4 mt-2 text-xs font-mono text-neutral-400 uppercase tracking-widest">
                                <span>{{ new Date(post.created_at).toLocaleDateString() }}</span>
                                <span>{{ post.category }}</span>
                                <span v-if="post.visibility !== 'Public'"
                                    class="px-1 border border-neutral-200 text-[10px] rounded">{{ post.visibility }}</span>
                            </div>
                        </div>
                    </div>
                </div>

                <!-- Knowledge Tab -->
                <div v-if="activeTab === 'knowledge'" class="animate-in fade-in slide-in-from-bottom-4 duration-300">
                    <div v-if="loadingKbs" class="space-y-4">
                        <div class="h-4 bg-ash w-3/4 animate-pulse"></div>
                    </div>

                    <div v-else-if="kbs.length === 0" class="text-neutral-400 italic">
                        No knowledge bases found.
                    </div>

                    <div v-else class="grid grid-cols-1 md:grid-cols-2 gap-6">
                        <div v-for="kb in kbs" :key="kb.id"
                            class="group cursor-pointer border border-ash/50 p-6 hover:border-ink transition-colors"
                            @click="openKb(kb)">
                            <div class="flex items-center justify-between mb-4">
                                <div class="w-10 h-10 bg-ash/30 flex items-center justify-center text-ink/40">
                                    <i class="ri-book-read-line text-lg"></i>
                                </div>
                                <span v-if="kb.visibility !== 'Public'"
                                    class="px-2 py-0.5 border border-neutral-200 text-[10px] uppercase font-bold text-neutral-400 rounded">{{ kb.visibility }}</span>
                            </div>
                            <h4 class="text-lg font-bold mb-2 group-hover:text-accent transition-colors">{{ kb.title }}
                            </h4>
                            <p class="text-sm text-neutral-500 line-clamp-2">
                                {{ kb.description || 'No description provided.' }}
                            </p>
                        </div>
                    </div>
                </div>

                <!-- Memos Tab -->
                <div v-if="activeTab === 'memos'" class="animate-in fade-in slide-in-from-bottom-4 duration-300">
                    <div v-if="loadingMemos" class="space-y-4">
                        <div class="h-4 bg-ash w-3/4 animate-pulse"></div>
                    </div>

                    <div v-else-if="memos.length === 0" class="text-neutral-400 italic">
                        No memos found.
                    </div>

                    <div v-else class="space-y-6 columns-1 md:columns-2 gap-6">
                        <div v-for="memo in memos" :key="memo.id"
                            class="break-inside-avoid border-l-2 border-ash/50 pl-4 py-2 hover:border-accent transition-colors">
                            <div
                                class="text-xs font-mono text-neutral-400 mb-2 uppercase tracking-tight flex justify-between">
                                <span>{{ new Date(memo.created_at).toLocaleDateString() }}</span>
                                <span v-if="memo.visibility !== 'Public'"
                                    class="px-1 border border-neutral-200 text-[10px] rounded">{{ memo.visibility }}</span>
                            </div>
                            <p class="text-sm text-neutral-700 whitespace-pre-wrap font-serif leading-relaxed">
                                {{ memo.content }}
                            </p>
                            <div class="mt-3 flex gap-2 flex-wrap">
                                <span v-for="tag in memo.tags" :key="tag" class="text-[10px] text-accent">#{{ tag
                                }}</span>
                            </div>
                        </div>
                    </div>
                </div>

            </div>
        </div>
    </div>
</template>
