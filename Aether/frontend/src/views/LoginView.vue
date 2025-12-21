<script setup lang="ts">
import { ref, reactive } from 'vue';
import { useRouter } from 'vue-router';
import { useAuthStore } from '../stores/auth';
import { MessagePlugin } from 'tdesign-vue-next';
import axios from 'axios';

const router = useRouter();
const authStore = useAuthStore();

const isLogin = ref(true);
const loading = ref(false);

const form = reactive({
    username: '',
    password: '',
    email: ''
});

const toggleMode = () => {
    isLogin.value = !isLogin.value;
    form.username = '';
    form.password = '';
    form.email = '';
};

const handleSubmit = async () => {
    loading.value = true;
    try {
        if (isLogin.value) {
            const res = await axios.post('/api/auth/login', { username: form.username, password: form.password });
            const token = res.data.token; // Use the real token from backend
            authStore.login(token, res.data.user);
            router.push('/');
        } else {
            await axios.post('/api/auth/register', { username: form.username, email: form.email, password: form.password });
            MessagePlugin.success('Account created.');
            isLogin.value = true;
        }
    } catch (err: any) {
        MessagePlugin.error(err.response?.data?.error || 'Authentication failed');
    } finally {
        loading.value = false;
    }
};
</script>

<template>
    <div class="h-screen w-full flex items-center justify-center p-6 bg-ash">
        <div class="w-full max-w-sm bg-paper p-12 border border-neutral-200 shadow-sm relative overflow-hidden group">
            <!-- Top Accent -->
            <div
                class="absolute top-0 left-0 w-full h-1 bg-ink scale-x-0 group-hover:scale-x-100 transition-transform duration-500 origin-left">
            </div>

            <div class="mb-12">
                <h1 class="text-4xl font-bold tracking-tighter mb-2">Aether.</h1>
                <p class="text-neutral-500 font-mono text-xs uppercase tracking-widest">
                    {{ isLogin ? 'Access Terminal' : 'New Identification' }}
                </p>
            </div>

            <form @submit.prevent="handleSubmit" class="space-y-8">
                <div class="space-y-6">
                    <div class="relative">
                        <input v-model="form.username" type="text" placeholder="Username"
                            class="w-full border-b border-neutral-200 py-2 text-lg font-medium focus:outline-none focus:border-ink transition-colors placeholder:text-neutral-300 bg-transparent"
                            required />
                    </div>

                    <div v-if="!isLogin" class="relative">
                        <input v-model="form.email" type="email" placeholder="Email"
                            class="w-full border-b border-neutral-200 py-2 text-lg font-medium focus:outline-none focus:border-ink transition-colors placeholder:text-neutral-300 bg-transparent"
                            required />
                    </div>

                    <div class="relative">
                        <input v-model="form.password" type="password" placeholder="Password"
                            class="w-full border-b border-neutral-200 py-2 text-lg font-medium focus:outline-none focus:border-ink transition-colors placeholder:text-neutral-300 bg-transparent"
                            required />
                    </div>
                </div>

                <div class="pt-4">
                    <button type="submit" :disabled="loading"
                        class="w-full bg-ink text-paper py-4 text-sm font-bold uppercase tracking-widest hover:bg-neutral-800 disabled:opacity-50 transition-colors flex justify-between px-6 items-center group/btn">
                        <span>{{ isLogin ? 'Enter' : 'Create' }}</span>
                        <i class="ri-arrow-right-line group-hover/btn:translate-x-1 transition-transform"></i>
                    </button>
                </div>
            </form>

            <div class="mt-8 text-center">
                <button @click="toggleMode"
                    class="text-xs font-mono text-neutral-400 hover:text-ink uppercase tracking-widest transition-colors">
                    {{ isLogin ? 'Need an account?' : 'Have an account?' }}
                </button>
            </div>
        </div>
    </div>
</template>
