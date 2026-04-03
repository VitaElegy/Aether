<template>
    <div class="h-full flex flex-col bg-paper-2 rounded-xl border border-ash/20 overflow-hidden">
        <!-- Toolbar -->
        <div class="flex items-center justify-between p-4 border-b border-ash/20 bg-white/50 backdrop-blur-sm">
            <div class="flex items-center gap-3">
                <div class="flex items-center text-ink/40 text-sm font-mono">
                    <span class="hover:text-ink cursor-pointer transition-colors">root</span>
                    <span class="mx-2">/</span>
                    <span class="text-ink font-bold">{{ currentPath || '' }}</span>
                </div>
            </div>
            <div class="flex gap-2">
                <button @click="openUploadDialog" class="p-2 text-ink/60 hover:text-accent hover:bg-accent/10 rounded-lg transition-colors" title="Upload File">
                    <i class="ri-upload-cloud-2-line text-lg"></i>
                </button>
                <button @click="pickExistingAsset" class="p-2 text-ink/60 hover:text-green-600 hover:bg-green-50 rounded-lg transition-colors" title="Link Existing Asset">
                    <i class="ri-link text-lg"></i>
                </button>
                <button class="p-2 text-ink/60 hover:text-ink hover:bg-ash rounded-lg transition-colors" title="New Folder">
                    <i class="ri-folder-add-line text-lg"></i>
                </button>
                <div class="w-px h-6 bg-ash/50 mx-1"></div>
                <button class="p-2 text-ink/60 hover:text-ink hover:bg-ash rounded-lg transition-colors" :class="{'text-accent bg-accent/10': viewMode === 'grid'}" @click="viewMode = 'grid'">
                    <i class="ri-grid-line text-lg"></i>
                </button>
                <button class="p-2 text-ink/60 hover:text-ink hover:bg-ash rounded-lg transition-colors" :class="{'text-accent bg-accent/10': viewMode === 'list'}" @click="viewMode = 'list'">
                    <i class="ri-list-check text-lg"></i>
                </button>
            </div>
        </div>

        <!-- File Area -->
        <div class="flex-1 overflow-y-auto p-4" @dragover.prevent @drop.prevent="handleDrop">
            <!-- Loading State -->
            <div v-if="loading" class="flex items-center justify-center h-full text-ink/20">
                <i class="ri-loader-4-line text-3xl animate-spin"></i>
            </div>

            <!-- Empty State -->
            <div v-else-if="assets.length === 0" class="flex flex-col items-center justify-center h-full text-ink/40 border-2 border-dashed border-ash/30 rounded-xl m-4">
                <i class="ri-hard-drive-2-line text-4xl mb-4 opacity-30"></i>
                <p class="font-serif font-bold text-lg">No Assets Found</p>
                <p class="text-xs mt-2">Drag and drop files here, or use the link button to attach existing assets</p>
            </div>

            <!-- Grid View -->
            <div v-else-if="viewMode === 'grid'" class="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-5 gap-4">
                <div 
                    v-for="asset in assets" 
                    :key="asset.id"
                    class="group relative bg-white border border-ash/20 rounded-xl p-4 flex flex-col items-center text-center hover:border-accent/50 hover:shadow-md transition-all cursor-pointer aspect-square"
                    @click="showAssetDetail(asset)"
                >
                    <div class="flex-1 flex items-center justify-center text-ink/20 group-hover:text-accent/80 transition-colors">
                        <i :class="getFileIcon(asset.type)" class="text-4xl"></i>
                    </div>
                    <div class="w-full mt-3">
                        <p class="text-xs font-bold text-ink truncate w-full">{{ asset.name }}</p>
                        <p class="text-[10px] text-ink/40 font-mono mt-0.5">{{ formatBytes(asset.size) }}</p>
                    </div>
                    <!-- Quick actions overlay -->
                    <div class="absolute top-2 right-2 flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                        <button @click.stop="viewUsage(asset.id)" class="w-6 h-6 rounded bg-blue-50 text-blue-500 hover:bg-blue-100 flex items-center justify-center" title="View Usage">
                            <i class="ri-links-line text-xs"></i>
                        </button>
                        <button @click.stop="unlinkAsset(asset.id)" class="w-6 h-6 rounded bg-orange-50 text-orange-500 hover:bg-orange-100 flex items-center justify-center" title="Unlink">
                            <i class="ri-link-unlink text-xs"></i>
                        </button>
                        <button @click.stop="deleteAsset(asset.id)" class="w-6 h-6 rounded bg-red-50 text-red-500 hover:bg-red-100 flex items-center justify-center" title="Delete">
                            <i class="ri-delete-bin-line text-xs"></i>
                        </button>
                    </div>
                </div>
            </div>

            <!-- List View -->
            <div v-else class="flex flex-col gap-1">
                <div 
                    v-for="asset in assets" 
                    :key="asset.id"
                    class="flex items-center justify-between p-3 bg-white border border-ash/20 rounded-lg hover:border-accent/30 hover:bg-ash/10 transition-colors cursor-pointer group"
                    @click="showAssetDetail(asset)"
                >
                    <div class="flex items-center gap-3">
                        <i :class="getFileIcon(asset.type)" class="text-xl text-ink/40 group-hover:text-accent"></i>
                        <span class="text-sm font-medium text-ink">{{ asset.name }}</span>
                    </div>
                    <div class="flex items-center gap-6 text-xs text-ink/40 font-mono">
                        <span>{{ formatBytes(asset.size) }}</span>
                        <span>{{ formatDate(asset.updated_at) }}</span>
                        <div class="w-32 flex justify-end gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
                            <button @click.stop="viewUsage(asset.id)" class="hover:text-blue-500" title="View Usage"><i class="ri-links-line"></i></button>
                            <button class="hover:text-accent"><i class="ri-download-line"></i></button>
                            <button @click.stop="unlinkAsset(asset.id)" class="hover:text-orange-500" title="Unlink"><i class="ri-link-unlink"></i></button>
                            <button @click.stop="deleteAsset(asset.id)" class="hover:text-red-500"><i class="ri-delete-bin-line"></i></button>
                        </div>
                    </div>
                </div>
            </div>
        </div>

        <!-- VRKB-06: Asset Detail / Usage Panel -->
        <div v-if="selectedAsset" class="border-t border-ash/20 bg-white/80 backdrop-blur-sm p-4">
            <div class="flex items-center justify-between mb-3">
                <h4 class="font-bold text-sm text-ink">{{ selectedAsset.name || 'Asset Detail' }}</h4>
                <button @click="selectedAsset = null" class="text-ink/40 hover:text-ink"><i class="ri-close-line"></i></button>
            </div>
            <div class="flex items-center gap-4 text-xs text-ink/60">
                <span class="font-mono">{{ formatBytes(selectedAsset.size) }}</span>
                <span class="font-mono">{{ selectedAsset.mime_type }}</span>
            </div>
            <!-- Usage list -->
            <div v-if="assetUsages.length > 0" class="mt-3">
                <p class="text-[10px] font-bold uppercase tracking-wider text-ink/40 mb-2">Linked To</p>
                <div v-for="usage in assetUsages" :key="`${usage.target_type}-${usage.target_id}`" class="flex items-center gap-2 py-1 text-xs text-ink/70">
                    <i :class="usage.target_type === 'finding' ? 'ri-bug-line' : usage.target_type === 'doc' ? 'ri-file-text-line' : 'ri-folder-line'" class="text-accent"></i>
                    <span>{{ usage.target_title }}</span>
                    <span class="text-ink/30 font-mono">({{ usage.target_type }})</span>
                </div>
            </div>
            <div v-else class="mt-3 text-xs text-ink/40 italic">No linked references found.</div>
        </div>

        <!-- Hidden file input for upload -->
        <input ref="fileInput" type="file" multiple class="hidden" @change="handleFileInput" />
    </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { DateTime } from 'luxon';
