<template>
  <Teleport to="body">
    <Transition name="picker-fade">
      <div
        v-if="isOpen"
        class="fixed inset-0 z-[9999] flex items-center justify-center bg-black/40 backdrop-blur-sm"
        @click.self="handleClose"
      >
        <div
          class="flex w-full max-w-4xl flex-col overflow-hidden rounded-3xl border border-stone-200 bg-white shadow-2xl dark:border-stone-700 dark:bg-stone-900"
          style="max-height: 85vh"
        >
          <!-- Header -->
          <div class="flex items-center justify-between border-b border-stone-200 px-6 py-4 dark:border-stone-700">
            <div>
              <h2 class="font-serif text-xl font-semibold text-stone-900 dark:text-stone-100">
                Select Asset{{ multiple ? 's' : '' }}
              </h2>
              <p class="mt-0.5 text-sm text-stone-500 dark:text-stone-400">
                Choose from your asset library or upload a new file.
              </p>
            </div>
            <button
              class="rounded-full p-2 text-stone-400 transition hover:bg-stone-100 hover:text-stone-600 dark:hover:bg-stone-800 dark:hover:text-stone-200"
              @click="handleClose"
            >
              <svg class="h-5 w-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>

          <!-- Search & Filters -->
          <div class="flex flex-wrap items-center gap-3 border-b border-stone-100 px-6 py-3 dark:border-stone-800">
            <div class="relative flex-1">
              <svg
                class="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-stone-400"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
              </svg>
              <input
                v-model="localSearch"
                class="w-full rounded-full border border-stone-200 bg-stone-50 py-2 pl-10 pr-4 text-sm text-stone-900 transition focus:border-stone-400 focus:outline-none focus:ring-2 focus:ring-stone-200 dark:border-stone-700 dark:bg-stone-800 dark:text-stone-100 dark:focus:border-stone-500 dark:focus:ring-stone-700"
                placeholder="Search assets..."
                @input="onSearchInput"
              />
            </div>

            <!-- Type Filter Pills -->
            <div class="flex flex-wrap gap-1.5">
              <button
                v-for="filter in typeFilters"
                :key="filter.value"
                class="rounded-full border px-3 py-1 text-xs font-medium transition"
                :class="
                  activeTypeFilter === filter.value
                    ? 'border-stone-900 bg-stone-900 text-white dark:border-stone-100 dark:bg-stone-100 dark:text-stone-900'
                    : 'border-stone-200 text-stone-600 hover:border-stone-400 dark:border-stone-700 dark:text-stone-300 dark:hover:border-stone-500'
                "
                @click="setTypeFilter(filter.value)"
              >
                {{ filter.label }}
              </button>
            </div>

            <!-- Tab: Recent / All -->
            <div class="flex rounded-full border border-stone-200 p-0.5 dark:border-stone-700">
              <button
                class="rounded-full px-3 py-1 text-xs font-medium transition"
                :class="activeTab === 'all' ? 'bg-stone-900 text-white dark:bg-stone-100 dark:text-stone-900' : 'text-stone-500 hover:text-stone-700 dark:hover:text-stone-200'"
                @click="activeTab = 'all'"
              >
                All
              </button>
              <button
                class="rounded-full px-3 py-1 text-xs font-medium transition"
                :class="activeTab === 'recent' ? 'bg-stone-900 text-white dark:bg-stone-100 dark:text-stone-900' : 'text-stone-500 hover:text-stone-700 dark:hover:text-stone-200'"
                @click="activeTab = 'recent'"
              >
                Recent
              </button>
            </div>
          </div>

          <!-- Asset Grid -->
          <div class="flex-1 overflow-y-auto p-6">
            <div v-if="pickerLoading" class="flex h-48 items-center justify-center text-sm text-stone-500">
              Loading assets...
            </div>

            <div v-else-if="displayAssets.length === 0" class="flex h-48 flex-col items-center justify-center text-sm text-stone-500">
              <p>{{ activeTab === 'recent' ? 'No recent assets.' : 'No assets found.' }}</p>
              <button
                class="mt-3 rounded-full border border-stone-300 px-4 py-1.5 text-xs font-medium text-stone-600 transition hover:border-stone-500 hover:text-stone-900 dark:border-stone-600 dark:text-stone-300 dark:hover:border-stone-400"
                @click="triggerUpload"
              >
                Upload New Asset
              </button>
            </div>

            <div v-else class="grid grid-cols-2 gap-3 sm:grid-cols-3 lg:grid-cols-4">
              <button
                v-for="asset in displayAssets"
                :key="asset.id"
                class="group relative overflow-hidden rounded-2xl border text-left transition"
                :class="
                  pickerIsSelected(asset.id)
                    ? 'border-stone-900 bg-stone-50 shadow-md ring-2 ring-stone-900/20 dark:border-stone-100 dark:bg-stone-800 dark:ring-stone-100/20'
                    : 'border-stone-200 bg-white hover:border-stone-400 hover:shadow-sm dark:border-stone-700 dark:bg-stone-800/50 dark:hover:border-stone-500'
                "
                @click="handleAssetClick(asset)"
              >
                <!-- Selection Indicator -->
                <div
                  v-if="pickerIsSelected(asset.id)"
                  class="absolute right-2 top-2 z-10 flex h-6 w-6 items-center justify-center rounded-full bg-stone-900 text-white dark:bg-stone-100 dark:text-stone-900"
                >
                  <svg class="h-3.5 w-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="3" d="M5 13l4 4L19 7" />
                  </svg>
                </div>

                <!-- Preview -->
                <div class="aspect-square border-b border-stone-100 bg-stone-50 dark:border-stone-700 dark:bg-stone-900">
                  <img
                    v-if="isImageType(asset)"
                    :src="getAssetPreviewUrl(asset)"
                    :alt="getDisplayName(asset)"
                    class="h-full w-full object-cover"
                    loading="lazy"
                  />
                  <div v-else class="flex h-full flex-col items-center justify-center gap-2 text-stone-400">
                    <span class="rounded-full border border-stone-300 px-2.5 py-0.5 text-[10px] font-bold uppercase tracking-widest dark:border-stone-600">
                      {{ getTypeLabel(asset) }}
                    </span>
                    <span class="text-[10px] uppercase tracking-wider">
                      {{ getExtension(asset) || 'file' }}
                    </span>
                  </div>
                </div>

                <!-- Info -->
                <div class="p-2.5">
                  <p class="truncate text-xs font-semibold text-stone-800 dark:text-stone-200">
                    {{ getDisplayName(asset) }}
                  </p>
                  <p class="mt-0.5 text-[10px] text-stone-400">
                    {{ formatSize(getPayload(asset).size_bytes) }}
                  </p>
                </div>
              </button>
            </div>
          </div>

          <!-- Footer -->
          <div class="flex items-center justify-between border-t border-stone-200 px-6 py-3 dark:border-stone-700">
            <div class="flex items-center gap-3">
              <input
                ref="fileInputRef"
                class="hidden"
                multiple
                type="file"
                @change="handleFileUpload"
              />
              <button
                class="rounded-full border border-stone-300 px-4 py-2 text-sm font-medium text-stone-600 transition hover:border-stone-500 hover:text-stone-900 dark:border-stone-600 dark:text-stone-300 dark:hover:border-stone-400"
                @click="triggerUpload"
              >
                Upload New
              </button>
              <span v-if="selectedCount > 0" class="text-sm text-stone-500 dark:text-stone-400">
                {{ selectedCount }} selected
              </span>
            </div>

            <div class="flex items-center gap-2">
              <button
                class="rounded-full border border-stone-300 px-5 py-2 text-sm font-medium text-stone-600 transition hover:border-stone-500 hover:text-stone-900 dark:border-stone-600 dark:text-stone-300 dark:hover:border-stone-400"
                @click="handleClose"
              >
                Cancel
              </button>
              <button
                class="rounded-full bg-stone-900 px-5 py-2 text-sm font-medium text-white transition hover:bg-stone-700 disabled:cursor-not-allowed disabled:opacity-40 dark:bg-stone-100 dark:text-stone-900 dark:hover:bg-white"
                :disabled="selectedCount === 0"
                @click="handleConfirm"
              >
                {{ multiple ? `Insert ${selectedCount} Asset${selectedCount !== 1 ? 's' : ''}` : 'Insert Asset' }}
              </button>
            </div>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import {
    assetsApi,
    type AssetNode,
    type AssetType,
    extractAssetPayload,
    getAssetDisplayName,
    getAssetType,
    getAssetTypeLabel,
    isImageAsset,
} from '@/api/assets';
import { useAssetPicker } from '@/composables/useAssetPicker';

