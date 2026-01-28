<template>
    <div class="h-full flex flex-col bg-paper relative overflow-hidden">
        <Transition name="fade" mode="out-in">
            <VulnerabilityKanban v-if="currentProject" />
            <ProjectList v-else />
        </Transition>
    </div>
</template>

<script setup lang="ts">
import { computed, watch, onMounted } from 'vue';
import { useRoute, useRouter } from 'vue-router'; // [NEW]
import ProjectList from './ProjectList.vue';
import VulnerabilityKanban from './VulnerabilityKanban.vue';
import { useVrkbStore } from '@/stores/vrkb';

const store = useVrkbStore();
const route = useRoute();
const router = useRouter();

const currentProject = computed(() => store.currentProject);

// Sync URL (Deep Linking Rule)
// Sync URL (Deep Linking Rule)
// 1. Initialize from URL
onMounted(() => {
    const pid = route.query.project as string;
    if (pid) {
        store.selectProject(pid);
    } else {
        // CRITICAL: If no project in URL, ensure store is cleared
        // This prevents stale state from persisting when switching modules
        store.selectProject(null);
    }
});

// 2. Watch URL -> Store (Forward Sync)
// Handles browser back/forward buttons, or clicking Dock icon (which resets URL)
watch(() => route.query.project, (newPid) => {
    if (newPid && typeof newPid === 'string') {
        if (store.currentProject?.id !== newPid) {
            store.selectProject(newPid);
        }
    } else {
        if (store.currentProject) {
            store.selectProject(null);
        }
    }
});

watch(currentProject, (newVal) => {
    const query = { ...route.query };
    if (newVal) {
        query.project = newVal.id;
    } else {
        delete query.project;
        // Also clear view if we go back to project list
        delete query.view;
    }
    router.replace({ query });
});
</script>

<style scoped>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