import { vrkbApi } from '@/api/vrkb';
import { useAssetPicker } from '@/composables/useAssetPicker';

const props = defineProps<{
    projectId: string
}>();

const viewMode = ref<'grid' | 'list'>('grid');
const loading = ref(false);
const currentPath = ref('');
const assets = ref<any[]>([]);
const selectedAsset = ref<any>(null);
const assetUsages = ref<any[]>([]);
const fileInput = ref<HTMLInputElement | null>(null);

const assetPicker = useAssetPicker();

const loadAssets = async () => {
    loading.value = true;
    try {
        assets.value = await vrkbApi.listAssets(props.projectId);
    } catch(e) {
        console.error("Failed to list assets", e);
    } finally {
        loading.value = false;
    }
};

onMounted(() => {
    loadAssets();
});

// --- VRKB-06: Upload dialog ---
const openUploadDialog = () => {
    fileInput.value?.click();
};

const handleFileInput = async (e: Event) => {
    const target = e.target as HTMLInputElement;
    const files = target.files;
    if (!files) return;

    loading.value = true;
    try {
        for (let i = 0; i < files.length; i++) {
            const uploaded = await vrkbApi.uploadAsset(files[i]);
            // Auto-link to project
            await vrkbApi.linkAsset({
                asset_id: uploaded.id,
                target_type: 'project',
                target_id: props.projectId,
            });
        }
        await loadAssets();
    } catch (e) {
        console.error("Upload failed", e);
    } finally {
        loading.value = false;
        if (target) target.value = '';
    }
};

