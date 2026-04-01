<template>
  <div class="h-screen w-full bg-[#f8f9fa] text-gray-900 font-sans flex overflow-hidden">
    <!-- Sidebar -->
    <aside class="w-64 border-r border-gray-200 bg-white flex flex-col shrink-0">
      <div class="p-6 border-b border-gray-100">
        <h1 class="font-serif text-xl font-bold tracking-tight text-gray-800">Papers</h1>
        <p class="text-xs text-gray-500 mt-1 uppercase tracking-wider">Research Knowledge Base</p>
      </div>

      <nav class="flex-1 overflow-y-auto py-4 px-3 space-y-1">
        <div class="px-3 mb-2 text-xs font-semibold text-gray-400 uppercase tracking-wider">Flow</div>
        <button
          @click="$emit('update:activeTab', 'inbox')"
          :class="[activeTab === 'inbox' ? 'bg-gray-100 text-gray-900' : 'text-gray-600 hover:bg-gray-50 hover:text-gray-900']"
          class="w-full flex items-center px-3 py-2 text-sm font-medium rounded-md group"
        >
          <i class="ri-inbox-line mr-3 text-gray-500"></i> Inbox
          <span v-if="store.inboxTotalCount > 0" class="ml-auto bg-gray-200 py-0.5 px-2 rounded-full text-xs text-gray-600">{{ store.inboxTotalCount }}</span>
        </button>
        <button
          @click="$emit('update:activeTab', 'library')"
          :class="[activeTab === 'library' ? 'bg-gray-100 text-gray-900' : 'text-gray-600 hover:bg-gray-50 hover:text-gray-900']"
          class="w-full flex items-center px-3 py-2 text-sm font-medium rounded-md group"
        >
          <i class="ri-book-2-line mr-3 text-gray-400"></i> Library
          <span v-if="store.library.length > 0" class="ml-auto bg-gray-200 py-0.5 px-2 rounded-full text-xs text-gray-600">{{ store.library.length }}</span>
        </button>
        <button
          @click="$emit('update:activeTab', 'collections')"
          :class="[activeTab === 'collections' ? 'bg-gray-100 text-gray-900' : 'text-gray-600 hover:bg-gray-50 hover:text-gray-900']"
          class="w-full flex items-center px-3 py-2 text-sm font-medium rounded-md group"
        >
          <i class="ri-folder-3-line mr-3 text-gray-400"></i> Collections
          <span v-if="store.collections.length > 0" class="ml-auto bg-gray-200 py-0.5 px-2 rounded-full text-xs text-gray-600">{{ store.collections.length }}</span>
        </button>

        <!-- PRKB-01: Feeds section -->
        <div class="mt-6 px-3 mb-2 flex items-center justify-between text-xs font-semibold text-gray-400 uppercase tracking-wider">
          <span>Feeds</span>
          <div class="flex space-x-2">
            <span class="text-[10px] cursor-pointer hover:text-gray-600" @click="$emit('update:activeTab', 'feeds')">MANAGE</span>
            <span class="text-[10px] cursor-pointer hover:text-gray-600" @click="store.selectAllFeeds()">TOGGLE</span>
          </div>
        </div>

        <div v-if="store.feeds.length === 0" class="px-3 py-2 text-sm text-gray-500 italic">
          No feeds added.
        </div>

        <div v-for="feed in store.feeds" :key="feed.id"
          class="group flex items-center justify-between px-3 py-2 text-sm text-gray-600 rounded-md hover:bg-gray-50 transition-colors cursor-pointer"
          @click="store.toggleFeedSelection(feed.id)"
        >
          <span class="truncate max-w-[150px] flex items-center" :title="feed.name">
            <input
              type="checkbox"
              :checked="store.selectedFeeds.has(feed.id)"
              @click.stop
              @change="store.toggleFeedSelection(feed.id)"
              class="mr-2 cursor-pointer w-3.5 h-3.5 text-blue-600 rounded border-gray-300 focus:ring-blue-500"
            />
            <!-- Health indicator -->
            <span v-if="store.loadingFeeds.has(feed.id)" class="w-1.5 h-1.5 mr-1.5 inline-block rounded-full border-2 border-blue-400 border-t-transparent animate-spin"></span>
            <span v-else class="w-1.5 h-1.5 mr-1.5 inline-block rounded-full"
              :class="{
                'bg-green-500': feed.health_status === 'healthy',
                'bg-yellow-500': feed.health_status === 'degraded',
                'bg-red-500': feed.health_status === 'error',
                'bg-gray-300': feed.health_status === 'unknown' || !feed.health_status,
              }"
              :title="feed.health_status || 'unknown'"
            ></span>
            <span :class="{ 'opacity-50 line-through': !feed.enabled }">{{ feed.name }}</span>
          </span>
          <div class="flex items-center opacity-0 group-hover:opacity-100 transition-opacity space-x-1">
            <button
              @click.stop="store.refreshFeeds(feed.id)"
              class="p-1 text-gray-400 hover:text-blue-600 rounded hover:bg-blue-50"
              :class="{'animate-spin text-blue-600': store.loadingFeeds.has(feed.id)}"
              title="Fetch updates now"
              :disabled="store.loadingFeeds.has(feed.id)"
            >
              <i class="ri-refresh-line"></i>
            </button>
            <button
              @click.stop="deleteFeed(feed.id)"
              class="p-1 text-gray-400 hover:text-red-600 rounded hover:bg-red-50"
              title="Remove feed"
            >
              <i class="ri-delete-bin-line"></i>
            </button>
          </div>
        </div>

        <!-- VENUES FACET -->
        <div class="mt-6 px-3 mb-2 flex items-center justify-between text-xs font-semibold text-gray-400 uppercase tracking-wider">
          <span>Venues</span>
        </div>
        <div v-if="store.venues && store.venues.length === 0" class="px-3 py-2 text-sm text-gray-500 italic">
          No venues found.
        </div>
        <div v-for="venue in (store.venues || [])" :key="venue.id"
          class="group flex items-center justify-between px-3 py-2 text-sm text-gray-600 rounded-md hover:bg-gray-50 transition-colors cursor-pointer"
          :class="{'bg-blue-50 text-blue-700': selectedVenueId === venue.id}"
          @click="selectVenue(venue.id)"
        >
          <span class="truncate">{{ venue.name }}</span>
          <span v-if="venue.tier" class="text-[10px] bg-gray-100 text-gray-500 px-1 rounded">{{ venue.tier }}</span>
        </div>

        <!-- Tools section -->
        <div class="mt-6 px-3 mb-2 text-xs font-semibold text-gray-400 uppercase tracking-wider">Tools</div>
        <button
          @click="$emit('update:activeTab', 'portability')"
          :class="[activeTab === 'portability' ? 'bg-gray-100 text-gray-900' : 'text-gray-600 hover:bg-gray-50 hover:text-gray-900']"
          class="w-full flex items-center px-3 py-2 text-sm font-medium rounded-md group"
        >
          <i class="ri-upload-cloud-line mr-3 text-gray-400"></i> Import / Export
        </button>
      </nav>

      <div class="p-4 border-t border-gray-200">
        <button
          @click="$emit('add-feed')"
          class="w-full flex justify-center items-center px-4 py-2 border border-gray-300 shadow-sm text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50"
        >
          <span class="mr-2">+</span> Add Feed
        </button>
      </div>
    </aside>

    <!-- Main Content -->
    <main class="flex-1 overflow-y-auto bg-[#f8f9fa] p-8">
      <div class="max-w-7xl mx-auto">
        <div class="flex items-center justify-between mb-8">
          <h2 class="text-2xl font-serif font-semibold text-gray-900 capitalize">{{ activeTab }}</h2>
          <div class="flex space-x-2 items-center">
            <slot name="tools" />
          </div>
        </div>
        <div class="min-h-[500px]">
          <slot />
        </div>
      </div>
    </main>
  </div>
</template>

<script setup lang="ts">
import { usePrkbStore } from '@/stores/prkb';
import { ref, onMounted, watch } from 'vue';

const props = defineProps<{
  activeTab: string
}>();

const emit = defineEmits(['update:activeTab', 'add-feed']);

const store = usePrkbStore();
const selectedVenueId = ref<string | undefined>(undefined);

const deleteFeed = async (id: string) => {
  if (confirm('Are you sure you want to remove this feed?')) {
    await store.deleteFeed(id);
  }
};

const selectVenue = (id: string) => {
  if (selectedVenueId.value === id) {
    selectedVenueId.value = undefined;
  } else {
    selectedVenueId.value = id;
  }
  emit('update:activeTab', 'library');
  store.fetchLibrary(selectedVenueId.value);
};

watch(() => props.activeTab, (newTab) => {
  if (newTab !== 'library') {
    selectedVenueId.value = undefined;
  }
});

onMounted(() => {
  if (store.fetchPublications) store.fetchPublications();
  if (store.fetchVenues) store.fetchVenues();
});
</script>
