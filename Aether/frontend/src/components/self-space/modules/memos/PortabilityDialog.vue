<script setup lang="ts">
import { ref, computed } from 'vue';
import { useMemosStore } from '@/stores/memos';
import { Icon } from 'tdesign-vue-next';

const store = useMemosStore();

// Export state
const exportFormat = ref<'json' | 'markdown' | 'daily_archive'>('json');
const exportScope = ref<'all' | 'selected' | 'filtered'>('all');
const exporting = ref(false);
const exportResult = ref<string | null>(null);

// Import state
const importTab = ref<'export' | 'import'>('export');
const importFile = ref<File | null>(null);
const importOptions = ref({
    mergeTags: true,
    mergeChannels: true,
    detectDuplicates: true,
});
const importing = ref(false);
const importResult = ref<{ imported: number; skipped: number; total: number } | null>(null);

function close() {
    store.ui.showExportDialog = false;
    store.ui.showImportDialog = false;
    exportResult.value = null;
    importResult.value = null;
}

async function handleExport() {
    exporting.value = true;
    try {
        let ids: string[] | undefined;
        if (exportScope.value === 'selected') {
            ids = Array.from(store.ui.selectedIds);
        } else if (exportScope.value === 'filtered') {
            ids = store.filteredMemos.map(m => m.id);
        }

        const data = await store.exportMemos(exportFormat.value, ids);

        if (exportFormat.value === 'json') {
            const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' });
            downloadBlob(blob, `memos-export-${new Date().toISOString().slice(0, 10)}.json`);
        } else {
            // markdown / daily_archive — data is a string
            const blob = new Blob([typeof data === 'string' ? data : JSON.stringify(data)], { type: 'text/markdown' });
            downloadBlob(blob, `memos-export-${new Date().toISOString().slice(0, 10)}.md`);
        }
        exportResult.value = 'Export complete!';
    } catch (e) {
        exportResult.value = 'Export failed.';
    } finally {
        exporting.value = false;
    }
}

function downloadBlob(blob: Blob, filename: string) {
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = filename;
    a.click();
    URL.revokeObjectURL(url);
}

function handleFileSelect(e: Event) {
    const input = e.target as HTMLInputElement;
    if (input.files && input.files[0]) {
        importFile.value = input.files[0];
    }
}

async function handleImport() {
    if (!importFile.value) return;
    importing.value = true;
    try {
        const text = await importFile.value.text();
        const data = JSON.parse(text);

        // Support both flat array and bundle format
        const memos = Array.isArray(data) ? data : (data.memos || []);

        const result = await store.importMemos(memos, {
            mergeTags: importOptions.value.mergeTags,
            mergeChannels: importOptions.value.mergeChannels,
            detectDuplicates: importOptions.value.detectDuplicates,
        });
        importResult.value = result;
    } catch (e) {
        console.error('Import failed', e);
        importResult.value = { imported: 0, skipped: 0, total: 0 };
    } finally {
        importing.value = false;
    }
}
</script>

