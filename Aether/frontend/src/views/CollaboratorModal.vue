<template>
  <div v-if="visible" class="fixed inset-0 z-[100] flex items-center justify-center bg-black/80 backdrop-blur-md transition-all duration-300">
    <div class="bg-paper w-full max-w-md rounded-2xl shadow-2xl overflow-hidden border border-ash/50 transform transition-all">
      <!-- Header -->
      <div class="px-6 py-5 border-b border-ash/50 flex justify-between items-center bg-paper/50 backdrop-blur-sm">
        <h3 class="text-xl font-bold tracking-tight text-ink">Manage Collaborators</h3>
        <button @click="$emit('close')" class="text-ink/40 hover:text-red-500 transition-colors bg-ash/20 hover:bg-red-500/10 p-2 rounded-full">
          <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18"></line><line x1="6" y1="6" x2="18" y2="18"></line></svg>
        </button>
      </div>

      <!-- Content -->
      <div class="p-6 space-y-8 bg-paper">
        
        <!-- Add New -->
        <div class="space-y-3">
          <label class="text-[10px] font-black uppercase tracking-[0.2em] text-accent">Add Collaborator</label>
          <div class="relative group">
            <div class="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                <svg class="h-4 w-4 text-ink/30" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"/></svg>
            </div>
            <input 
              v-model="searchQuery" 
              @input="handleSearch"
              type="text" 
              placeholder="Search by username..." 
              class="w-full bg-ash/10 border border-ash/30 rounded-xl pl-10 pr-4 py-3 text-sm font-medium text-ink placeholder-ink/30 focus:outline-none focus:ring-2 focus:ring-accent/50 focus:bg-paper transition-all"
            />
            <!-- Search Results Dropdown -->
            <div v-if="searchResults.length > 0" class="absolute top-full left-0 right-0 mt-3 bg-paper border border-ash rounded-xl shadow-[0_10px_40px_-10px_rgba(0,0,0,0.3)] max-h-60 overflow-y-auto z-50 divide-y divide-ash/30">
              <div 
                v-for="user in searchResults" 
                :key="user.id"
                @click="addCollaborator(user)"
                class="px-4 py-3 hover:bg-accent/5 cursor-pointer flex items-center gap-4 transition-colors group/item"
              >
                <div class="w-10 h-10 rounded-full bg-gradient-to-br from-ash/20 to-ash/40 flex items-center justify-center text-ink font-black text-sm group-hover/item:from-accent/20 group-hover/item:to-accent/40 group-hover/item:text-accent transition-all">
                  {{ user.username.charAt(0).toUpperCase() }}
                </div>
                <div>
                  <div class="text-sm font-bold text-ink group-hover/item:text-accent transition-colors">{{ user.display_name || user.username }}</div>
                  <div class="text-[10px] font-mono text-ink/50 group-hover/item:text-accent/60">@{{ user.username }}</div>
                </div>
                <div class="ml-auto opacity-0 group-hover/item:opacity-100 transition-opacity">
                    <span class="text-[10px] font-bold uppercase bg-accent text-paper px-2 py-1 rounded-md">Add</span>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- List -->
        <div class="space-y-3">
           <label class="text-[10px] font-black uppercase tracking-[0.2em] text-ink/40">Current Access</label>
           <div v-if="collaborators.length === 0" class="text-sm text-ink/30 font-medium py-8 text-center border-2 border-dashed border-ash/30 rounded-xl">
             No collaborators yet.
           </div>
           <div v-else class="space-y-3 max-h-60 overflow-y-auto pr-1">
             <div v-for="collab in collaborators" :key="collab.user_id" class="flex items-center justify-between p-3 bg-ash/5 border border-ash/20 rounded-xl group hover:border-accent/30 transition-all">
                <div class="flex items-center gap-4">
                   <div class="w-10 h-10 rounded-full bg-blue-500/10 text-blue-500 flex items-center justify-center font-black text-sm">
                      {{ collab.username.charAt(0).toUpperCase() }}
                   </div>
                   <div>
                       <div class="text-sm font-bold text-ink">{{ collab.username }}</div>
                       <span class="text-[9px] uppercase font-black tracking-wider text-green-600 bg-green-500/10 px-2 py-0.5 rounded-sm">Editor</span>
                   </div>
                </div>
                <button @click="removeCollaborator(collab.user_id)" class="text-red-400 hover:text-red-600 hover:bg-red-500/10 p-2 rounded-lg transition-all" title="Remove Access">
                  <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="3 6 5 6 21 6"></polyline><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path></svg>
                </button>
             </div>
           </div>
        </div>

      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import axios from 'axios';
import { useAuthStore } from '../stores/auth';

const props = defineProps<{
  visible: boolean;
  articleId: string;
}>();

const emit = defineEmits(['close']);
const auth = useAuthStore();

const collaborators = ref<any[]>([]);
const searchQuery = ref('');
const searchResults = ref<any[]>([]);

const fetchCollaborators = async () => {
  if (!props.articleId) return;
  try {
    const res = await axios.get(`/api/content/${props.articleId}/collaborators`, {
      headers: { Authorization: `Bearer ${auth.token}` }
    });
    collaborators.value = res.data;
  } catch (e) {
    console.error('Failed to fetch collaborators', e);
  }
};

const handleSearch = async () => {
  if (searchQuery.value.length < 2) {
    searchResults.value = [];
    return;
  }
  try {
    const res = await axios.get(`/api/users/search?q=${searchQuery.value}`, {
      headers: { Authorization: `Bearer ${auth.token}` }
    });
    // Filter out already added users + self
    searchResults.value = res.data.filter((u: any) => 
      u.id !== auth.user?.id && 
      !collaborators.value.some(c => c.user_id === u.id)
    );
  } catch (e) {
    console.error('Search failed', e);
  }
};

const addCollaborator = async (user: any) => {
  try {
    await axios.post(`/api/content/${props.articleId}/collaborators`, { user_id: user.id }, {
      headers: { Authorization: `Bearer ${auth.token}` }
    });
    searchQuery.value = '';
    searchResults.value = [];
    await fetchCollaborators();
    await fetchCollaborators();
  } catch (e: any) {
    console.error(e);
    alert(e.response?.data?.error || 'Failed to add collaborator');
  }
};

const removeCollaborator = async (userId: string) => {
  if (!confirm('Remove this collaborator?')) return;
  try {
    await axios.delete(`/api/content/${props.articleId}/collaborators/${userId}`, {
      headers: { Authorization: `Bearer ${auth.token}` }
    });
    await fetchCollaborators();
  } catch (e) {
    alert('Failed to remove collaborator');
  }
};

watch(() => props.visible, (val) => {
  if (val) fetchCollaborators();
});
</script>
