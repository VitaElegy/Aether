<template>
    <div class="p-8 max-w-7xl mx-auto min-h-screen bg-surface">
        <!-- Header -->
        <div class="flex items-end justify-between mb-10">
            <div>
                <h1 class="text-3xl font-serif font-bold text-ink mb-2">Layout Templates</h1>
                <p class="text-ink/50 text-sm max-w-lg leading-relaxed">
                    Design and manage the visual presentation of your Knowledge Bases. 
                    Choose from specialized renderers or create custom visual styles.
                </p>
            </div>
            <button 
                @click="openCreateModal" 
                class="px-5 py-2.5 bg-accent text-white rounded-lg shadow-lg shadow-accent/20 hover:bg-accent/90 hover:shadow-xl hover:-translate-y-0.5 transition-all duration-300 flex items-center gap-2 text-sm font-medium"
            >
                <i class="ri-add-line text-lg"></i>
                <span>Create Template</span>
            </button>
        </div>

        <!-- Grid -->
        <div v-if="loading" class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8 animate-pulse">
            <div v-for="i in 3" :key="i" class="h-80 bg-ash/10 rounded-xl"></div>
        </div>

        <div v-else class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8">
            <div v-for="template in templates" :key="template.id" 
                class="group bg-paper rounded-xl overflow-hidden hover:shadow-2xl hover:shadow-ink/5 transition-all duration-500 cursor-pointer border border-ink/5 relative flex flex-col h-full hover:-translate-y-1"
                @click="openEditModal(template)">
                
                <!-- Thumbnail Stage -->
                <div class="aspect-video w-full relative bg-ash/5 overflow-hidden group-hover:scale-[1.02] transition-transform duration-700">
                    <!-- Image Validity Check -->
                    <img 
                        v-if="isValidImageForDisplay(template)" 
                        :src="template.thumbnail" 
                        class="w-full h-full object-cover transition-all duration-700 filter group-hover:brightness-110" 
                        :style="getCoverStyle(template)"
                        @error="handleImageError(template)"
                    />
                    
                    <!-- Fallback Icon -->
                    <div v-if="!isValidImageForDisplay(template)" class="w-full h-full flex flex-col items-center justify-center text-ink/20 group-hover:text-ink/30 transition-colors gap-3 bg-gradient-to-br from-ash/5 to-ash/20">
                        <i :class="[getIconForRenderer(template.renderer_id), 'text-6xl']"></i>
                        <span class="text-[10px] uppercase tracking-widest font-bold opacity-50">No Cover</span>
                    </div>

                    <!-- Overlay Actions -->
                    <div class="absolute inset-0 bg-ink/60 opacity-0 group-hover:opacity-100 transition-opacity duration-300 flex items-center justify-center backdrop-blur-[2px]">
                        <span class="bg-white text-ink px-5 py-2 rounded-full text-xs font-bold tracking-wide shadow-xl transform scale-95 group-hover:scale-100 transition-transform duration-300">
                            Edit Template
                        </span>
                    </div>

                    <!-- Renderer Badge -->
                    <div class="absolute top-3 right-3 px-2.5 py-1 bg-white/90 backdrop-blur text-ink border border-ink/5 shadow-sm rounded-md text-[10px] uppercase font-bold tracking-wider z-10">
                        {{ getRendererLabel(template.renderer_id) }}
                    </div>
                </div>

                <!-- Content -->
                <div class="p-6 flex flex-col flex-grow">
                    <div class="mb-4">
                        <h3 class="font-bold text-lg text-ink group-hover:text-accent transition-colors duration-300 font-serif leading-tight">
                            {{ template.title }}
                        </h3>
                        <p class="text-[10px] text-ink/40 font-mono mt-1 opacity-60">ID: {{ template.id }}</p>
                    </div>
                    
                    <!-- Relaxed line clamp to show more text -->
                    <p class="text-sm text-ink/70 line-clamp-4 leading-relaxed mb-4 flex-grow" :title="template.description">
                        {{ template.description }}
                    </p>
                    
                    <div class="pt-4 border-t border-ink/5 flex flex-wrap gap-2 mt-auto">
                         <span v-for="tag in template.tags" :key="tag" 
                            class="text-[10px] text-ink/50 bg-ash/10 px-2 py-1 rounded-md font-medium transition-colors group-hover:bg-accent/5 group-hover:text-accent">
                            #{{ tag }}
                        </span>
                        <span v-if="template.tags.length === 0" class="text-[10px] text-ink/20 italic">No tags</span>
                    </div>
                </div>
            </div>
            
            <!-- Create New Card -->
             <div 
                @click="openCreateModal"
                class="group min-h-[400px] rounded-xl border-2 border-dashed border-ink/10 flex flex-col items-center justify-center gap-4 text-ink/30 hover:border-accent/50 hover:text-accent hover:bg-accent/5 transition-all duration-300 cursor-pointer"
            >
                <div class="w-16 h-16 rounded-full bg-ink/5 group-hover:bg-accent/10 flex items-center justify-center transition-colors">
                    <i class="ri-add-line text-3xl"></i>
                </div>
                <span class="text-sm font-bold uppercase tracking-wider">Create New</span>
            </div>
        </div>

        <!-- Editor Modal -->
        <t-dialog 
            v-model:visible="showModal" 
            :header="false"
            width="1000px"
            top="5vh"
            :footer="false"
            class="template-dialog overflow-hidden rounded-2xl"
        >
            <div class="flex flex-col md:flex-row h-[70vh] min-h-[600px]">
                
                <!-- Left: Visual Preview (Sidebar) -->
                <div class="md:w-5/12 bg-ash/5 border-r border-ink/5 relative flex flex-col">
                     <!-- Interactive Image Area -->
                     <div class="flex-grow relative group overflow-hidden bg-ash/10">
                         <t-upload
                            action="/api/upload" 
                            name="file"
                            :headers="uploadHeaders"
                            :show-upload-list="false"
                            @success="handleUploadSuccess"
                            @fail="handleUploadFail"
                            class="w-full h-full block"
                        >
                            <div class="w-full h-full absolute inset-0 cursor-pointer">
                                <!-- Image Preview -->
                                <img v-if="isValidFormImage()" 
                                    :src="form.thumbnail" 
                                    class="w-full h-full object-cover transition-all duration-300"
                                    :style="{ objectPosition: `center ${form.cover_offset_y}%` }"
                                 />
                                 
                                 <!-- Empty State -->
                                 <div v-else class="w-full h-full flex flex-col items-center justify-center text-ink/20 gap-4 p-8 text-center bg-gradient-to-b from-transparent to-black/5">
                                    <i :class="[getIconForRenderer(form.renderer_id), 'text-8xl opacity-80']"></i>
                                    <div>
                                        <p class="text-sm font-bold uppercase tracking-widest text-ink/40">No Cover Image</p>
                                        <p class="text-xs text-ink/30 mt-1">Click anywhere to upload</p>
                                    </div>
                                 </div>

                                 <!-- Hover Overlay -->
                                 <div class="absolute inset-0 bg-black/50 opacity-0 group-hover:opacity-100 transition-opacity flex flex-col items-center justify-center text-white backdrop-blur-sm z-10">
                                    <i class="ri-upload-cloud-2-line text-6xl mb-3"></i>
                                    <span class="text-xs font-bold uppercase tracking-wider">Replace Cover</span>
                                    <span class="text-[10px] opacity-70 mt-1">Supports JPG, PNG, WEBP</span>
                                 </div>
                                 
                                 <!-- Focal Line Guide (Only visible when dragging slider in Right panel or hovering image) -->
                                 <div v-if="isValidFormImage()" 
                                      class="absolute left-0 right-0 border-t border-white/80 border-dashed pointer-events-none transition-opacity duration-300 opacity-60"
                                      :style="{ top: `${form.cover_offset_y}%` }">
                                 </div>
                            </div>
                        </t-upload>
                     </div>
                     
                     <!-- Focal Point Mini Control (Bottom of Image) -->
                     <div class="p-4 bg-white/50 backdrop-blur border-t border-ink/5">
                        <div class="flex justify-between items-center mb-2">
                            <label class="text-[10px] uppercase font-bold text-ink/50">Focal Point</label>
                            <span class="text-[10px] font-mono bg-white px-1.5 py-0.5 rounded text-ink/70 shadow-sm border border-ink/5">{{ form.cover_offset_y }}%</span>
                        </div>
                        <input 
                            type="range" 
                            min="0" max="100" 
                            v-model.number="form.cover_offset_y" 
                            class="w-full h-1 bg-ink/10 rounded-lg appearance-none cursor-pointer accent-accent hover:accent-accent/80 transition-all"
                        >
                     </div>
                </div>

                <!-- Right: Form Controls -->
                <div class="md:w-7/12 bg-surface p-8 flex flex-col overflow-y-auto">
                    
                    <!-- Header -->
                    <div class="mb-6 pb-6 border-b border-ink/5">
                        <h2 class="text-2xl font-serif font-bold text-ink mb-1">{{ isEditing ? 'Edit Template' : 'New Template' }}</h2>
                        <p class="text-sm text-ink/50">Configure how your content is presented to the world.</p>
                    </div>

                    <!-- Fields -->
                    <div class="space-y-6 flex-grow">
                        
                        <!-- Title -->
                        <div class="group">
                            <label class="block text-xs font-bold uppercase tracking-wider text-ink/40 mb-1.5 group-focus-within:text-accent transition-colors">Title</label>
                            <input v-model="form.title" type="text" class="w-full bg-ash/5 border border-transparent rounded-lg px-4 py-3 text-ink font-serif text-lg focus:bg-white focus:border-accent focus:shadow-lg focus:ring-0 transition-all outline-none" placeholder="e.g. Modern Math V2" />
                        </div>

                         <!-- Renderer Selector -->
                         <div class="group">
                            <label class="block text-xs font-bold uppercase tracking-wider text-ink/40 mb-1.5 group-focus-within:text-accent transition-colors">Rendering Engine</label>
                            <div class="relative">
                                <select v-model="form.renderer_id" class="w-full appearance-none bg-ash/5 border border-transparent rounded-lg px-4 py-3 text-sm text-ink focus:bg-white focus:border-accent focus:shadow-lg focus:ring-0 transition-all outline-none cursor-pointer">
                                    <option v-for="(label, id) in RENDERER_LABELS" :key="id" :value="id">{{ label }}</option>
                                </select>
                                <div class="absolute inset-y-0 right-4 flex items-center pointer-events-none text-ink/40">
                                    <i class="ri-arrow-down-s-line text-lg"></i>
                                </div>
                            </div>
                        </div>

                        <!-- Description -->
                        <div class="group">
                            <label class="block text-xs font-bold uppercase tracking-wider text-ink/40 mb-1.5 group-focus-within:text-accent transition-colors">Description</label>
                            <textarea v-model="form.description" rows="5" class="w-full bg-ash/5 border border-transparent rounded-lg px-4 py-3 text-sm text-ink/80 focus:bg-white focus:border-accent focus:shadow-lg focus:ring-0 transition-all outline-none resize-none leading-relaxed" placeholder="Describe the purpose and style of this template..."></textarea>
                        </div>

                        <!-- Tags -->
                        <div class="group">
                            <div class="flex justify-between items-center mb-1.5">
                                <label class="block text-xs font-bold uppercase tracking-wider text-ink/40 group-focus-within:text-accent transition-colors">Tags</label>
                            </div>
                            <input v-model="form.tagsInput" type="text" class="w-full bg-ash/5 border border-transparent rounded-lg px-3 py-2 text-sm focus:bg-white focus:border-accent focus:shadow-sm outline-none transition-all mb-3" placeholder="Math, Academic, v2" />
                            <div class="flex flex-wrap gap-1.5 min-h-[30px]">
                                <span v-for="tag in form.tagsInput.split(',').filter(t => t.trim())" :key="tag" 
                                    class="text-[10px] text-accent bg-accent/5 px-2 py-0.5 rounded border border-accent/10">
                                    #{{ tag.trim() }}
                                </span>
                            </div>
                        </div>
                        
                         <!-- Manual URL -->
                         <div class="group pt-4 border-t border-ink/5">
                            <div class="flex items-center justify-between mb-1.5">
                                <label class="text-[10px] uppercase font-bold text-ink/30 group-focus-within:text-accent/70 transition-colors">Image URL (Optional)</label>
                                <button v-if="form.thumbnail" @click="form.thumbnail = ''" class="text-[10px] text-red-400 hover:text-red-500 font-bold uppercase tracking-wider">Clear Image</button>
                            </div>
                            <input v-model="form.thumbnail" type="text" class="w-full bg-ash/5 border border-transparent rounded px-2 py-1.5 text-xs text-ink/60 focus:bg-white focus:border-accent/50 outline-none transition-all font-mono" placeholder="https://..." />
                        </div>

                    </div>

                    <!-- Actions -->
                    <div class="flex items-center justify-end gap-4 mt-8 pt-6 border-t border-ink/5">
                        <button @click="showModal = false" class="px-6 py-2.5 text-sm font-medium text-ink/50 hover:text-ink hover:bg-ash/10 rounded-lg transition-colors">
                            Cancel
                        </button>
                        <button 
                            @click="saveTemplate" 
                            :disabled="saving"
                            class="px-8 py-2.5 bg-accent text-white rounded-lg shadow-lg shadow-accent/20 hover:bg-accent/90 hover:shadow-xl hover:-translate-y-0.5 transition-all duration-300 text-sm font-bold flex items-center gap-2"
                        >
                            <i v-if="saving" class="ri-loader-4-line animate-spin"></i>
                            <span>{{ isEditing ? 'Save Changes' : 'Create Template' }}</span>
                        </button>
                    </div>

                </div>
            </div>
        </t-dialog>
    </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue';
