<template>
    <div class="h-full flex flex-col p-4">
        <div class="flex items-center justify-between mb-6">
            <h3 class="text-xl font-bold font-serif text-ink">Project Team</h3>
            <div class="flex items-center gap-2">
                <button @click="showPermMatrix = !showPermMatrix" class="px-3 py-2 bg-ash/20 text-ink/60 rounded-lg text-xs font-bold uppercase tracking-wider flex items-center gap-2 hover:bg-ash/40 transition-colors" :class="{'bg-accent/10 text-accent': showPermMatrix}">
                    <i class="ri-shield-keyhole-line text-lg"></i> Permissions
                </button>
                <button @click="showInviteModal = true" class="px-4 py-2 bg-accent text-white rounded-lg text-xs font-bold uppercase tracking-wider flex items-center gap-2 hover:bg-accent/90 transition-colors">
                    <i class="ri-user-add-line text-lg"></i> Invite Member
                </button>
            </div>
        </div>

        <!-- VRKB-08: Permission Matrix Panel -->
        <div v-if="showPermMatrix" class="mb-6 bg-white border border-ash/20 rounded-xl overflow-hidden">
            <div class="p-4 border-b border-ash/10 bg-ash/5">
                <h4 class="text-sm font-bold text-ink uppercase tracking-wider">Permission Matrix</h4>
            </div>
            <div class="overflow-x-auto">
                <table class="w-full text-left border-collapse">
                    <thead>
                        <tr class="border-b border-ash/20 text-xs font-bold text-ink/40 uppercase tracking-wider">
                            <th class="p-3 sticky left-0 bg-white">Permission</th>
                            <th v-for="role in permissionMatrix.roles" :key="role" class="p-3 text-center">{{ role }}</th>
                        </tr>
                    </thead>
                    <tbody>
                        <tr v-for="perm in permissionMatrix.permissions" :key="perm.action" class="border-b border-ash/10 hover:bg-ash/5">
                            <td class="p-3 text-xs font-medium text-ink sticky left-0 bg-white">{{ formatPermAction(perm.action) }}</td>
                            <td v-for="role in permissionMatrix.roles" :key="role" class="p-3 text-center">
                                <i v-if="perm.allowed_roles.includes(role)" class="ri-checkbox-circle-fill text-green-500"></i>
                                <i v-else class="ri-close-circle-line text-ink/20"></i>
                            </td>
                        </tr>
                    </tbody>
                </table>
            </div>
        </div>

        <!-- Members Table -->
        <div class="bg-white border border-ash/20 rounded-xl overflow-hidden">
            <table class="w-full text-left border-collapse">
                <thead>
                    <tr class="bg-ash/50 border-b border-ash/20 text-xs font-bold text-ink/40 uppercase tracking-wider">
                        <th class="p-4">Member</th>
                        <th class="p-4">Role</th>
                        <th class="p-4">Joined</th>
                        <th class="p-4 text-right">Actions</th>
                    </tr>
                </thead>
                <tbody>
                    <tr v-for="member in members" :key="member.id" class="border-b border-ash/10 hover:bg-ash/5 transition-colors group">
                        <td class="p-4 flex items-center gap-3">
                            <div class="w-8 h-8 rounded-full bg-gradient-to-br from-blue-400 to-purple-500 text-white flex items-center justify-center font-bold text-xs shadow-sm">
                                {{ member.name.charAt(0) }}
                            </div>
                            <div>
                                <div class="font-bold text-ink text-sm">{{ member.name }}</div>
                                <div class="text-xs text-ink/40 font-mono">{{ member.email }}</div>
                            </div>
                        </td>
                        <td class="p-4">
                            <!-- VRKB-08: Role selector (inline) -->
                            <div v-if="editingRoleMemberId === member.id" class="flex items-center gap-2">
                                <select v-model="editingRoleValue" class="px-2 py-1 text-xs border border-ash/30 rounded-lg focus:ring-2 focus:ring-accent/20 focus:outline-none bg-white">
                                    <option v-for="r in validRoles" :key="r" :value="r">{{ r }}</option>
                                </select>
                                <button @click="saveRole(member)" class="text-green-500 hover:text-green-700"><i class="ri-check-line"></i></button>
                                <button @click="editingRoleMemberId = null" class="text-ink/40 hover:text-ink"><i class="ri-close-line"></i></button>
                            </div>
                            <span v-else class="px-2 py-1 rounded text-[10px] font-bold uppercase tracking-wider border cursor-pointer hover:ring-2 hover:ring-accent/20" :class="getRoleClass(member.role)" @click="startEditRole(member)">
                                {{ member.role }}
                            </span>
                        </td>
                        <td class="p-4 text-xs text-ink/60 font-mono">
                            {{ member.joined }}
                        </td>
                        <td class="p-4 text-right opacity-0 group-hover:opacity-100 transition-opacity">
                            <button @click="removeMember(member.id)" class="p-1.5 text-ink/40 hover:text-red-500 rounded transition-colors" title="Remove">
                                <i class="ri-delete-bin-line"></i>
                            </button>
                            <button @click="startEditRole(member)" class="p-1.5 text-ink/40 hover:text-accent rounded transition-colors" title="Edit Role">
                                <i class="ri-settings-3-line"></i>
                            </button>
                        </td>
                    </tr>
                </tbody>
            </table>
        </div>

        <InviteMemberModal 
            :is-open="showInviteModal"
            @close="showInviteModal = false"
            @invite="handleInvite"
        />
    </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { vrkbApi } from '@/api/vrkb';
