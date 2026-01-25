<template>
    <div v-if="isOpen" class="fixed inset-0 z-50 flex items-center justify-center p-4">
        <!-- Backdrop -->
        <div class="absolute inset-0 bg-ink/20 backdrop-blur-sm" @click="close"></div>

        <!-- Modal -->
        <div class="relative w-full max-w-md bg-white rounded-xl shadow-2xl border border-ash/20 overflow-hidden flex flex-col max-h-[80vh]">
            <!-- Header -->
            <div class="p-4 border-b border-ash/10 flex items-center justify-between bg-paper-2">
                <h3 class="font-bold font-serif text-ink text-lg">Invite Member</h3>
                <button @click="close" class="w-8 h-8 rounded-full hover:bg-ash/50 flex items-center justify-center text-ink/40 transition-colors">
                    <i class="ri-close-line text-xl"></i>
                </button>
            </div>

            <!-- Search Bar -->
            <div class="p-4 border-b border-ash/10 bg-white">
                <div class="relative">
                    <i class="ri-search-line absolute left-3 top-1/2 -translate-y-1/2 text-ink/40"></i>
                    <input 
                        v-model="query" 
                        @input="handleSearch"
                        type="text" 
                        class="w-full pl-10 pr-4 py-2 bg-ash/30 rounded-lg text-sm font-medium text-ink focus:outline-none focus:ring-2 focus:ring-accent/20 transition-all placeholder-ink/40"
                        placeholder="Search by name or email..."
                        auto-focus
                    >
                </div>
            </div>

            <!-- Results List -->
            <div class="flex-1 overflow-y-auto min-h-0 p-2 space-y-1 bg-paper-2 h-[300px]">
                <div v-if="loading" class="flex items-center justify-center p-8 text-ink/40">
                    <i class="ri-loader-4-line animate-spin text-2xl"></i>
                </div>
                
                <div v-else-if="users.length === 0" class="flex flex-col items-center justify-center p-8 text-ink/40">
                     <i class="ri-user-search-line text-3xl mb-2 opacity-50"></i>
                     <p class="text-xs">No users found.</p>
                </div>

                <div 
                    v-for="user in users" 
                    :key="user.id" 
                    class="flex items-center justify-between p-3 rounded-lg hover:bg-white border border-transparent hover:border-ash/20 transition-all group"
                >
                    <div class="flex items-center gap-3">
                        <div class="w-10 h-10 rounded-full bg-gradient-to-br from-indigo-400 to-purple-500 text-white flex items-center justify-center font-bold shadow-sm">
                            {{ user.username.charAt(0).toUpperCase() }}
                        </div>
                        <div>
                            <div class="font-bold text-ink text-sm">{{ user.username }}</div>
                            <div class="text-xs text-ink/40 font-mono">{{ user.email }}</div>
                        </div>
                    </div>
                    <button 
                        @click="invite(user)" 
                        class="px-3 py-1.5 bg-ash/50 hover:bg-accent hover:text-white text-ink/60 rounded-md text-xs font-bold transition-colors"
                    >
                        Invite
                    </button>
                </div>
            </div>
            
            <!-- Default Suggestions (if search is empty) -->
            <div v-if="!query && users.length === 0" class="p-4 text-xs text-ink/40 text-center border-t border-ash/10">
                Start typing to search for collaborators.
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import { vrkbApi } from '@/api/vrkb';

const props = defineProps<{
    isOpen: boolean
}>();

const emit = defineEmits(['close', 'invite']);

const query = ref('');
const users = ref<any[]>([]);
const loading = ref(false);
let debounceTimeout: any;

const close = () => {
    emit('close');
    query.value = '';
    users.value = [];
};

const handleSearch = () => {
    if (debounceTimeout) clearTimeout(debounceTimeout);
    debounceTimeout = setTimeout(() => {
        search();
    }, 300);
};

const search = async () => {
    loading.value = true;
    try {
        // Empty query fetches suggested or all users handled by backend logic
        users.value = await vrkbApi.searchUsers(query.value);
    } catch (e) {
        console.error("Search failed", e);
    } finally {
        loading.value = false;
    }
};

const invite = (user: any) => {
    emit('invite', user);
};

// Initial load (suggestions) when opening
watch(() => props.isOpen, (newVal) => {
    if (newVal) {
        search();
    }
});
</script>
