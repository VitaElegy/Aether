<template>
    <t-drawer
        v-model:visible="internalVisible"
        header="User Access & Permissions"
        size="large"
        :close-on-overlay-click="true"
        @close="handleClose"
    >
        <div class="space-y-6">
            <!-- User Header -->
            <div class="flex items-center gap-4 border-b border-ink/10 pb-4">
                <t-avatar size="large">{{ userInitials }}</t-avatar>
                <div>
                    <h3 class="text-lg font-bold">{{ user?.display_name || user?.username }}</h3>
                    <p class="text-ink/40 text-sm">Target User ID: {{ user?.id }}</p>
                </div>
            </div>

            <!-- Tabs -->
            <t-tabs v-model="activeTab">
                <t-tab-panel value="roles" label="Roles & Groups">
                    <div class="py-4 space-y-4">
                        <t-alert theme="info" class="mb-4">
                            <template #message>
                                Groups determine broad access roles. Users can belong to multiple groups.
                            </template>
                        </t-alert>
                        
                        <div v-if="loading" class="text-center py-4"><t-loading /></div>
                        <div v-else class="space-y-2">
                            <div v-for="group in allGroups" :key="group.id" class="flex items-center justify-between p-3 bg-ash/5 rounded">
                                <div class="flex items-center gap-3">
                                    <t-checkbox 
                                        :checked="userGroups.includes(group.id)"
                                        @change="(val: boolean) => toggleGroup(group.id, val)"
                                        :disabled="processing"
                                    />
                                    <span>{{ group.name }}</span>
                                </div>
                                <t-tag variant="light">Group</t-tag>
                            </div>
                        </div>
                    </div>
                </t-tab-panel>

                <t-tab-panel value="direct" label="Explicit Grants">
                    <div class="py-4 space-y-4">
                        <div class="flex justify-between items-center">
                            <p class="text-sm text-ink/60">Specific permissions granted directly to this user on resources.</p>
                            <t-button theme="primary" variant="outline" @click="showSearch = true">
                                <template #icon><t-icon name="add" /></template>
                                Add Grant
                            </t-button>
                        </div>

                        <t-table
                            :data="directGrants"
                            :columns="columns"
                            :loading="loading"
                            row-key="id"
                            stripe
                        >
                            <template #op="{ row }">
                                <t-popconfirm content="Are you sure you want to revoke this permission?" @confirm="revokeGrant(row)">
                                    <t-button theme="danger" variant="text" size="small">Revoke</t-button>
                                </t-popconfirm>
                            </template>
                        </t-table>
                    </div>
                </t-tab-panel>

                <t-tab-panel value="history" label="History">
                   <div class="py-4 text-center text-ink/40">
                       <t-icon name="time" size="large" class="mb-2" />
                       <p>Audit logs for permission changes will appear here.</p>
                       <t-button variant="text" size="small">Refresh Logs</t-button>
                   </div>
                </t-tab-panel>
            </t-tabs>
        </div>

        <!-- Resource Search Modal -->
        <ResourceSearchModal 
            v-model:visible="showSearch"
            title="Grant Access to Resource"
            @select="handleResourceSelect"
        />
        
        <!-- Relation Selection Dialog (Simple) -->
        <t-dialog v-model:visible="showRelationDialog" header="Select Permission Level" @confirm="confirmGrant">
            <t-radio-group v-model="selectedRelation" variant="default-filled">
                <t-radio-button value="viewer">Viewer</t-radio-button>
                <t-radio-button value="editor">Editor</t-radio-button>
                <t-radio-button value="owner">Owner</t-radio-button>
            </t-radio-group>
        </t-dialog>

    </t-drawer>
</template>

<script setup lang="ts">
import { ref, watch, computed } from 'vue';
import axios from 'axios';
import { MessagePlugin } from 'tdesign-vue-next';
import ResourceSearchModal from '@/components/common/ResourceSearchModal.vue';


const props = defineProps({
    visible: Boolean,
    user: Object, // User object
});
const emit = defineEmits(['update:visible']);

