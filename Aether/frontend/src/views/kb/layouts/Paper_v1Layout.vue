<template>
  <div class="h-screen w-full bg-[#f8f9fa] text-gray-900 font-sans flex overflow-hidden">
    <!-- Sidebar -->
    <aside class="w-64 border-r border-gray-200 bg-white flex flex-col shrink-0">
      <div class="p-6 border-b border-gray-100">
        <h1 class="font-serif text-xl font-bold tracking-tight text-gray-800">Papers</h1>
        <p class="text-xs text-gray-500 mt-1 uppercase tracking-wider">Computer Science</p>
      </div>

      <nav class="flex-1 overflow-y-auto py-4 px-3 space-y-1">
        <div class="px-3 mb-2 text-xs font-semibold text-gray-400 uppercase tracking-wider">Flow</div>
        <button 
          @click="$emit('update:activeTab', 'inbox')"
          :class="[activeTab === 'inbox' ? 'bg-gray-100 text-gray-900' : 'text-gray-600 hover:bg-gray-50 hover:text-gray-900']"
          class="w-full flex items-center px-3 py-2 text-sm font-medium rounded-md group"
        >
          <span class="mr-3 text-gray-500">📥</span> Inbox 
          <span v-if="store.inbox.length > 0" class="ml-auto bg-gray-200 py-0.5 px-2 rounded-full text-xs text-gray-600">{{ store.inbox.length }}</span>
        </button>
        <button 
          @click="$emit('update:activeTab', 'library')"
          :class="[activeTab === 'library' ? 'bg-gray-100 text-gray-900' : 'text-gray-600 hover:bg-gray-50 hover:text-gray-900']"
          class="w-full flex items-center px-3 py-2 text-sm font-medium rounded-md group"
        >
          <span class="mr-3 text-gray-400">📚</span> Library
          <span v-if="store.library.length > 0" class="ml-auto bg-gray-200 py-0.5 px-2 rounded-full text-xs text-gray-600">{{ store.library.length }}</span>
        </button>
        <button 
           @click="$emit('update:activeTab', 'favorites')"
           :class="[activeTab === 'favorites' ? 'bg-gray-100 text-gray-900' : 'text-gray-600 hover:bg-gray-50 hover:text-gray-900']"
           class="w-full flex items-center px-3 py-2 text-sm font-medium rounded-md group"
        >
          <span class="mr-3 text-gray-400">⭐️</span> Favorites
        </button>

        <div class="mt-8 px-3 mb-2 flex items-center justify-between text-xs font-semibold text-gray-400 uppercase tracking-wider">
            <span>Feeds</span>
            <span class="text-[10px] cursor-pointer hover:text-gray-600" @click="store.fetchFeeds()">REFRESH LIST</span>
        </div>
        
        <div v-if="store.feeds.length === 0" class="px-3 py-2 text-sm text-gray-500 italic">
            No feeds added.
        </div>

        <div v-for="feed in store.feeds" :key="feed.id" class="group flex items-center justify-between px-3 py-2 text-sm text-gray-600 rounded-md hover:bg-gray-50 transition-colors">
            <span class="truncate max-w-[120px] flex items-center" :title="feed.name">
                 <!-- Status Dot / Spinner -->
                 <span v-if="store.loadingFeeds.has(feed.id)" class="w-2 h-2 mr-2 inline-block rounded-full border-2 border-blue-400 border-t-transparent animate-spin"></span>
                 <span v-else class="w-2 h-2 mr-2 inline-block rounded-full" 
                       :class="feed.last_fetched_at ? 'bg-green-500' : 'bg-gray-300'"
                       :title="feed.last_fetched_at ? 'Last fetched: ' + new Date(feed.last_fetched_at).toLocaleString() : 'Never fetched'"
                 ></span>
                 {{ feed.name }}
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
        <!-- Breadcrumbs / Tools -->
        <div class="flex items-center justify-between mb-8">
            <h2 class="text-2xl font-serif font-semibold text-gray-900 capitalize">{{ activeTab }}</h2>
            <div class="flex space-x-2 items-center">
                <slot name="tools" />
                <input type="text" placeholder="Search papers..." class="px-4 py-2 border border-gray-300 rounded-md text-sm focus:outline-none focus:ring-1 focus:ring-gray-400 w-64">
            </div>
        </div>

        <!-- Grid / Slot -->
        <div class="min-h-[500px]">
            <slot />
        </div>
      </div>
    </main>
  </div>
</template>

<script setup lang="ts">
import { usePrkbStore } from '@/stores/prkb';

defineProps<{
  activeTab: string
}>();

defineEmits(['update:activeTab', 'add-feed']);

const store = usePrkbStore();

const deleteFeed = async (id: string) => {
    if (confirm('Are you sure you want to remove this feed?')) {
        await store.deleteFeed(id);
    }
};
</script>


<style scoped>
/* Scoped overrides if needed, generally using Tailwind */
</style>
