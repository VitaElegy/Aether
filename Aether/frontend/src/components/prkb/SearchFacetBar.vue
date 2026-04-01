<template>
  <div class="flex items-center gap-2">
    <!-- Search input with DSL support -->
    <div class="relative flex-1">
      <i class="ri-search-line absolute left-3 top-1/2 -translate-y-1/2 text-gray-400"></i>
      <input
        v-model="query"
        type="text"
        :placeholder="'Search papers... (try author:Name venue:ICLR year:2024 tag:ai)'"
        class="w-full pl-9 pr-4 py-2 border border-gray-300 rounded-md text-sm focus:outline-none focus:ring-1 focus:ring-blue-400"
        @keydown.enter="search"
      />
      <button v-if="query" @click="clearSearch" class="absolute right-2 top-1/2 -translate-y-1/2 text-gray-400 hover:text-gray-600">
        <i class="ri-close-line"></i>
      </button>
    </div>

    <!-- Facet filters -->
    <select v-model="selectedVenue" @change="search" class="text-sm border border-gray-300 rounded-md px-3 py-2">
      <option value="">All Venues</option>
      <option v-for="v in venues" :key="v.id" :value="v.id">{{ v.name }} {{ v.tier ? `(${v.tier})` : '' }}</option>
    </select>

    <select v-model="selectedYear" @change="search" class="text-sm border border-gray-300 rounded-md px-3 py-2 w-28">
      <option value="">All Years</option>
      <option v-for="y in yearOptions" :key="y" :value="y">{{ y }}</option>
    </select>

    <select v-model="selectedState" @change="search" class="text-sm border border-gray-300 rounded-md px-3 py-2 w-28">
      <option value="">All States</option>
      <option value="Inbox">Inbox</option>
      <option value="Reading">Reading</option>
      <option value="Done">Done</option>
    </select>

    <label class="flex items-center gap-1 text-sm text-gray-600 cursor-pointer">
      <input type="checkbox" v-model="hasPdf" @change="search" class="rounded border-gray-300" />
      Has PDF
    </label>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import type { Venue } from '@/stores/prkb';

const props = defineProps<{ venues: Venue[] }>();
const emit = defineEmits(['search']);

const query = ref('');
const selectedVenue = ref('');
const selectedYear = ref('');
const selectedState = ref('');
const hasPdf = ref(false);

const currentYear = new Date().getFullYear();
const yearOptions = computed(() => {
  const years = [];
  for (let y = currentYear; y >= currentYear - 10; y--) {
    years.push(y);
  }
  return years;
});

const search = () => {
  // Build DSL query
  let dsl = query.value;

  // Append facet filters to DSL
  if (selectedYear.value) dsl += ` year:${selectedYear.value}`;
  if (selectedState.value) dsl += ` state:${selectedState.value}`;

  emit('search', {
    q: dsl.trim() || undefined,
    venue_id: selectedVenue.value || undefined,
    year: selectedYear.value ? parseInt(selectedYear.value) : undefined,
    state: selectedState.value || undefined,
    has_pdf: hasPdf.value || undefined,
  });
};

const clearSearch = () => {
  query.value = '';
  selectedVenue.value = '';
  selectedYear.value = '';
  selectedState.value = '';
  hasPdf.value = false;
  emit('search', {});
};
</script>
