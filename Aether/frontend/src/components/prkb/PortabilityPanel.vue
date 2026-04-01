<template>
  <div class="border rounded-lg bg-white p-6 space-y-4">
    <h3 class="text-sm font-semibold text-gray-400 uppercase tracking-wider">Import / Export</h3>

    <!-- Export -->
    <div>
      <h4 class="text-sm font-medium text-gray-700 mb-2">Export Library</h4>
      <div class="flex gap-2">
        <button @click="$emit('export', 'bibtex')" class="px-3 py-1.5 text-xs border rounded text-gray-600 hover:bg-gray-50 flex items-center gap-1">
          <i class="ri-file-text-line"></i> BibTeX
        </button>
        <button @click="$emit('export', 'json')" class="px-3 py-1.5 text-xs border rounded text-gray-600 hover:bg-gray-50 flex items-center gap-1">
          <i class="ri-code-line"></i> JSON
        </button>
        <button @click="$emit('export', 'markdown')" class="px-3 py-1.5 text-xs border rounded text-gray-600 hover:bg-gray-50 flex items-center gap-1">
          <i class="ri-markdown-line"></i> Markdown Digest
        </button>
      </div>
    </div>

    <!-- Import -->
    <div>
      <h4 class="text-sm font-medium text-gray-700 mb-2">Import BibTeX</h4>
      <textarea
        v-model="bibtexInput"
        placeholder="Paste BibTeX entries here..."
        class="w-full border rounded p-3 text-xs font-mono min-h-[120px] focus:outline-none focus:ring-1 focus:ring-blue-400"
      ></textarea>
      <div class="flex items-center gap-4 mt-2">
        <label class="flex items-center gap-1 text-xs text-gray-600">
          <input type="checkbox" v-model="mergeTags" class="rounded border-gray-300" /> Merge tags
        </label>
        <label class="flex items-center gap-1 text-xs text-gray-600">
          <input type="checkbox" v-model="mergeNotes" class="rounded border-gray-300" /> Merge notes
        </label>
        <button
          @click="doImport"
          :disabled="!bibtexInput.trim()"
          class="px-4 py-1.5 text-xs bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50 ml-auto"
        >
          Import
        </button>
      </div>
    </div>

    <!-- Import result -->
    <div v-if="importResult" class="bg-gray-50 rounded p-3 text-xs space-y-1">
      <div class="font-medium text-gray-700">Import Result:</div>
      <div class="text-green-600">Imported: {{ importResult.imported }}</div>
      <div class="text-amber-600">Duplicates (merged): {{ importResult.duplicates }}</div>
      <div v-if="importResult.errors > 0" class="text-red-600">Errors: {{ importResult.errors }}</div>
      <details v-if="importResult.details.length > 0" class="mt-2">
        <summary class="cursor-pointer text-gray-500 hover:text-gray-700">Details</summary>
        <ul class="mt-1 space-y-0.5 text-gray-500">
          <li v-for="(d, i) in importResult.details" :key="i">{{ d }}</li>
        </ul>
      </details>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import type { ImportResult } from '@/stores/prkb';

const emit = defineEmits(['export', 'import']);

const bibtexInput = ref('');
const mergeTags = ref(true);
const mergeNotes = ref(true);
const importResult = ref<ImportResult | null>(null);

const doImport = () => {
  emit('import', bibtexInput.value, mergeTags.value, mergeNotes.value);
};

// Expose setter for parent to update result
defineExpose({
  setImportResult: (result: ImportResult | null) => {
    importResult.value = result;
  }
});
</script>