<template>
    <Transition name="fade">
        <div v-if="store.ui.showExportDialog || store.ui.showImportDialog" class="fixed inset-0 z-[90] flex items-center justify-center p-4">
            <div class="absolute inset-0 bg-black/30 backdrop-blur-sm" @click="close"></div>

            <div class="relative bg-white rounded-2xl shadow-2xl w-full max-w-md overflow-hidden" @click.stop>
                <!-- Tabs -->
                <div class="flex border-b border-gray-100">
                    <button
                        @click="importTab = 'export'"
                        class="flex-1 py-3 text-sm font-semibold transition-colors"
                        :class="importTab === 'export' ? 'text-blue-600 border-b-2 border-blue-600' : 'text-gray-400 hover:text-gray-600'"
                    >Export</button>
                    <button
                        @click="importTab = 'import'"
                        class="flex-1 py-3 text-sm font-semibold transition-colors"
                        :class="importTab === 'import' ? 'text-blue-600 border-b-2 border-blue-600' : 'text-gray-400 hover:text-gray-600'"
                    >Import</button>
                </div>

                <!-- Export Panel -->
                <div v-if="importTab === 'export'" class="p-6 space-y-4">
                    <div>
                        <label class="text-xs font-semibold text-gray-500 uppercase tracking-wider block mb-2">Format</label>
                        <div class="flex gap-2">
                            <button
                                v-for="fmt in [
                                    { value: 'json', label: 'JSON Bundle', icon: 'file' },
                                    { value: 'markdown', label: 'Markdown', icon: 'file-paste' },
                                    { value: 'daily_archive', label: 'Daily Archive', icon: 'calendar' },
                                ]"
                                :key="fmt.value"
                                @click="exportFormat = fmt.value as any"
                                class="flex-1 flex flex-col items-center gap-1 p-3 rounded-lg border transition-colors text-center"
                                :class="exportFormat === fmt.value ? 'border-blue-300 bg-blue-50 text-blue-700' : 'border-gray-200 text-gray-500 hover:bg-gray-50'"
                            >
                                <Icon :name="fmt.icon" size="20px" />
                                <span class="text-xs font-medium">{{ fmt.label }}</span>
                            </button>
                        </div>
                    </div>

                    <div>
                        <label class="text-xs font-semibold text-gray-500 uppercase tracking-wider block mb-2">Scope</label>
                        <select v-model="exportScope" class="w-full border border-gray-200 rounded-lg px-3 py-2 text-sm focus:ring-1 focus:ring-blue-300 focus:outline-none">
                            <option value="all">All Memos</option>
                            <option value="filtered">Current Filter ({{ store.filteredMemos.length }})</option>
                            <option value="selected" :disabled="store.ui.selectedIds.size === 0">Selected ({{ store.ui.selectedIds.size }})</option>
                        </select>
                    </div>

                    <div v-if="exportResult" class="text-sm text-green-600 bg-green-50 px-3 py-2 rounded-lg">
                        {{ exportResult }}
                    </div>

                    <div class="flex justify-end gap-2">
                        <button @click="close" class="px-4 py-2 text-sm text-gray-500 hover:text-gray-700">Cancel</button>
                        <button
                            @click="handleExport"
                            :disabled="exporting"
                            class="px-4 py-2 bg-blue-600 text-white text-sm font-medium rounded-lg hover:bg-blue-700 disabled:opacity-50 flex items-center gap-2"
                        >
                            <div v-if="exporting" class="w-3 h-3 border-2 border-white border-t-transparent rounded-full animate-spin"></div>
                            Export
                        </button>
                    </div>
                </div>

                <!-- Import Panel -->
                <div v-if="importTab === 'import'" class="p-6 space-y-4">
                    <div>
                        <label class="text-xs font-semibold text-gray-500 uppercase tracking-wider block mb-2">JSON File</label>
                        <div class="border-2 border-dashed border-gray-200 rounded-lg p-6 text-center hover:border-blue-300 transition-colors cursor-pointer relative">
                            <input type="file" accept=".json" @change="handleFileSelect" class="absolute inset-0 opacity-0 cursor-pointer" />
                            <Icon name="upload" size="24px" class="mx-auto text-gray-400 mb-2" />
                            <div class="text-sm text-gray-500">
                                {{ importFile ? importFile.name : 'Drop or click to select JSON file' }}
                            </div>
                        </div>
                    </div>

                    <div class="space-y-2">
                        <label class="text-xs font-semibold text-gray-500 uppercase tracking-wider block">Options</label>
                        <label class="flex items-center gap-2 text-sm text-gray-600">
                            <input type="checkbox" v-model="importOptions.detectDuplicates" class="rounded border-gray-300" />
                            Detect duplicates (skip existing titles)
                        </label>
                        <label class="flex items-center gap-2 text-sm text-gray-600">
                            <input type="checkbox" v-model="importOptions.mergeTags" class="rounded border-gray-300" />
                            Merge tags with existing
                        </label>
                        <label class="flex items-center gap-2 text-sm text-gray-600">
                            <input type="checkbox" v-model="importOptions.mergeChannels" class="rounded border-gray-300" />
                            Merge channels with existing
                        </label>
                    </div>

                    <div v-if="importResult" class="text-sm bg-green-50 px-3 py-2 rounded-lg space-y-1">
                        <div class="text-green-600 font-medium">Import complete</div>
                        <div class="text-green-700">{{ importResult.imported }} imported, {{ importResult.skipped }} skipped ({{ importResult.total }} total)</div>
                    </div>

                    <div class="flex justify-end gap-2">
                        <button @click="close" class="px-4 py-2 text-sm text-gray-500 hover:text-gray-700">Cancel</button>
                        <button
                            @click="handleImport"
                            :disabled="!importFile || importing"
                            class="px-4 py-2 bg-blue-600 text-white text-sm font-medium rounded-lg hover:bg-blue-700 disabled:opacity-50 flex items-center gap-2"
                        >
                            <div v-if="importing" class="w-3 h-3 border-2 border-white border-t-transparent rounded-full animate-spin"></div>
                            Import
                        </button>
                    </div>
                </div>
            </div>
        </div>
    </Transition>
</template>

<style scoped>
.fade-enter-active, .fade-leave-active { transition: opacity 0.2s ease; }
.fade-enter-from, .fade-leave-to { opacity: 0; }
</style>
