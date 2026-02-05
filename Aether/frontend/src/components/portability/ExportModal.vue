<template>
  <div v-if="isOpen" class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm">
    <div class="bg-white dark:bg-gray-800 rounded-xl shadow-2xl w-full max-w-lg overflow-hidden border border-gray-200 dark:border-gray-700">
      
      <!-- Header -->
      <div class="px-6 py-4 border-b border-gray-100 dark:border-gray-700 flex justify-between items-center">
        <h3 class="text-lg font-semibold text-gray-900 dark:text-gray-100">Export Knowledge Base</h3>
        <button @click="close" class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-200">
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path></svg>
        </button>
      </div>

      <!-- Content -->
      <div class="p-6">
        
        <!-- Step 1: Analyzing -->
        <div v-if="step === 'analyzing'" class="flex flex-col items-center py-8 text-gray-500">
          <svg class="animate-spin h-8 w-8 mb-4 text-blue-500" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
          </svg>
          <p>{{ statusMessage }}</p>
          <p v-if="isLongWait" class="text-xs text-orange-500 mt-2">Analysis is taking longer than expected...</p>
        </div>

        <!-- Step 2: Preview -->
        <div v-else-if="step === 'preview'" class="space-y-4">
          <div class="bg-blue-50 dark:bg-blue-900/20 p-4 rounded-lg border border-blue-100 dark:border-blue-800">
            <div class="flex justify-between items-center mb-2">
              <span class="text-sm font-medium text-blue-800 dark:text-blue-300">Total Items</span>
              <span class="text-lg font-bold text-blue-900 dark:text-blue-100">{{ summary?.total_items }}</span>
            </div>
            <div class="text-xs text-blue-600 dark:text-blue-400">Estimated Size: {{ summary?.estimated_size }}</div>
          </div>

          <div class="space-y-2">
            <h4 class="text-sm font-medium text-gray-700 dark:text-gray-300">Included Content</h4>
            <div v-for="section in summary?.sections" :key="section.name" class="flex justify-between items-center p-3 bg-gray-50 dark:bg-gray-700/50 rounded-md">
              <div>
                <div class="font-medium text-gray-900 dark:text-gray-100">{{ section.name }}</div>
                <div class="text-xs text-gray-500">{{ section.details }}</div>
              </div>
              <div class="font-mono text-sm font-semibold">{{ section.count }}</div>
            </div>
          </div>
        </div>

        <!-- Step 3: Exporting -->
        <div v-else-if="step === 'exporting'" class="space-y-6 py-4">
          <div class="space-y-2">
            <div class="flex justify-between text-sm">
              <span class="font-medium">{{ progress?.stage }}</span>
              <span>{{ progress?.percent }}%</span>
            </div>
            <div class="h-2 bg-gray-200 dark:bg-gray-700 rounded-full overflow-hidden">
              <div class="h-full bg-blue-500 transition-all duration-300 ease-out" :style="{ width: `${progress?.percent}%` }"></div>
            </div>
            <p class="text-xs text-gray-500 text-center">{{ progress?.message }}</p>
          </div>
        </div>

        <!-- Step 4: Completed -->
        <div v-else-if="step === 'completed'" class="text-center py-6">
          <div class="w-16 h-16 bg-green-100 dark:bg-green-900/30 text-green-600 dark:text-green-400 rounded-full flex items-center justify-center mx-auto mb-4">
            <svg class="w-8 h-8" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7"></path></svg>
          </div>
          <h3 class="text-xl font-bold text-gray-900 dark:text-gray-100 mb-2">Export Ready</h3>
          <p class="text-gray-500 mb-6">Your portability package has been generated successfully.</p>
          
          <a :href="downloadUrl" download class="inline-flex items-center justify-center w-full px-4 py-3 bg-blue-600 hover:bg-blue-700 text-white rounded-lg font-medium transition-colors">
            <svg class="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"></path></svg>
            Download Package
          </a>
        </div>

        <!-- Error -->
        <div v-else-if="step === 'error'" class="text-center py-6">
          <div class="w-16 h-16 bg-red-100 dark:bg-red-900/30 text-red-600 dark:text-red-400 rounded-full flex items-center justify-center mx-auto mb-4">
            <svg class="w-8 h-8" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path></svg>
          </div>
          <h3 class="text-xl font-bold text-gray-900 dark:text-gray-100 mb-2">Export Failed</h3>
          <div class="bg-red-50 dark:bg-red-900/10 p-3 rounded-md text-left mb-4">
             <p class="text-xs font-mono text-red-600 dark:text-red-300 break-words">{{ errorMessage }}</p>
          </div>
          <button @click="reset(); analyze()" class="px-4 py-2 bg-gray-200 dark:bg-gray-700 hover:bg-gray-300 dark:hover:bg-gray-600 rounded text-sm font-medium mr-2">Retry</button>
          <button @click="close" class="text-gray-500 hover:text-gray-700 underline text-sm">Close</button>
        </div>

      </div>

      <!-- Footer Actions -->
      <div v-if="step === 'preview'" class="px-6 py-4 bg-gray-50 dark:bg-gray-700/30 border-t border-gray-100 dark:border-gray-700 flex justify-end gap-3">
        <button @click="close" class="px-4 py-2 text-gray-600 hover:text-gray-800 dark:text-gray-300 dark:hover:text-white">Cancel</button>
        <button @click="startExport" class="px-4 py-2 bg-black dark:bg-white text-white dark:text-black rounded-lg hover:opacity-90 font-medium">
          Start Export
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted } from 'vue';
import { portabilityApi, type ExportSummary, type ProgressEvent } from '../../api/portability';

