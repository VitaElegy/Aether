<template>
  <div class="space-y-4">
    <div class="flex items-center justify-between">
      <h3 class="text-sm font-semibold text-gray-400 uppercase tracking-wider">Collections</h3>
      <button @click="showCreate = true" class="text-xs text-blue-600 hover:text-blue-800">+ New</button>
    </div>

    <!-- Collection list -->
    <div v-if="collections.length === 0" class="text-sm text-gray-400 italic py-2">
      No collections yet.
    </div>
    <div v-for="col in collections" :key="col.id"
      class="flex items-center justify-between px-3 py-2 rounded-md hover:bg-gray-50 transition-colors cursor-pointer"
      :class="selectedId === col.id ? 'bg-blue-50 text-blue-700' : 'text-gray-600'"
      @click="$emit('select', col.id)"
    >
      <div class="flex items-center gap-2 min-w-0">
        <i :class="typeIcon(col.collection_type)" class="text-gray-400"></i>
        <span class="text-sm truncate">{{ col.name }}</span>
      </div>
      <div class="flex items-center gap-2 shrink-0">
        <span class="text-xs text-gray-400 bg-gray-100 px-1.5 py-0.5 rounded-full">{{ col.paper_count }}</span>
        <button @click.stop="$emit('delete', col.id)" class="p-1 text-gray-400 hover:text-red-600 opacity-0 group-hover:opacity-100">
          <i class="ri-delete-bin-line text-xs"></i>
        </button>
      </div>
    </div>

    <!-- Create dialog -->
    <div v-if="showCreate" class="border rounded-lg p-4 bg-white space-y-3">
      <input v-model="newName" placeholder="Collection name" class="w-full border rounded px-3 py-2 text-sm" />
      <select v-model="newType" class="w-full border rounded px-3 py-2 text-sm">
        <option value="watchlist">Watchlist</option>
        <option value="reading_queue">Reading Queue</option>
        <option value="archive">Archive</option>
        <option value="topic_collection">Topic Collection</option>
      </select>
      <textarea v-model="newDesc" placeholder="Description (optional)" class="w-full border rounded px-3 py-2 text-sm" rows="2"></textarea>
      <div class="flex gap-2">
        <button @click="create" class="px-3 py-1.5 text-xs bg-blue-600 text-white rounded hover:bg-blue-700">Create</button>
        <button @click="showCreate = false" class="px-3 py-1.5 text-xs border rounded text-gray-600 hover:bg-gray-50">Cancel</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import type { Collection } from '@/stores/prkb';

defineProps<{ collections: Collection[]; selectedId?: string }>();
const emit = defineEmits(['select', 'delete', 'create']);

const showCreate = ref(false);
const newName = ref('');
const newType = ref('topic_collection');
const newDesc = ref('');

const typeIcon = (type: string) => {
  switch (type) {
    case 'watchlist': return 'ri-eye-line';
    case 'reading_queue': return 'ri-book-read-line';
    case 'archive': return 'ri-archive-line';
    case 'topic_collection': return 'ri-folder-line';
    default: return 'ri-folder-line';
  }
};

const create = () => {
  if (!newName.value.trim()) return;
  emit('create', newName.value.trim(), newType.value, newDesc.value.trim() || undefined);
  newName.value = '';
  newDesc.value = '';
  showCreate.value = false;
};
</script>
