<template>
  <div class="rounded-2xl border border-stone-200 bg-stone-50 px-4 py-3 dark:border-stone-800 dark:bg-stone-950/80">
    <div class="flex items-center justify-between gap-4">
      <p class="text-[11px] font-semibold uppercase tracking-[0.24em] text-stone-400">Used In</p>
      <span class="text-xs text-stone-500 dark:text-stone-400">{{ references.length }} refs</span>
    </div>

    <div v-if="loading" class="mt-3 text-sm text-stone-500 dark:text-stone-400">
      Loading reference contexts...
    </div>

    <div v-else-if="references.length === 0" class="mt-3 text-sm text-stone-500 dark:text-stone-400">
      No content references this asset yet.
    </div>

    <div v-else class="mt-3 space-y-3" data-testid="asset-reference-list">
      <article
        v-for="reference in references"
        :key="reference.content_id"
        class="cursor-pointer rounded-2xl border border-stone-200 bg-white px-3 py-3 transition hover:border-stone-400 hover:shadow-sm dark:border-stone-700 dark:bg-stone-900 dark:hover:border-stone-500"
        @click="navigateToReference(reference)"
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
        <p class="mt-2 text-[11px] text-stone-400">{{ formatDate(reference.updated_at) }}</p>
      </article>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { AssetReferenceItem } from '../../api/assets';

defineProps<{
  references: AssetReferenceItem[];
  loading: boolean;
}>();

function navigateToReference(reference: AssetReferenceItem) {
  // Navigate to the article that references this asset
  const kbId = reference.knowledge_base_id;
  const contentId = reference.content_id;
  if (kbId && contentId) {
    window.open(`/kb/${kbId}/article/${contentId}`, '_blank');
  }
}

function formatDate(raw: string): string {
  const date = new Date(raw);
  if (Number.isNaN(date.getTime())) return raw;
  return new Intl.DateTimeFormat('en-US', {
    month: 'short',
    day: 'numeric',
    year: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  }).format(date);
}
</script>