import { useVrkbStore } from '@/stores/vrkb';
import InviteMemberModal from './InviteMemberModal.vue';

const store = useVrkbStore();
const members = ref<any[]>([]);
const showInviteModal = ref(false);
const showPermMatrix = ref(false);

// VRKB-08: Role editing state
const editingRoleMemberId = ref<string | null>(null);
const editingRoleValue = ref('researcher');
const validRoles = ['owner', 'lead', 'researcher', 'observer'];

// VRKB-08: Permission matrix data
const permissionMatrix = ref<{ roles: string[]; permissions: any[] }>({
    roles: validRoles,
    permissions: []
});

const loadTeam = async () => {
    if (!store.currentProject) return;
    try {
        members.value = await vrkbApi.getTeam(store.currentProject.id);
    } catch (e) {
        console.error("Failed to load team", e);
    }
};

// VRKB-08: Load permission matrix
const loadPermissionMatrix = async () => {
    if (!store.currentProject) return;
    try {
        permissionMatrix.value = await vrkbApi.getPermissionMatrix(store.currentProject.id);
    } catch (e) {
        console.error("Failed to load permission matrix", e);
    }
};

const handleInvite = async (user: any) => {
    if (!store.currentProject) return;
    try {
        await vrkbApi.addMember(store.currentProject.id, user.id, "researcher");
        await loadTeam();
        alert(`Invited ${user.username}`);
    } catch (e) {
        console.error("Failed to add member", e);
        alert("Failed to invite.");
    }
};

const removeMember = async (userId: string) => {
    if (!store.currentProject || !confirm("Remove this member?")) return;
    try {
        await vrkbApi.removeMember(store.currentProject.id, userId);
        await loadTeam();
    } catch (e) {
        console.error("Failed to remove member", e);
    }
};

// VRKB-08: Inline role editing
const startEditRole = (member: any) => {
    editingRoleMemberId.value = member.id;
    editingRoleValue.value = member.role?.toLowerCase() || 'researcher';
};

const saveRole = async (member: any) => {
    if (!store.currentProject) return;
    try {
        await vrkbApi.updateMemberRole(store.currentProject.id, member.id || member.user_id, editingRoleValue.value);
        editingRoleMemberId.value = null;
        await loadTeam();
    } catch (e) {
        console.error("Failed to update role", e);
    }
};

// VRKB-08: Format permission action for display
const formatPermAction = (action: string) => {
    return action.replace(/_/g, ' ').replace(/\b\w/g, c => c.toUpperCase());
};

onMounted(() => {
    loadTeam();
    loadPermissionMatrix();
});

const getRoleClass = (role: string) => {
    switch(role?.toLowerCase()) {
        case 'owner': return 'bg-purple-50 text-purple-600 border-purple-100';
        case 'lead': return 'bg-blue-50 text-blue-600 border-blue-100';
        case 'researcher': return 'bg-green-50 text-green-600 border-green-100';
        case 'observer': return 'bg-gray-50 text-gray-500 border-gray-200';
        default: return 'bg-ash text-ink/40 border-ash';
    }
};
</script>
