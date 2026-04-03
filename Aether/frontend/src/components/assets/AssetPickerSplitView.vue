<template>
  <Transition name="split-slide">
    <div
      v-if="isOpen && mode === 'split'"
      class="flex h-full w-80 flex-col border-l border-stone-200 bg-white dark:border-stone-700 dark:bg-stone-900"
    >
      <!-- Header -->
      <div class="flex items-center justify-between border-b border-stone-200 px-4 py-3 dark:border-stone-700">
        <h3 class="text-sm font-semibold text-stone-900 dark:text-stone-100">Assets</h3>
        <button
          class="rounded-full p-1.5 text-stone-400 transition hover:bg-stone-100 hover:text-stone-600 dark:hover:bg-stone-800 dark:hover:text-stone-200"
          @click="handleClose"
        >
          <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <!-- Search -->
      <div class="border-b border-stone-100 p-3 dark:border-stone-800">
        <div class="relative">
          <svg
            class="absolute left-2.5 top-1/2 h-3.5 w-3.5 -translate-y-1/2 text-stone-400"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
          </svg>
          <input
            v-model="localSearch"
            class="w-full rounded-lg border border-stone-200 bg-stone-50 py-1.5 pl-8 pr-3 text-xs text-stone-900 transition focus:border-stone-400 focus:outline-none dark:border-stone-700 dark:bg-stone-800 dark:text-stone-100"
            placeholder="Search..."
            @input="onSearchInput"
          />
        </div>

        <!-- Quick type filters -->
        <div class="mt-2 flex flex-wrap gap-1">
          <button
            v-for="filter in compactFilters"
            :key="filter.value"
            class="rounded-full border px-2 py-0.5 text-[10px] font-medium transition"
            :class="
              activeTypeFilter === filter.value
                ? 'border-stone-900 bg-stone-900 text-white dark:border-stone-100 dark:bg-stone-100 dark:text-stone-900'
                : 'border-stone-200 text-stone-500 hover:border-stone-400 dark:border-stone-700 dark:text-stone-400'
            "
            @click="setTypeFilter(filter.value)"
          >
            {{ filter.label }}
          </button>
        </div>
      </div>

      <!-- Asset List -->
      <div class="flex-1 overflow-y-auto p-2">
        <div v-if="pickerLoading" class="flex h-32 items-center justify-center text-xs text-stone-500">
          Loading...
        </div>

        <div v-else-if="displayAssets.length === 0" class="flex h-32 flex-col items-center justify-center text-xs text-stone-500">
          <p>No assets found.</p>
          <button
            class="mt-2 rounded-full border border-stone-300 px-3 py-1 text-[10px] font-medium text-stone-600 transition hover:border-stone-500 dark:border-stone-600 dark:text-stone-300"
            @click="triggerUpload"
          >
            Upload
          </button>
        </div>

        <div v-else class="space-y-1">
          <button
            v-for="asset in displayAssets"
            :key="asset.id"
            class="flex w-full items-center gap-2.5 rounded-xl p-2 text-left transition"
            :class="
              pickerIsSelected(asset.id)
                ? 'bg-stone-100 ring-1 ring-stone-300 dark:bg-stone-800 dark:ring-stone-600'
                : 'hover:bg-stone-50 dark:hover:bg-stone-800/50'
            "
            @click="handleAssetClick(asset)"
          >
            <!-- Thumbnail -->
            <div class="h-10 w-10 flex-shrink-0 overflow-hidden rounded-lg border border-stone-200 bg-stone-50 dark:border-stone-700 dark:bg-stone-800">
              <img
                v-if="isImageType(asset)"
                :src="getAssetPreviewUrl(asset)"
                :alt="getDisplayName(asset)"
                class="h-full w-full object-cover"
                loading="lazy"
              />
              <div v-else class="flex h-full items-center justify-center text-[8px] font-bold uppercase text-stone-400">
                {{ getExtension(asset) || 'F' }}
              </div>
            </div>

            <!-- Info -->
            <div class="min-w-0 flex-1">
              <p class="truncate text-xs font-medium text-stone-800 dark:text-stone-200">
                {{ getDisplayName(asset) }}
              </p>
              <p class="text-[10px] text-stone-400">
                {{ getTypeLabel(asset) }} · {{ formatSize(getPayload(asset).size_bytes) }}
              </p>
            </div>

            <!-- Selected check -->
            <div
              v-if="pickerIsSelected(asset.id)"
              class="flex h-5 w-5 flex-shrink-0 items-center justify-center rounded-full bg-stone-900 text-white dark:bg-stone-100 dark:text-stone-900"
            >
              <svg class="h-3 w-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="3" d="M5 13l4 4L19 7" />
              </svg>
            </div>
          </button>
        </div>
      </div>

      <!-- Footer -->
      <div class="border-t border-stone-200 p-3 dark:border-stone-700">
        <div class="flex items-center gap-2">
          <input ref="fileInputRef" class="hidden" multiple type="file" @change="handleFileUpload" />
          <button
            class="flex-1 rounded-full border border-stone-300 py-1.5 text-xs font-medium text-stone-600 transition hover:border-stone-500 dark:border-stone-600 dark:text-stone-300"
            @click="triggerUpload"
          >
            Upload
          </button>
          <button
            class="flex-1 rounded-full bg-stone-900 py-1.5 text-xs font-medium text-white transition hover:bg-stone-700 disabled:opacity-40 dark:bg-stone-100 dark:text-stone-900 dark:hover:bg-white"
            :disabled="selectedCount === 0"
            @click="handleConfirm"
          >
            Insert{{ selectedCount > 0 ? ` (${selectedCount})` : '' }}
          </button>
        </div>
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import {
    assetsApi,
    type AssetNode,
    extractAssetPayload,
    getAssetDisplayName,
    getAssetType,
    getAssetTypeLabel,
    isImageAsset,
} from '@/api/assets';
import { useAssetPicker } from '@/composables/useAssetPicker';

