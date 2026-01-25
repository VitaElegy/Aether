<template>
    <div class="h-full flex overflow-hidden bg-paper-2 rounded-xl border border-ash/20 shadow-sm">
        <!-- Sidebar: Doc List -->
        <div class="w-72 bg-white/60 backdrop-blur-md border-r border-ash/20 flex flex-col">
            <!-- Header -->
            <div class="p-4 border-b border-ash/10">
                <div class="flex items-center justify-between mb-4">
                    <span class="text-xs font-bold uppercase tracking-widest text-ink/40">Knowledge Base</span>
                    <div class="flex items-center gap-1">
                        <button @click="toggleTrash" class="w-8 h-8 rounded-lg hover:bg-ash/50 text-ink/40 hover:text-red-500 flex items-center justify-center transition-all shadow-sm" :class="{'bg-red-50 text-red-500': isTrashMode}" title="Recycle Bin">
                            <i class="ri-delete-bin-2-line text-lg"></i>
                        </button>
                        <button v-if="!isTrashMode" @click="createNewDoc" class="w-8 h-8 rounded-lg bg-accent/10 hover:bg-accent hover:text-white text-accent flex items-center justify-center transition-all shadow-sm" title="New Page">
                            <i class="ri-add-line text-lg"></i>
                        </button>
                    </div>
                </div>
                <div class="relative group">
                    <i class="ri-search-line absolute left-3 top-1/2 -translate-y-1/2 text-ink/30 group-focus-within:text-accent transition-colors"></i>
                    <input 
                        v-model="searchQuery" 
                        type="text" 
                        class="w-full pl-9 pr-4 py-2 bg-ash/30 rounded-lg text-sm font-medium text-ink focus:outline-none focus:ring-2 focus:ring-accent/20 transition-all placeholder-ink/30"
                        placeholder="Filter pages..."
                    >
                </div>
            </div>
            
            <!-- Breadcrumbs (Hidden in Trash Mode) -->
            <div v-if="!isTrashMode" class="px-4 py-2 border-b border-ash/10 text-xs font-mono text-ink/60 flex items-center gap-2 overflow-x-auto whitespace-nowrap">
                <div 
                    v-for="(crumb, idx) in breadcrumbs" 
                    :key="idx"
                    class="flex items-center gap-1"
                    @dragover.prevent="onDragOver($event, crumb.id)"
                    @dragleave="onDragLeave"
                    @drop="onDrop($event, crumb.id)"
                >
                    <span 
                        @click="navigateTo(crumb.id)" 
                        class="hover:text-accent cursor-pointer hover:underline transition-colors px-1 rounded"
                        :class="[
                            idx === breadcrumbs.length - 1 ? 'font-bold text-ink' : '',
                            dragOverId === crumb.id ? 'bg-accent/20 text-accent ring-2 ring-accent/50' : ''
                        ]"
                    >
                        {{ crumb.title }}
                    </span>
                    <span v-if="idx < breadcrumbs.length - 1" class="text-ash/60">/</span>
                </div>
            </div>
            
            <!-- Trash Header -->
            <div v-if="isTrashMode" class="px-4 py-2 border-b border-red-100 bg-red-50/50 text-xs font-bold text-red-500 flex items-center gap-2">
                 <i class="ri-delete-bin-line"></i> Recycle Bin
            </div>

            <!-- List -->
            <div 
                class="flex-1 overflow-y-auto p-3 space-y-1 relative"
            >
                <div v-if="filteredDocs.length === 0" class="text-center py-8 text-ink/30 text-xs">
                    {{ isTrashMode ? 'Trash is empty.' : 'Empty folder.' }}
                </div>
                
                <div 
                    v-for="doc in filteredDocs" 
                    :key="doc.id" 
                    @click="handleDocClick(doc)"
                    
                    :draggable="!isTrashMode"
                    @dragstart="!isTrashMode ? onDragStart($event, doc) : null"
                    @dragend="!isTrashMode ? onDragEnd() : null"
                    @dragover.prevent="(!isTrashMode && doc.content?.is_folder) ? onDragOver($event, doc.id) : null"
                    @dragleave="(!isTrashMode && doc.content?.is_folder) ? onDragLeave($event) : null"
                    @drop="(!isTrashMode && doc.content?.is_folder) ? onDrop($event, doc.id) : null"

                    class="p-3 rounded-lg cursor-pointer flex items-center gap-3 group transition-all duration-200 relative overflow-hidden ring-transparent border border-transparent"
                    :class="[
                        selectedDoc?.id === doc.id ? 'bg-white shadow-md border-ash/20' : 'hover:bg-white/50 hover:shadow-sm',
                        doc.content?.is_folder ? 'bg-yellow-50/10' : '',
                        dragOverId === doc.id ? '!bg-accent/10 !border-accent !ring-1 !ring-accent shadow-lg scale-[1.02] z-10' : '',
                        draggingId === doc.id ? 'opacity-40 grayscale' : ''
                    ]"
                >
                    <div class="w-8 h-8 rounded-md flex items-center justify-center transition-colors" 
                        :class="[
                            doc.content?.is_folder ? 'bg-yellow-100 text-yellow-600' : (selectedDoc?.id === doc.id ? 'bg-accent/10 text-accent' : 'bg-ash/50 text-ink/40'),
                            isTrashMode ? 'opacity-50' : ''
                        ]"
                    >
                        <i :class="doc.content?.is_folder ? 'ri-folder-3-fill text-lg' : 'ri-file-text-line text-lg'"></i>
                    </div>
                    <div class="flex-1 min-w-0 pointer-events-none">
                        <h4 class="text-sm font-bold text-ink truncate" :class="[selectedDoc?.id === doc.id ? 'text-ink' : 'text-ink/70', isTrashMode ? 'line-through opacity-60' : '']">{{ doc.title }}</h4>
                        <!-- Only show details for files -->
                        <p v-if="isTrashMode" class="text-[10px] text-red-400 truncate">
                            Deleted {{ formatDate(doc.deleted_at) }}
                        </p>
                        <p v-else-if="!doc.content?.is_folder" class="text-[10px] text-ink/40 truncate">
                            Updated {{ formatDate(doc.updated_at) }}
                        </p>
                    </div>
                    
                    <!-- Trash Actions -->
                    <div v-if="isTrashMode" class="flex items-center gap-1 opacity-100 sm:opacity-0 group-hover:opacity-100 transition-opacity absolute right-2">
                        <button @click.stop="restoreDoc(doc)" class="p-1.5 bg-white/80 hover:bg-green-50 text-green-600 rounded-md shadow-sm" title="Restore">
                            <i class="ri-restart-line"></i>
                        </button>
                        <button @click.stop="permanentDeleteDoc(doc)" class="p-1.5 bg-white/80 hover:bg-red-50 text-red-500 rounded-md shadow-sm" title="Delete Forever">
                            <i class="ri-close-circle-line"></i>
                        </button>
                    </div>
                    
                    <div v-if="selectedDoc?.id === doc.id && !doc.content?.is_folder" class="absolute right-0 top-0 bottom-0 w-1 bg-accent"></div>
                </div>
            </div>

            <!-- Sidebar Footer: Recycle Bin -->
            <div class="p-3 border-t border-ash/10 bg-ash/5 text-ink/60">
                <button 
                    @click="toggleTrash" 
                    class="w-full flex items-center gap-3 px-3 py-2 rounded-lg transition-all"
                    :class="isTrashMode ? 'bg-red-50 text-red-500 shadow-sm ring-1 ring-red-100' : 'hover:bg-white hover:shadow-sm hover:text-ink'"
                >
                    <i :class="isTrashMode ? 'ri-delete-bin-2-fill' : 'ri-delete-bin-line'" class="text-lg"></i>
                    <span class="text-sm font-bold">Recycle Bin</span>
                </button>
            </div>
        </div>

        <!-- Main Area: Editor -->
        <div class="flex-1 flex flex-col bg-white/80 relative">
             <!-- Background Pattern -->
            <div class="absolute inset-0 opacity-[0.03] pointer-events-none" style="background-image: radial-gradient(#000 1px, transparent 1px); background-size: 20px 20px;"></div>

            <div v-if="selectedDoc && !isTrashMode" class="flex-1 flex flex-col relative z-10">
                <!-- Metadata Header -->
                <div class="h-14 border-b border-ash/10 flex items-center px-6 justify-between bg-white/50 backdrop-blur-sm">
                    <div class="flex items-center gap-4 text-xs font-bold uppercase tracking-wider text-ink/40">
                        <span class="flex items-center gap-1.5 px-2 py-1 rounded bg-green-50 text-green-600 border border-green-100">
                            <i class="ri-checkbox-circle-fill"></i> Saved
                        </span>
                        <span class="flex items-center gap-1.5">
                            <i class="ri-time-line"></i> {{ formatDate(selectedDoc.updated_at) }}
                        </span>
                    </div>
                    <div class="flex items-center gap-2">
                        <button class="flex items-center gap-2 px-3 py-1.5 hover:bg-ash/50 rounded-lg text-ink/60 text-xs font-bold transition-colors">
                            <i class="ri-share-line"></i> Share
                        </button>
                        <div class="w-px h-4 bg-ash/50 mx-1"></div>
                        <button @click="deleteDoc(selectedDoc.id)" class="p-1.5 hover:bg-red-50 text-ink/40 hover:text-red-500 rounded-lg transition-colors" title="Move to Trash">
                            <i class="ri-delete-bin-line text-lg"></i>
                        </button>
                    </div>
                </div>
                
                <!-- ... Editor Toolbar ... -->
                <div class="px-6 py-2 border-b border-ash/10 flex items-center gap-2 bg-white/30 backdrop-blur-sm sticky top-0 z-20">
                    <button class="w-8 h-8 rounded hover:bg-ash/50 flex items-center justify-center text-ink/60 transition-colors" title="Bold"><i class="ri-bold"></i></button>
                    <button class="w-8 h-8 rounded hover:bg-ash/50 flex items-center justify-center text-ink/60 transition-colors" title="Italic"><i class="ri-italic"></i></button>
                    <button class="w-8 h-8 rounded hover:bg-ash/50 flex items-center justify-center text-ink/60 transition-colors" title="Underline"><i class="ri-underline"></i></button>
                    <div class="w-px h-4 bg-ash/50 mx-2"></div>
                    <button class="w-8 h-8 rounded hover:bg-ash/50 flex items-center justify-center text-ink/60 transition-colors" title="Code"><i class="ri-code-line"></i></button>
                    <button class="w-8 h-8 rounded hover:bg-ash/50 flex items-center justify-center text-ink/60 transition-colors" title="Link"><i class="ri-link"></i></button>
                    <button class="w-8 h-8 rounded hover:bg-ash/50 flex items-center justify-center text-ink/60 transition-colors" title="Image"><i class="ri-image-line"></i></button>
                    <div class="flex-1"></div>
                    <span class="text-[10px] font-bold text-ink/20 uppercase tracking-widest">Markdown Supported</span>
                </div>

                <!-- Content -->
                <div class="flex-1 overflow-y-auto">
                    <div class="max-w-4xl mx-auto w-full py-12 px-12">
                        <textarea 
                            ref="titleTextarea"
                            v-model="selectedDoc.title" 
                            class="text-4xl font-black font-serif text-ink w-full bg-transparent focus:outline-none mb-8 placeholder-ink/20 resize-none overflow-hidden h-auto" 
                            placeholder="Untitled Page"
                            rows="1"
                            @input="autoResize($event)"
                        ></textarea>
                        
                        <textarea 
                            v-model="selectedDoc.content"
                            class="w-full h-[calc(100vh-300px)] resize-none focus:outline-none font-serif text-lg leading-relaxed text-ink/80 placeholder-ink/20 bg-transparent selection:bg-accent/20" 
                            placeholder="Start writing your documentation..."
                        ></textarea>
                    </div>
                </div>
            </div>
            
            <!-- Trash Selected State -->
             <div v-else-if="selectedDoc && isTrashMode" class="flex-1 flex flex-col items-center justify-center text-ink/40 bg-ash/5 relative z-10">
                 <i class="ri-delete-bin-2-line text-6xl text-red-200 mb-4"></i>
                 <h3 class="text-xl font-bold text-ink">This item is in the Recycle Bin</h3>
                 <p class="text-sm opacity-60 mt-2 max-w-sm text-center">Restore this item to view or edit its contents. Items in the Recycle Bin are automatically deleted after 30 days.</p>
                 <div class="flex gap-4 mt-8">
                      <button @click="restoreDoc(selectedDoc)" class="px-6 py-2 bg-white border border-ash/20 shadow-sm rounded-lg text-sm font-bold text-green-600 hover:text-green-700 hover:bg-green-50 transition-colors">
                        Restore Item
                    </button>
                    <button @click="permanentDeleteDoc(selectedDoc)" class="px-6 py-2 bg-red-500 border border-red-600 shadow-sm rounded-lg text-sm font-bold text-white hover:bg-red-600 transition-colors">
                        Delete Forever
                    </button>
                 </div>
             </div>

            <!-- Empty State -->
            <div v-else class="flex-1 flex flex-col items-center justify-center text-ink/20 bg-ash/5">
                <div class="w-24 h-24 bg-white rounded-full flex items-center justify-center shadow-sm mb-6 animate-pulse-slow">
                    <i class="ri-book-read-line text-4xl text-accent/50"></i>
                </div>
                <h3 class="text-xl font-black font-serif text-ink/40 mb-2">No Document Selected</h3>
                <p class="text-sm">Select a page from the sidebar or create a new one.</p>
                <button v-if="!isTrashMode" @click="createNewDoc" class="mt-8 px-6 py-2.5 bg-white border border-ash/30 shadow-sm rounded-lg text-sm font-bold text-ink hover:text-accent hover:border-accent/40 transition-all flex items-center gap-2">
                    <i class="ri-add-line"></i> Create New Page
                </button>
            </div>
        </div>
        
        <CreateDocModal 
            :is-open="showCreateModal"
            @close="showCreateModal = false"
            @create="handleCreate"
        />
    </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch, computed } from 'vue';
