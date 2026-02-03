<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import { useRouter } from 'vue-router';
import { useAuthStore, type ExperienceItem } from '../stores/auth';
import { usePreferencesStore } from '../stores/preferences';
import axios from 'axios';

const router = useRouter();
const authStore = useAuthStore();
const prefStore = usePreferencesStore();

const activeTab = ref<'profile' | 'preferences'>('profile');

const form = ref({
    display_name: '',
    bio: '',
    email: '',
    avatar_url: '',
    experience: [] as ExperienceItem[]
});

const status = ref<'idle' | 'saving' | 'uploading' | 'success' | 'error'>('idle');
const errorMessage = ref('');
const fileInput = ref<HTMLInputElement | null>(null);

const syncForm = () => {
    if (authStore.user) {
        form.value = {
            display_name: authStore.user.display_name || authStore.user.username || '',
            bio: authStore.user.bio || '',
            email: authStore.user.email || '',
            avatar_url: authStore.user.avatar_url || '',
            experience: authStore.user.experience ? JSON.parse(JSON.stringify(authStore.user.experience)) : []
        };
    }
};

onMounted(async () => {
    if (!authStore.user || !authStore.user.username) {
        await authStore.fetchUser();
    }

    if (!authStore.user) {
        router.push('/login');
        return;
    }

    syncForm();
});

watch(() => authStore.user, () => {
    syncForm();
}, { deep: true });

const triggerFileInput = () => {
    fileInput.value?.click();
};

const handleFileChange = async (event: Event) => {
    const target = event.target as HTMLInputElement;
    if (target.files && target.files[0]) {
        const file = target.files[0];
        await uploadAvatar(file);
    }
};

const uploadAvatar = async (file: File) => {
    status.value = 'uploading';
    const formData = new FormData();
    formData.append('file', file);

    try {
        const token = localStorage.getItem('aether_token');
        const res = await axios.post('/api/upload', formData, {
            headers: {
                'Content-Type': 'multipart/form-data',
                'Authorization': `Bearer ${token}`
            }
        });
        form.value.avatar_url = res.data.url;
        status.value = 'idle';
    } catch (e: any) {
        status.value = 'error';
        errorMessage.value = e.response?.data?.error || 'Failed to upload image';
    }
};

const addExperience = () => {
    form.value.experience.push({
        id: crypto.randomUUID(),
        title: '',
        organization: '',
        start_date: '',
        description: ''
    });
};

const removeExperience = (index: number) => {
    form.value.experience.splice(index, 1);
};

const save = async () => {
    status.value = 'saving';
    errorMessage.value = '';
    try {
        await authStore.updateUser({
            display_name: form.value.display_name,
            bio: form.value.bio,
            email: form.value.email,
            avatar_url: form.value.avatar_url,
            experience: form.value.experience
        });
        status.value = 'success';
        setTimeout(() => status.value = 'idle', 2000);
    } catch (e: any) {
        status.value = 'error';
        errorMessage.value = e.response?.data?.error || 'Failed to save changes';
    }
};
</script>