const picker = useAssetPicker();

const localSearch = ref('');
const activeTypeFilter = ref<string>('all');
const fileInputRef = ref<HTMLInputElement | null>(null);

const compactFilters = [
    { value: 'all', label: 'All' },
    { value: 'image_asset', label: 'Img' },
    { value: 'pdf_asset', label: 'PDF' },
    { value: 'file_asset', label: 'File' },
];

const isOpen = computed(() => picker.isOpen.value);
const mode = computed(() => picker.mode.value);
const pickerLoading = computed(() => picker.loading.value);
const selectedCount = computed(() => picker.selectedAssets.value.length);

const displayAssets = computed(() => {
    let items = picker.assets.value;
    if (picker.acceptTypes.value.length > 0) {
        items = items.filter((a) => picker.acceptTypes.value.includes(getAssetType(a)));
    }
    if (activeTypeFilter.value !== 'all') {
        items = items.filter((a) => getAssetType(a) === activeTypeFilter.value);
    }
    return items;
});

function pickerIsSelected(assetId: string): boolean {
    return picker.isAssetSelected(assetId);
}

function handleAssetClick(asset: AssetNode) {
    picker.toggleAssetSelection(asset);
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
            console.error('[AssetPickerSplit] Upload failed', error);
        }
    }
    target.value = '';
    await picker.fetchPickerAssets();
}

// Helpers
function isImageType(asset: AssetNode): boolean { return isImageAsset(asset); }
function getAssetPreviewUrl(asset: AssetNode): string { return assetsApi.getAssetUrl(asset.id); }
function getDisplayName(asset: AssetNode): string { return getAssetDisplayName(asset); }
function getTypeLabel(asset: AssetNode): string { return getAssetTypeLabel(getAssetType(asset)); }
function getExtension(asset: AssetNode): string | null | undefined { return extractAssetPayload(asset).metadata?.extension; }
function getPayload(asset: AssetNode) { return extractAssetPayload(asset); }
function formatSize(bytes?: number): string {
    if (!bytes) return '0 B';
    const units = ['B', 'KB', 'MB', 'GB'];
    const exponent = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1);
    const value = bytes / Math.pow(1024, exponent);
    return `${value.toFixed(value >= 10 || exponent === 0 ? 0 : 1)} ${units[exponent]}`;
}

watch(() => picker.searchQuery.value, (val) => { localSearch.value = val; });
watch(isOpen, (val) => {
    if (val) { localSearch.value = ''; activeTypeFilter.value = 'all'; }
});
</script>

<style scoped>
.split-slide-enter-active,
.split-slide-leave-active {
    transition: transform 0.25s ease, opacity 0.2s ease;
}
.split-slide-enter-from,
.split-slide-leave-to {
    transform: translateX(100%);
    opacity: 0;
}
</style>
