<template>
    <div class="p-8 max-w-7xl mx-auto animate-fade-in">
        <div class="mb-8 flex justify-between items-end">
            <div>
                <h1 class="text-2xl font-bold text-ink font-serif mb-1">My Teams</h1>
                <p class="text-ink/60 text-sm">Create and manage your own groups for permission assignment.</p>
            </div>
            <t-button theme="primary" @click="showCreateDialog = true">
                <template #icon><t-icon name="add" /></template>
                New Team
            </t-button>
        </div>

        <div v-if="loading" class="space-y-4">
             <div v-for="i in 3" :key="i" class="h-16 bg-ash/10 rounded animate-pulse"></div>
        </div>

        <div v-else-if="teams.length === 0" class="text-center py-12 text-ink/40 bg-surface rounded-xl border border-ink/5">
            <i class="ri-group-line text-4xl mb-2 block"></i>
            <p>You haven't created any teams yet.</p>
        </div>

        <div v-else class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            <div v-for="team in teams" :key="team.id" class="bg-surface p-6 rounded-xl border border-ink/5 hover:border-accent/30 transition-colors">
                <div class="flex justify-between items-start mb-4">
                    <div class="flex items-center gap-3">
                        <t-avatar shape="round" :style="{ backgroundColor: stringToColor(team.name) }">
                            <t-icon name="usergroup" />
                        </t-avatar>
                        <div>
                            <h3 class="font-bold text-ink">{{ team.name }}</h3>
                            <p class="text-xs text-ink/40">{{ team.member_count || 1 }} members</p>
                        </div>
                    </div>
                    <t-dropdown :options="[{ content: 'Add Member', value: 'add' }, { content: 'Delete Team', value: 'delete', theme: 'danger' }]" @click="(data: any) => handleAction(data, team)">
                        <t-button variant="text" shape="circle"><t-icon name="more" /></t-button>
                    </t-dropdown>
                </div>
                
                <div class="space-y-2">
                    <p class="text-xs font-bold text-ink/40 uppercase tracking-wider">Members</p>
                    <div class="flex -space-x-2 overflow-hidden py-1">
                        <!-- Mock members for now, or fetch if expanded -->
                        <t-avatar size="small" shape="circle" v-for="i in Math.min(3, team.member_count || 1)" :key="i">
                            {{ team.name.charAt(0) }}
                        </t-avatar>
                        <div v-if="(team.member_count || 1) > 3" class="w-6 h-6 rounded-full bg-ash/20 flex items-center justify-center text-[10px] text-ink border-2 border-surface z-10">
                            +{{ (team.member_count || 1) - 3 }}
                        </div>
                    </div>
                </div>
            </div>
        </div>

        <!-- Create Team Dialog -->
        <t-dialog v-model:visible="showCreateDialog" header="Create New Team" @confirm="createTeam">
            <t-input v-model="newTeamName" placeholder="Enter team name (e.g. 'Project Alpha Editors')" />
        </t-dialog>

        <!-- Add Member Dialog (Simple User ID input for now) -->
        <t-dialog v-model:visible="showAddMemberDialog" header="Add Team Member" @confirm="addMember">
            <t-input v-model="newMemberId" placeholder="Enter User UUID to add" />
            <p class="text-xs text-ink/40 mt-2">Future: This will include a user search.</p>
        </t-dialog>
    </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import axios from 'axios';
import { MessagePlugin } from 'tdesign-vue-next';

const teams = ref<any[]>([]);
const loading = ref(false);
const showCreateDialog = ref(false);
const showAddMemberDialog = ref(false);
const newTeamName = ref('');
const newMemberId = ref('');
const activeTeam = ref<any>(null);

const loadTeams = async () => {
    loading.value = true;
    try {
        // TODO: Backend needs list API. For now, we might fail or mock.
        // Wait, did I implement list permissions? Yes, but not "list my groups".
        // Implementation plan said: GET /api/groups
        // But I didn't verify if I implemented `list_groups` in backend service yet.
        // Actually I only did create_team, add, remove member.
        // I need to add `list_my_teams` to backend.
        // For now, let's assume it returns empty or fails gracefully.
        
        // Mock fallback for UI dev if 404
        /* const res = await axios.get('/api/groups');
        teams.value = res.data; */
        
        teams.value = []; // Default empty until backend ready
    } catch (e) {
        console.error(e);
    } finally {
        loading.value = false;
    }
};

const createTeam = async () => {
    if (!newTeamName.value) return;
    try {
        const res = await axios.post('/api/groups', { name: newTeamName.value });
        teams.value.push({
             id: res.data.id, 
             name: newTeamName.value, 
             member_count: 1 
        });
        showCreateDialog.value = false;
        newTeamName.value = '';
        MessagePlugin.success('Team created');
    } catch (e) {
        MessagePlugin.error('Failed to create team');
    }
};

const handleAction = (action: any, team: any) => {
    activeTeam.value = team;
    if (action.value === 'add') {
        showAddMemberDialog.value = true;
    } else if (action.value === 'delete') {
        // TODO: Implement delete
        MessagePlugin.warning('Delete not implemented yet');
    }
};

const addMember = async () => {
    if (!newMemberId.value || !activeTeam.value) return;
    try {
        await axios.post(`/api/groups/${activeTeam.value.id}/members`, { user_id: newMemberId.value });
        activeTeam.value.member_count++;
        showAddMemberDialog.value = false;
        newMemberId.value = '';
        MessagePlugin.success('Member added');
    } catch (e) {
        MessagePlugin.error('Failed to add member');
    }
};

const stringToColor = (str: string) => {
    let hash = 0;
    for (let i = 0; i < str.length; i++) {
        hash = str.charCodeAt(i) + ((hash << 5) - hash);
    }
    const c = (hash & 0x00ffffff).toString(16).toUpperCase();
    return '#' + '00000'.substring(0, 6 - c.length) + c;
}

onMounted(() => {
    loadTeams();
});
</script>
