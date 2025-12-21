<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import { useRouter } from 'vue-router';
import { useAuthStore } from '../stores/auth';
import axios from 'axios';

const router = useRouter();
const authStore = useAuthStore();

const form = ref({
  display_name: '',
  bio: '',
  email: '',
  avatar_url: ''
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
            avatar_url: authStore.user.avatar_url || ''
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

const save = async () => {
    status.value = 'saving';
    errorMessage.value = '';
    try {
        await authStore.updateUser({
            display_name: form.value.display_name,
            bio: form.value.bio,
            email: form.value.email,
            avatar_url: form.value.avatar_url
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
  <div class="min-h-screen w-full bg-paper flex items-center justify-center p-6">
    <div class="w-full max-w-lg relative">
      <!-- Header -->
      <div class="flex justify-between items-center mb-12">
        <button @click="router.back()" class="text-neutral-400 hover:text-ink transition-colors">
          <i class="ri-arrow-left-line text-xl"></i>
        </button>
        <h1 class="text-xs font-bold uppercase tracking-widest text-neutral-400">Settings</h1>
      </div>

      <div class="space-y-8">
          <!-- Avatar Preview -->
          <div class="flex items-center gap-6">
              <div class="w-20 h-20 bg-ash grayscale overflow-hidden relative group cursor-pointer" @click="triggerFileInput">
                   <img :src="form.avatar_url || `https://api.dicebear.com/9.x/notionists/svg?seed=${form.display_name}`" class="w-full h-full object-cover mix-blend-multiply" />
                   <div class="absolute inset-0 bg-black/50 hidden group-hover:flex items-center justify-center text-white text-xs uppercase tracking-widest font-bold">
                       Upload
                   </div>
              </div>
              <input type="file" ref="fileInput" class="hidden" accept="image/*" @change="handleFileChange" />

              <div class="flex-1">
                  <label class="block text-xs font-bold uppercase tracking-widest mb-2 text-neutral-400">Avatar URL</label>
                  <div class="flex gap-2">
                      <input v-model="form.avatar_url" type="text" placeholder="https://..." class="flex-1 bg-transparent border-b border-neutral-200 py-2 text-ink font-mono text-sm focus:outline-none focus:border-ink transition-colors" />
                      <button @click="triggerFileInput" class="text-xs font-bold uppercase tracking-widest text-ink hover:text-neutral-600">Upload</button>
                  </div>
              </div>
          </div>

          <!-- Name -->
          <div>
              <label class="block text-xs font-bold uppercase tracking-widest mb-2 text-neutral-400">Display Name</label>
              <input v-model="form.display_name" type="text" class="w-full bg-transparent border-b border-neutral-200 py-2 text-xl font-bold text-ink focus:outline-none focus:border-ink transition-colors" />
          </div>

          <!-- Bio -->
          <div>
              <label class="block text-xs font-bold uppercase tracking-widest mb-2 text-neutral-400">Bio</label>
              <textarea v-model="form.bio" rows="3" class="w-full bg-transparent border-b border-neutral-200 py-2 text-serif italic text-neutral-600 focus:outline-none focus:border-ink transition-colors resize-none"></textarea>
          </div>

          <!-- Email -->
          <div>
              <label class="block text-xs font-bold uppercase tracking-widest mb-2 text-neutral-400">Email</label>
              <input v-model="form.email" type="email" class="w-full bg-transparent border-b border-neutral-200 py-2 text-ink font-mono text-sm focus:outline-none focus:border-ink transition-colors" />
          </div>

          <!-- Actions -->
          <div class="pt-8 flex items-center justify-between">
              <div class="text-sm">
                  <span v-if="status === 'success'" class="text-green-600">Changes saved.</span>
                  <span v-if="status === 'uploading'" class="text-neutral-400">Uploading image...</span>
                  <span v-if="status === 'error'" class="text-red-500">{{ errorMessage }}</span>
              </div>
              <button @click="save" :disabled="status === 'saving' || status === 'uploading'" class="bg-ink text-paper px-8 py-3 text-xs font-bold uppercase tracking-widest hover:bg-neutral-800 transition-colors disabled:opacity-50">
                  {{ status === 'saving' ? 'Saving...' : 'Save Changes' }}
              </button>
          </div>
      </div>

    </div>
  </div>
</template>
