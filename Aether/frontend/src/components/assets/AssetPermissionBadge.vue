<template>
  <div class="inline-flex flex-col">
    <!-- Badge trigger -->
    <button
      class="inline-flex items-center gap-1.5 rounded-full border px-2.5 py-1 text-[11px] font-semibold uppercase tracking-[0.18em] transition"
      :class="badgeClasses"
      :title="explanation?.reason_text || 'Check permission'"
      @click="expanded = !expanded"
    >
      <span class="h-1.5 w-1.5 rounded-full" :class="dotClass" />
      {{ badgeLabel }}
      <svg
        v-if="explanation"
        class="h-3 w-3 transition-transform"
        :class="{ 'rotate-180': expanded }"
        viewBox="0 0 20 20"
        fill="currentColor"
      >
        <path
          fill-rule="evenodd"
          d="M5.23 7.21a.75.75 0 011.06.02L10 11.168l3.71-3.938a.75.75 0 111.08 1.04l-4.25 4.5a.75.75 0 01-1.08 0l-4.25-4.5a.75.75 0 01.02-1.06z"
          clip-rule="evenodd"
        />
      </svg>
    </button>

    <!-- Expandable detail panel -->
    <Transition name="slide">
      <div
        v-if="expanded && explanation"
        class="mt-2 rounded-2xl border border-stone-200 bg-stone-50 px-4 py-3 dark:border-stone-800 dark:bg-stone-950/80"
      >
        <!-- Reason -->
        <p class="text-xs text-stone-700 dark:text-stone-200">
          {{ explanation.reason_text }}
        </p>
        <span
          class="mt-1 inline-block rounded-full border border-stone-300 px-2 py-0.5 text-[10px] font-semibold uppercase tracking-[0.2em] text-stone-600 dark:border-stone-700 dark:text-stone-300"
        >
          {{ explanation.reason_code }}
        </span>

        <!-- Context chain -->
        <div v-if="explanation.context_chain.length > 0" class="mt-3">
          <p class="text-[11px] font-semibold uppercase tracking-[0.24em] text-stone-400">
            Permission Chain
          </p>
          <ul class="mt-1.5 space-y-1.5">
            <li
              v-for="(link, index) in explanation.context_chain"
              :key="index"
              class="flex items-center gap-2 text-xs text-stone-600 dark:text-stone-300"
            >
              <span class="font-mono text-[10px] text-stone-400">{{ index + 1 }}.</span>
              <span class="rounded bg-stone-200 px-1.5 py-0.5 text-[10px] dark:bg-stone-800">
                {{ link.entity_type }}
              </span>
              <span class="truncate font-mono text-[10px]">{{ shortId(link.entity_id) }}</span>
              <span class="text-stone-400">&rarr;</span>
              <span class="font-medium">{{ link.relation }}</span>
              <span class="text-stone-400">via</span>
              <span
                class="rounded-full border border-stone-300 px-1.5 py-0.5 text-[10px] dark:border-stone-700"
              >
                {{ link.via }}
              </span>
            </li>
          </ul>
        </div>

        <!-- Referenced by -->
        <div v-if="explanation.referenced_by.length > 0" class="mt-3">
          <p class="text-[11px] font-semibold uppercase tracking-[0.24em] text-stone-400">
            Referenced By
          </p>
          <ul class="mt-1.5 space-y-1">
            <li
              v-for="refId in explanation.referenced_by"
              :key="refId"
              class="truncate font-mono text-[10px] text-stone-500 dark:text-stone-400"
            >
              {{ refId }}
            </li>
          </ul>
        </div>
      </div>
    </Transition>

    <!-- Loading state -->
    <p v-if="loading" class="mt-1 text-[10px] text-stone-400">Checking permissions...</p>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue';
import { permissionsApi, type PermissionExplanation } from '../../api/permissions';

const props = defineProps<{
  assetId: string;
  contextId?: string;
}>();

const loading = ref(false);
const expanded = ref(false);
const explanation = ref<PermissionExplanation | null>(null);

const badgeLabel = computed(() => {
  if (loading.value) return 'Checking';
  if (!explanation.value) return 'Unknown';
  return explanation.value.allowed ? 'Allowed' : 'Denied';
});

const badgeClasses = computed(() => {
  if (loading.value || !explanation.value) {
    return 'border-stone-300 text-stone-500 dark:border-stone-600 dark:text-stone-400';
  }
  return explanation.value.allowed
    ? 'border-emerald-300 text-emerald-700 hover:border-emerald-400 dark:border-emerald-700 dark:text-emerald-300'
    : 'border-red-300 text-red-700 hover:border-red-400 dark:border-red-700 dark:text-red-300';
});

const dotClass = computed(() => {
  if (loading.value || !explanation.value) {
    return 'bg-stone-400';
  }
  return explanation.value.allowed ? 'bg-emerald-500' : 'bg-red-500';
});

function shortId(id: string): string {
  return id.length > 12 ? `${id.slice(0, 8)}...` : id;
}

async function fetchExplanation() {
  loading.value = true;
  try {
    explanation.value = await permissionsApi.explainAssetAccess(
      props.assetId,
      props.contextId,
    );
  } catch {
    explanation.value = null;
  } finally {
    loading.value = false;
  }
}

onMounted(fetchExplanation);

watch(
  () => [props.assetId, props.contextId],
  () => {
    expanded.value = false;
    fetchExplanation();
  },
);
</script>

<style scoped>
.slide-enter-active,
.slide-leave-active {
  transition: all 0.2s ease;
}
.slide-enter-from,
.slide-leave-to {
  opacity: 0;
  transform: translateY(-4px);
}
</style>
