<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue';

const props = defineProps<{
    kbId?: string;
}>();
import { useRouter } from 'vue-router';
import { knowledgeApi, type KnowledgeBase } from '../../../api/knowledge';
import { contentApi, type Content } from '../../../api/content';
import { uploadApi } from '../../../api/upload';
import TagInput from '../../common/TagInput.vue';
import UserSelect from '../../common/UserSelect.vue';
import BlockRenderer from '../renderer/BlockRenderer.vue'; // [FIXED]
import { DialogPlugin, MessagePlugin } from 'tdesign-vue-next';

// -- State --
const router = useRouter();
const viewMode = ref<'list' | 'detail' | 'settings' | 'article'>('list'); // [MODIFIED]
const knowledgeBases = ref<KnowledgeBase[]>([]);
const currentKb = ref<KnowledgeBase | null>(null);
const currentPath = ref<Content[]>([]); // Stack of folder objects
const contents = ref<Content[]>([]); // Current folder contents
const currentArticle = ref<any>(null); // [NEW]
const currentBlocks = ref<any[]>([]);  // [NEW] Computed blocks for renderer
const isLoading = ref(false);

const kbFormVisible = ref(false);
const kbForm = ref({ title: '', description: '', tags: '', cover_image: '', cover_offset_y: 50, renderer_id: 'default' });

// Settings Form
const settingsForm = ref({
    title: '',
    description: '',
    tags: '',
    cover_image: '',
    cover_offset_y: 50,
    visibility: 'Private' as 'Public' | 'Private' | 'Internal',
    renderer_id: 'default'
});
const collaborators = ref<any[]>([]); // { user_id, username, role, ... }
const newCollaboratorRole = ref('Viewer');
const newCollaboratorUser = ref<any>(null);

const contentFormVisible = ref(false);
const contentFormType = ref<'Folder' | 'Article'>('Article');
const contentForm = ref({ title: '' });

// Context Menu State
const contextMenuVisible = ref(false);
const contextMenuTarget = ref<Content | null>(null);
const contextMenuPosition = ref({ x: 0, y: 0 });


// -- Computed --
const currentParentId = computed(() => {
    const len = currentPath.value.length;
    return len > 0 ? currentPath.value[len - 1].id : undefined;
});

const currentParentContext = computed(() => {
    const len = currentPath.value.length;
    return len > 0 ? currentPath.value[len - 1] : null;
});

// -- Actions --

const fetchKBs = async () => {
    isLoading.value = true;
    try {
        knowledgeBases.value = await knowledgeApi.list();
    } catch (e) {
        console.error("Failed to list KBs", e);
    } finally {
        isLoading.value = false;
    }
};

const openKb = (kb: KnowledgeBase) => {
    // Navigate to the dedicated Knowledge Base Dashboard
    router.push(`/kb/${kb.id}`);
};

const openSettings = () => {
    if (!currentKb.value) return;
    settingsForm.value = {
        title: currentKb.value.title,
        description: currentKb.value.description || '',
        tags: currentKb.value.tags ? currentKb.value.tags.join(', ') : '',
        cover_image: currentKb.value.cover_image || '',
        cover_offset_y: currentKb.value.cover_offset_y || 50,
        visibility: currentKb.value.visibility || 'Private',
        renderer_id: currentKb.value.renderer_id || 'default'
    };
    viewMode.value = 'settings';
    fetchCollaborators();
};

const fetchCollaborators = async () => {
    if (!currentKb.value) return;
    try {
        collaborators.value = await knowledgeApi.listCollaborators(currentKb.value.id);
    } catch (e) {
        console.error("Failed to list collaborators", e);
    }
};

const addCollaborator = async () => {
    if (!currentKb.value || !newCollaboratorUser.value) return;
    try {
        await knowledgeApi.addCollaborator(currentKb.value.id, newCollaboratorUser.value.id, newCollaboratorRole.value);
        newCollaboratorUser.value = null; // Reset selection
        await fetchCollaborators();
    } catch (e) {
        console.error("Failed to add collaborator", e);
        alert("Failed to add collaborator");
    }
};

