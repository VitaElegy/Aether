<script setup lang="ts">
import { ref, watch, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { contentApi, type Content } from '../../api/content';

const props = defineProps<{
    knowledgeBaseId: string;
    currentArticleId?: string;
}>();

const router = useRouter();
const loading = ref(true);
const treeItems = ref<{ item: Content; level: number; isOpen: boolean }[]>([]);
const rawContents = ref<Content[]>([]);
// Map of folder_id -> isOpen
const openFolders = ref<Record<string, boolean>>({});

const loadContents = async () => {
    loading.value = true;
    try {
        // Fetch all contents (limit 1000 to be safe for now)
        // Note: Ideally backend should support filtering by KB ID.
        // We follow existing pattern of fetching list and filtering client-side.
        const all = await contentApi.list({ limit: 1000 });
        rawContents.value = all.filter(c => c.knowledge_base_id === props.knowledgeBaseId);
        rebuildTree();

        // Auto-expand path to current article
        if (props.currentArticleId) {
            expandPathTo(props.currentArticleId);
        }
    } catch (e) {
        console.error("Failed to load directory", e);
    } finally {
        loading.value = false;
    }
};

const getChildren = (parentId?: string) => {
    if (!parentId) {
        return rawContents.value.filter(c => !c.parent_id);
    }
    return rawContents.value.filter(c => c.parent_id === parentId);
};

// Flatten tree to list for rendering
const rebuildTree = () => {
    const result: { item: Content; level: number; isOpen: boolean }[] = [];

    const traverse = (parentId: string | undefined, level: number) => {
        const children = getChildren(parentId);
        // Sort: Directories first, then Articles. Alphabetical within.
        children.sort((a, b) => {
            if (a.type !== b.type) {
                return a.type === 'Folder' ? -1 : 1;
            }
            return a.title.localeCompare(b.title);
        });

        for (const child of children) {
            const isOpen = openFolders.value[child.id] || false;
            result.push({ item: child, level, isOpen });

            if (child.type === 'Folder' && isOpen) {
                traverse(child.id, level + 1);
            }
        }
    };

    traverse(undefined, 0);
    treeItems.value = result;
};

const toggleFolder = (folder: Content) => {
    console.log("[DirectoryTree] Toggling folder:", folder.id, openFolders.value[folder.id]);
    if (folder.type !== 'Folder') return;
    openFolders.value[folder.id] = !openFolders.value[folder.id];
    rebuildTree();
};

const expandPathTo = (targetId: string) => {
    const target = rawContents.value.find(c => c.id === targetId);
    if (!target || !target.parent_id) return;

    let pid = target.parent_id;
    while (pid) {
        openFolders.value[pid] = true;
        const parent = rawContents.value.find(c => c.id === pid);
        pid = parent?.parent_id || ''; // stop if no parent found or moved to root
    }
    rebuildTree();
};

const navigateTo = (item: Content) => {
    console.log("[DirectoryTree] Clicked item:", item.title, item.type, item.id);
    if (item.type === 'Folder') {
        toggleFolder(item);
    } else {
        console.log("[DirectoryTree] Navigating to article:", item.id);
        router.push(`/article/${item.id}`);
    }
};

onMounted(() => {
    loadContents();
});

watch(() => props.knowledgeBaseId, () => {
    loadContents();
});
</script>

<template>
    <div class="px-4 py-8">
        <div v-if="loading" class="text-xs text-ink/40 uppercase tracking-widest animate-pulse px-4">
            Loading Tree...
        </div>

        <nav v-else-if="treeItems.length > 0" class="flex flex-col gap-1">
            <div v-for="node in treeItems" :key="node.item.id"
                class="flex items-center gap-2 px-3 py-1.5 rounded cursor-pointer transition-colors text-xs font-medium select-none"
                :class="{
                    'bg-accent/10 text-accent': node.item.id === currentArticleId,
                    'text-ink/60 hover:bg-ash/30 hover:text-ink': node.item.id !== currentArticleId,
                    'pl-3': node.level === 0,
                    'pl-6': node.level === 1,
                    'pl-9': node.level === 2,
                    'pl-12': node.level === 3,
                    'pl-16': node.level > 3,
                }" @click="navigateTo(node.item)">

                <!-- Icon -->
                <span class="opacity-70 flex-shrink-0 w-4 text-center">
                    <template v-if="node.item.type === 'Folder'">
                        <i v-if="node.isOpen" class="ri-folder-open-fill text-yellow-500/80"></i>
                        <i v-else class="ri-folder-3-fill text-yellow-500/80"></i>
                    </template>
                    <i v-else class="ri-file-text-line"></i>
                </span>

                <span class="truncate">{{ node.item.title }}</span>
            </div>
        </nav>

        <div v-else class="text-xs text-ink/20 italic font-mono px-4">
            Empty Knowledge Base.
        </div>
    </div>
</template>
