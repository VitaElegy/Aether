<template>
  <AssetUploadQueue ref="uploadQueueRef" @uploaded="fetchAssets">
    <div class="flex h-full flex-col bg-stone-100 text-stone-900 dark:bg-stone-950 dark:text-stone-100">
      <header class="border-b border-stone-200 bg-white/90 px-6 py-5 backdrop-blur dark:border-stone-800 dark:bg-stone-900/90">
        <div class="flex flex-col gap-5 lg:flex-row lg:items-end lg:justify-between">
          <div class="space-y-2">
            <p class="text-[11px] font-semibold uppercase tracking-[0.3em] text-stone-500 dark:text-stone-400">
              Asset Center
            </p>
            <div>
              <h1 class="font-serif text-3xl font-semibold tracking-tight">My Assets</h1>
              <p class="mt-1 max-w-2xl text-sm text-stone-600 dark:text-stone-300">
                Centralize uploaded files, inspect their metadata, and copy resilient internal references from one place.
              </p>
            </div>
          </div>

          <div class="flex flex-col gap-3 lg:items-end">
            <div class="flex flex-wrap gap-2">
              <div class="rounded-full border border-stone-200 bg-stone-100 px-3 py-1.5 text-sm text-stone-700 dark:border-stone-700 dark:bg-stone-800 dark:text-stone-200">
                {{ stats.total }} assets
              </div>
              <div class="rounded-full border border-stone-200 bg-stone-100 px-3 py-1.5 text-sm text-stone-700 dark:border-stone-700 dark:bg-stone-800 dark:text-stone-200">
                {{ stats.images }} images
              </div>
              <div class="rounded-full border border-stone-200 bg-stone-100 px-3 py-1.5 text-sm text-stone-700 dark:border-stone-700 dark:bg-stone-800 dark:text-stone-200">
                {{ stats.pdfs }} PDFs
              </div>
              <div v-if="stats.ip_assets" class="rounded-full border border-stone-200 bg-stone-100 px-3 py-1.5 text-sm text-stone-700 dark:border-stone-700 dark:bg-stone-800 dark:text-stone-200">
                {{ stats.ip_assets }} IPs
              </div>
              <div v-if="stats.domain_assets" class="rounded-full border border-stone-200 bg-stone-100 px-3 py-1.5 text-sm text-stone-700 dark:border-stone-700 dark:bg-stone-800 dark:text-stone-200">
                {{ stats.domain_assets }} domains
              </div>
              <div v-if="stats.credential_stubs" class="rounded-full border border-stone-200 bg-stone-100 px-3 py-1.5 text-sm text-stone-700 dark:border-stone-700 dark:bg-stone-800 dark:text-stone-200">
                {{ stats.credential_stubs }} credentials
              </div>
              <div v-if="stats.snippets" class="rounded-full border border-stone-200 bg-stone-100 px-3 py-1.5 text-sm text-stone-700 dark:border-stone-700 dark:bg-stone-800 dark:text-stone-200">
                {{ stats.snippets }} snippets
              </div>
            </div>

            <div class="flex items-center gap-3">
              <input
                ref="fileInput"
                class="hidden"
                multiple
                type="file"
                @change="handleFileUpload"
              />
              <button
                class="rounded-full bg-stone-900 px-4 py-2.5 text-sm font-medium text-white transition hover:bg-stone-700 disabled:cursor-not-allowed disabled:opacity-60 dark:bg-stone-100 dark:text-stone-900 dark:hover:bg-white"
                :disabled="uploading"
                @click="triggerUpload"
              >
                {{ uploading ? 'Uploading...' : 'Upload Assets' }}
              </button>
            </div>
          </div>
        </div>
      </header>

      <AssetFiltersBar
        v-model:searchQuery="searchQuery"
        v-model:activeFilter="activeFilter"
        v-model:viewMode="viewMode"
        v-model:sortBy="sortBy"
        :stats="stats"
        :actionMessage="actionMessage"
      />

      <div class="min-h-0 flex-1 overflow-hidden">
        <div v-if="loading" class="flex h-full items-center justify-center text-sm text-stone-500 dark:text-stone-400">
          Loading asset catalog...
        </div>

        <div v-else-if="stats.total === 0" class="flex h-full items-center justify-center px-6">
          <div class="max-w-md rounded-[2rem] border border-dashed border-stone-300 bg-white/90 p-10 text-center shadow-sm dark:border-stone-700 dark:bg-stone-900/80">
            <p class="text-[11px] font-semibold uppercase tracking-[0.3em] text-stone-400">Empty Library</p>
            <h2 class="mt-3 font-serif text-2xl font-semibold">No assets yet</h2>
            <p class="mt-3 text-sm text-stone-600 dark:text-stone-300">
              Upload files here first so other special knowledge bases can reference them reliably.
            </p>
            <button
              class="mt-6 rounded-full bg-stone-900 px-4 py-2.5 text-sm font-medium text-white transition hover:bg-stone-700 dark:bg-stone-100 dark:text-stone-900 dark:hover:bg-white"
              @click="triggerUpload"
            >
              Upload the first asset
            </button>
          </div>
        </div>

        <div v-else class="grid h-full min-h-0 lg:grid-cols-[minmax(0,1fr)_22rem]">
          <main class="min-h-0 overflow-auto px-6 py-6">
            <!-- Grid View -->
            <template v-if="viewMode === 'grid'">
              <div v-if="sortedAssets.length === 0" class="rounded-[1.75rem] border border-dashed border-stone-300 bg-white/80 p-8 text-center text-sm text-stone-600 dark:border-stone-700 dark:bg-stone-900/70 dark:text-stone-300">
                No assets match the current search or filter.
              </div>

              <div v-else class="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
                <button
                  v-for="asset in sortedAssets"
                  :key="asset.id"
                  :data-testid="`asset-card-${asset.id}`"
                  class="group overflow-hidden rounded-[1.6rem] border text-left transition"
                  :class="selectedAsset?.id === asset.id
                    ? 'border-stone-900 bg-white shadow-lg shadow-stone-300/40 dark:border-stone-100 dark:bg-stone-900 dark:shadow-black/40'
                    : 'border-stone-200 bg-white/90 hover:border-stone-400 hover:shadow-md dark:border-stone-800 dark:bg-stone-900/80 dark:hover:border-stone-600'"
                  @click="selectedAssetId = asset.id"
                >
                  <div class="aspect-[4/3] border-b border-stone-200 bg-stone-100 dark:border-stone-800 dark:bg-stone-950">
                    <img
                      v-if="isImageAsset(asset)"
                      :src="getAssetUrl(asset)"
                      :alt="getAssetDisplayName(asset)"
                      class="h-full w-full object-cover"
                      loading="lazy"
                    />
                    <div v-else class="flex h-full flex-col items-center justify-center gap-3 text-stone-500 dark:text-stone-400">
                      <div class="rounded-full border border-stone-300 px-4 py-1 text-xs font-semibold uppercase tracking-[0.3em] dark:border-stone-700">
                        {{ getAssetTypeLabel(getAssetType(asset)) }}
                      </div>
                      <p class="text-xs uppercase tracking-[0.24em]">{{ assetExtension(asset) || 'binary' }}</p>
                    </div>
                  </div>

                  <div class="space-y-3 p-4">
                    <div class="flex items-start justify-between gap-3">
                      <div class="min-w-0">
                        <h3 class="truncate text-base font-semibold text-stone-900 dark:text-stone-100">
                          {{ getAssetDisplayName(asset) }}
                        </h3>
                        <p class="mt-1 truncate text-sm text-stone-500 dark:text-stone-400">
                          {{ assetMime(asset) }}
                        </p>
                      </div>
                      <span class="rounded-full bg-stone-900 px-2.5 py-1 text-[11px] font-semibold uppercase tracking-[0.18em] text-white dark:bg-stone-100 dark:text-stone-900">
                        {{ getAssetTypeLabel(getAssetType(asset)) }}
                      </span>
                    </div>

                    <div class="flex items-center justify-between text-xs text-stone-500 dark:text-stone-400">
                      <span>{{ formatSize(extractPayload(asset).size_bytes) }}</span>
                      <span>{{ formatDate(asset.created_at) }}</span>
                    </div>
                  </div>
                </button>
              </div>
            </template>

            <!-- Table View -->
            <AssetTable
              v-else
              :assets="sortedAssets"
              :selectedAssetId="selectedAssetId"
              @select="selectedAssetId = $event"
            />
          </main>

          <aside class="border-t border-stone-200 bg-white/90 p-6 lg:min-h-0 lg:overflow-auto lg:border-l lg:border-t-0 dark:border-stone-800 dark:bg-stone-900/90">
            <div v-if="selectedAsset" data-testid="asset-detail-panel" class="space-y-6">
              <div class="space-y-3">
                <p class="text-[11px] font-semibold uppercase tracking-[0.3em] text-stone-400">Selection</p>
                <div>
                  <h2 class="font-serif text-2xl font-semibold leading-tight">{{ getAssetDisplayName(selectedAsset) }}</h2>
                  <p class="mt-2 text-sm text-stone-600 dark:text-stone-300">
                    {{ getAssetTypeLabel(getAssetType(selectedAsset)) }} asset stored for cross-KB reuse.
                  </p>
                </div>
              </div>

              <div class="overflow-hidden rounded-[1.6rem] border border-stone-200 bg-stone-100 dark:border-stone-800 dark:bg-stone-950">
                <img
                  v-if="isImageAsset(selectedAsset)"
                  :src="getAssetUrl(selectedAsset)"
                  :alt="getAssetDisplayName(selectedAsset)"
                  class="max-h-72 w-full object-cover"
                />
                <div v-else class="flex min-h-56 flex-col items-center justify-center gap-4 px-6 text-center text-stone-500 dark:text-stone-400">
                  <div class="rounded-full border border-stone-300 px-4 py-1 text-xs font-semibold uppercase tracking-[0.3em] dark:border-stone-700">
                    {{ getAssetTypeLabel(getAssetType(selectedAsset)) }}
                  </div>
                  <p class="text-sm">
                    Preview is metadata-first for non-image assets. Open the original file for the raw document.
                  </p>
                </div>
              </div>

              <div class="grid gap-3">
                <div class="rounded-2xl border border-stone-200 bg-stone-50 px-4 py-3 dark:border-stone-800 dark:bg-stone-950/80">
                  <p class="text-[11px] font-semibold uppercase tracking-[0.24em] text-stone-400">Stored Reference</p>
                  <p class="mt-2 break-all font-mono text-xs text-stone-700 dark:text-stone-200">{{ assetReference(selectedAsset) }}</p>
                </div>

                <div class="rounded-2xl border border-stone-200 bg-stone-50 px-4 py-3 dark:border-stone-800 dark:bg-stone-950/80">
                  <p class="text-[11px] font-semibold uppercase tracking-[0.24em] text-stone-400">Metadata</p>
                  <dl class="mt-3 space-y-3 text-sm">
                    <div class="flex items-start justify-between gap-4">
                      <dt class="text-stone-500 dark:text-stone-400">MIME</dt>
                      <dd class="text-right font-medium">{{ assetMime(selectedAsset) }}</dd>
                    </div>
                    <div class="flex items-start justify-between gap-4">
                      <dt class="text-stone-500 dark:text-stone-400">Extension</dt>
                      <dd class="text-right font-medium">{{ assetExtension(selectedAsset) || 'n/a' }}</dd>
                    </div>
                    <div class="flex items-start justify-between gap-4">
                      <dt class="text-stone-500 dark:text-stone-400">Size</dt>
                      <dd class="text-right font-medium">{{ formatSize(extractPayload(selectedAsset).size_bytes) }}</dd>
                    </div>
                    <div class="flex items-start justify-between gap-4">
                      <dt class="text-stone-500 dark:text-stone-400">Uploaded</dt>
                      <dd class="text-right font-medium">{{ formatDate(selectedAsset.created_at, true) }}</dd>
                    </div>
                    <div class="flex items-start justify-between gap-4">
                      <dt class="text-stone-500 dark:text-stone-400">Hash</dt>
                      <dd class="max-w-[14rem] break-all text-right font-mono text-xs">{{ truncateHash(extractPayload(selectedAsset).hash) }}</dd>
                    </div>
                  </dl>
                </div>
              </div>

              <AssetUsagePanel
                :references="assetReferences"
                :loading="loadingReferences"
              />

              <div class="grid gap-3">
                <button
                  class="rounded-full bg-stone-900 px-4 py-2.5 text-sm font-medium text-white transition hover:bg-stone-700 dark:bg-stone-100 dark:text-stone-900 dark:hover:bg-white"
                  title="Copy Markdown Link"
                  @click="copyAssetReference(selectedAsset, 'embed')"
                >
                  Copy Embed Syntax
                </button>
                <button
                  class="rounded-full border border-stone-300 px-4 py-2.5 text-sm font-medium text-stone-700 transition hover:border-stone-500 hover:text-stone-900 dark:border-stone-700 dark:text-stone-200 dark:hover:border-stone-500 dark:hover:text-white"
                  title="Copy Asset Reference"
                  @click="copyAssetReference(selectedAsset, 'reference')"
                >
                  Copy Asset Reference
                </button>
                <a
                  class="rounded-full border border-stone-300 px-4 py-2.5 text-center text-sm font-medium text-stone-700 transition hover:border-stone-500 hover:text-stone-900 dark:border-stone-700 dark:text-stone-200 dark:hover:border-stone-500 dark:hover:text-white"
                  :href="getAssetUrl(selectedAsset)"
                  rel="noreferrer"
                  target="_blank"
                  title="Open Original"
                >
                  Open Original File
                </a>
                <button
                  class="rounded-full border border-red-300 px-4 py-2.5 text-sm font-medium text-red-700 transition hover:border-red-500 hover:bg-red-50 dark:border-red-800 dark:text-red-300 dark:hover:border-red-600 dark:hover:bg-red-950/40"
                  title="Delete Asset"
                  @click="deleteSelectedAsset"
                >
                  Delete Asset
                </button>
              </div>
            </div>

            <div v-else class="flex h-full items-center justify-center text-sm text-stone-500 dark:text-stone-400">
              Select an asset to inspect its metadata.
            </div>
          </aside>
        </div>
      </div>
    </div>
  </AssetUploadQueue>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue';