const removeCollaborator = async (userId: string) => {
     if (!currentKb.value) return;
     if(!confirm("Remove this collaborator?")) return;
     try {
        await knowledgeApi.removeCollaborator(currentKb.value.id, userId);
        await fetchCollaborators();
     } catch (e) {
        console.error("Failed to remove collaborator", e);
     }
};

const updateKbSettings = async () => {
    if (!currentKb.value) return;
    try {
        await knowledgeApi.update(currentKb.value.id, {
            title: settingsForm.value.title,
            description: settingsForm.value.description,
            tags: settingsForm.value.tags.split(',').map(t => t.trim()).filter(t => t),
            cover_image: settingsForm.value.cover_image,
            cover_offset_y: settingsForm.value.cover_offset_y,
            visibility: settingsForm.value.visibility,
            renderer_id: settingsForm.value.renderer_id
        });

        // Update local state
        currentKb.value.title = settingsForm.value.title;
        currentKb.value.description = settingsForm.value.description;
        currentKb.value.tags = settingsForm.value.tags.split(',').map(t => t.trim()).filter(t => t);
        currentKb.value.cover_image = settingsForm.value.cover_image;
        currentKb.value.cover_offset_y = settingsForm.value.cover_offset_y;
        currentKb.value.visibility = settingsForm.value.visibility;
        currentKb.value.renderer_id = settingsForm.value.renderer_id;

        viewMode.value = 'detail';
        fetchKBs(); // Type of reload
    } catch (e: any) {
        if (e.response && e.response.status === 409) {
            alert("A Knowledge Base with this title already exists.");
        } else {
            console.error("Failed to update KB", e);
        }
    }
};

const handleImageUpload = async (event: Event) => {
    const input = event.target as HTMLInputElement;
    if (!input.files || input.files.length === 0) return;

    const file = input.files[0];
    try {
        const url = await uploadApi.uploadFile(file);
        settingsForm.value.cover_image = url;
    } catch (e: any) {
        if (e.response && e.response.status === 413) {
            alert("File is too large. Max size is 5MB.");
        } else {
             console.error("Failed to upload image", e);
             alert("Failed to upload image");
        }
    }
};

const handleKbCreateImageUpload = async (event: Event) => {
    const input = event.target as HTMLInputElement;
    if (!input.files || input.files.length === 0) return;

    const file = input.files[0];
    try {
        const url = await uploadApi.uploadFile(file);
        kbForm.value.cover_image = url;
    } catch (e: any) {
         if (e.response && e.response.status === 413) {
            alert("File is too large. Max size is 5MB.");
        } else {
            console.error("Failed to upload image", e);
            alert("Failed to upload image");
        }
    }
};

const deleteKb = async () => {
    if (!currentKb.value) return;
    if (!confirm("Are you sure you want to delete this Knowledge Base? This cannot be undone.")) return;
    try {
        await knowledgeApi.delete(currentKb.value.id);
        currentKb.value = null;
        viewMode.value = 'list';
        fetchKBs();
    } catch (e) {
        console.error("Failed to delete KB", e);
    }
};

const refreshContent = async () => {
    if (!currentKb.value) return;
    isLoading.value = true;
    try {
        const allContent = await contentApi.list({ 
            limit: 100,
            knowledge_base_id: currentKb.value.id
        });

        contents.value = allContent.filter(c => {
            // Match parent
            if (currentParentId.value) {
                return c.parent_id === currentParentId.value;
            } else {
                return !c.parent_id; // Root
            }
        });

    } catch (e) {
        console.error(e);
    } finally {
        isLoading.value = false;
    }
};

const navigateInto = (folder: Content) => {
    // Backend returns 'type' field, checking strictly
    if (folder.type !== 'Folder') return;
    currentPath.value.push(folder);
    refreshContent();
};

const navigateToArticle = async (article: Content) => {
    isLoading.value = true;
    try {
        const fullArticle = await contentApi.get(article.id);
        currentArticle.value = fullArticle;
        
        // Adapt Body to Blocks
        if (fullArticle.body) {
            if (fullArticle.body.type === 'Custom' && fullArticle.body.data?.blocks) {
                // New AST Protocol
                currentBlocks.value = fullArticle.body.data.blocks;
            } else if (fullArticle.body.type === 'Markdown') {
                // Legacy Markdown - Wrap in single block
                currentBlocks.value = [{
                    id: 'legacy-md',
                    type: 'markdown',
                    payload: { content: fullArticle.body.data }
                }];
            } else {
                // Fallback / Other types
                currentBlocks.value = []; 
                console.warn("Unknown body type", fullArticle.body);
            }
        } else {
             currentBlocks.value = [];
        }

        viewMode.value = 'article';
    } catch (e) {
        console.error("Failed to fetch article", e);
    } finally {
        isLoading.value = false;
    }
};