import { vrkbApi } from '@/api/vrkb';
import { useVrkbStore } from '@/stores/vrkb';
import { DateTime } from 'luxon';
import CreateDocModal from './CreateDocModal.vue';

const store = useVrkbStore();
const docs = ref<any[]>([]);
const selectedDoc = ref<any>(null);
const searchQuery = ref('');
const showCreateModal = ref(false);
const titleTextarea = ref<HTMLTextAreaElement | null>(null);
const currentFolderId = ref<string | null>(null);

// Drag & Drop State
const draggingId = ref<string | null>(null);
const dragOverId = ref<string | null>(null);

// Trash State
const isTrashMode = ref(false);

const toggleTrash = () => {
    isTrashMode.value = !isTrashMode.value;
    loadDocs();
    // Clear selection if switching modes
    selectedDoc.value = null;
    currentFolderId.value = null; // Reset folder nav in trash mode
};

// Compute breadcrumbs path
const breadcrumbs = computed(() => {
    const path = [{ id: null, title: 'Root' }];
    if (!currentFolderId.value) return path;
    
    // Simple traversal up (could be optimized if we had map)
    let curr = docs.value.find(d => d.id === currentFolderId.value);
    const stack = [];
    while (curr) {
        stack.unshift({ id: curr.id, title: curr.title });
        curr = docs.value.find(d => d.id === curr.parent_id);
    }
    return [...path, ...stack];
});

