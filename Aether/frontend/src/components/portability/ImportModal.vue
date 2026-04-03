<template>
  <div v-if="isOpen" class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm">
    <div class="bg-white dark:bg-gray-800 rounded-xl shadow-2xl w-full max-w-lg overflow-hidden border border-gray-200 dark:border-gray-700">
      
      <!-- Header -->
      <div class="px-6 py-4 border-b border-gray-100 dark:border-gray-700 flex justify-between items-center">
        <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100">Import Knowledge Base</h3>
        <button @click="close" class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-200">
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path></svg>
        </button>
      </div>

      <!-- Content -->
      <div class="p-6">

        <!-- Step 1: Select File -->
        <div v-if="step === 'select'" class="space-y-4">
          <div
            class="relative border-2 border-dashed rounded-xl p-8 text-center transition-colors cursor-pointer"
            :class="isDragOver
              ? 'border-blue-400 bg-blue-50 dark:bg-blue-900/20'
              : 'border-gray-300 dark:border-gray-600 hover:border-gray-400 dark:hover:border-gray-500'"
            @dragover.prevent="isDragOver = true"
            @dragleave.prevent="isDragOver = false"
            @drop.prevent="handleDrop"
            @click="triggerFileInput"
          >
            <input
              ref="fileInputRef"
              type="file"
              accept=".zip,.akb"
              class="hidden"
              @change="handleFileSelect"
            />
            <div class="flex flex-col items-center gap-3">
              <div class="w-12 h-12 rounded-full bg-gray-100 dark:bg-gray-700 flex items-center justify-center">
                <svg class="w-6 h-6 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12"></path>
                </svg>
              </div>
              <div>
                <p class="text-sm font-medium text-gray-700 dark:text-gray-200">
                  Drop your archive here, or <span class="text-blue-600 dark:text-blue-400">browse</span>
                </p>
                <p class="text-xs text-gray-500 mt-1">
                  Supports Smart Portability ZIP (.zip) and legacy AKB (.akb) archives
                </p>
              </div>
            </div>
          </div>

          <!-- Selected file info -->
          <div v-if="selectedFile" class="flex items-center gap-3 p-3 bg-gray-50 dark:bg-gray-700/50 rounded-lg">
            <div class="w-8 h-8 rounded-lg bg-blue-100 dark:bg-blue-900/30 flex items-center justify-center flex-shrink-0">
              <svg class="w-4 h-4 text-blue-600 dark:text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"></path>
              </svg>
            </div>
            <div class="min-w-0 flex-1">
              <p class="text-sm font-medium text-gray-900 dark:text-gray-100 truncate">{{ selectedFile.name }}</p>
              <p class="text-xs text-gray-500">{{ formatFileSize(selectedFile.size) }}</p>
            </div>
            <button @click.stop="clearFile" class="text-gray-400 hover:text-red-500 flex-shrink-0">
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path></svg>
            </button>
          </div>
        </div>

        <!-- Step 2: Analyzing -->
        <div v-else-if="step === 'analyzing'" class="flex flex-col items-center py-8 text-gray-500">
          <svg class="animate-spin h-8 w-8 mb-4 text-blue-500" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
          </svg>
          <p>{{ statusMessage }}</p>
          <p v-if="isLongWait" class="text-xs text-orange-500 mt-2">Analysis is taking longer than expected...</p>
        </div>

        <!-- Step 3: Preview -->
        <div v-else-if="step === 'preview'" class="space-y-4">
          <!-- Summary -->
          <div class="bg-blue-50 dark:bg-blue-900/20 p-4 rounded-lg border border-blue-100 dark:border-blue-800">
            <div class="flex justify-between items-center mb-2">
              <span class="text-sm font-medium text-blue-800 dark:text-blue-300">Total Items</span>
              <span class="text-lg font-bold text-blue-900 dark:text-blue-100">{{ preview?.summary.total_items }}</span>
            </div>
          </div>

          <!-- Sections -->
          <div class="space-y-2">
            <h4 class="text-sm font-medium text-gray-700 dark:text-gray-300">Content to Import</h4>
            <div v-for="section in preview?.summary.sections" :key="section.name" class="flex justify-between items-center p-3 bg-gray-50 dark:bg-gray-700/50 rounded-md">
              <div class="flex items-center gap-2">
                <div class="font-medium text-gray-900 dark:text-gray-100">{{ section.name }}</div>
                <span
                  class="px-2 py-0.5 rounded text-[10px] font-semibold uppercase tracking-wider"
                  :class="actionBadgeClass(section.action)"
                >
                  {{ section.action }}
                </span>
              </div>
              <div class="font-mono text-sm font-semibold">{{ section.count }}</div>
            </div>
          </div>

          <!-- Conflicts -->
          <div v-if="preview && preview.conflicts.length > 0" class="space-y-2">
            <h4 class="text-sm font-medium text-amber-700 dark:text-amber-400 flex items-center gap-1">
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4.5c-.77-.833-2.694-.833-3.464 0L3.34 16.5c-.77.833.192 2.5 1.732 2.5z"></path></svg>
              Conflicts ({{ preview.conflicts.length }})
            </h4>
            <div class="max-h-40 overflow-y-auto space-y-1.5">
              <div v-for="conflict in preview.conflicts" :key="conflict.item_id" class="p-2.5 bg-amber-50 dark:bg-amber-900/10 border border-amber-200 dark:border-amber-800/40 rounded-md">
                <div class="flex items-start justify-between gap-2">
                  <p class="text-sm font-medium text-gray-900 dark:text-gray-100">{{ conflict.item_name }}</p>
                  <span class="text-[10px] font-semibold uppercase tracking-wider px-2 py-0.5 bg-amber-200/60 dark:bg-amber-800/30 text-amber-800 dark:text-amber-300 rounded flex-shrink-0">
                    {{ conflict.conflict_type }}
                  </span>
                </div>
                <div v-if="conflict.existing_value || conflict.incoming_value" class="mt-1 text-xs text-gray-500 dark:text-gray-400">
                  <span v-if="conflict.existing_value">Existing: {{ conflict.existing_value }}</span>
                  <span v-if="conflict.existing_value && conflict.incoming_value"> → </span>
                  <span v-if="conflict.incoming_value">Incoming: {{ conflict.incoming_value }}</span>
                </div>
              </div>
            </div>
          </div>

          <!-- Suggested Actions -->
          <div v-if="preview && preview.suggested_actions.length > 0" class="space-y-2">
            <h4 class="text-sm font-medium text-gray-700 dark:text-gray-300">Suggested Actions</h4>
            <div class="space-y-1">
              <div v-for="action in preview.suggested_actions" :key="action.item_id" class="flex items-center gap-2 text-xs text-gray-600 dark:text-gray-400">
                <span class="px-1.5 py-0.5 bg-gray-200 dark:bg-gray-700 rounded font-mono">{{ action.action }}</span>
                <span>{{ action.reason }}</span>
              </div>
            </div>
          </div>
        </div>

        <!-- Step 4: Importing -->
        <div v-else-if="step === 'importing'" class="space-y-6 py-4">
          <div class="space-y-2">
            <div class="flex justify-between text-sm">
              <span class="font-medium">{{ progress?.stage || 'Starting...' }}</span>
              <span>{{ progress?.percent ?? 0 }}%</span>
            </div>
            <div class="h-2 bg-gray-200 dark:bg-gray-700 rounded-full overflow-hidden">
              <div class="h-full bg-blue-500 transition-all duration-300 ease-out" :style="{ width: `${progress?.percent ?? 0}%` }"></div>
            </div>
            <p class="text-xs text-gray-500 text-center">{{ progress?.message || 'Preparing import...' }}</p>
          </div>
        </div>

        <!-- Step 5: Completed -->
        <div v-else-if="step === 'completed'" class="text-center py-6">
          <div class="w-16 h-16 bg-green-100 dark:bg-green-900/30 text-green-600 dark:text-green-400 rounded-full flex items-center justify-center mx-auto mb-4">
            <svg class="w-8 h-8" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7"></path></svg>
          </div>
          <h3 class="text-xl font-bold text-gray-900 dark:text-gray-100 mb-2">Import Complete</h3>
          <p class="text-gray-500 mb-3">Your assets have been imported successfully.</p>
          <div v-if="importStats" class="text-xs text-emerald-700 bg-emerald-50 dark:bg-emerald-900/20 rounded-md px-3 py-2 mb-4">
            {{ importStats }}
          </div>
        </div>

        <!-- Error -->
        <div v-else-if="step === 'error'" class="text-center py-6">
          <div class="w-16 h-16 bg-red-100 dark:bg-red-900/30 text-red-600 dark:text-red-400 rounded-full flex items-center justify-center mx-auto mb-4">
            <svg class="w-8 h-8" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path></svg>
          </div>
          <h3 class="text-xl font-bold text-gray-900 dark:text-gray-100 mb-2">Import Failed</h3>
          <div class="bg-red-50 dark:bg-red-900/10 p-3 rounded-md text-left mb-4">
            <p class="text-xs font-mono text-red-600 dark:text-red-300 break-words">{{ errorMessage }}</p>
          </div>
          <button @click="resetToSelect" class="px-4 py-2 bg-gray-200 dark:bg-gray-700 hover:bg-gray-300 dark:hover:bg-gray-600 rounded text-sm font-medium mr-2">
            Try Again
          </button>
          <button @click="close" class="text-gray-500 hover:text-gray-700 underline text-sm">Close</button>
        </div>

      </div>

      <!-- Footer Actions -->
      <div class="px-6 py-4 bg-gray-50 dark:bg-gray-700/30 border-t border-gray-100 dark:border-gray-700 flex justify-end gap-3">
        <template v-if="step === 'select'">
          <button @click="close" class="px-4 py-2 text-gray-600 hover:text-gray-800 dark:text-gray-300 dark:hover:text-white">
            Cancel
          </button>
          <button
            :disabled="!selectedFile"
            class="px-4 py-2 bg-black dark:bg-white text-white dark:text-black rounded-lg hover:opacity-90 font-medium disabled:opacity-40 disabled:cursor-not-allowed"
            @click="analyzeFile"
          >
            Analyze Archive
          </button>
        </template>

        <template v-else-if="step === 'preview'">
          <button @click="resetToSelect" class="px-4 py-2 text-gray-600 hover:text-gray-800 dark:text-gray-300 dark:hover:text-white">
            Back
          </button>
          <button
            class="px-4 py-2 bg-black dark:bg-white text-white dark:text-black rounded-lg hover:opacity-90 font-medium"
            @click="startImport"
          >
            Start Import
          </button>
        </template>

        <template v-else-if="step === 'completed'">
          <button
            class="px-4 py-2 bg-black dark:bg-white text-white dark:text-black rounded-lg hover:opacity-90 font-medium"
            @click="close"
          >
            Done
          </button>
        </template>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onUnmounted } from 'vue';