const navigateUp = (index?: number) => {
    if (viewMode.value === 'article') {
        // If in article mode, "Back" or "Up" goes to the folder list
        viewMode.value = 'detail';
        currentArticle.value = null;
        // Don't modify path if just going back from article
        if (index !== undefined) {
             currentPath.value = currentPath.value.slice(0, index + 1);
             refreshContent();
        }
    } else {
        if (index === undefined) {
            // Go to root
            currentPath.value = [];
        } else {
            // Go to specific index
            currentPath.value = currentPath.value.slice(0, index + 1);
        }
        refreshContent();
    }
    viewMode.value = 'detail'; // Ensure looking at detail
};

const goBackToList = () => {
    viewMode.value = 'list';
    currentKb.value = null;
    currentPath.value = [];
};

// -- Creation --

const createKb = async () => {
    if (!kbForm.value.title) return;
    try {
        await knowledgeApi.create({
            title: kbForm.value.title,
            description: kbForm.value.description,
            tags: kbForm.value.tags.split(',').map(t => t.trim()).filter(t => t),
            cover_image: kbForm.value.cover_image,
            cover_offset_y: kbForm.value.cover_offset_y,
            visibility: 'Private', // Default
            renderer_id: kbForm.value.renderer_id
        });
        kbFormVisible.value = false;
        kbForm.value = { title: '', description: '', tags: '', cover_image: '', cover_offset_y: 50, renderer_id: 'default' };
        fetchKBs();
    } catch (e: any) {
        if (e.response && e.response.status === 409) {
            alert("A Knowledge Base with this title already exists.");
        } else {
            console.error(e);
        }
    }
};

const createContent = async () => {
    if (!contentForm.value.title) return;
    if (!currentKb.value) return;

    // Only handle Directory creation here
    if (contentFormType.value === 'Article') {
         // Should not happen, but safeguard
         return;
    }

    try {
        // Determine Visibility Default
        // If Parent is Public/Internal, inherit?
        // For now, default to Private unless user explicitly changes (UI doesn't show selector yet per req "minimal").
        // "当目录为公共模式的时候在这个目录下的文章默认为公共模式"
        let visibility: 'Public' | 'Private' | 'Internal' = 'Private';

        if (currentParentContext.value) {
            if (currentParentContext.value.visibility === 'Public') visibility = 'Public';
            if (currentParentContext.value.visibility === 'Internal') visibility = 'Internal';
        }

        await contentApi.create({
            title: contentForm.value.title,
            body: '', // Empty body
            tags: [],
            visibility: visibility,
            knowledge_base_id: currentKb.value.id,
            parent_id: currentParentId.value,
            type: 'Folder',
            status: 'Published' 
        });
        contentFormVisible.value = false;
        contentForm.value = { title: '' };
        refreshContent();
    } catch (e) {
        console.error(e);
    }
};

// -- Context Menu & Deletion --
const handleContextMenu = (e: MouseEvent, item: Content) => {
    e.preventDefault();
    contextMenuTarget.value = item;
    contextMenuPosition.value = { x: e.clientX, y: e.clientY };
    contextMenuVisible.value = true;
};

const closeContextMenu = () => {
    contextMenuVisible.value = false;
    contextMenuTarget.value = null;
};

const handleDeleteFromMenu = () => {
    if (!contextMenuTarget.value) return;
    const item = contextMenuTarget.value;
    closeContextMenu();

    const isFolder = item.type === 'Folder';
    const confirmBody = isFolder 
        ? `Are you sure you want to delete the folder "${item.title}"? All contents inside will be permanently deleted.`
        : `Are you sure you want to delete "${item.title}"?`;

    const confirmDialog = DialogPlugin.confirm({
        header: 'Confirm Deletion',
        body: confirmBody,
        theme: 'danger',
        onConfirm: async () => {
            try {
                await contentApi.delete(item.id);
                MessagePlugin.success('Deleted successfully');
                confirmDialog.hide();
                refreshContent();
            } catch (e) {
                console.error("Delete failed", e);
                MessagePlugin.error('Failed to delete item');
            }
        }
    });
};