const filteredDocs = computed(() => {
    let list = docs.value;
    
    // Search Mode: Show all matching (flattened)
    if (searchQuery.value) {
        const q = searchQuery.value.toLowerCase();
        return list.filter(d => d.title.toLowerCase().includes(q));
    }

    // Browse Mode: Filter by parent (Only if not in Search Mode)
    // In Trash Mode, we show flat list of deleted items (or maybe tree? usually flat in trash)
    if (isTrashMode.value) {
         // Flat list sorted by deleted date
         return list.sort((a, b) => {
             return new Date(b.deleted_at).getTime() - new Date(a.deleted_at).getTime();
         });
    }

    list = list.filter(d => d.parent_id === currentFolderId.value);

    // Sort: Folders first, then by title
    return list.sort((a, b) => {
        const aIsFolder = a.content?.is_folder ? 1 : 0;
        const bIsFolder = b.content?.is_folder ? 1 : 0;
        if (aIsFolder !== bIsFolder) return bIsFolder - aIsFolder;
        return a.title.localeCompare(b.title);
    });
});

const loadDocs = async () => {
    if (!store.currentProject) return;
    try {
        if (isTrashMode.value) {
            // Need API endpoint for trash, or we can use dedicated trash fetcher if implemented in API client
            // Assuming vrkbApi.listTrash exists, otherwise fallback to custom fetch
             const res = await fetch(`/api/vrkb/projects/${store.currentProject.id}/trash`);
             if (res.ok) {
                 docs.value = await res.json();
             }
        } else {
            docs.value = await vrkbApi.listDocs(store.currentProject.id);
        }
    } catch (e) {
        console.error("Failed to load docs", e);
    }
};