import {
  assetsApi,
  type AssetNode,
  type AssetPayload,
  type AssetReferenceItem,
  type AssetStats as AssetStatsShape,
  type AssetType,
  extractAssetPayload,
  getAssetDisplayName,
  getAssetType,
  getAssetTypeLabel,
  isImageAsset,
} from '../../api/assets';
import AssetFiltersBar from '../../components/assets/AssetFiltersBar.vue';
import AssetTable from '../../components/assets/AssetTable.vue';
import AssetUploadQueue from '../../components/assets/AssetUploadQueue.vue';
import AssetUsagePanel from '../../components/assets/AssetUsagePanel.vue';

type AssetFilter = 'all' | AssetType;
type ViewMode = 'grid' | 'table';
type SortBy = 'newest' | 'largest' | 'name';

const assets = ref<AssetNode[]>([]);
const stats = ref<AssetStatsShape>({
  total: 0,
  images: 0,
  pdfs: 0,
  files: 0,
  ip_assets: 0,
  domain_assets: 0,
  credential_stubs: 0,
  snippets: 0,
});
const loading = ref(true);
const uploading = ref(false);
const fileInput = ref<HTMLInputElement | null>(null);
const uploadQueueRef = ref<InstanceType<typeof AssetUploadQueue> | null>(null);
const searchQuery = ref('');
const activeFilter = ref<AssetFilter>('all');
const viewMode = ref<ViewMode>('grid');
const sortBy = ref<SortBy>('newest');
const selectedAssetId = ref<string | null>(null);
const actionMessage = ref('');
const assetReferences = ref<AssetReferenceItem[]>([]);
const loadingReferences = ref(false);