import { useNavigationStore } from '@/stores/navigation';
import { usePreferencesStore } from '@/stores/preferences'; // [NEW]
import { onActivated, onDeactivated, onUnmounted } from 'vue';

const navStore = useNavigationStore();
const prefStore = usePreferencesStore(); // [NEW]

const setNavState = () => {
    // Knowledge Module uses standard header (no custom center/right for now)
    // So we ensure we reset any custom state from other modules
    navStore.reset();
};
// ... rest of logic

const navigateToNewArticle = () => {
    if (!currentKb.value) return;
    
    const query: any = {
        knowledge_base_id: currentKb.value.id
    };
    if (currentParentId.value) {
        query.parent_id = currentParentId.value;
    }
    
    router.push({ path: '/editor', query });
};

onMounted(() => {
    if (props.kbId) {
        // Direct Mode
        knowledgeApi.get(props.kbId).then(kb => {
            currentKb.value = kb;
            viewMode.value = 'detail';
            refreshContent();
        }).catch(e => {
            console.error("Failed to load initial KB", e);
            fetchKBs(); // Fallback
        });
    } else {
        fetchKBs();
    }
    document.addEventListener('click', closeContextMenu);
    setNavState();
});

// Watch for prop change (if switching between pinned KBs of same type)
watch(() => props.kbId, (newId) => {
    if (newId) {
         knowledgeApi.get(newId).then(kb => {
            currentKb.value = kb;
            viewMode.value = 'detail';
            refreshContent();
        });
    } else {
        viewMode.value = 'list';
        currentKb.value = null;
        fetchKBs();
    }
});

onActivated(() => {
    setNavState();
});

onDeactivated(() => {
    // No specific cleanup needed if we just use defaults, but good practice
    navStore.reset();
});

onUnmounted(() => {
    // No specific cleanup needed
});

</script>

