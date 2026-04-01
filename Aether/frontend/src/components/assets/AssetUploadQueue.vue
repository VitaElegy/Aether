<template>
  <div
    class="relative"
    @dragenter.prevent="onDragEnter"
    @dragover.prevent="onDragOver"
    @dragleave.prevent="onDragLeave"
    @drop.prevent="onDrop"
  >
    <!-- Drag overlay -->
    <Transition name="fade">
      <div
        v-if="isDragging"
        class="absolute inset-0 z-50 flex items-center justify-center rounded-[2rem] border-2 border-dashed border-stone-400 bg-stone-100/90 backdrop-blur dark:border-stone-500 dark:bg-stone-900/90"
      >
        <div class="text-center">
          <p class="text-[11px] font-semibold uppercase tracking-[0.3em] text-stone-400">Drop Zone</p>
          <p class="mt-2 font-serif text-xl font-semibold text-stone-700 dark:text-stone-200">
            Drop files to upload
          </p>
        </div>
      </div>
    </Transition>

    <slot />

    <!-- Upload queue panel -->
    <Transition name="slide-up">
      <div
        v-if="queue.length > 0"
        class="fixed bottom-4 right-4 z-40 w-80 overflow-hidden rounded-2xl border border-stone-200 bg-white shadow-xl dark:border-stone-700 dark:bg-stone-900"
      >
        <div class="flex items-center justify-between border-b border-stone-200 px-4 py-3 dark:border-stone-700">
          <p class="text-xs font-semibold uppercase tracking-[0.2em] text-stone-400">Upload Queue</p>
          <span class="text-xs text-stone-500">{{ completedCount }}/{{ queue.length }}</span>
        </div>

        <div class="max-h-60 overflow-auto">
          <div
            v-for="item in queue"
            :key="item.id"
            class="flex items-center gap-3 border-b border-stone-100 px-4 py-2.5 last:border-0 dark:border-stone-800"
          >
            <div class="min-w-0 flex-1">
              <p class="truncate text-sm font-medium text-stone-700 dark:text-stone-200">{{ item.fileName }}</p>
              <p class="text-xs" :class="statusColor(item.status)">{{ statusLabel(item.status) }}</p>
            </div>
            <div v-if="item.status === 'duplicate'" class="text-[10px] font-semibold uppercase text-amber-600 dark:text-amber-400">
              DUP
            </div>
          </div>
        </div>

        <div v-if="allDone" class="border-t border-stone-200 px-4 py-2 dark:border-stone-700">
          <button
            class="w-full rounded-lg bg-stone-100 py-1.5 text-xs font-medium text-stone-600 transition hover:bg-stone-200 dark:bg-stone-800 dark:text-stone-300 dark:hover:bg-stone-700"
            @click="clearQueue"
          >
            Dismiss
          </button>
        </div>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue';
import { assetsApi } from '../../api/assets';

type UploadStatus = 'pending' | 'uploading' | 'done' | 'failed' | 'duplicate';

interface QueueItem {
  id: string;
  fileName: string;
  file: File;
  status: UploadStatus;
}

const emit = defineEmits<{
  uploaded: [];
}>();

const isDragging = ref(false);
const dragCounter = ref(0);
const queue = ref<QueueItem[]>([]);

const completedCount = computed(() => queue.value.filter((i) => i.status === 'done' || i.status === 'duplicate').length);
const allDone = computed(() => queue.value.every((i) => i.status !== 'pending' && i.status !== 'uploading'));

function onDragEnter() {
  dragCounter.value++;
  isDragging.value = true;
}

function onDragOver() {
  isDragging.value = true;
}

function onDragLeave() {
  dragCounter.value--;
  if (dragCounter.value <= 0) {
    isDragging.value = false;
    dragCounter.value = 0;
  }
}

async function onDrop(event: DragEvent) {
  isDragging.value = false;
  dragCounter.value = 0;

  const files = event.dataTransfer?.files;
  if (!files?.length) return;

  await enqueueFiles(Array.from(files));
}

async function enqueueFiles(files: File[]) {
  const newItems: QueueItem[] = files.map((file) => ({
    id: `${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
    fileName: file.name,
    file,
    status: 'pending' as UploadStatus,
  }));

  queue.value.push(...newItems);

  for (const item of newItems) {
    await processItem(item);
  }
}

async function processItem(item: QueueItem) {
  // Duplicate hash detection: compute SHA-256 then check server
  try {
    const hashHex = await computeFileHash(item.file);
    const existing = await assetsApi.list({ q: hashHex, limit: 1 });
    if (existing.items.length > 0) {
      item.status = 'duplicate';
      return;
    }
  } catch {
    // If hash check fails, proceed with upload anyway
  }

  item.status = 'uploading';
  try {
    await assetsApi.upload(item.file);
    item.status = 'done';
    emit('uploaded');
  } catch {
    item.status = 'failed';
  }
}

async function computeFileHash(file: File): Promise<string> {
  const buffer = await file.arrayBuffer();
  const hashBuffer = await crypto.subtle.digest('SHA-256', buffer);
  return Array.from(new Uint8Array(hashBuffer))
    .map((b) => b.toString(16).padStart(2, '0'))
    .join('');
}

function clearQueue() {
  queue.value = [];
}

function statusLabel(status: UploadStatus): string {
  switch (status) {
    case 'pending': return 'Waiting...';
    case 'uploading': return 'Uploading...';
    case 'done': return 'Uploaded';
    case 'failed': return 'Failed';
    case 'duplicate': return 'Duplicate (skipped)';
  }
}

function statusColor(status: UploadStatus): string {
  switch (status) {
    case 'done': return 'text-green-600 dark:text-green-400';
    case 'failed': return 'text-red-600 dark:text-red-400';
    case 'duplicate': return 'text-amber-600 dark:text-amber-400';
    default: return 'text-stone-400';
  }
}

// Public method for triggering upload from parent (file input)
function uploadFiles(files: File[]) {
  void enqueueFiles(files);
}

defineExpose({ uploadFiles });
</script>

<style scoped>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}
.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
.slide-up-enter-active,
.slide-up-leave-active {
  transition: all 0.3s ease;
}
.slide-up-enter-from,
.slide-up-leave-to {
  opacity: 0;
  transform: translateY(1rem);
}
</style>
