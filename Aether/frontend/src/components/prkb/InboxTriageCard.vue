<template>
  <div class="group relative bg-white border rounded-sm hover:border-blue-400 hover:shadow-md transition-all duration-200 flex flex-col"
    :class="[
      item.priority && item.priority >= 4 ? 'border-amber-300 bg-amber-50/30' : 'border-gray-200',
      item.state === 'read' ? 'opacity-75' : ''
    ]"
  >
    <!-- Priority indicator -->
    <div v-if="item.priority" class="absolute top-0 right-0 px-1.5 py-0.5 text-[10px] font-bold rounded-bl"
      :class="{
        'bg-red-100 text-red-700': item.priority >= 4,
        'bg-amber-100 text-amber-700': item.priority === 3,
        'bg-gray-100 text-gray-600': item.priority <= 2,
      }"
    >
      P{{ item.priority }}
    </div>

    <!-- Main content -->
    <a :href="item.pdf_url || item.url" target="_blank" class="block p-4 flex-1" @click.stop="$emit('mark-read', item.id)">
      <div class="mb-2">
        <h3 class="text-sm font-bold text-gray-900 leading-snug group-hover:text-blue-700 mb-1 pr-8">
          {{ item.title }}
        </h3>
        <div class="flex items-center flex-wrap gap-1.5 text-xs text-gray-500">
          <span class="px-1.5 py-0.5 rounded-sm text-[10px] font-bold uppercase"
            :class="stateClasses">
            {{ item.state }}
          </span>
          <span v-if="item.publication" class="px-1 py-0.5 bg-gray-100 text-gray-600 rounded text-[10px]">
            {{ item.publication }}
          </span>
          <span class="text-gray-400">{{ displayDate }}</span>
        </div>
      </div>
      <div class="text-xs text-gray-600 truncate mb-2">{{ displayAuthors }}</div>
      <div class="text-xs text-gray-500 line-clamp-2 mb-2">{{ item.abstract_text }}</div>
      <!-- Note -->
      <div v-if="item.note" class="text-xs text-blue-600 bg-blue-50 px-2 py-1 rounded mt-1">
        <i class="ri-sticky-note-line mr-1"></i>{{ item.note }}
      </div>
    </a>

    <!-- Actions footer -->
    <div class="px-3 py-2 border-t border-gray-50 bg-gray-50/50 flex justify-between items-center mt-auto">
      <div class="flex items-center gap-1">
        <!-- Priority buttons -->
        <button
          v-for="p in [1,2,3,4,5]" :key="p"
          @click.stop="$emit('set-priority', item.id, p)"
          class="w-5 h-5 text-[10px] rounded transition-colors"
          :class="item.priority === p ? 'bg-amber-200 text-amber-800 font-bold' : 'text-gray-400 hover:bg-gray-200'"
          :title="'Priority ' + p"
        >
          {{ p }}
        </button>
      </div>
      <div class="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
        <button
          @click.stop="$emit('add-note', item.id)"
          class="p-1.5 text-gray-400 hover:text-blue-600 hover:bg-blue-50 rounded transition-colors"
          title="Add note"
        >
          <i class="ri-sticky-note-line text-lg"></i>
        </button>
        <button
          @click.stop="$emit('save', item)"
          class="p-1.5 text-gray-400 hover:text-green-600 hover:bg-green-50 rounded transition-colors"
          title="Save to Library"
        >
          <i class="ri-check-line text-lg"></i>
        </button>
        <button
          @click.stop="$emit('skip', item.id)"
          class="p-1.5 text-gray-400 hover:text-yellow-600 hover:bg-yellow-50 rounded transition-colors"
          title="Skip"
        >
          <i class="ri-skip-forward-line text-lg"></i>
        </button>
        <button
          @click.stop="$emit('trash', item)"
          class="p-1.5 text-gray-400 hover:text-red-600 hover:bg-red-50 rounded transition-colors"
          title="Trash"
        >
          <i class="ri-delete-bin-line text-lg"></i>
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import type { InboxItem } from '@/stores/prkb';

const props = defineProps<{ item: InboxItem }>();
defineEmits(['save', 'skip', 'trash', 'mark-read', 'set-priority', 'add-note']);

const stateClasses = computed(() => {
  switch (props.item.state) {
    case 'new': return 'bg-blue-100 text-blue-700';
    case 'read': return 'bg-gray-100 text-gray-600';
    case 'saved': return 'bg-green-100 text-green-700';
    case 'skipped': return 'bg-yellow-100 text-yellow-700';
    default: return 'bg-gray-100 text-gray-600';
  }
});

const displayAuthors = computed(() => {
  const list = props.item.authors || [];
  return list.length > 0 ? list.join(', ') : 'Unknown Author';
});

const displayDate = computed(() => {
  if (!props.item.publish_date) return '';
  return new Date(props.item.publish_date).toLocaleDateString();
});
</script>

<style scoped>
.line-clamp-2 {
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}
</style>
