<template>
  <div class="overflow-auto">
    <table class="w-full text-sm" data-testid="asset-table">
      <thead>
        <tr class="border-b border-stone-200 text-left text-[11px] font-semibold uppercase tracking-[0.2em] text-stone-400 dark:border-stone-800">
          <th class="px-4 py-3">Name</th>
          <th class="px-4 py-3">Type</th>
          <th class="hidden px-4 py-3 md:table-cell">Size</th>
          <th class="hidden px-4 py-3 lg:table-cell">MIME</th>
          <th class="hidden px-4 py-3 xl:table-cell">Date</th>
          <th class="hidden px-4 py-3 xl:table-cell">Hash</th>
          <th class="px-4 py-3 text-right">Refs</th>
        </tr>
      </thead>
      <tbody>
        <tr
          v-for="asset in assets"
          :key="asset.id"
          :data-testid="`asset-row-${asset.id}`"
          class="cursor-pointer border-b border-stone-100 transition dark:border-stone-800/60"
          :class="selectedAssetId === asset.id
            ? 'bg-stone-100 dark:bg-stone-800/60'
            : 'hover:bg-stone-50 dark:hover:bg-stone-900/50'"
          @click="$emit('select', asset.id)"
        >
          <td class="max-w-[16rem] truncate px-4 py-3 font-medium text-stone-900 dark:text-stone-100">
            <div class="flex items-center gap-3">
              <img
                v-if="isImageAsset(asset)"
                :src="getAssetUrl(asset)"
                :alt="getAssetDisplayName(asset)"
                class="h-8 w-8 rounded-lg object-cover"
                loading="lazy"
              />
              <div v-else class="flex h-8 w-8 items-center justify-center rounded-lg bg-stone-200 text-[10px] font-semibold uppercase text-stone-500 dark:bg-stone-800 dark:text-stone-400">
                {{ getAssetTypeLabel(getAssetType(asset)).slice(0, 3) }}
              </div>
              <span class="truncate">{{ getAssetDisplayName(asset) }}</span>
            </div>
          </td>
          <td class="px-4 py-3">
            <span class="rounded-full bg-stone-100 px-2 py-0.5 text-xs font-medium text-stone-600 dark:bg-stone-800 dark:text-stone-300">
              {{ getAssetTypeLabel(getAssetType(asset)) }}
            </span>
          </td>
          <td class="hidden px-4 py-3 text-stone-500 dark:text-stone-400 md:table-cell">
            {{ formatSize(extractPayload(asset).size_bytes) }}
          </td>
          <td class="hidden px-4 py-3 text-stone-500 dark:text-stone-400 lg:table-cell">
            {{ extractPayload(asset).mime_type || 'n/a' }}
          </td>
          <td class="hidden px-4 py-3 text-stone-500 dark:text-stone-400 xl:table-cell">
            {{ formatDate(asset.created_at) }}
          </td>
          <td class="hidden px-4 py-3 font-mono text-xs text-stone-400 xl:table-cell">
            {{ truncateHash(extractPayload(asset).hash) }}
          </td>
          <td class="px-4 py-3 text-right text-stone-500 dark:text-stone-400">
            {{ getUsedInCount(asset) }}
          </td>
        </tr>
      </tbody>
    </table>

    <div v-if="assets.length === 0" class="rounded-[1.75rem] border border-dashed border-stone-300 bg-white/80 p-8 text-center text-sm text-stone-600 dark:border-stone-700 dark:bg-stone-900/70 dark:text-stone-300">
      No assets match the current search or filter.
    </div>
  </div>
</template>

<script setup lang="ts">
import {
  type AssetNode,
  type AssetPayload,
  extractAssetPayload,
  getAssetDisplayName,
  getAssetType,
  getAssetTypeLabel,
  isImageAsset,
  assetsApi,
} from '../../api/assets';

defineProps<{
  assets: AssetNode[];
  selectedAssetId: string | null;
}>();

defineEmits<{
  select: [id: string];
}>();

function extractPayload(asset: AssetNode): AssetPayload {
  return extractAssetPayload(asset);
}

function getAssetUrl(asset: AssetNode): string {
  return assetsApi.getAssetUrl(asset.id);
}

function formatSize(bytes?: number): string {
  if (!bytes) return '0 B';
  const units = ['B', 'KB', 'MB', 'GB'];
  const exponent = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1);
  const value = bytes / Math.pow(1024, exponent);
  return `${value.toFixed(value >= 10 || exponent === 0 ? 0 : 1)} ${units[exponent]}`;
}

function formatDate(raw: string): string {
  const date = new Date(raw);
  if (Number.isNaN(date.getTime())) return raw;
  return new Intl.DateTimeFormat('en-US', { month: 'short', day: 'numeric', year: 'numeric' }).format(date);
}

function truncateHash(hash?: string): string {
  if (!hash) return 'n/a';
  if (hash.length <= 18) return hash;
  return `${hash.slice(0, 10)}...${hash.slice(-8)}`;
}

function getUsedInCount(asset: AssetNode): string | number {
  const raw = asset.body?.data as unknown as Record<string, unknown> | undefined;
  const count = raw?.used_in_count;
  return typeof count === 'number' ? count : '-';
}
</script>
