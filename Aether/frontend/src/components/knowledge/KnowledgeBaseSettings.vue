<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import { knowledgeApi, type KnowledgeBase } from '@/api/knowledge';
import { uploadApi } from '@/api/upload';
import TagInput from '@/components/common/TagInput.vue';
import { DialogPlugin, MessagePlugin } from 'tdesign-vue-next';

interface Props {
    kb: KnowledgeBase;
}

const props = defineProps<Props>();
const emit = defineEmits(['close', 'update', 'delete']);

const form = ref({
    title: '',
    description: '',
    tags: '',
    cover_image: '',
    cover_offset_y: 50,
    visibility: 'Private' as 'Public' | 'Private' | 'Internal',
    renderer_id: 'default'
});

const initForm = () => {
    if (!props.kb) return;
    form.value = {
        title: props.kb.title,
        description: props.kb.description || '',
        tags: props.kb.tags ? props.kb.tags.join(', ') : '',
        cover_image: props.kb.cover_image || '',
        cover_offset_y: props.kb.cover_offset_y || 50,
        visibility: props.kb.visibility || 'Private',
        renderer_id: props.kb.renderer_id || 'default'
    };
};

watch(() => props.kb, initForm, { immediate: true });

const handleImageUpload = async (event: Event) => {
    const input = event.target as HTMLInputElement;
    if (!input.files || input.files.length === 0) return;

    const file = input.files[0];
    try {
        const url = await uploadApi.uploadFile(file);
        form.value.cover_image = url;
    } catch (e: any) {
        if (e.response && e.response.status === 413) {
            alert("File is too large. Max size is 5MB.");
        } else {
             console.error("Failed to upload image", e);
             alert("Failed to upload image");
        }
    }
};

const updateKbSettings = async () => {
    try {
        await knowledgeApi.update(props.kb.id, {
            title: form.value.title,
            description: form.value.description,
            tags: form.value.tags.split(',').map(t => t.trim()).filter(t => t),
            cover_image: form.value.cover_image,
            cover_offset_y: form.value.cover_offset_y,
            visibility: form.value.visibility,
            renderer_id: form.value.renderer_id
        });
        emit('update');
        emit('close');
        MessagePlugin.success('Settings saved successfully');
    } catch (e: any) {
        if (e.response && e.response.status === 409) {
            alert("A Knowledge Base with this title already exists.");
        } else {
            console.error("Failed to update KB", e);
            alert("Failed to save settings");
        }
    }
};

const deleteKb = async () => {
    const confirmDialog = DialogPlugin.confirm({
        header: 'Delete Knowledge Base',
        body: 'Are you sure you want to delete this Knowledge Base? This cannot be undone.',
        theme: 'danger',
        onConfirm: async () => {
            try {
                await knowledgeApi.delete(props.kb.id);
                confirmDialog.hide();
                // We should redirect after delete, but that depends on parent context.
                // For now, emit close/update and let parent handle navigation usually.
                // But in KnowledgeBaseDetail, we are on the page of the KB.
                // We'll treat this as a special event or let the user handle it.
                // Since this component is just settings, we'll emit 'delete' or just assume 'update' implies state change.
                // Actually, if deleted, we should probably redirect home.
                // Let's emit a specific 'deleted' event.
                emit('delete'); 
                emit('close');
            } catch (e) {
                console.error("Failed to delete KB", e);
            }
        }
    });
};
</script>