const internalVisible = ref(false);
const activeTab = ref('roles');
const loading = ref(false);
const processing = ref(false);
const showSearch = ref(false);
const showRelationDialog = ref(false);

const userGroups = ref<string[]>([]);
const directGrants = ref<any[]>([]);
// Mock Groups for now - TODO: fetch from backend
const allGroups = ref([
    { id: 'g1', name: 'Editors' },
    { id: 'g2', name: 'Viewers' },
    { id: 'g3', name: 'Admins' }
]);

const selectedResource = ref<any>(null);
const selectedRelation = ref('viewer');

watch(() => props.visible, (val: boolean) => {
    internalVisible.value = val;
    if (val && props.user) {
        fetchPermissions();
    }
});

const handleClose = () => emit('update:visible', false);

const userInitials = computed(() => {
    const name = props.user?.display_name || props.user?.username || '?';
    return name.substring(0, 2).toUpperCase();
});

const columns = [
    { colKey: 'entity_type', title: 'Type', width: 100 },
    { colKey: 'entity_id', title: 'Resource ID', ellipsis: true },
    { colKey: 'relation', title: 'Access Level' },
    { colKey: 'op', title: 'Action', width: 100, fixed: 'right' },
];

const fetchPermissions = async () => {
    loading.value = true;
    try {
        if (!props.user) return;
        const res = await axios.get(`/api/permissions/user/${props.user.id}`);
        // Backend returns { groups: [], direct_grants: [] }
        // For groups, backend currently returns basic UUIDs or stubbed group objects. 
        // We'll trust the backend response structure.
        userGroups.value = res.data.groups || [];
        
        // Direct grants from backend: (EntityId, EntityType, Relation) tuple probably?
        // Let's check backend service: "direct_grants": direct // List of (EntityId, EntityType, Relation)
        // Wait, backend returns tuples (Uuid, String, String).
        // Frontend expects object with keys. Map it.
        if (res.data.direct_grants) {
            directGrants.value = res.data.direct_grants.map((t: any) => ({
                id: `${t[1]}-${t[0]}-${t[2]}`, // Composite key
                entity_id: t[0],
                entity_type: t[1],
                relation: t[2]
            }));
        } else {
             directGrants.value = [];
        }

    } catch (e) {
        console.error(e);
        MessagePlugin.error('Failed to load permissions');
    } finally {
        loading.value = false;
    }
};


const toggleGroup = async (groupId: string, checked: boolean) => {
    processing.value = true;
    try {
        // TODO: Call API to add/remove group membership
        await new Promise(r => setTimeout(r, 300));
        if (checked) userGroups.value.push(groupId);
        else userGroups.value = userGroups.value.filter(g => g !== groupId);
        MessagePlugin.success('Group updated');
    } catch (e) {
        MessagePlugin.error('Update failed');
    } finally {
        processing.value = false;
    }
};

const handleResourceSelect = (item: any) => {
    selectedResource.value = item;
    showRelationDialog.value = true;
};

const confirmGrant = async () => {
    if (!selectedResource.value || !props.user) return;
    processing.value = true;
    try {
        await axios.post('/api/permissions/grant', {
            user_id: props.user.id,
            entity_id: selectedResource.value.id,
            relation: selectedRelation.value
        });
        
        // Optimistic update or refresh? Let's refresh to be safe and simple.
        await fetchPermissions();
        
        showRelationDialog.value = false;
        MessagePlugin.success('Access granted');
    } catch (e) {
        console.error(e);
        MessagePlugin.error('Grant failed');
    } finally {
        processing.value = false;
    }
};


const revokeGrant = async (row: any) => {
     if (!props.user) return;
     try {
        await axios.post('/api/permissions/revoke', {
            user_id: props.user.id,
            entity_id: row.entity_id,
            relation: row.relation
        });
        
        await fetchPermissions();
        MessagePlugin.success('Permission revoked');
    } catch (e) {
        console.error(e);
        MessagePlugin.error('Revoke failed');
    }
};
</script>