<template>
    <div class="h-full flex flex-col p-6 max-w-5xl mx-auto w-full">

        <!-- HEADER -->
        <div class="flex items-center justify-between mb-8">
            <div class="flex items-center gap-3">
                <button v-if="viewMode === 'detail' || viewMode === 'settings' || viewMode === 'article'" @click="viewMode === 'article' ? navigateUp() : goBackToList()"
                    class="text-ink/60 hover:text-accent">
                    <i class="ri-arrow-left-line text-xl"></i>
                </button>
                <div v-if="viewMode === 'list'">
                    <h2 class="text-2xl font-bold font-serif tracking-tight">知识库</h2>
                    <p class="text-ink/40 text-xs uppercase tracking-widest mt-1">Knowledge Bases</p>
                </div>
                <div v-else>
                    <h2 class="text-2xl font-bold font-serif tracking-tight">{{ currentKb?.title }}</h2>
                    <div class="flex items-center gap-2 text-xs text-ink/40 font-mono mt-1">
                        <span @click="navigateUp(undefined)" class="cursor-pointer hover:text-accent">Root</span>
                        <template v-for="(folder, idx) in currentPath" :key="folder.id">
                            <span>/</span>
                            <span @click="navigateUp(idx)"
                                class="cursor-pointer hover:text-accent">{{ folder.title }}</span>
                        </template>
                    </div>
                </div>
            </div>

            <div class="flex gap-2">
                <button v-if="viewMode === 'list'" @click="kbFormVisible = true"
                    class="bg-ink text-paper px-4 py-2 text-sm font-medium rounded hover:bg-accent transition-colors">
                    <i class="ri-add-line mr-1"></i> 新建知识库
                </button>
                <template v-else>
                    <button @click="contentFormType = 'Folder'; contentFormVisible = true"
                        class="bg-paper border border-ink/10 text-ink px-3 py-1.5 text-xs font-medium rounded hover:border-accent hover:text-accent transition-colors">
                        <i class="ri-folder-add-line mr-1"></i> 文件夹
                    </button>
                    <button @click="navigateToNewArticle()"
                        class="bg-ink text-paper px-3 py-1.5 text-xs font-medium rounded hover:bg-accent transition-colors">
                        <i class="ri-file-add-line mr-1"></i> 文章
                    </button>
                    <button @click="openSettings"
                        class="bg-surface border border-ink/10 text-ink px-3 py-1.5 text-xs font-medium rounded hover:border-accent hover:text-accent transition-colors ml-2">
                        <i class="ri-settings-3-line mr-1"></i> 设置
                    </button>
                </template>
            </div>
        </div>

        <!-- CONTENT -->
        <div class="flex-1 overflow-y-auto">

            <!-- LIST VIEW -->
            <div v-if="viewMode === 'list'" class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                <div v-for="kb in knowledgeBases" :key="kb.id" @click="openKb(kb)"
                    class="group relative aspect-[4/3] rounded-2xl overflow-hidden cursor-pointer border border-ink/5 hover:shadow-2xl hover:shadow-black/5 transition-all duration-500">

                    <!-- Background Image or Placeholder -->
                    <div class="absolute inset-0 bg-ash/10">
                        <img v-if="kb.cover_image" :src="kb.cover_image"
                            class="w-full h-full object-cover transition-transform duration-700 group-hover:scale-110"
                            :style="{ objectPosition: `50% ${kb.cover_offset_y || 50}%` }" />
                        <div v-else
                            class="w-full h-full flex items-center justify-center bg-gradient-to-br from-ash/20 to-ash/5 text-ink/10">
                            <i class="ri-book-mark-fill text-6xl"></i>
                        </div>
                    </div>

                    <!-- Pin Button (Top Right) -->
                    <button @click.stop="prefStore.togglePin(kb.id)"
                        class="absolute top-3 right-3 z-20 w-8 h-8 rounded-full bg-black/20 backdrop-blur-md flex items-center justify-center text-white hover:bg-black/40 transition-colors"
                        :class="{ 'bg-accent/80 text-white': prefStore.isPinned(kb.id) }"
                        title="Pin to Dock">
                        <i :class="prefStore.isPinned(kb.id) ? 'ri-pushpin-fill' : 'ri-pushpin-line'"></i>
                    </button>

                    <!-- Gradient Overlay -->
                    <div
                        class="absolute inset-0 bg-gradient-to-t from-black/90 via-black/40 to-transparent opacity-80 group-hover:opacity-90 transition-opacity duration-300">
                    </div>

                    <!-- Content Overlay -->
                    <div
                        class="absolute bottom-0 left-0 w-full p-6 text-white transform transition-transform duration-500 group-hover:translate-y-[-4px]">
                        <div
                            class="flex items-center justify-between mb-2 opacity-0 group-hover:opacity-100 transition-opacity duration-300 -translate-y-2 group-hover:translate-y-0">
                            <div class="flex gap-2">
                                <span v-for="tag in kb.tags?.slice(0, 3)" :key="tag"
                                    class="text-[10px] uppercase tracking-wider font-bold bg-white/20 backdrop-blur-sm px-2 py-0.5 rounded-full text-white/90">
                                    {{ tag }}
                                </span>
                            </div>
                            <i class="ri-arrow-right-line text-white/60"></i>
                        </div>

                        <h3
                            class="font-bold text-2xl font-serif mb-2 leading-tight group-hover:text-accent transition-colors">
                            {{ kb.title }}
                        </h3>
                        <p class="text-sm text-white/70 line-clamp-2 leading-relaxed">
                            {{ kb.description || 'No description provided.' }}
                        </p>
                    </div>
                </div>

                <!-- Empty State -->
                <div v-if="knowledgeBases.length === 0 && !isLoading"
                    class="col-span-full py-12 text-center text-ink/40">
                    <i class="ri-inbox-line text-4xl mb-4 block"></i>
                    <p>No Knowledge Bases found.</p>
                </div>
            </div>

            <!-- SETTINGS VIEW -->
            <div v-else-if="viewMode === 'settings'" class="max-w-2xl mx-auto pb-32">
                <div class="bg-surface rounded-lg border border-ink/5 p-8">
                    <h3 class="text-xl font-bold font-serif mb-6 pb-4 border-b border-ink/5">Knowledge Base Settings
                    </h3>

                    <div class="space-y-4">
                        <div>
                            <label
                                class="block text-xs font-bold uppercase tracking-wider text-ink/40 mb-1">Title</label>
                            <input v-model="settingsForm.title"
                                class="w-full bg-paper border border-ink/10 rounded px-3 py-2 text-sm focus:outline-none focus:border-accent" />
                        </div>
                        <div>
                            <label
                                class="block text-xs font-bold uppercase tracking-wider text-ink/40 mb-1">Description</label>
                            <textarea v-model="settingsForm.description" rows="3"
                                class="w-full bg-paper border border-ink/10 rounded px-3 py-2 text-sm focus:outline-none focus:border-accent"></textarea>
                        </div>

                        <div class="grid grid-cols-2 gap-4">
                            <div>
                                <label
                                    class="block text-xs font-bold uppercase tracking-wider text-ink/40 mb-1">Visibility</label>
                                <select v-model="settingsForm.visibility"
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
                                <label
                                    class="block text-xs font-bold uppercase tracking-wider text-ink/40 mb-1">Layout Renderer</label>
                                <select v-model="settingsForm.renderer_id"
                                    class="w-full bg-paper border border-ink/10 rounded px-3 py-2 text-sm focus:outline-none focus:border-accent">
                                    <option value="default">Standard (Blog)</option>
                                    <option value="math_v1">Math Archive (Graph)</option>
                                    <option value="math_v3">Math Manuscript (Book)</option>
                                    <option value="english_v1">English Analysis (Academic)</option>
                                </select>
                            </div>
                            <div class="col-span-2">
                                <label
                                    class="block text-xs font-bold uppercase tracking-wider text-ink/40 mb-1">Tags</label>
                                <TagInput v-model="settingsForm.tags" placeholder="Add tags..." />
                            </div>
                        </div>

                        <div>
                            <label class="block text-xs font-bold uppercase tracking-wider text-ink/40 mb-1">Cover
                                Image</label>

                            <div
                                class="relative group w-full h-48 rounded-lg overflow-hidden bg-ash/10 border border-ink/10">
                                <!-- Preview -->
                                <img v-if="settingsForm.cover_image" :src="settingsForm.cover_image"
                                    class="w-full h-full object-cover" 
                                    :style="{ objectPosition: `50% ${settingsForm.cover_offset_y || 50}%` }" />
                                <div v-else class="w-full h-full flex items-center justify-center text-ink/20">
                                    <i class="ri-image-add-line text-4xl"></i>
                                </div>

                                <!-- Overlay Action -->
                                <div
                                    class="absolute inset-0 bg-black/50 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center">
                                    <label
                                        class="cursor-pointer bg-white text-black px-4 py-2 rounded-full font-medium text-sm hover:bg-white/90 transition-colors flex items-center gap-2">
                                        <i class="ri-upload-cloud-line"></i> Change Image
                                        <input type="file" accept="image/*" class="hidden"
                                            @change="handleImageUpload" />
                                    </label>
                                </div>
                            </div>

                             <!-- Position Slider -->
                            <div v-if="settingsForm.cover_image" class="mt-4 bg-surface p-3 rounded border border-ink/5">
                                <label class="block text-[10px] font-bold uppercase tracking-wider text-ink/40 mb-2 flex justify-between">
                                    <span>Cover Position (Vertical)</span>
                                    <span>{{ settingsForm.cover_offset_y }}%</span>
                                </label>
                                <input type="range" min="0" max="100" v-model.number="settingsForm.cover_offset_y" 
                                    class="w-full h-1 bg-ink/10 rounded-lg appearance-none cursor-pointer accent-ink" />
                            </div>

                            <!-- Fallback URL Input (optional, keeps flexibility) -->
                            <input v-model="settingsForm.cover_image" placeholder="Or enter image URL..."
                                class="mt-2 w-full bg-transparent border-b border-ink/10 py-1 text-xs text-ink/60 focus:outline-none focus:border-accent" />
                        </div>

                        <div class="pt-6 border-t border-ink/5 flex justify-between items-center">
                            <button @click="deleteKb"
                                class="text-red-500 hover:text-red-700 text-sm font-medium px-4 py-2 hover:bg-red-50 rounded transition-colors">
                                <i class="ri-delete-bin-line mr-1"></i> Delete Knowledge Base
                            </button>
                            <button @click="updateKbSettings"
                                class="bg-ink text-paper px-6 py-2 rounded font-medium hover:bg-accent transition-colors">
                                Save Changes
                            </button>
                        </div>
                    </div>
                </div>
            </div>

            <!-- ARTICLE READ VIEW -->
            <div v-else-if="viewMode === 'article'" class="max-w-4xl mx-auto pb-32">
                 <div class="mb-8 border-b border-ink/5 pb-6">
                    <h1 class="text-4xl font-serif font-bold mb-4">{{ currentArticle?.title }}</h1>
                    <div class="flex items-center gap-4 text-xs text-ink/40 font-mono">
                        <div class="flex items-center gap-2">
                            <span v-for="tag in currentArticle?.tags" :key="tag" 
                                class="bg-surface px-2 py-0.5 rounded border border-ink/10">
                                #{{ tag }}
                            </span>
                        </div>
                        <span v-if="currentArticle?.updated_at">
                            Updated {{ new Date(currentArticle.updated_at).toLocaleDateString() }}
                        </span>
                    </div>
                </div>
                
                <BlockRenderer :blocks="currentBlocks" />
            </div>

            <!-- DETAIL VIEW -->
            <div v-else class="space-y-1">
                <div v-if="contents.length === 0 && !isLoading" class="py-12 text-center text-ink/40">
                    <p>This folder is empty.</p>
                </div>

                <div v-for="item in contents" :key="item.id"
                    @click="item.type === 'Folder' ? navigateInto(item) : navigateToArticle(item)"
                    @contextmenu="handleContextMenu($event, item)"
                    class="flex items-center justify-between p-3 rounded hover:bg-surface/50 border border-transparent hover:border-ink/5 group transition-colors cursor-pointer relative">



                    <div class="flex items-center gap-3">
                        <i v-if="item.type === 'Folder'"
                            class="ri-folder-fill text-xl text-yellow-500/80"></i>
                        <i v-else class="ri-file-text-line text-xl text-ink/40"></i>

                        <span class="font-medium text-sm">{{ item.title }}</span>
                    </div>

                    <div class="flex items-center gap-4 text-xs text-ink/40">
                        <span v-if="item.visibility === 'Private'" title="Private"><i class="ri-lock-line"></i></span>
                        <span v-if="item.visibility === 'Public'" title="Public" class="text-green-500"><i
                                class="ri-global-line"></i></span>
                        <span v-if="item.visibility === 'Internal'" title="Internal" class="text-blue-500"><i
                                class="ri-group-line"></i></span>

                        <span
                            class="opacity-0 group-hover:opacity-100 transition-opacity">{{ new Date(item.updated_at).toLocaleDateString() }}</span>
                    </div>
                </div>
            </div>

            <div v-if="isLoading" class="py-12 text-center text-ink/40 animate-pulse">
                Loading...
            </div>
        </div>

        <!-- MODALS (Simple inline implementation for minimal deps) -->
        
        <!-- Context Menu -->
        <div v-if="contextMenuVisible" 
             class="fixed z-[100] bg-paper border border-ink/10 shadow-xl rounded py-1 min-w-[160px] flex flex-col"
             :style="{ top: `${contextMenuPosition.y}px`, left: `${contextMenuPosition.x}px` }"
             @click.stop>
            <button @click="handleDeleteFromMenu" 
                class="text-left px-4 py-2 text-sm text-red-500 hover:bg-red-50 transition-colors flex items-center gap-2">
                <i class="ri-delete-bin-line"></i> Delete
            </button>
        </div>

        <div v-if="kbFormVisible"
            class="fixed inset-0 z-50 flex items-center justify-center bg-black/20 backdrop-blur-sm"
            @click.self="kbFormVisible = false">
            <div class="bg-paper p-6 rounded-lg border border-ink/10 shadow-xl w-96 max-w-full">
                <h3 class="text-lg font-bold mb-4">New Knowledge Base</h3>
                <input v-model="kbForm.title" placeholder="Title"
                    class="w-full bg-surface border border-ink/10 rounded px-3 py-2 mb-3 text-sm focus:outline-none focus:border-accent"
                    autoFocus />
                
                <div class="mb-3">
                     <label class="block text-xs font-bold uppercase tracking-wider text-ink/40 mb-1">Layout</label>
                     <select v-model="kbForm.renderer_id"
                        class="w-full bg-surface border border-ink/10 rounded px-3 py-2 text-sm focus:outline-none focus:border-accent">
                        <option value="default">Standard (Blog)</option>
                        <option value="math_v1">Math Archive (Graph)</option>
                        <option value="math_v3">Math Manuscript (Book)</option>
                        <option value="english_v1">English Analysis (Academic)</option>
                        <option value="memo">Memos (Sticky Notes)</option>
                        <option value="vocabulary">Vocabulary (English)</option>
                    </select>
                </div>

                <textarea v-model="kbForm.description" placeholder="Description" rows="3"
                    class="w-full bg-surface border border-ink/10 rounded px-3 py-2 mb-3 text-sm focus:outline-none focus:border-accent"></textarea>

                <TagInput v-model="kbForm.tags" placeholder="Tags" class="mb-3" />

                <div class="mb-4">
                    <label class="block text-xs font-bold uppercase tracking-wider text-ink/40 mb-1">Cover Image</label>
                    <div class="relative group w-full h-32 rounded-lg overflow-hidden bg-ash/10 border border-ink/10">
                        <img v-if="kbForm.cover_image" :src="kbForm.cover_image" class="w-full h-full object-cover" 
                             :style="{ objectPosition: `50% ${kbForm.cover_offset_y || 50}%` }" />
                        <div v-else class="w-full h-full flex items-center justify-center text-ink/20">
                            <i class="ri-image-add-line text-3xl"></i>
                        </div>
                        <div class="absolute inset-0 bg-black/50 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center">
                            <label class="cursor-pointer bg-white text-black px-3 py-1.5 rounded-full font-medium text-xs hover:bg-white/90 transition-colors flex items-center gap-1">
                                <i class="ri-upload-cloud-line"></i> Upload
                                <input type="file" accept="image/*" class="hidden" @change="handleKbCreateImageUpload" />
                            </label>
                        </div>
                    </div>

                    
                    <!-- Position Slider -->
                    <div v-if="kbForm.cover_image" class="mt-2 bg-surface p-2 rounded border border-ink/5">
                         <label class="block text-[10px] font-bold uppercase tracking-wider text-ink/40 mb-1 flex justify-between">
                            <span>Position</span>
                            <span>{{ kbForm.cover_offset_y }}%</span>
                        </label>
                        <input type="range" min="0" max="100" v-model.number="kbForm.cover_offset_y" 
                            class="w-full h-1 bg-ink/10 rounded-lg appearance-none cursor-pointer accent-ink" />
                    </div>

                    <input v-model="kbForm.cover_image" placeholder="Or enter image URL..."
                        class="mt-2 w-full bg-transparent border-b border-ink/10 py-1 text-xs text-ink/60 focus:outline-none focus:border-accent" />
                </div>
                <div class="flex justify-end gap-2 text-xs font-medium">
                    <button @click="kbFormVisible = false" class="px-3 py-1.5 hover:bg-surface rounded">Cancel</button>
                    <button @click="createKb"
                        class="bg-ink text-paper px-3 py-1.5 rounded hover:bg-accent transition-colors">Create</button>
                </div>
            </div>
        </div>

        <div v-if="contentFormVisible"
            class="fixed inset-0 z-50 flex items-center justify-center bg-black/20 backdrop-blur-sm"
            @click.self="contentFormVisible = false">
            <div class="bg-paper p-6 rounded-lg border border-ink/10 shadow-xl w-96 max-w-full">
                <h3 class="text-lg font-bold mb-4">New Folder</h3>
                <input v-model="contentForm.title" placeholder="Title"
                    class="w-full bg-surface border border-ink/10 rounded px-3 py-2 mb-4 text-sm focus:outline-none focus:border-accent"
                    autoFocus @keyup.enter="createContent" />
                <div class="flex justify-end gap-2 text-xs font-medium">
                    <button @click="contentFormVisible = false"
                        class="px-3 py-1.5 hover:bg-surface rounded">Cancel</button>
                    <button @click="createContent"
                        class="bg-ink text-paper px-3 py-1.5 rounded hover:bg-accent transition-colors">Create</button>
                </div>
            </div>
        </div>
    </div>
</template>