<template>
    <div
        class="min-h-screen w-full bg-paper flex items-center justify-center p-6 md:p-12 transition-colors duration-500">
        <div class="w-full max-w-5xl flex flex-col md:flex-row min-h-[600px] gap-12 md:gap-24">

            <!-- Sidebar / Navigation -->
            <aside class="w-full md:w-64 flex flex-col pt-8">
                <div class="flex items-center gap-2 mb-12 cursor-pointer group" @click="router.push('/')">
                    <i class="ri-arrow-left-line text-neutral-300 group-hover:text-ink transition-colors"></i>
                    <span
                        class="text-[10px] font-mono uppercase tracking-[0.2em] text-neutral-300 group-hover:text-ink transition-colors">Return</span>
                </div>

                <nav class="space-y-6">
                    <button @click="activeTab = 'profile'"
                        class="w-full text-left text-xs font-bold uppercase tracking-widest transition-all duration-300 group flex items-center justify-between"
                        :class="activeTab === 'profile' ? 'text-ink pl-4 border-l-2 border-ink' : 'text-neutral-400 hover:text-ink hover:pl-2'">
                        <span>Identity</span>
                    </button>

                    <button @click="activeTab = 'preferences'"
                        class="w-full text-left text-xs font-bold uppercase tracking-widest transition-all duration-300 group flex items-center justify-between"
                        :class="activeTab === 'preferences' ? 'text-ink pl-4 border-l-2 border-ink' : 'text-neutral-400 hover:text-ink hover:pl-2'">
                        <span>Interface</span>
                    </button>

                    <button @click="router.push('/admin/templates')"
                        class="w-full text-left text-xs font-bold uppercase tracking-widest transition-all duration-300 group flex items-center justify-between text-neutral-400 hover:text-ink hover:pl-2">
                        <span>Templates (Admin)</span>
                    </button>
                </nav>
            </aside>

            <!-- Main Content Area -->
            <main class="flex-1 pt-8 md:pt-12">

                <!-- TAB: Profile -->
                <div v-if="activeTab === 'profile'"
                    class="space-y-12 max-w-lg animate-in fade-in slide-in-from-bottom-4 duration-500">
                    <div class="border-b border-neutral-100 pb-4">
                        <h2 class="text-3xl font-bold tracking-tight text-ink">Public Profile</h2>
                        <p class="text-xs text-neutral-400 font-mono uppercase tracking-widest mt-2">Manage your digital
                            presence</p>
                    </div>

                    <!-- Avatar -->
                    <div class="flex items-center gap-8">
                        <div class="w-24 h-24 bg-ash grayscale overflow-hidden relative group cursor-pointer ring-1 ring-neutral-100"
                            @click="triggerFileInput">
                            <img :src="form.avatar_url || `https://api.dicebear.com/9.x/notionists/svg?seed=${form.display_name}`"
                                class="w-full h-full object-cover mix-blend-multiply transition-transform duration-700 group-hover:scale-110" />
                            <div
                                class="absolute inset-0 bg-black/40 hidden group-hover:flex items-center justify-center text-white backdrop-blur-[1px]">
                                <i class="ri-upload-2-line text-xl"></i>
                            </div>
                        </div>
                        <div class="flex-1">
                            <input type="file" ref="fileInput" class="hidden" accept="image/*"
                                @change="handleFileChange" />
                            <label
                                class="block text-[10px] font-mono uppercase tracking-widest text-neutral-400 mb-2">Avatar
                                Source</label>
                            <input v-model="form.avatar_url" type="text" placeholder="HTTPS://..."
                                class="w-full bg-transparent border-b border-neutral-200 py-2 text-xs font-mono text-ink focus:outline-none focus:border-black transition-colors" />
                        </div>
                    </div>

                    <!-- Form Fields -->
                    <div class="space-y-8">
                        <div class="group">
                            <label
                                class="block text-[10px] font-bold uppercase tracking-widest text-neutral-400 mb-2 group-focus-within:text-ink transition-colors">Display
                                Name</label>
                            <input v-model="form.display_name" type="text"
                                class="w-full bg-transparent border-b border-neutral-200 py-2 text-xl font-bold text-ink focus:outline-none focus:border-black transition-colors" />
                        </div>

                        <div class="group">
                            <label
                                class="block text-[10px] font-bold uppercase tracking-widest text-neutral-400 mb-2 group-focus-within:text-ink transition-colors">Bio</label>
                            <textarea v-model="form.bio" rows="3"
                                class="w-full bg-transparent border-b border-neutral-200 py-2 text-lg font-serif italic text-neutral-600 focus:text-ink focus:outline-none focus:border-black transition-colors resize-none"></textarea>
                        </div>

                        <div class="group">
                            <label
                                class="block text-[10px] font-bold uppercase tracking-widest text-neutral-400 mb-2 group-focus-within:text-ink transition-colors">Email</label>
                            <input v-model="form.email" type="email"
                                class="w-full bg-transparent border-b border-neutral-200 py-2 text-ink font-mono text-sm focus:outline-none focus:border-black transition-colors" />
                        </div>
                    </div>

                    <!-- Experience Section -->
                    <div class="pt-8 border-t border-neutral-100">
                         <div class="flex items-center justify-between mb-8">
                             <div>
                                <h3 class="text-lg font-bold text-ink">Experience</h3>
                                <p class="text-[10px] text-neutral-400 font-mono uppercase tracking-widest mt-1">Career & Education Ops</p>
                             </div>
                             <button @click="addExperience" 
                                class="w-8 h-8 flex items-center justify-center rounded-full border border-neutral-200 hover:border-black hover:bg-black hover:text-white transition-all">
                                 <i class="ri-add-line"></i>
                             </button>
                         </div>

                         <div class="space-y-8">
                             <div v-for="(item, index) in form.experience" :key="index" 
                                class="relative p-6 bg-neutral-50/50 rounded-xl border border-neutral-100 group transition-all hover:bg-neutral-50 hover:shadow-sm">
                                 
                                 <button @click="removeExperience(index)"
                                    class="absolute top-4 right-4 text-neutral-300 hover:text-red-500 transition-colors opacity-0 group-hover:opacity-100">
                                     <i class="ri-delete-bin-line"></i>
                                 </button>

                                 <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mb-4">
                                     <div class="group/field">
                                         <label class="block text-[10px] font-bold uppercase tracking-widest text-neutral-400 mb-1">Title</label>
                                         <input v-model="item.title" type="text" placeholder="e.g. Senior Eng"
                                             class="w-full bg-transparent border-b border-neutral-200 py-1 text-sm font-bold text-ink focus:outline-none focus:border-black transition-colors" />
                                     </div>
                                     <div class="group/field">
                                         <label class="block text-[10px] font-bold uppercase tracking-widest text-neutral-400 mb-1">Organization</label>
                                         <input v-model="item.organization" type="text" placeholder="e.g. Acme Corp"
                                             class="w-full bg-transparent border-b border-neutral-200 py-1 text-sm font-bold text-ink focus:outline-none focus:border-black transition-colors" />
                                     </div>
                                 </div>

                                 <div class="grid grid-cols-2 gap-4 mb-4">
                                      <div class="group/field">
                                         <label class="block text-[10px] font-bold uppercase tracking-widest text-neutral-400 mb-1">Start</label>
                                         <input v-model="item.start_date" type="text" placeholder="YYYY-MM"
                                             class="w-full bg-transparent border-b border-neutral-200 py-1 text-xs font-mono text-ink focus:outline-none focus:border-black transition-colors" />
                                     </div>
                                     <div class="group/field">
                                         <label class="block text-[10px] font-bold uppercase tracking-widest text-neutral-400 mb-1">End</label>
                                         <input v-model="item.end_date" type="text" placeholder="Present"
                                             class="w-full bg-transparent border-b border-neutral-200 py-1 text-xs font-mono text-ink focus:outline-none focus:border-black transition-colors" />
                                     </div>
                                 </div>

                                 <div class="group/field">
                                     <label class="block text-[10px] font-bold uppercase tracking-widest text-neutral-400 mb-1">Description</label>
                                     <textarea v-model="item.description" rows="2"
                                         class="w-full bg-transparent border-b border-neutral-200 py-1 text-sm text-neutral-600 focus:text-ink focus:outline-none focus:border-black transition-colors resize-none"></textarea>
                                 </div>
                             </div>
                             
                             <div v-if="form.experience.length === 0" class="text-center py-8 text-neutral-300 text-xs italic">
                                 No experience listed. Add your journey.
                             </div>
                         </div>
                    </div>

                    <div class="pt-8 flex items-center justify-between">
                        <button @click="save" :disabled="status === 'saving'"
                            class="bg-ink text-paper px-8 py-3 text-xs font-bold uppercase tracking-[0.2em] hover:bg-neutral-800 transition-all disabled:opacity-50">
                            {{ status === 'saving' ? 'SAVING...' : 'SAVE CHANGES' }}
                        </button>
                        <span
                            class="text-[10px] text-neutral-400 font-mono uppercase tracking-widest transition-opacity duration-300"
                            :class="status === 'success' ? 'opacity-100' : 'opacity-0'">
                            Configuration Synced
                        </span>
                    </div>
                </div>

                <!-- TAB: Preferences -->
                <div v-if="activeTab === 'preferences'"
                    class="space-y-12 max-w-lg animate-in fade-in slide-in-from-bottom-4 duration-500">
                    <div class="border-b border-neutral-100 pb-4">
                        <h2 class="text-3xl font-bold tracking-tight text-ink">System Interface</h2>
                        <p class="text-xs text-neutral-400 font-mono uppercase tracking-widest mt-2">Customize your
                            viewing experience</p>
                    </div>

                    <div class="space-y-8">

                        <!-- Toggle Item -->
                        <div class="flex items-center justify-between group py-4 hover:bg-neutral-50 dark:hover:bg-white/5 -mx-4 px-4 rounded-lg transition-colors cursor-pointer"
                            @click="prefStore.toggleTheme">
                            <div>
                                <div class="font-bold text-ink mb-1 group-hover:translate-x-1 transition-transform">Dark
                                    Mode</div>
                                <div class="text-xs text-neutral-400">Reduce luminance for low-light environments.</div>
                            </div>
                            <div class="relative w-10 h-10 flex items-center justify-center">
                                <div class="w-8 h-4 rounded-full border border-neutral-300 transition-colors duration-300"
                                    :class="{ 'bg-ink border-ink': prefStore.theme === 'dark' }"></div>
                                <div class="absolute w-3 h-3 bg-neutral-400 rounded-full transition-all duration-300 shadow-sm"
                                    :class="{ 'translate-x-2 bg-paper': prefStore.theme === 'dark', '-translate-x-2': prefStore.theme !== 'dark' }">
                                </div>
                            </div>
                        </div>

                        <!-- Toggle Item -->
                        <div class="flex items-center justify-between group py-4 hover:bg-neutral-50 dark:hover:bg-white/5 -mx-4 px-4 rounded-lg transition-colors cursor-pointer"
                            @click="prefStore.toggleSidebar">
                            <div>
                                <div class="font-bold text-ink mb-1 group-hover:translate-x-1 transition-transform">
                                    Reader Sidebar</div>
                                <div class="text-xs text-neutral-400">Toggle the visibility of the Table of Contents.
                                </div>
                            </div>
                            <div class="relative w-10 h-10 flex items-center justify-center">
                                <div class="w-8 h-4 rounded-full border border-neutral-300 transition-colors duration-300"
                                    :class="{ 'bg-ink border-ink': prefStore.isSidebarCollapsed }"></div>
                                <div class="absolute w-3 h-3 bg-neutral-400 rounded-full transition-all duration-300 shadow-sm"
                                    :class="{ 'translate-x-2 bg-paper': prefStore.isSidebarCollapsed, '-translate-x-2': !prefStore.isSidebarCollapsed }">
                                </div>
                            </div>
                        </div>

                        <!-- Default Commit Message -->
                        <div class="pt-8 border-t border-neutral-100">
                             <div class="font-bold text-ink mb-2">Default Change Reason</div>
                             <div class="text-xs text-neutral-400 mb-4">Template for your version history messages.</div>
                             <textarea
                                v-model="prefStore.defaultCommitMessage"
                                rows="2"
                                class="w-full bg-transparent border-b border-neutral-200 py-2 text-sm font-mono text-ink focus:outline-none focus:border-black transition-colors resize-none placeholder:text-neutral-300"
                                placeholder="Update content..."
                             ></textarea>
                        </div>

                    </div>
                </div>

            </main>
        </div>
    </div>
</template>