import { templateApi, type LayoutTemplate } from '@/api/template';
import { MessagePlugin } from 'tdesign-vue-next';
import { useAuthStore } from '@/stores/auth';

const authStore = useAuthStore();
const loading = ref(true);

// Auth headers for Upload
const uploadHeaders = computed(() => {
    return authStore.token ? { Authorization: `Bearer ${authStore.token}` } : {};
});

const ICON_MAP: Record<string, string> = {
    default: 'ri-article-line',
    math_v3: 'ri-function-line',
    math_v1: 'ri-compass-3-line',
    english_v1: 'ri-book-read-line',
    vrkb: 'ri-bug-line',
    memo: 'ri-sticky-note-line',
    admin_system: 'ri-settings-line'
};

const RENDERER_LABELS: Record<string, string> = {
    default: 'Blog & Standard',
    math_v3: 'Math Ms. V3',
    math_v1: 'Math Archive V1',
    english_v1: 'English Analysis',
    vrkb: 'Vuln. Research',
    memo: 'Memo Board',
    admin_system: 'System Control'
};

const getIconForRenderer = (id: string) => ICON_MAP[id] || 'ri-layout-grid-line';
const getRendererLabel = (id: string) => RENDERER_LABELS[id] || id;

// --- Image Handling Helpers ---