const sortedAssets = computed(() => {
  const items = [...assets.value];
  switch (sortBy.value) {
    case 'largest':
      return items.sort((a, b) => {
        const sizeA = extractAssetPayload(a).size_bytes ?? 0;
        const sizeB = extractAssetPayload(b).size_bytes ?? 0;
        return sizeB - sizeA;
      });
    case 'name':
      return items.sort((a, b) => {
        const nameA = getAssetDisplayName(a).toLowerCase();
        const nameB = getAssetDisplayName(b).toLowerCase();
        return nameA.localeCompare(nameB);
      });
    default: // newest
      return items.sort((a, b) => {
        const dateA = new Date(a.updated_at || a.created_at).getTime();
        const dateB = new Date(b.updated_at || b.created_at).getTime();
        return dateB - dateA;
      });
  }
});

const selectedAsset = computed(() => {
  if (!sortedAssets.value.length) {
    return null;
  }

  return sortedAssets.value.find((asset) => asset.id === selectedAssetId.value) || sortedAssets.value[0];
});

watch(sortedAssets, (items) => {
  if (!items.length) {
    selectedAssetId.value = null;
    return;
  }

  if (!selectedAssetId.value || !items.some((asset) => asset.id === selectedAssetId.value)) {
    selectedAssetId.value = items[0].id;
  }
}, { immediate: true });