// --- VRKB-06: Pick existing asset from Asset Center ---
const pickExistingAsset = async () => {
    const result = await assetPicker.openPicker({ multiple: true });
    if (result.cancelled || result.assets.length === 0) return;

    loading.value = true;
    try {
        for (const asset of result.assets) {
            await vrkbApi.linkAsset({
                asset_id: asset.id,
                target_type: 'project',
                target_id: props.projectId,
            });
        }
        await loadAssets();
    } catch (e) {
        console.error("Failed to link assets", e);
    } finally {
        loading.value = false;
    }
};

// --- VRKB-06: Show asset detail and usage ---
const showAssetDetail = (asset: any) => {
    selectedAsset.value = asset;
    viewUsage(asset.id);
};

const viewUsage = async (assetId: string) => {
    try {
        assetUsages.value = await vrkbApi.getAssetUsage(assetId);
    } catch (e) {
        console.error("Failed to load asset usage", e);
        assetUsages.value = [];
    }
};

// --- VRKB-06: Unlink asset from project ---
const unlinkAsset = async (assetId: string) => {
    if (!confirm("Unlink this asset from the project?")) return;
    try {
        await vrkbApi.unlinkAsset({
            asset_id: assetId,
            target_type: 'project',
            target_id: props.projectId,
        });
        await loadAssets();
        if (selectedAsset.value?.id === assetId) {
            selectedAsset.value = null;
        }
    } catch (e) {
        console.error("Failed to unlink asset", e);
    }
};

const handleDrop = async (e: DragEvent) => {
    const files = e.dataTransfer?.files;
    if (!files) return;

    loading.value = true;
    try {
        for (let i = 0; i < files.length; i++) {
            const file = files[i];
            const uploaded = await vrkbApi.uploadAsset(file);
            await vrkbApi.linkAsset({
                asset_id: uploaded.id,
                target_type: 'project',
                target_id: props.projectId,
            });
        }
        await loadAssets();
    } catch(e) {
        console.error("Failed to upload", e);
    } finally {
        loading.value = false;
    }
};

const getFileIcon = (type: string) => {
    switch(type) {
        case 'image': return 'ri-image-line';
        case 'code': return 'ri-code-line';
        case 'binary': return 'ri-file-binary-line';
        case 'log': return 'ri-file-text-line';
        case 'folder': return 'ri-folder-3-fill';
        default: return 'ri-file-line';
    }
};

const deleteAsset = async (assetId: string) => {
    if (!confirm("Permanently delete this asset?")) return;
    try {
        await vrkbApi.deleteAsset(assetId);
        await loadAssets();
        if (selectedAsset.value?.id === assetId) {
            selectedAsset.value = null;
        }
    } catch (e) {
        console.error("Failed to delete asset", e);
    }
};

const formatBytes = (bytes: number) => {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
};

const formatDate = (iso: string) => {
    return DateTime.fromISO(iso).toRelative();
};
</script>
