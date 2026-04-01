<template>
  <div class="bg-white rounded-lg border border-gray-200 overflow-hidden">
    <div class="px-4 py-3 border-b border-gray-100 flex items-center justify-between">
      <h3 class="text-sm font-semibold text-gray-700 uppercase tracking-wider">Feed Control Center</h3>
      <div class="flex items-center gap-2">
        <span class="text-xs text-gray-400">{{ feeds.length }} feeds</span>
        <span v-if="healthyCount > 0" class="text-xs text-green-600">{{ healthyCount }} healthy</span>
        <span v-if="errorCount > 0" class="text-xs text-red-600">{{ errorCount }} errors</span>
      </div>
    </div>
    <div class="divide-y divide-gray-50">
      <div
        v-for="feed in feeds"
        :key="feed.id"
        class="px-4 py-3 hover:bg-gray-50 transition-colors"
      >
        <div class="flex items-center justify-between">
          <div class="flex items-center gap-3 flex-1 min-w-0">
            <!-- Health indicator -->
            <span
              class="w-2 h-2 rounded-full shrink-0"
              :class="{
                'bg-green-500': feed.health_status === 'healthy',
                'bg-yellow-500': feed.health_status === 'degraded',
                'bg-red-500': feed.health_status === 'error',
                'bg-gray-300': feed.health_status === 'unknown',
              }"
              :title="feed.health_status"
            ></span>
            <!-- Enable toggle -->
            <label class="relative inline-flex items-center cursor-pointer">
              <input
                type="checkbox"
                :checked="feed.enabled"
                @change="$emit('toggle-enabled', feed.id, !feed.enabled)"
                class="sr-only peer"
              />
              <div class="w-7 h-4 bg-gray-200 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:bg-blue-600 after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-3 after:w-3 after:transition-all"></div>
            </label>
            <!-- Feed info -->
            <div class="min-w-0 flex-1">
              <div class="flex items-center gap-2">
                <span class="text-sm font-medium text-gray-900 truncate">{{ feed.name }}</span>
                <span class="text-[10px] px-1.5 py-0.5 rounded bg-gray-100 text-gray-500 uppercase font-mono">{{ feed.feed_type }}</span>
              </div>
              <div class="text-xs text-gray-400 truncate">{{ feed.url }}</div>
            </div>
          </div>
          <!-- Stats -->
          <div class="flex items-center gap-4 text-xs text-gray-500 shrink-0 ml-4">
            <div class="text-right">
              <div class="font-mono">{{ feed.total_fetched }}</div>
              <div class="text-gray-400">fetched</div>
            </div>
            <div class="text-right">
              <div class="font-mono" :class="feed.parse_errors > 0 ? 'text-red-500' : ''">{{ feed.parse_errors }}</div>
              <div class="text-gray-400">errors</div>
            </div>
            <div class="text-right min-w-[90px]">
              <div v-if="feed.last_fetched_at" class="font-mono">{{ formatRelativeTime(feed.last_fetched_at) }}</div>
              <div v-else class="text-gray-400 italic">never</div>
              <div class="text-gray-400">last fetch</div>
            </div>
            <!-- Actions -->
            <div class="flex items-center gap-1">
              <button
                @click="$emit('manual-fetch', feed.id)"
                class="p-1.5 text-gray-400 hover:text-blue-600 hover:bg-blue-50 rounded transition-colors"
                title="Fetch now"
              >
                <i class="ri-refresh-line"></i>
              </button>
              <button
                @click="$emit('test-parser', feed.id)"
                class="p-1.5 text-gray-400 hover:text-green-600 hover:bg-green-50 rounded transition-colors"
                title="Test parser"
              >
                <i class="ri-bug-line"></i>
              </button>
              <button
                @click="$emit('delete-feed', feed.id)"
                class="p-1.5 text-gray-400 hover:text-red-600 hover:bg-red-50 rounded transition-colors"
                title="Delete feed"
              >
                <i class="ri-delete-bin-line"></i>
              </button>
            </div>
          </div>
        </div>
        <!-- Error message -->
        <div v-if="feed.last_error" class="mt-1 ml-12 text-xs text-red-500 truncate">
          <i class="ri-error-warning-line mr-1"></i>{{ feed.last_error }}
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import type { Feed } from '@/stores/prkb';

const props = defineProps<{ feeds: Feed[] }>();

defineEmits(['toggle-enabled', 'manual-fetch', 'test-parser', 'delete-feed']);

const healthyCount = computed(() => props.feeds.filter(f => f.health_status === 'healthy').length);
const errorCount = computed(() => props.feeds.filter(f => f.health_status === 'error').length);

const formatRelativeTime = (dateStr: string) => {
  const diff = Date.now() - new Date(dateStr).getTime();
  const mins = Math.floor(diff / 60000);
  if (mins < 1) return 'just now';
  if (mins < 60) return `${mins}m ago`;
  const hrs = Math.floor(mins / 60);
  if (hrs < 24) return `${hrs}h ago`;
  const days = Math.floor(hrs / 24);
  return `${days}d ago`;
};
</script>
