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
                <button class="p-2 text-ink/60 hover:text-accent hover:bg-accent/10 rounded-lg transition-colors" title="Upload File">
                    <i class="ri-upload-cloud-2-line text-lg"></i>
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
                <p class="text-xs mt-2">Drag and drop files here to upload</p>
            </div>

            <!-- Grid View -->
            <div v-else-if="viewMode === 'grid'" class="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-5 gap-4">
                <div 
                    v-for="asset in assets" 
                    :key="asset.id"
                    class="group relative bg-white border border-ash/20 rounded-xl p-4 flex flex-col items-center text-center hover:border-accent/50 hover:shadow-md transition-all cursor-pointer aspect-square"
                >
                    <div class="flex-1 flex items-center justify-center text-ink/20 group-hover:text-accent/80 transition-colors">
                        <i :class="getFileIcon(asset.type)" class="text-4xl"></i>
                    </div>
                    <div class="w-full mt-3">
                        <p class="text-xs font-bold text-ink truncate w-full">{{ asset.name }}</p>
                        <p class="text-[10px] text-ink/40 font-mono mt-0.5">{{ formatBytes(asset.size) }}</p>
                    </div>
                </div>
            </div>

            <!-- List View -->
            <div v-else class="flex flex-col gap-1">
                <div 
                    v-for="asset in assets" 
                    :key="asset.id"
                    class="flex items-center justify-between p-3 bg-white border border-ash/20 rounded-lg hover:border-accent/30 hover:bg-ash/10 transition-colors cursor-pointer group"
                >
                    <div class="flex items-center gap-3">
                        <i :class="getFileIcon(asset.type)" class="text-xl text-ink/40 group-hover:text-accent"></i>
                        <span class="text-sm font-medium text-ink">{{ asset.name }}</span>
                    </div>
                    <div class="flex items-center gap-6 text-xs text-ink/40 font-mono">
                        <span>{{ formatBytes(asset.size) }}</span>
                        <span>{{ formatDate(asset.updated_at) }}</span>
                        <div class="w-20 flex justify-end gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
                            <button class="hover:text-accent"><i class="ri-download-line"></i></button>
                            <button @click.stop="deleteAsset(asset.id)" class="hover:text-red-500"><i class="ri-delete-bin-line"></i></button>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { DateTime } from 'luxon';
import { vrkbApi } from '@/api/vrkb';

const props = defineProps<{
    projectId: string
}>();

const viewMode = ref<'grid' | 'list'>('grid');
const loading = ref(false);
const currentPath = ref('');
const assets = ref<any[]>([]);

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

const handleDrop = async (e: DragEvent) => {
    const files = e.dataTransfer?.files;
    if (!files) return;

    for (let i = 0; i < files.length; i++) {
        const file = files[i];
        try {
            await vrkbApi.uploadAsset(file);
        } catch(e) {
            alert(`Failed to upload ${file.name}`);
        }
    }
    await loadAssets();
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
    if (!confirm("Delete this asset?")) return;
    try {
        await vrkbApi.deleteAsset(assetId);
        await loadAssets();
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
