<template>
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

    <section class="border-b border-stone-200 bg-stone-50/80 px-6 py-4 dark:border-stone-800 dark:bg-stone-900/50">
      <div class="flex flex-col gap-4 xl:flex-row xl:items-center xl:justify-between">
        <label class="flex min-w-0 flex-1 items-center rounded-2xl border border-stone-200 bg-white px-4 py-3 shadow-sm dark:border-stone-700 dark:bg-stone-900">
          <span class="mr-3 text-xs font-semibold uppercase tracking-[0.24em] text-stone-400">Search</span>
          <input
            v-model="searchQuery"
            class="min-w-0 flex-1 bg-transparent text-sm outline-none placeholder:text-stone-400"
            placeholder="Name, hash, extension, or MIME type"
            type="text"
          />
        </label>

        <div class="flex flex-wrap gap-2">
          <button
            v-for="filterOption in FILTER_OPTIONS"
            :key="filterOption.id"
            :data-testid="`asset-filter-${filterOption.id}`"
            class="rounded-full border px-3 py-2 text-sm font-medium transition"
            :class="activeFilter === filterOption.id
              ? 'border-stone-900 bg-stone-900 text-white dark:border-stone-100 dark:bg-stone-100 dark:text-stone-900'
              : 'border-stone-200 bg-white text-stone-700 hover:border-stone-400 dark:border-stone-700 dark:bg-stone-900 dark:text-stone-200 dark:hover:border-stone-500'"
            @click="activeFilter = filterOption.id"
          >
            {{ filterOption.label }}
            <span class="ml-2 text-xs opacity-70">{{ countForFilter(filterOption.id) }}</span>
          </button>
        </div>
      </div>

      <p
        v-if="actionMessage"
        data-testid="asset-action-message"
        class="mt-3 text-sm text-stone-600 dark:text-stone-300"
      >
        {{ actionMessage }}
      </p>
    </section>

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

            <div class="rounded-2xl border border-stone-200 bg-stone-50 px-4 py-3 dark:border-stone-800 dark:bg-stone-950/80">
              <div class="flex items-center justify-between gap-4">
                <p class="text-[11px] font-semibold uppercase tracking-[0.24em] text-stone-400">Used In</p>
                <span class="text-xs text-stone-500 dark:text-stone-400">{{ assetReferences.length }} refs</span>
              </div>

              <div v-if="loadingReferences" class="mt-3 text-sm text-stone-500 dark:text-stone-400">
                Loading reference contexts...
              </div>

              <div v-else-if="assetReferences.length === 0" class="mt-3 text-sm text-stone-500 dark:text-stone-400">
                No content references this asset yet.
              </div>

              <div v-else class="mt-3 space-y-3" data-testid="asset-reference-list">
                <article
                  v-for="reference in assetReferences"
                  :key="reference.content_id"
                  class="rounded-2xl border border-stone-200 bg-white px-3 py-3 dark:border-stone-700 dark:bg-stone-900"
                >
                  <div class="flex items-start justify-between gap-3">
                    <div class="min-w-0">
                      <p class="truncate text-sm font-semibold text-stone-900 dark:text-stone-100">{{ reference.title }}</p>
                      <p class="mt-1 text-xs text-stone-500 dark:text-stone-400">
                        {{ reference.knowledge_base_title || reference.category || 'Uncategorized' }}
                      </p>
                    </div>
                    <span class="rounded-full border border-stone-300 px-2 py-1 text-[10px] font-semibold uppercase tracking-[0.2em] text-stone-600 dark:border-stone-700 dark:text-stone-300">
                      {{ reference.reference_type }}
                    </span>
                  </div>
                  <p class="mt-2 line-clamp-2 text-xs text-stone-600 dark:text-stone-300">{{ reference.snippet }}</p>
                  <p class="mt-2 text-[11px] text-stone-400">{{ formatDate(reference.updated_at, true) }}</p>
                </article>
              </div>
            </div>

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

type AssetFilter = 'all' | AssetType;

const FILTER_OPTIONS: Array<{ id: AssetFilter; label: string }> = [
  { id: 'all', label: 'All' },
  { id: 'image_asset', label: 'Images' },
  { id: 'pdf_asset', label: 'PDFs' },
  { id: 'file_asset', label: 'Files' },
];

const assets = ref<AssetNode[]>([]);
const stats = ref<AssetStatsShape>({ total: 0, images: 0, pdfs: 0, files: 0 });
const loading = ref(true);
const uploading = ref(false);
const fileInput = ref<HTMLInputElement | null>(null);
const searchQuery = ref('');
const activeFilter = ref<AssetFilter>('all');
const selectedAssetId = ref<string | null>(null);
const actionMessage = ref('');
const assetReferences = ref<AssetReferenceItem[]>([]);
const loadingReferences = ref(false);

const sortedAssets = computed(() =>
  [...assets.value].sort((left, right) => {
    const leftDate = new Date(left.updated_at || left.created_at).getTime();
    const rightDate = new Date(right.updated_at || right.created_at).getTime();
    return rightDate - leftDate;
  }),
);

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
  target.value = '';
  uploading.value = false;

  if (failureCount === 0) {
    actionMessage.value = `Uploaded ${successCount} asset${successCount === 1 ? '' : 's'}.`;
  } else {
    actionMessage.value = `Uploaded ${successCount} asset${successCount === 1 ? '' : 's'}, ${failureCount} failed.`;
  }
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

function countForFilter(filterId: AssetFilter): number {
  switch (filterId) {
    case 'image_asset':
      return stats.value.images;
    case 'pdf_asset':
      return stats.value.pdfs;
    case 'file_asset':
      return stats.value.files;
    default:
      return stats.value.total;
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

watch([searchQuery, activeFilter], () => {
  void fetchAssets();
});
</script>