const createNewDoc = () => {
    showCreateModal.value = true;
};

const handleCreate = async (data: { name: string, type: 'file' | 'folder' }) => {
    if (!store.currentProject) return;
    try {
        showCreateModal.value = false;
        
        // Pass parent_id if inside a folder
        const newDoc = await vrkbApi.createDoc(store.currentProject.id, data.name, currentFolderId.value);
        
        if (data.type === 'folder') {
             newDoc.content = { is_folder: true };
             await vrkbApi.updateDoc(newDoc.id, newDoc.title, newDoc.content, newDoc.parent_id);
        }

        await loadDocs();
        
        if (data.type === 'file') {
            selectedDoc.value = docs.value.find(d => d.id === newDoc.id);
            setTimeout(() => {
                if (titleTextarea.value) {
                    try {
                        titleTextarea.value.focus();
                        titleTextarea.value.select();
                    } catch(e) {}
                }
            }, 100);
        }
    } catch (e) {
        console.error("Failed to create doc", e);
    }
};

// Navigate to folder
const navigateTo = (folderId: string | null) => {
    currentFolderId.value = folderId;
};

const handleDocClick = (doc: any) => {
    if (isTrashMode.value) {
        selectedDoc.value = doc;
    } else {
        doc.content?.is_folder ? navigateTo(doc.id) : selectedDoc.value = doc;
    }
}