<template>
    <div class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm p-4" @click.self="$emit('close')">
        <div class="bg-surface rounded-lg border border-ink/5 p-8 max-w-2xl w-full max-h-[90vh] overflow-y-auto shadow-2xl relative">
            
            <button @click="$emit('close')" class="absolute top-4 right-4 text-ink/40 hover:text-ink">
                <i class="ri-close-line text-2xl"></i>
            </button>

            <h3 class="text-xl font-bold font-serif mb-6 pb-4 border-b border-ink/5">Knowledge Base Settings</h3>

            <div class="space-y-4">
                <div>
                    <label class="block text-xs font-bold uppercase tracking-wider text-ink/40 mb-1">Title</label>
                    <input v-model="form.title"
                        class="w-full bg-paper border border-ink/10 rounded px-3 py-2 text-sm focus:outline-none focus:border-accent" />
                </div>
                <div>
                    <label class="block text-xs font-bold uppercase tracking-wider text-ink/40 mb-1">Description</label>
                    <textarea v-model="form.description" rows="3"
                        class="w-full bg-paper border border-ink/10 rounded px-3 py-2 text-sm focus:outline-none focus:border-accent"></textarea>
                </div>

                <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                    <div>
                        <label class="block text-xs font-bold uppercase tracking-wider text-ink/40 mb-1">Visibility</label>
                        <select v-model="form.visibility"
                            class="w-full bg-paper border border-ink/10 rounded px-3 py-2 text-sm focus:outline-none focus:border-accent">
                            <option value="Private">Private</option>
                            <option value="Internal">Internal</option>
                            <option value="Public">Public</option>
                        </select>
                        <p class="text-[10px] text-ink/40 mt-1">
                            Private: Only me. Internal: All logged in users. Public: Everyone.
                        </p>
                    </div>
                    <div>
                        <label class="block text-xs font-bold uppercase tracking-wider text-ink/40 mb-1">Layout Renderer</label>
                        <select v-model="form.renderer_id"
                            class="w-full bg-paper border border-ink/10 rounded px-3 py-2 text-sm focus:outline-none focus:border-accent">
                            <option value="default">Standard (Blog)</option>
                            <option value="math_v1">Math Archive (Graph)</option>
                            <option value="math_v3">Math Manuscript (Book)</option>
                            <option value="english_v1">English Analysis (Academic)</option>
                        </select>
                    </div>
                    <div class="md:col-span-2">
                        <label class="block text-xs font-bold uppercase tracking-wider text-ink/40 mb-1">Tags</label>
                        <TagInput v-model="form.tags" placeholder="Add tags..." />
                    </div>
                </div>

                <div>
                    <label class="block text-xs font-bold uppercase tracking-wider text-ink/40 mb-1">Cover Image</label>
                    <div class="relative group w-full h-48 rounded-lg overflow-hidden bg-ash/10 border border-ink/10">
                        <img v-if="form.cover_image" :src="form.cover_image"
                            class="w-full h-full object-cover" 
                            :style="{ objectPosition: `50% ${form.cover_offset_y || 50}%` }" />
                        <div v-else class="w-full h-full flex items-center justify-center text-ink/20">
                            <i class="ri-image-add-line text-4xl"></i>
                        </div>
                        <div class="absolute inset-0 bg-black/50 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center">
                            <label class="cursor-pointer bg-white text-black px-4 py-2 rounded-full font-medium text-sm hover:bg-white/90 transition-colors flex items-center gap-2">
                                <i class="ri-upload-cloud-line"></i> Change Image
                                <input type="file" accept="image/*" class="hidden" @change="handleImageUpload" />
                            </label>
                        </div>
                    </div>

                    <div v-if="form.cover_image" class="mt-4 bg-surface p-3 rounded border border-ink/5">
                        <label class="block text-[10px] font-bold uppercase tracking-wider text-ink/40 mb-2 flex justify-between">
                            <span>Cover Position (Vertical)</span>
                            <span>{{ form.cover_offset_y }}%</span>
                        </label>
                        <input type="range" min="0" max="100" v-model.number="form.cover_offset_y" 
                            class="w-full h-1 bg-ink/10 rounded-lg appearance-none cursor-pointer accent-ink" />
                    </div>

                    <input v-model="form.cover_image" placeholder="Or enter image URL..."
                        class="mt-2 w-full bg-transparent border-b border-ink/10 py-1 text-xs text-ink/60 focus:outline-none focus:border-accent" />
                </div>

                <div class="pt-6 border-t border-ink/5 flex justify-between items-center">
                    <button @click="deleteKb"
                        class="text-red-500 hover:text-red-700 text-sm font-medium px-4 py-2 hover:bg-red-50 rounded transition-colors">
                        <i class="ri-delete-bin-line mr-1"></i> Delete Knowledge Base
                    </button>
                    <div class="flex gap-2">
                         <button @click="$emit('close')"
                            class="px-4 py-2 text-sm font-medium text-ink/60 hover:text-ink hover:bg-ash/10 rounded transition-colors">
                            Cancel
                        </button>
                        <button @click="updateKbSettings"
                            class="bg-ink text-paper px-6 py-2 rounded font-medium hover:bg-accent transition-colors">
                            Save Changes
                        </button>
                    </div>
                </div>
            </div>
        </div>
    </div>
</template>