watch(selectedAsset, (asset) => {
  if (!asset) {
    assetReferences.value = [];
    return;
  }

  void fetchAssetReferences(asset.id);
}, { immediate: true });

async function fetchAssets() {
  loading.value = true;
  try {
    const response = await assetsApi.list({
      limit: 200,
      q: searchQuery.value.trim() || undefined,
      asset_type: activeFilter.value === 'all' ? undefined : activeFilter.value,
      sort_by: sortBy.value,
    });
    assets.value = response.items;
    stats.value = response.stats;
  } catch (error) {
    console.error('Failed to load assets', error);
    actionMessage.value = 'Failed to load assets.';
  } finally {
    loading.value = false;
  }
}

async function fetchAssetReferences(assetId: string) {
  loadingReferences.value = true;
  try {
    assetReferences.value = await assetsApi.listReferences(assetId);
  } catch (error) {
    console.error('Failed to load asset references', error);
    assetReferences.value = [];
  } finally {
    loadingReferences.value = false;
  }
}

function triggerUpload() {
  fileInput.value?.click();
}

async function handleFileUpload(event: Event) {
  const target = event.target as HTMLInputElement;
  if (!target.files?.length) {
    return;
  }

  uploading.value = true;
  actionMessage.value = '';

  const files = Array.from(target.files);

  // Delegate to upload queue component
  if (uploadQueueRef.value) {
    uploadQueueRef.value.uploadFiles(files);
  } else {
    // Fallback: direct upload
    let successCount = 0;
    let failureCount = 0;

    for (const file of files) {
      try {
        await assetsApi.upload(file);
        successCount += 1;
      } catch (error) {
        console.error('Upload failed', error);
        failureCount += 1;
      }
    }

    await fetchAssets();

    if (failureCount === 0) {
      actionMessage.value = `Uploaded ${successCount} asset${successCount === 1 ? '' : 's'}.`;
    } else {
      actionMessage.value = `Uploaded ${successCount} asset${successCount === 1 ? '' : 's'}, ${failureCount} failed.`;
    }
  }

  target.value = '';
  uploading.value = false;
}

