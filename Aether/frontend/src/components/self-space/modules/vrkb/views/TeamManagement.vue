<template>
    <div class="h-full flex flex-col p-4">
        <div class="flex items-center justify-between mb-6">
            <h3 class="text-xl font-bold font-serif text-ink">Project Team</h3>
            <button @click="showInviteModal = true" class="px-4 py-2 bg-accent text-white rounded-lg text-xs font-bold uppercase tracking-wider flex items-center gap-2 hover:bg-accent/90 transition-colors">
                <i class="ri-user-add-line text-lg"></i> Invite Member
            </button>
        </div>

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
                            <span class="px-2 py-1 rounded text-[10px] font-bold uppercase tracking-wider border" :class="getRoleClass(member.role)">
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
                            <button class="p-1.5 text-ink/40 hover:text-accent rounded transition-colors" title="Edit Role">
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

const loadTeam = async () => {
    if (!store.currentProject) return;
    try {
        members.value = await vrkbApi.getTeam(store.currentProject.id);
    } catch (e) {
        console.error("Failed to load team", e);
    }
};

const handleInvite = async (user: any) => {
    if (!store.currentProject) return;
    try {
        await vrkbApi.addMember(store.currentProject.id, user.id, "Member");
        await loadTeam();
        // Optional: Keep modal open for multi-invite or close
        // showInviteModal.value = false;
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

onMounted(() => {
    loadTeam();
});

const getRoleClass = (role: string) => {
    switch(role) {
        case 'Owner': return 'bg-purple-50 text-purple-600 border-purple-100';
        case 'Admin': return 'bg-blue-50 text-blue-600 border-blue-100';
        case 'Editor': return 'bg-green-50 text-green-600 border-green-100';
        default: return 'bg-ash text-ink/40 border-ash';
    }
};
</script>
