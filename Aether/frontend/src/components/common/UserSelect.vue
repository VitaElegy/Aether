<script setup lang="ts">
import { ref, watch } from 'vue';
import axios from 'axios';

defineProps<{
    modelValue: any; // User object or null
}>();

const emit = defineEmits(['update:modelValue']);

const searchQuery = ref('');
const results = ref<any[]>([]);
const isSearching = ref(false);
const showResults = ref(false);

let debounceTimer: any = null;

const handleSearch = () => {
    if (debounceTimer) clearTimeout(debounceTimer);
    if (searchQuery.value.length < 2) {
        results.value = [];
        return;
    }

    debounceTimer = setTimeout(async () => {
        isSearching.value = true;
        try {
            const token = localStorage.getItem('token');
            const res = await axios.get('/api/users/search', {
                params: { q: searchQuery.value },
                headers: { Authorization: `Bearer ${token}` }
            });
            results.value = res.data;
            showResults.value = true;
        } catch (e) {
            console.error(e);
        } finally {
            isSearching.value = false;
        }
    }, 300);
};

const selectUser = (user: any) => {
    emit('update:modelValue', user);
    searchQuery.value = '';
    showResults.value = false;
};
</script>

<template>
    <div class="relative">
        <div class="flex items-center bg-paper border border-ink/10 rounded px-3 py-2 text-sm focus-within:border-accent">
            <i class="ri-search-line text-ink/40 mr-2"></i>
            <input 
                v-model="searchQuery" 
                @input="handleSearch"
                placeholder="Search users by name or email..." 
                class="flex-1 bg-transparent outline-none"
            />
        </div>

        <div v-if="showResults && results.length > 0" 
             class="absolute z-50 left-0 right-0 mt-1 bg-paper border border-ink/10 rounded-lg shadow-xl max-h-48 overflow-y-auto">
            <div v-for="user in results" :key="user.id" 
                 @click="selectUser(user)"
                 class="px-3 py-2 hover:bg-surface cursor-pointer flex items-center gap-3">
                <div class="w-8 h-8 rounded-full bg-surface border border-ink/5 overflow-hidden flex-shrink-0">
                    <img v-if="user.avatar_url" :src="user.avatar_url" class="w-full h-full object-cover"/>
                    <div v-else class="w-full h-full flex items-center justify-center text-xs font-bold text-ink/40">
                        {{ user.username.substring(0, 2).toUpperCase() }}
                    </div>
                </div>
                <div>
                    <div class="font-medium text-sm">{{ user.display_name || user.username }}</div>
                    <div class="text-xs text-ink/40">@{{ user.username }}</div>
                </div>
            </div>
        </div>
        <div v-if="showResults && results.length === 0 && !isSearching" class="absolute z-50 left-0 right-0 mt-1 bg-paper border border-ink/10 rounded-lg shadow-xl p-3 text-center text-xs text-ink/40">
            No users found.
        </div>
    </div>
</template>