function extractPayload(asset: AssetNode): AssetPayload {
  return extractAssetPayload(asset);
}

function assetMime(asset: AssetNode): string {
  return extractPayload(asset).mime_type || 'application/octet-stream';
}

function assetExtension(asset: AssetNode): string | null | undefined {
  return extractPayload(asset).metadata?.extension;
}

function assetReference(asset: AssetNode): string {
  return `[[asset:${asset.id}]]`;
}

function getAssetUrl(asset: AssetNode): string {
  return assetsApi.getAssetUrl(asset.id);
}

async function copyAssetReference(asset: AssetNode, mode: 'embed' | 'reference') {
  const displayName = getAssetDisplayName(asset);
  const reference = assetReference(asset);
  const value = mode === 'embed' ? `![${displayName}](${reference})` : reference;
  await navigator.clipboard.writeText(value);
  actionMessage.value = mode === 'embed'
    ? `Copied embed syntax for ${displayName}.`
    : `Copied asset reference for ${displayName}.`;
}

async function deleteSelectedAsset() {
  if (!selectedAsset.value) {
    return;
  }

  const displayName = getAssetDisplayName(selectedAsset.value);
  const confirmed = window.confirm(`Delete asset "${displayName}"? This removes the asset record from your library.`);
  if (!confirmed) {
    return;
  }

  try {
    await assetsApi.delete(selectedAsset.value.id);
    actionMessage.value = `Deleted ${displayName}.`;
    await fetchAssets();
  } catch (error) {
    console.error('Failed to delete asset', error);
    actionMessage.value = `Failed to delete ${displayName}.`;
  }
}

function formatSize(bytes?: number): string {
  if (!bytes) {
    return '0 B';
  }

  const units = ['B', 'KB', 'MB', 'GB'];
  const exponent = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1);
  const value = bytes / Math.pow(1024, exponent);
  return `${value.toFixed(value >= 10 || exponent === 0 ? 0 : 1)} ${units[exponent]}`;
}

function formatDate(raw: string, includeTime = false): string {
  const date = new Date(raw);
  if (Number.isNaN(date.getTime())) {
    return raw;
  }

  return new Intl.DateTimeFormat('en-US', includeTime
    ? { month: 'short', day: 'numeric', year: 'numeric', hour: '2-digit', minute: '2-digit' }
    : { month: 'short', day: 'numeric', year: 'numeric' }).format(date);
}

function truncateHash(hash?: string): string {
  if (!hash) {
    return 'n/a';
  }

  if (hash.length <= 18) {
    return hash;
  }

  return `${hash.slice(0, 10)}...${hash.slice(-8)}`;
}

onMounted(() => {
  void fetchAssets();
});

watch([searchQuery, activeFilter, sortBy], () => {
  void fetchAssets();
});
</script>
