<template>
    <div class="p-8 max-w-7xl mx-auto">
        <div class="flex items-center justify-between mb-8">
            <div>
                <h1 class="text-2xl font-bold text-ink font-serif mb-1">Layout Templates</h1>
                <p class="text-ink/60 text-sm">Manage the visual templates available for Knowledge Bases.</p>
            </div>
            <button @click="openCreateModal" class="px-4 py-2 bg-accent text-white rounded hover:bg-accent/90 transition-colors flex items-center gap-2">
                <i class="ri-add-line"></i>
                New Template
            </button>
        </div>

        <!-- Grid -->
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            <div v-for="template in templates" :key="template.id" 
                class="group bg-paper border border-ink/10 rounded-lg overflow-hidden hover:shadow-lg transition-all cursor-pointer relative"
                @click="openEditModal(template)">
                
                <!-- Thumbnail -->
                <div class="h-40 w-full relative bg-ash/10">
                    <img v-if="template.thumbnail?.startsWith('http')" :src="template.thumbnail" class="w-full h-full object-cover" />
                    <div v-else class="w-full h-full" :class="template.thumbnail || 'bg-gradient-to-br from-gray-100 to-gray-200'"></div>
                    
                    <div class="absolute inset-0 bg-black/0 group-hover:bg-black/10 transition-colors flex items-center justify-center opacity-0 group-hover:opacity-100">
                        <span class="bg-white/90 text-ink px-3 py-1 rounded-full text-xs font-bold shadow-sm">Edit</span>
                    </div>
                </div>

                <!-- Content -->
                <div class="p-4">
                    <div class="flex items-start justify-between mb-2">
                        <h3 class="font-bold text-ink truncate">{{ template.title }}</h3>
                        <span class="text-[10px] uppercase font-bold tracking-wider text-ink/40 bg-ink/5 px-2 py-0.5 rounded">
                            {{ template.renderer_id }}
                        </span>
                    </div>
                    <p class="text-xs text-ink/60 line-clamp-2 mb-3 h-8">{{ template.description }}</p>
                    <div class="flex flex-wrap gap-1">
                        <span v-for="tag in template.tags" :key="tag" class="text-[10px] text-accent bg-accent/5 px-1.5 py-0.5 rounded">
                            #{{ tag }}
                        </span>
                    </div>
                </div>
            </div>
        </div>

        <!-- Editor Modal -->
        <t-dialog v-model:visible="showModal" :header="isEditing ? 'Edit Template' : 'New Template'" @confirm="saveTemplate" :confirm-loading="saving" width="600px">
            <div class="space-y-4">
                
                <!-- ID/Renderer (Read-onlyish or Select) -->
                 <div>
                    <label class="block text-xs font-bold uppercase tracking-wider text-ink/40 mb-1">Renderer ID (Codebase)</label>
                    <select v-model="form.renderer_id" class="w-full bg-surface border border-ink/10 rounded px-3 py-2 text-sm">
                        <option value="default">Default (Blog)</option>
                        <option value="math_v3">Math Manuscript (V3)</option>
                        <option value="math_v1">Math Archive (V1)</option>
                        <option value="english_v1">English Analysis (V1)</option>
                        <option value="vrkb">Vulnerability Research (Kanban)</option>
                        <option value="memo">Memo Board</option>
                        <option value="admin_system">System Control (Protected)</option>
                    </select>
                </div>

                <!-- Title -->
                <div>
                    <label class="block text-xs font-bold uppercase tracking-wider text-ink/40 mb-1">Title</label>
                    <input v-model="form.title" type="text" class="w-full bg-surface border border-ink/10 rounded px-3 py-2 text-sm focus:border-accent focus:outline-none" placeholder="e.g. Math Book V2" />
                </div>

                <!-- Description -->
                <div>
                    <label class="block text-xs font-bold uppercase tracking-wider text-ink/40 mb-1">Description</label>
                    <textarea v-model="form.description" rows="3" class="w-full bg-surface border border-ink/10 rounded px-3 py-2 text-sm focus:border-accent focus:outline-none" placeholder="Describe the purpose..."></textarea>
                </div>

                <!-- Image / Thumbnail -->
                <div>
                    <label class="block text-xs font-bold uppercase tracking-wider text-ink/40 mb-1">Thumbnail URL / Class</label>
                    <div class="flex gap-2">
                         <input v-model="form.thumbnail" type="text" class="flex-1 bg-surface border border-ink/10 rounded px-3 py-2 text-sm focus:border-accent focus:outline-none" placeholder="https://... or bg-red-500" />
                         <!-- Simple Upload TRIGGER (Implementation of Upload is complex, assume user pastes URL for now or we build upload later) -->
                         <!-- Actually user requested verify upload capability -->
                         <t-upload
                            action="/api/uploads" 
                            name="file"
                            :show-upload-list="false"
                            @success="handleUploadSuccess"
                         >
                            <button class="px-3 py-2 bg-ash/10 hover:bg-ash/20 rounded text-xs font-bold text-ink">Upload</button>
                         </t-upload>
                    </div>
                    <p class="text-[10px] text-ink/40 mt-1">Paste a URL or upload an image. Defaults to CSS gradient if empty.</p>
                </div>

                <!-- Tags -->
                <div>
                    <label class="block text-xs font-bold uppercase tracking-wider text-ink/40 mb-1">Tags (Comma separated)</label>
                    <input v-model="form.tagsInput" type="text" class="w-full bg-surface border border-ink/10 rounded px-3 py-2 text-sm focus:border-accent focus:outline-none" placeholder="Math, Academic, v2" />
                </div>
            </div>
        </t-dialog>
    </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { templateApi, type LayoutTemplate } from '@/api/template';
import { MessagePlugin } from 'tdesign-vue-next';

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
    tagsInput: ''
});

const fetchTemplates = async () => {
    try {
        const { data } = await templateApi.list();
        templates.value = data;
    } catch (e) {
        MessagePlugin.error('Failed to load templates');
    }
};

const openCreateModal = () => {
    isEditing.value = false;
    editingId.value = null;
    form.value = { renderer_id: 'default', title: '', description: '', thumbnail: '', tagsInput: '' };
    showModal.value = true;
};

const openEditModal = (t: LayoutTemplate) => {
    isEditing.value = true;
    editingId.value = t.id;
    form.value = {
        renderer_id: t.renderer_id,
        title: t.title,
        description: t.description,
        thumbnail: t.thumbnail || '',
        tagsInput: t.tags.join(', ')
    };
    showModal.value = true;
};

const handleUploadSuccess = (context: any) => {
    if (context.response && context.response.url) {
        form.value.thumbnail = context.response.url;
        MessagePlugin.success('Image uploaded');
    }
};

const saveTemplate = async () => {
    saving.value = true;
    try {
        const tags = form.value.tagsInput.split(',').map(s => s.trim()).filter(s => s);
        
        const payload = {
            renderer_id: form.value.renderer_id,
            title: form.value.title,
            description: form.value.description,
            thumbnail: form.value.thumbnail || undefined,
            tags
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
        MessagePlugin.error('Failed to save template');
    } finally {
        saving.value = false;
    }
};

onMounted(fetchTemplates);
</script>
