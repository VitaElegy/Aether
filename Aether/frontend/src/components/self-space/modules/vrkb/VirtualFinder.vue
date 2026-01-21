<template>
    <div class="h-full flex">
        <!-- Sidebar -->
        <div class="w-64 border-r border-ink/10 p-4 bg-ash/30 flex flex-col">
            <h3 class="font-bold text-ink/60 mb-4 uppercase text-xs tracking-wider px-2">Locations</h3>
            <ul class="space-y-1">
                <li class="p-2 bg-accent/10 text-accent rounded font-medium flex items-center gap-2 cursor-pointer">
                    <i class="ri-cloud-line text-lg"></i>
                    Uploads
                </li>
                <li class="p-2 text-ink/60 hover:bg-ash/50 rounded flex items-center gap-2 cursor-pointer">
                    <i class="ri-folder-line text-lg"></i>
                    Project Assets
                </li>
            </ul>
        </div>

        <!-- Main Content -->
        <div class="flex-1 flex flex-col p-6 bg-paper">
            <h2 class="text-xl font-serif text-ink mb-6">Virtual Finder</h2>

            <!-- Drop Zone -->
            <div 
                class="border-2 border-dashed border-ink/10 rounded-xl h-64 flex flex-col items-center justify-center text-ink/60 transition-colors relative"
                :class="{ 'border-accent bg-accent/5': isDragging }"
                @dragover.prevent="isDragging = true"
                @dragleave.prevent="isDragging = false"
                @drop.prevent="handleDrop"
            >
                <div v-if="isUploading" class="flex flex-col items-center">
                    <div class="animate-spin h-8 w-8 border-2 border-accent border-t-transparent rounded-full mb-4"></div>
                    <span class="text-sm">Uploading...</span>
                </div>
                <div v-else class="flex flex-col items-center pointer-events-none">
                    <i class="ri-upload-cloud-2-line text-4xl mb-2 opacity-50"></i>
                    <p class="font-medium">Drag & Drop files here</p>
                    <p class="text-xs opacity-70 mt-1">or click to browse</p>
                </div>
                
                <input 
                    type="file" 
                    ref="fileInput" 
                    class="absolute inset-0 opacity-0 cursor-pointer" 
                    @change="handleFileSelect" 
                />
            </div>

            <!-- Recent Uploads (Local State) -->
            <div class="mt-8 flex-1 overflow-hidden flex flex-col">
                <h3 class="font-bold text-ink mb-4">Session Uploads</h3>
                <div class="overflow-y-auto flex-1 grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4 pb-4">
                    <div 
                        v-for="asset in recentAssets" 
                        :key="asset.id"
                        class="bg-ash/30 p-3 rounded border border-ink/10 flex flex-col gap-2 group relative overflow-hidden"
                    >
                        <div class="aspect-square bg-ash/50 rounded flex items-center justify-center text-ink/60">
                            <span v-if="asset.mime_type.startsWith('image/')" class="text-xs">IMG</span>
                            <i v-else class="ri-file-text-line text-2xl"></i>
                        </div>
                        <div class="flex flex-col min-w-0">
                            <span class="text-xs font-bold text-ink truncate" :title="asset.hash">{{ asset.hash.substring(0, 12) }}...</span>
                            <span class="text-[10px] text-ink-muted">{{ asset.mime_type }}</span>
                        </div>
                        
                        <!-- Quick Copy Action -->
                        <div class="absolute inset-0 bg-black/50 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center gap-2">
                            <button @click="copyHash(asset)" class="p-2 bg-white rounded-full text-ink hover:text-accent shadow-sm" title="Copy Hash">
                                <i class="ri-file-copy-line"></i>
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { useVrkbStore, type VrkbAsset } from '@/stores/vrkb';

const store = useVrkbStore();
const isDragging = ref(false);
const isUploading = ref(false);
const recentAssets = ref<VrkbAsset[]>([]);
const fileInput = ref<HTMLInputElement | null>(null);

const handleDrop = (e: DragEvent) => {
    isDragging.value = false;
    const files = e.dataTransfer?.files;
    if (files && files.length > 0) {
        uploadFiles(files);
    }
};

const handleFileSelect = (e: Event) => {
    const input = e.target as HTMLInputElement;
    if (input.files && input.files.length > 0) {
        uploadFiles(input.files);
    }
};

const uploadFiles = async (files: FileList) => {
    isUploading.value = true;
    try {
        for (let i = 0; i < files.length; i++) {
            const asset = await store.uploadAsset(files[i]);
            recentAssets.value.unshift(asset);
        }
    } catch (e) {
        console.error("Upload failed", e);
        alert("Upload failed");
    } finally {
        isUploading.value = false;
        if (fileInput.value) fileInput.value.value = '';
    }
};

const copyHash = (asset: VrkbAsset) => {
    navigator.clipboard.writeText(asset.hash);
};
</script>