const deleteDoc = async (id: string) => {
    if (!confirm("Move this item to the Recycle Bin?")) return;
    try {
        await vrkbApi.deleteDoc(id);
        if (selectedDoc.value?.id === id) selectedDoc.value = null;
        await loadDocs();
    } catch (e) {
        console.error("Failed to delete doc", e);
    }
};

const restoreDoc = async (doc: any) => {
    if (!confirm(`Restore "${doc.title}"?`)) return;
    try {
        await fetch(`/api/vrkb/docs/${doc.id}/restore`, { method: 'POST' });
        if (selectedDoc.value?.id === doc.id) selectedDoc.value = null;
        await loadDocs();
    } catch (e) {
        console.error("Failed to restore doc", e);
    }
};

const permanentDeleteDoc = async (doc: any) => {
    if (!confirm(`Permanently delete "${doc.title}"? This cannot be undone.`)) return;
    try {
        await fetch(`/api/vrkb/docs/${doc.id}/permanent`, { method: 'DELETE' });
        if (selectedDoc.value?.id === doc.id) selectedDoc.value = null;
        await loadDocs();
    } catch (e) {
        console.error("Failed to permanent delete doc", e);
    }
};

const saveDoc = async () => {
    if (!selectedDoc.value || isTrashMode.value) return; // Don't save if in trash
    try {
        await vrkbApi.updateDoc(selectedDoc.value.id, selectedDoc.value.title, selectedDoc.value.content, selectedDoc.value.parent_id);
        // Refresh list logic... 
        // Actually for simple list update:
        const idx = docs.value.findIndex(d => d.id === selectedDoc.value.id);
        if (idx !== -1) {
             docs.value[idx] = { ...selectedDoc.value, updated_at: new Date().toISOString() };
        }
    } catch (e) {
        console.error("Failed to save doc", e);
    }
};