const isValidImage = (url?: string) => {
    if (!url || typeof url !== 'string') return false;
    // Allow http(s) and local absolute paths
    return url.startsWith('http') || url.startsWith('/');
};

const failedImages = ref(new Set<string>());

const handleImageError = (template: LayoutTemplate) => {
    failedImages.value.add(template.id);
};

const isValidImageForDisplay = (template: LayoutTemplate) => {
    return isValidImage(template.thumbnail) && !failedImages.value.has(template.id);
};

const isValidFormImage = () => {
    return isValidImage(form.value.thumbnail);
};

// Redefine getCoverStyle to handle offset
const getCoverStyle = (template: LayoutTemplate) => {
    // @ts-ignore
    const cfg = template.config || {};
    const offset = cfg.cover_offset_y !== undefined ? cfg.cover_offset_y : 50;
    return {
        objectPosition: `center ${offset}%`
    };
};

const templates = ref<LayoutTemplate[]>([]);
const showModal = ref(false);
const isEditing = ref(false);
const saving = ref(false);
const editingId = ref<string | null>(null);

const form = ref({
    renderer_id: 'default',
    title: '',
    description: '',
    thumbnail: '',
    tagsInput: '',
    cover_offset_y: 50
});

const fetchTemplates = async () => {
    loading.value = true;
    try {
        const { data } = await templateApi.list();
        templates.value = data;
        failedImages.value.clear(); // Reset errors on reload
    } catch (e) {
        MessagePlugin.error('Failed to load templates');
    } finally {
        loading.value = false;
    }
};