import { portabilityApi, type ImportPreview, type TaskProgress } from '../../api/portability';

const props = defineProps<{
  isOpen: boolean;
  kbId?: string;
}>();

const emit = defineEmits(['close']);

type ImportStep = 'select' | 'analyzing' | 'preview' | 'importing' | 'completed' | 'error';

const step = ref<ImportStep>('select');
const selectedFile = ref<File | null>(null);
const isDragOver = ref(false);
const fileInputRef = ref<HTMLInputElement | null>(null);
const preview = ref<ImportPreview | null>(null);
const progress = ref<TaskProgress | null>(null);
const errorMessage = ref('');
const statusMessage = ref('Analyzing archive...');
const isLongWait = ref(false);
const importStats = ref('');
let currentEventSource: EventSource | null = null;

watch(() => props.isOpen, (newVal) => {
  if (newVal) {
    resetAll();
  }
});

onUnmounted(() => {
  if (currentEventSource) {
    currentEventSource.close();
    currentEventSource = null;
  }
});

function resetAll() {
  step.value = 'select';
  selectedFile.value = null;
  isDragOver.value = false;
  preview.value = null;
  progress.value = null;
  errorMessage.value = '';
  statusMessage.value = 'Analyzing archive...';
  isLongWait.value = false;
  importStats.value = '';
  if (currentEventSource) {
    currentEventSource.close();
    currentEventSource = null;
  }
}

