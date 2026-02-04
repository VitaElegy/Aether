<template>
    <div class="p-8 max-w-7xl mx-auto animate-fade-in">
        <div class="mb-8">
            <h1 class="text-2xl font-bold text-ink font-serif mb-1">User Management</h1>
            <p class="text-ink/60 text-sm">Search and view users in the system.</p>
        </div>

        <!-- Search Bar -->
        <div class="mb-8 max-w-xl">
             <div class="relative">
                <i class="ri-search-line absolute left-3 top-1/2 -translate-y-1/2 text-ink/40"></i>
                <input 
                    v-model="searchQuery" 
                    @keydown.enter="handleSearch"
                    type="text" 
                    class="w-full bg-surface border border-ink/10 rounded-lg pl-10 pr-4 py-3 text-sm focus:border-accent focus:outline-none shadow-sm"
                    placeholder="Search by username (min 2 chars)..."
                />
                <button 
                    @click="handleSearch"
                    class="absolute right-2 top-1/2 -translate-y-1/2 px-3 py-1 bg-ink/5 hover:bg-ink/10 rounded text-xs font-bold text-ink/60 transition-colors"
                >
                    Search
                </button>
             </div>
        </div>

        <div v-if="loading" class="space-y-4">
            <div v-for="i in 3" :key="i" class="h-20 bg-ash/10 rounded animate-pulse"></div>
        </div>

        <!-- Empty State -->
        <div v-else-if="users.length === 0 && !loading" class="text-center py-12 text-ink/40">
            <div v-if="searchQuery">
                <i class="ri-search-line text-4xl mb-2 block"></i>
                <p>No users found matching "{{ searchQuery }}"</p>
            </div>
            <div v-else>
                <i class="ri-ghost-line text-4xl mb-2 block"></i>
                <p>No users found.</p>
            </div>
        </div>

        <!-- Initial State (Removed - Auto Load) -->

        <!-- Results -->
        <div v-else class="space-y-8">
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                <div v-for="user in users" :key="user.id" class="bg-surface p-6 rounded-xl border border-ink/5 flex items-center gap-4 animate-fade-in-up">
                    <!-- Avatar -->
                    <div class="w-12 h-12 rounded-full bg-gradient-to-br from-indigo-500 to-purple-500 text-white flex items-center justify-center font-bold text-lg shadow-inner flex-shrink-0">
                       <img v-if="user.avatar_url" :src="user.avatar_url" class="w-full h-full object-cover rounded-full" />
                       <span v-else>{{ user.username.charAt(0).toUpperCase() }}</span>
                    </div>
                    
                    <div class="overflow-hidden min-w-0">
                        <h3 class="font-bold text-ink truncate">{{ user.display_name || user.username }}</h3>
                        <p class="text-xs text-ink/40 truncate">@{{ user.username }}</p>
                        <p class="text-[10px] text-ink/20 font-mono mt-1 truncate">{{ user.id }}</p>
                    </div>

                    <!-- Actions -->
                    <div class="ml-auto">
                        <t-dropdown 
                            :options="[
                                { content: 'Permissions', value: 'permissions', prefixIcon: () => h(UserSettingIcon) }, 
                                { content: 'Delete', value: 'delete', theme: 'danger', prefixIcon: () => h(DeleteIcon) }
                            ]" 
                            @click="(data: any) => handleUserAction(data, user)"
                        >
                            <t-button variant="text" shape="circle"><t-icon name="more" /></t-button>
                        </t-dropdown>
                    </div>
                </div>
            </div>

            <!-- Loader / Load More -->
            <div class="text-center pt-4 pb-8">
                 <button 
                    v-if="hasMore && !loading" 
                    @click="loadMore"
                    class="px-6 py-2 bg-surface hover:bg-ink/5 border border-ink/10 rounded-full text-sm text-ink/60 transition-colors"
                 >
                    Load More
                 </button>
                 <div v-if="loading" class="inline-block animate-spin text-ink/40">
                    <i class="ri-loader-4-line text-2xl"></i>
                 </div>
                 <p v-if="!hasMore && users.length > 0" class="text-xs text-ink/20 mt-4">End of list</p>
            </div>
        </div>


        <!-- Permissions Drawer -->
        <UserPermissionDrawer 
            v-model:visible="showPermissionsDrawer"
            :user="selectedUser || undefined"
        />
    </div>
</template>

<script setup lang="ts">
import { ref, onMounted, h } from 'vue';
import { userApi, type UserSummary } from '@/api/user';
import { MessagePlugin, DialogPlugin } from 'tdesign-vue-next';
import { UserSettingIcon, DeleteIcon } from 'tdesign-icons-vue-next'; 
import axios from 'axios';
import UserPermissionDrawer from '@/components/admin/UserPermissionDrawer.vue';

const users = ref<UserSummary[]>([]);
const loading = ref(false);
const searchQuery = ref('');
const hasSearched = ref(false); // Used to toggle between "Initial" and "Results" view, but now we auto-load.

// Permission Drawer State
const showPermissionsDrawer = ref(false);
const selectedUser = ref<UserSummary | null>(null);

const openPermissions = (user: UserSummary) => {
    selectedUser.value = user;
    showPermissionsDrawer.value = true;
};

const handleUserAction = (action: any, user: UserSummary) => {
    if (action.value === 'permissions') {
        openPermissions(user);
    } else if (action.value === 'delete') {
        confirmDeleteUser(user);
    }
};

const confirmDeleteUser = (user: UserSummary) => {
    const confirmDialog = DialogPlugin.confirm({
        header: 'Delete User',
        body: `Are you sure you want to delete ${user.username}? This action cannot be undone.`,
        theme: 'danger',
        onConfirm: async () => {
            try {
                await axios.delete(`/api/users/${user.id}`);
                MessagePlugin.success('User deleted successfully');
                loadUsers(true); // Reload list
                confirmDialog.hide();
            } catch (e: any) {
                console.error(e);
                const msg = e.response?.data?.error || 'Failed to delete user';
                if (msg.includes('Insufficient system permissions')) {
                     MessagePlugin.error('Permission Denied: You need system-level access.');
                } else {
                     MessagePlugin.error(msg);
                }
                confirmDialog.hide();
            }
        }
    });
};


// Pagination State
const offset = ref(0);
const limit = 20;
const hasMore = ref(true);

const loadUsers = async (reset = false) => {
    if (reset) {
        offset.value = 0;
        users.value = [];
        hasMore.value = true;
        hasSearched.value = true;
    }

    loading.value = true;
    try {
        const { data } = await userApi.search(searchQuery.value, limit, offset.value);
        
        if (data.length < limit) {
            hasMore.value = false;
        }

        if (reset) {
            users.value = data;
        } else {
            users.value.push(...data);
        }
        
        offset.value += limit;
    } catch (e) {
        console.error('[UserManagement] Search Error:', e);
        MessagePlugin.error('Failed to load users');
        if (reset) users.value = [];
    } finally {
        loading.value = false;
    }
};

const handleSearch = () => {
    loadUsers(true);
};

const loadMore = () => {
    loadUsers(false);
};

onMounted(() => {
    loadUsers(true);
});
</script>