const openCreateModal = () => {
    isEditing.value = false;
    editingId.value = null;
    form.value = { renderer_id: 'default', title: '', description: '', thumbnail: '', tagsInput: '', cover_offset_y: 50 };
    showModal.value = true;
};

const openEditModal = (t: LayoutTemplate) => {
    isEditing.value = true;
    editingId.value = t.id;
    const cfg = (t as any).config || {};
    form.value = {
        renderer_id: t.renderer_id,
        title: t.title,
        description: t.description,
        thumbnail: t.thumbnail || '',
        tagsInput: t.tags.join(', '),
        cover_offset_y: cfg.cover_offset_y !== undefined ? cfg.cover_offset_y : 50
    };
    showModal.value = true;
};

const handleUploadSuccess = (context: any) => {
    if (context.response && context.response.url) {
        form.value.thumbnail = context.response.url;
        MessagePlugin.success('Image uploaded');
    }
};

const handleUploadFail = (context: any) => {
    console.error("Upload Failed:", context);
    MessagePlugin.error('Upload failed');
};

const saveTemplate = async () => {
    if (!form.value.title) return MessagePlugin.warning('Title is required');
    
    saving.value = true;
    try {
        const tags = form.value.tagsInput.split(',').map(s => s.trim()).filter(s => s);
        
        const payload = {
            renderer_id: form.value.renderer_id,
            title: form.value.title,
            description: form.value.description,
            thumbnail: form.value.thumbnail || undefined,
            tags,
            config: {
                cover_offset_y: form.value.cover_offset_y
            }
        };

        if (isEditing.value && editingId.value) {
            await templateApi.update(editingId.value, payload);
            MessagePlugin.success('Template updated');
        } else {
            await templateApi.create(payload);
            MessagePlugin.success('Template created');
        }
        showModal.value = false;
        fetchTemplates();
    } catch (e) {
        MessagePlugin.error('Failed to save');
    } finally {
        saving.value = false;
    }
};

onMounted(fetchTemplates);
</script>

<style scoped>
:deep(.t-dialog__body) {
    padding: 0 !important;
}

/* Ensure modal fits in varied screens */
@media (max-height: 800px) {
  .template-dialog {
      top: 2vh !important;
  }
}
</style>