const picker = useAssetPicker();

const localSearch = ref('');
const activeTab = ref<'all' | 'recent'>('all');
const activeTypeFilter = ref<string>('all');
const fileInputRef = ref<HTMLInputElement | null>(null);

const typeFilters = [
    { value: 'all', label: 'All' },
    { value: 'image_asset', label: 'Images' },
    { value: 'pdf_asset', label: 'PDFs' },
    { value: 'file_asset', label: 'Files' },
    { value: 'ip_asset', label: 'IPs' },
    { value: 'snippet_asset', label: 'Snippets' },
];

const isOpen = computed(() => picker.isOpen.value);
const multiple = computed(() => picker.multiple.value);
const pickerLoading = computed(() => picker.loading.value);

const displayAssets = computed(() => {
    if (activeTab.value === 'recent') {
        return picker.recentAssets.value;
    }
    let items = picker.assets.value;

    // Client-side type filter (if acceptTypes restricts)
    if (picker.acceptTypes.value.length > 0) {
        items = items.filter((a) => {
            const t = getAssetType(a);
            return picker.acceptTypes.value.includes(t);
        });
    }

    // Additional client-side type filter from pills
    if (activeTypeFilter.value !== 'all') {
        items = items.filter((a) => getAssetType(a) === activeTypeFilter.value);
    }

    return items;
});

