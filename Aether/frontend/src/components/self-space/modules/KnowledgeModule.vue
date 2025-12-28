<script setup lang="ts">
import { ref, computed, onMounted, watch } from 'vue';
import { useRouter } from 'vue-router';
import { knowledgeApi, type KnowledgeBase } from '../../../api/knowledge';
import { contentApi, type Content } from '../../../api/content';
import { uploadApi } from '../../../api/upload';
import TagInput from '../../common/TagInput.vue';

// -- State --
// -- State --
const viewMode = ref<'list' | 'detail' | 'settings'>('list');
const knowledgeBases = ref<KnowledgeBase[]>([]);
const currentKb = ref<KnowledgeBase | null>(null);
const currentPath = ref<Content[]>([]); // Stack of folder objects
const contents = ref<Content[]>([]); // Current folder contents
const isLoading = ref(false);

const kbFormVisible = ref(false);
const kbForm = ref({ title: '', description: '', tags: '', cover_image: '' });

// Settings Form
const settingsForm = ref({
    title: '',
    description: '',
    tags: '',
    cover_image: '',
    visibility: 'Private' as 'Public' | 'Private' | 'Internal'
});

const contentFormVisible = ref(false);
const contentFormType = ref<'Directory' | 'Article'>('Article');
const contentForm = ref({ title: '' });


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
    currentKb.value = kb;
    currentPath.value = [];
    viewMode.value = 'detail';
    refreshContent();
};

const openSettings = () => {
    if (!currentKb.value) return;
    settingsForm.value = {
        title: currentKb.value.title,
        description: currentKb.value.description || '',
        tags: currentKb.value.tags ? currentKb.value.tags.join(', ') : '',
        cover_image: currentKb.value.cover_image || '',
        visibility: currentKb.value.visibility || 'Private'
    };
    viewMode.value = 'settings';
};

const updateKbSettings = async () => {
    if (!currentKb.value) return;
    try {
        await knowledgeApi.update(currentKb.value.id, {
            title: settingsForm.value.title,
            description: settingsForm.value.description,
            tags: settingsForm.value.tags.split(',').map(t => t.trim()).filter(t => t),
            cover_image: settingsForm.value.cover_image,
            visibility: settingsForm.value.visibility
        });

        // Update local state
        currentKb.value.title = settingsForm.value.title;
        currentKb.value.description = settingsForm.value.description;
        currentKb.value.tags = settingsForm.value.tags.split(',').map(t => t.trim()).filter(t => t);
        currentKb.value.cover_image = settingsForm.value.cover_image;
        currentKb.value.visibility = settingsForm.value.visibility;

        viewMode.value = 'detail';
        fetchKBs(); // Type of reload
    } catch (e) {
        console.error("Failed to update KB", e);
    }
};

const handleImageUpload = async (event: Event) => {
    const input = event.target as HTMLInputElement;
    if (!input.files || input.files.length === 0) return;

    const file = input.files[0];
    try {
        const url = await uploadApi.uploadFile(file);
        settingsForm.value.cover_image = url;
    } catch (e) {
        console.error("Failed to upload image", e);
        alert("Failed to upload image");
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
        const allContent = await contentApi.list({ limit: 100 });

        contents.value = allContent.filter(c => {
            if (c.knowledge_base_id !== currentKb.value?.id) return false;
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
    if (folder.content_type !== 'Directory') return;
    currentPath.value.push(folder);
    refreshContent();
};

const navigateUp = (index?: number) => {
    if (index === undefined) {
        // Go to root
        currentPath.value = [];
    } else {
        // Go to specific index
        currentPath.value = currentPath.value.slice(0, index + 1);
    }
    viewMode.value = 'detail'; // Ensure looking at detail
    refreshContent();
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
            visibility: 'Private' // Default
        });
        kbFormVisible.value = false;
        kbForm.value = { title: '', description: '', tags: '', cover_image: '' };
        fetchKBs();
    } catch (e) {
        console.error(e);
    }
};

const createContent = async () => {
    if (!contentForm.value.title) return;
    if (!currentKb.value) return;

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
            type: contentFormType.value,
            status: 'Published' // Default to Published so it shows up? Or Draft?
        });
        contentFormVisible.value = false;
        contentForm.value = { title: '' };
        refreshContent();
    } catch (e) {
        console.error(e);
    }
};

onMounted(() => {
    fetchKBs();
});

</script>

<template>
    <div class="h-full flex flex-col p-6 max-w-5xl mx-auto w-full">

        <!-- HEADER -->
        <div class="flex items-center justify-between mb-8">
            <div class="flex items-center gap-3">
                <button v-if="viewMode === 'detail' || viewMode === 'settings'" @click="goBackToList"
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
                    <button @click="contentFormType = 'Directory'; contentFormVisible = true"
                        class="bg-paper border border-ink/10 text-ink px-3 py-1.5 text-xs font-medium rounded hover:border-accent hover:text-accent transition-colors">
                        <i class="ri-folder-add-line mr-1"></i> 文件夹
                    </button>
                    <button @click="contentFormType = 'Article'; contentFormVisible = true"
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
                            class="w-full h-full object-cover transition-transform duration-700 group-hover:scale-110" />
                        <div v-else
                            class="w-full h-full flex items-center justify-center bg-gradient-to-br from-ash/20 to-ash/5 text-ink/10">
                            <i class="ri-book-mark-fill text-6xl"></i>
                        </div>
                    </div>

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
            <div v-else-if="viewMode === 'settings'" class="max-w-2xl mx-auto">
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
                                    class="w-full h-full object-cover" />
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

            <!-- DETAIL VIEW -->
            <div v-else class="space-y-1">
                <div v-if="contents.length === 0 && !isLoading" class="py-12 text-center text-ink/40">
                    <p>This folder is empty.</p>
                </div>

                <div v-for="item in contents" :key="item.id"
                    @click="item.content_type === 'Directory' ? navigateInto(item) : navigateToArticle(item)"
                    class="flex items-center justify-between p-3 rounded hover:bg-surface/50 border border-transparent hover:border-ink/5 group transition-colors cursor-pointer">

                    <div class="flex items-center gap-3">
                        <i v-if="item.content_type === 'Directory'"
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
        <div v-if="kbFormVisible"
            class="fixed inset-0 z-50 flex items-center justify-center bg-black/20 backdrop-blur-sm"
            @click.self="kbFormVisible = false">
            <div class="bg-paper p-6 rounded-lg border border-ink/10 shadow-xl w-96 max-w-full">
                <h3 class="text-lg font-bold mb-4">New Knowledge Base</h3>
                <input v-model="kbForm.title" placeholder="Title"
                    class="w-full bg-surface border border-ink/10 rounded px-3 py-2 mb-3 text-sm focus:outline-none focus:border-accent"
                    autoFocus />
                <textarea v-model="kbForm.description" placeholder="Description" rows="3"
                    class="w-full bg-surface border border-ink/10 rounded px-3 py-2 mb-3 text-sm focus:outline-none focus:border-accent"></textarea>

                <TagInput v-model="kbForm.tags" placeholder="Tags" class="mb-3" />

                <input v-model="kbForm.cover_image" placeholder="Cover Image URL (optional)"
                    class="w-full bg-surface border border-ink/10 rounded px-3 py-2 mb-6 text-sm focus:outline-none focus:border-accent" />
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
                <h3 class="text-lg font-bold mb-4">New {{ contentFormType }}</h3>
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