const formatDate = (iso: string) => {
    if (!iso) return 'Just now';
    return DateTime.fromISO(iso).toRelative();
};

const autoResize = (event: Event) => {
    const textarea = event.target as HTMLTextAreaElement;
    textarea.style.height = 'auto';
    textarea.style.height = textarea.scrollHeight + 'px';
};

// Drag & Drop Handlers
const onDragStart = (e: DragEvent, doc: any) => {
    if (e.dataTransfer) {
        draggingId.value = doc.id;
        e.dataTransfer.effectAllowed = 'move';
        e.dataTransfer.setData('text/plain', doc.id);
    }
};

const onDragEnd = () => {
    draggingId.value = null;
    dragOverId.value = null;
};

const onDragOver = (e: DragEvent, targetId: string | null) => {
    if (draggingId.value === targetId) return;
    dragOverId.value = targetId;
};

const onDragLeave = (e: DragEvent) => {
    // Optional: refine interactions
};

const onDrop = async (e: DragEvent, targetFolderId: string | null) => {
    const docId = e.dataTransfer?.getData('text/plain');
    if (!docId || docId === targetFolderId) return;

    // Check for circular dependency
    if (targetFolderId) {
        let curr = docs.value.find(d => d.id === targetFolderId);
        while (curr) {
            if (curr.id === docId) {
                alert("Cannot move a folder into itself.");
                return;
            }
            curr = docs.value.find(d => d.id === curr.parent_id);
        }
    }

    try {
        const item = docs.value.find(d => d.id === docId);
        if (item) {
             item.parent_id = targetFolderId;
             // Cancel any pending auto-save for this item if it's selected to avoid race conditions
             if (selectedDoc.value?.id === item.id && saveTimeout) {
                 clearTimeout(saveTimeout);
             }
             
             await vrkbApi.updateDoc(item.id, item.title, item.content, targetFolderId);
             await loadDocs(); 
        }
    } catch (e: any) {
        console.error("Failed to move doc", e);
        if (e.response) {
            console.error("Server Response:", e.response.status, e.response.data);
            alert(`Failed to move item: ${e.response.data?.message || e.message}`);
        } else {
            console.error("Error details:", e);
            alert(`Failed to move item: ${e.message}`);
        }
        await loadDocs(); 
    } finally {
        draggingId.value = null;
        dragOverId.value = null;
    }
};


onMounted(() => {
    loadDocs();
});

let saveTimeout: any;
watch(() => selectedDoc.value, (newVal) => {
    if (newVal && !isTrashMode.value) {
        if (saveTimeout) clearTimeout(saveTimeout);
        saveTimeout = setTimeout(() => {
            saveDoc();
        }, 2000);
    }
}, { deep: true });
</script>