function resetToSelect() {
  step.value = 'select';
  preview.value = null;
  progress.value = null;
  errorMessage.value = '';
  statusMessage.value = 'Analyzing archive...';
  isLongWait.value = false;
  importStats.value = '';
}

function close() {
  emit('close');
}

function triggerFileInput() {
  fileInputRef.value?.click();
}

function handleFileSelect(event: Event) {
  const target = event.target as HTMLInputElement;
  if (target.files?.length) {
    selectedFile.value = target.files[0];
  }
}

function handleDrop(event: DragEvent) {
  isDragOver.value = false;
  const files = event.dataTransfer?.files;
  if (files?.length) {
    const file = files[0];
    if (file.name.endsWith('.zip') || file.name.endsWith('.akb')) {
      selectedFile.value = file;
    } else {
      errorMessage.value = 'Please drop a .zip or .akb file.';
      step.value = 'error';
    }
  }
}

function clearFile() {
  selectedFile.value = null;
  if (fileInputRef.value) {
    fileInputRef.value.value = '';
  }
}

async function analyzeFile() {
  if (!selectedFile.value || !props.kbId) return;

  step.value = 'analyzing';
  statusMessage.value = 'Analyzing archive...';
  isLongWait.value = false;

  const timer = setTimeout(() => {
    isLongWait.value = true;
    statusMessage.value = 'Still analyzing, this may take a moment for large archives...';
  }, 3000);

  try {
    preview.value = await portabilityApi.analyzeImport(props.kbId, selectedFile.value);
    step.value = 'preview';
  } catch (e: any) {
    step.value = 'error';
    if (e.response?.data) {
      if (typeof e.response.data === 'object' && e.response.data.error) {
        errorMessage.value = `${e.response.data.error} (Type: ${e.response.data.renderer_id || 'Unknown'})`;
      } else {
        errorMessage.value = String(e.response.data);
      }
    } else {
      errorMessage.value = e.message || 'Failed to analyze archive';
    }
  } finally {
    clearTimeout(timer);
  }
}