const props = defineProps<{
  isOpen: boolean;
  kbId: string;
}>();

const emit = defineEmits(['close']);

const step = ref<'analyzing' | 'preview' | 'exporting' | 'completed' | 'error'>('analyzing');
const summary = ref<ExportSummary | null>(null);
const progress = ref<ProgressEvent | null>(null);
const errorMessage = ref('');
const downloadUrl = ref('');
const currentTaskId = ref('');
const statusMessage = ref('Analyzing Knowledge Base...');
const isLongWait = ref(false);

watch(() => props.isOpen, async (newVal) => {
  if (newVal) {
    reset();
    await analyze();
  }
});

onMounted(() => {
  if (props.isOpen) {
    console.log('[ExportModal] Mounted open, starting analysis...');
    reset();
    analyze();
  }
});

const reset = () => {
  step.value = 'analyzing';
  summary.value = null;
  progress.value = null;
  errorMessage.value = '';
  downloadUrl.value = '';
  currentTaskId.value = '';
  statusMessage.value = 'Analyzing Knowledge Base...';
  isLongWait.value = false;
};

const close = () => {
  emit('close');
};

const analyze = async () => {
  console.log('[ExportModal] Analyzing KB:', props.kbId);
  // Long Wait Timer
  const timer = setTimeout(() => {
    isLongWait.value = true;
    statusMessage.value = 'Connecting to server...';
  }, 2000);

  try {
    summary.value = await portabilityApi.analyzeExport(props.kbId);
    step.value = 'preview';
  } catch (e: any) {
    step.value = 'error';
    
    // Detailed Error Parsing
    if (e.response && e.response.data) {
       // Check if JSON error
       if (typeof e.response.data === 'object' && e.response.data.error) {
           errorMessage.value = `${e.response.data.error} (Type: ${e.response.data.renderer_id || 'Unknown'})`;
       } else {
           // Text error
           errorMessage.value = String(e.response.data);
       }
    } else {
        errorMessage.value = e.message || 'Network Error';
    }
  } finally {
    clearTimeout(timer);
  }
};

const startExport = async () => {
  try {
    step.value = 'exporting';
    const taskId = await portabilityApi.startExport(props.kbId);
    currentTaskId.value = taskId;
    
    // Connect SSE
    portabilityApi.connectProgress(
      taskId,
      (event) => {
        progress.value = event;
        if (event.stage === 'Completed') {
          step.value = 'completed';
          downloadUrl.value = portabilityApi.getDownloadUrl(taskId);
        } else if (event.error) {
          step.value = 'error';
          errorMessage.value = event.error;
        }
      },
      (err) => {
        // Only error if not completed
        if (step.value !== 'completed') {
           console.error("SSE Error", err);
           // Don't fail immediately on SSE glitch, usually it reconnects or just stops updates
           // But if connection refused, we might want to show error
        }
      }
    );

  } catch (e: any) {
    step.value = 'error';
    errorMessage.value = e.response?.data?.message || e.message || 'Failed to start export';
  }
};
</script>