const selectedCount = computed(() => picker.selectedAssets.value.length);

function pickerIsSelected(assetId: string): boolean {
    return picker.isAssetSelected(assetId);
}

function handleAssetClick(asset: AssetNode) {
    picker.toggleAssetSelection(asset);
    if (!multiple.value) {
        // Auto-confirm for single select
        handleConfirm();
    }
}

function handleConfirm() {
    picker.confirmSelection();
}

function handleClose() {
    picker.closePicker();
}

function setTypeFilter(value: string) {
    activeTypeFilter.value = value;
}

let searchDebounce: ReturnType<typeof setTimeout> | null = null;
function onSearchInput() {
    if (searchDebounce) clearTimeout(searchDebounce);
    searchDebounce = setTimeout(() => {
        void picker.searchAssets(localSearch.value);
    }, 300);
}

function triggerUpload() {
    fileInputRef.value?.click();
}

async function handleFileUpload(event: Event) {
    const target = event.target as HTMLInputElement;
    if (!target.files?.length) return;

    const files = Array.from(target.files);
    for (const file of files) {
        try {
            await assetsApi.upload(file);
        } catch (error) {
            console.error('[AssetPicker] Upload failed', error);
        }
    }
    target.value = '';
    await picker.fetchPickerAssets();
}

// Helpers
function isImageType(asset: AssetNode): boolean {
    return isImageAsset(asset);
}

function getAssetPreviewUrl(asset: AssetNode): string {
    return assetsApi.getAssetUrl(asset.id);
}

function getDisplayName(asset: AssetNode): string {
    return getAssetDisplayName(asset);
}

function getTypeLabel(asset: AssetNode): string {
    return getAssetTypeLabel(getAssetType(asset));
}

function getExtension(asset: AssetNode): string | null | undefined {
    return extractAssetPayload(asset).metadata?.extension;
}

function getPayload(asset: AssetNode) {
    return extractAssetPayload(asset);
}

function formatSize(bytes?: number): string {
    if (!bytes) return '0 B';
    const units = ['B', 'KB', 'MB', 'GB'];
    const exponent = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1);
    const value = bytes / Math.pow(1024, exponent);
    return `${value.toFixed(value >= 10 || exponent === 0 ? 0 : 1)} ${units[exponent]}`;
}

// Sync local search with global picker search
watch(() => picker.searchQuery.value, (val) => {
    localSearch.value = val;
});

// Reset state when opened
watch(isOpen, (val) => {
    if (val) {
        localSearch.value = '';
        activeTab.value = 'all';
        activeTypeFilter.value = 'all';
    }
});
</script>

<style scoped>
.picker-fade-enter-active,
.picker-fade-leave-active {
    transition: opacity 0.2s ease;
}
.picker-fade-enter-from,
.picker-fade-leave-to {
    opacity: 0;
}
</style>