async function startImport() {
  if (!selectedFile.value || !props.kbId) return;

  step.value = 'importing';
  progress.value = null;

  try {
    const result = await portabilityApi.startImport(props.kbId, selectedFile.value);
    const taskId = result.task_id;

    // Connect SSE for progress
    currentEventSource = portabilityApi.connectProgress(
      taskId,
      (event: TaskProgress) => {
        progress.value = event;
        if (event.stage === 'Completed') {
          step.value = 'completed';
          importStats.value = event.message || 'All items imported successfully.';
          if (currentEventSource) {
            currentEventSource.close();
            currentEventSource = null;
          }
        } else if (event.error) {
          step.value = 'error';
          errorMessage.value = event.error;
          if (currentEventSource) {
            currentEventSource.close();
            currentEventSource = null;
          }
        }
      },
      (err: Event) => {
        if (step.value !== 'completed') {
          console.error('SSE Error during import', err);
          step.value = 'error';
          errorMessage.value = 'Lost connection to server during import. The import may still be running.';
        }
      }
    );
  } catch (e: any) {
    step.value = 'error';
    errorMessage.value = e.response?.data?.message || e.message || 'Failed to start import';
  }
}

function actionBadgeClass(action: string): string {
  const lower = action.toLowerCase();
  if (lower === 'create' || lower === 'add' || lower === 'new') {
    return 'bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-400';
  }
  if (lower === 'update' || lower === 'merge' || lower === 'overwrite') {
    return 'bg-amber-100 dark:bg-amber-900/30 text-amber-700 dark:text-amber-400';
  }
  if (lower === 'skip' || lower === 'ignore') {
    return 'bg-gray-100 dark:bg-gray-700 text-gray-500 dark:text-gray-400';
  }
  return 'bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-400';
}

function formatFileSize(bytes: number): string {
  if (bytes === 0) return '0 B';
  const units = ['B', 'KB', 'MB', 'GB'];
  const exponent = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1);
  const value = bytes / Math.pow(1024, exponent);
  return `${value.toFixed(value >= 10 || exponent === 0 ? 0 : 1)} ${units[exponent]}`;
}
</script>
