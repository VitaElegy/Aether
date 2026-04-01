<template>
  <section class="border-b border-stone-200 bg-stone-50/80 px-6 py-4 dark:border-stone-800 dark:bg-stone-900/50">
    <div class="flex flex-col gap-4 xl:flex-row xl:items-center xl:justify-between">
      <label class="flex min-w-0 flex-1 items-center rounded-2xl border border-stone-200 bg-white px-4 py-3 shadow-sm dark:border-stone-700 dark:bg-stone-900">
        <span class="mr-3 text-xs font-semibold uppercase tracking-[0.24em] text-stone-400">Search</span>
        <input
          :value="searchQuery"
          class="min-w-0 flex-1 bg-transparent text-sm outline-none placeholder:text-stone-400"
          placeholder="Name, hash, extension, or MIME type"
          type="text"
          @input="$emit('update:searchQuery', ($event.target as HTMLInputElement).value)"
        />
      </label>

      <div class="flex items-center gap-3">
        <!-- Sort dropdown -->
        <div class="flex items-center gap-2">
          <span class="text-[11px] font-semibold uppercase tracking-[0.2em] text-stone-400">Sort</span>
          <select
            :value="sortBy"
            class="rounded-xl border border-stone-200 bg-white px-3 py-2 text-sm text-stone-700 outline-none dark:border-stone-700 dark:bg-stone-900 dark:text-stone-200"
            @change="$emit('update:sortBy', ($event.target as HTMLSelectElement).value as SortBy)"
          >
            <option value="newest">Newest</option>
            <option value="largest">Largest</option>
            <option value="name">Name</option>
          </select>
        </div>

        <!-- View toggle -->
        <div class="flex rounded-xl border border-stone-200 dark:border-stone-700">
          <button
            class="rounded-l-xl px-3 py-2 text-sm font-medium transition"
            :class="viewMode === 'grid'
              ? 'bg-stone-900 text-white dark:bg-stone-100 dark:text-stone-900'
              : 'bg-white text-stone-600 hover:text-stone-900 dark:bg-stone-900 dark:text-stone-400 dark:hover:text-stone-100'"
            title="Grid view"
            @click="$emit('update:viewMode', 'grid')"
          >
            <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="7" height="7" /><rect x="14" y="3" width="7" height="7" /><rect x="3" y="14" width="7" height="7" /><rect x="14" y="14" width="7" height="7" /></svg>
          </button>
          <button
            class="rounded-r-xl px-3 py-2 text-sm font-medium transition"
            :class="viewMode === 'table'
              ? 'bg-stone-900 text-white dark:bg-stone-100 dark:text-stone-900'
              : 'bg-white text-stone-600 hover:text-stone-900 dark:bg-stone-900 dark:text-stone-400 dark:hover:text-stone-100'"
            title="Table view"
            @click="$emit('update:viewMode', 'table')"
          >
            <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="3" y1="6" x2="21" y2="6" /><line x1="3" y1="12" x2="21" y2="12" /><line x1="3" y1="18" x2="21" y2="18" /></svg>
          </button>
        </div>
      </div>
    </div>

    <!-- Type filter pills -->
    <div class="mt-4 flex flex-wrap gap-2">
      <button
        v-for="filterOption in FILTER_OPTIONS"
        :key="filterOption.id"
        :data-testid="`asset-filter-${filterOption.id}`"
        class="rounded-full border px-3 py-2 text-sm font-medium transition"
        :class="activeFilter === filterOption.id
          ? 'border-stone-900 bg-stone-900 text-white dark:border-stone-100 dark:bg-stone-100 dark:text-stone-900'
          : 'border-stone-200 bg-white text-stone-700 hover:border-stone-400 dark:border-stone-700 dark:bg-stone-900 dark:text-stone-200 dark:hover:border-stone-500'"
        @click="$emit('update:activeFilter', filterOption.id)"
      >
        {{ filterOption.label }}
        <span class="ml-2 text-xs opacity-70">{{ getFilterCount(filterOption.id) }}</span>
      </button>
    </div>

    <p
      v-if="actionMessage"
      data-testid="asset-action-message"
      class="mt-3 text-sm text-stone-600 dark:text-stone-300"
    >
      {{ actionMessage }}
    </p>
  </section>
</template>

<script setup lang="ts">
import type { AssetType, AssetStats } from '../../api/assets';

type AssetFilter = 'all' | AssetType;
type ViewMode = 'grid' | 'table';
type SortBy = 'newest' | 'largest' | 'name';

const FILTER_OPTIONS: Array<{ id: AssetFilter; label: string }> = [
  { id: 'all', label: 'All' },
  { id: 'image_asset', label: 'Images' },
  { id: 'pdf_asset', label: 'PDFs' },
  { id: 'file_asset', label: 'Files' },
  { id: 'ip_asset', label: 'IPs' },
  { id: 'domain_asset', label: 'Domains' },
  { id: 'credential_stub', label: 'Credentials' },
  { id: 'snippet_asset', label: 'Snippets' },
];

const props = defineProps<{
  searchQuery: string;
  activeFilter: AssetFilter;
  viewMode: ViewMode;
  sortBy: SortBy;
  stats: AssetStats;
  actionMessage: string;
}>();

defineEmits<{
  'update:searchQuery': [value: string];
  'update:activeFilter': [value: AssetFilter];
  'update:viewMode': [value: ViewMode];
  'update:sortBy': [value: SortBy];
}>();

function getFilterCount(filterId: AssetFilter): number {
  switch (filterId) {
    case 'image_asset':
      return props.stats.images;
    case 'pdf_asset':
      return props.stats.pdfs;
    case 'file_asset':
      return props.stats.files;
    case 'ip_asset':
      return props.stats.ip_assets;
    case 'domain_asset':
      return props.stats.domain_assets;
    case 'credential_stub':
      return props.stats.credential_stubs;
    case 'snippet_asset':
      return props.stats.snippets;
    default:
      return props.stats.total;
  }
}
</script>
