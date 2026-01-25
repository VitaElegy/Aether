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
import ProjectList from './ProjectList.vue';
import VulnerabilityKanban from './VulnerabilityKanban.vue';
import { useVrkbStore } from '@/stores/vrkb';

const store = useVrkbStore();
const currentProject = computed(() => store.currentProject);

// Sync URL
onMounted(() => {
    const params = new URLSearchParams(window.location.search);
    const pid = params.get('project_id');
    if (pid) {
        store.selectProject(pid);
    }
});

watch(currentProject, (newVal) => {
    const url = new URL(window.location.href);
    if (newVal) {
        url.searchParams.set('project_id', newVal.id);
    } else {
        url.searchParams.delete('project_id');
    }
    window.history.replaceState({}, '', url.toString());
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
