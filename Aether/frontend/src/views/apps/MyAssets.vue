<template>
  <div class="h-full flex flex-col bg-gray-50 dark:bg-gray-900">
    <!-- Header -->
    <header class="flex items-center justify-between px-6 py-4 bg-white dark:bg-gray-800 border-b border-gray-200 dark:border-gray-700">
      <h1 class="text-xl font-serif font-bold text-gray-900 dark:text-gray-100">My Assets</h1>
      <div class="flex items-center gap-4">
        <input 
          type="file" 
          ref="fileInput" 
          class="hidden" 
          @change="handleFileUpload" 
          multiple
        />
        <button 
          @click="triggerUpload"
          class="px-4 py-2 bg-black dark:bg-white text-white dark:text-black rounded hover:opacity-80 transition-opacity flex items-center gap-2"
        >
          <span>Upload</span>
        </button>
      </div>
    </header>

    <!-- Content -->
    <div class="flex-1 overflow-auto p-6">
      <div v-if="loading" class="text-center py-10 text-gray-500">Loading assets...</div>
      
      <div v-else-if="assets.length === 0" class="text-center py-20">
        <div class="text-6xl mb-4">📂</div>
        <p class="text-gray-500">No assets found. Upload files to get started.</p>
      </div>

      <div v-else class="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-6 gap-6">
        <div 
          v-for="asset in assets" 
          :key="asset.id"
          class="group relative bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 hover:shadow-lg transition-all overflow-hidden"
        >
          <!-- Thumbnail -->
          <div class="aspect-square bg-gray-100 dark:bg-gray-900 flex items-center justify-center overflow-hidden">
            <img 
              v-if="isImage(asset)" 
              :src="getThumbnailUrl(asset)" 
              class="w-full h-full object-cover"
              loading="lazy"
            />
            <div v-else class="text-4xl text-gray-400">
              📄
            </div>
          </div>

          <!-- Info -->
          <div class="p-3">
            <h3 class="text-sm font-medium text-gray-900 dark:text-gray-100 truncate" :title="asset.title">
              {{ asset.title }}
            </h3>
            <p class="text-xs text-gray-500 mt-1 truncate">
               {{ formatSize(asset.body?.data?.size_bytes) }}
            </p>
          </div>

          <!-- Actions Overlay -->
          <div class="absolute inset-0 bg-black/50 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center gap-2">
            <button 
              @click="copyMarkdown(asset)"
              class="p-2 bg-white text-black rounded-full hover:scale-110 transition-transform"
              title="Copy Markdown Link"
            >
              📋
            </button>
            <a 
              :href="getDownloadUrl(asset)" 
              target="_blank"
              class="p-2 bg-white text-black rounded-full hover:scale-110 transition-transform"
              title="Open Original"
            >
              ⬇️
            </a>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { contentApi } from '../../api/content';
import { assetsApi } from '../../api/assets';

const assets = ref<any[]>([]);
const loading = ref(true);
const fileInput = ref<HTMLInputElement | null>(null);

const fetchAssets = async () => {
  loading.value = true;
  try {
    const res = await contentApi.list({ category: 'Asset', limit: 100 });
    assets.value = res;
  } catch (e) {
    console.error("Failed to load assets", e);
  } finally {
    loading.value = false;
  }
};

const triggerUpload = () => {
  fileInput.value?.click();
};

const handleFileUpload = async (event: Event) => {
  const target = event.target as HTMLInputElement;
  if (!target.files?.length) return;

  const files = Array.from(target.files);
  for (const file of files) {
    try {
      await assetsApi.upload(file);
    } catch (e) {
      console.error("Upload failed", e);
      alert(`Failed to upload ${file.name}`);
    }
  }
  
  // Refresh list
  await fetchAssets();
  target.value = ''; // Reset input
};

const isImage = (asset: any) => {
  // If body is ContentBody object with type Custom
  if (asset.body && typeof asset.body === 'object' && asset.body.type === 'Custom') {
      const mime = asset.body.data?.mime_type || '';
      return mime.startsWith('image/');
  }
  return false;
};

const getThumbnailUrl = (asset: any) => {
  // Direct access for owner
  return assetsApi.getAssetUrl(asset.id);
};

const getDownloadUrl = (asset: any) => {
  return assetsApi.getAssetUrl(asset.id);
};

const copyMarkdown = (asset: any) => {
  // We use the Internal Asset Link Syntax: ![[asset:UUID]]
  // Note: Standard Obsidian syntax is ![[filename]], but we need UUID for resilience.
  // Spec says: [[asset:uuid]]
  const link = `![${asset.title}]([[asset:${asset.id}]])`;
  navigator.clipboard.writeText(link);
  // Ideally show a toast
  alert("Copied to clipboard!"); 
};

const formatSize = (bytes?: number) => {
  if (!bytes) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
};

onMounted(() => {
  fetchAssets();
});
</script>
